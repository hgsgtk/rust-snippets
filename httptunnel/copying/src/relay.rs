use core::fmt;
use std::time::{Duration, Instant};

use crate::tunnel::TunnelCtx;

use log::{error, info, debug};
use std::future::Future;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::time::timeout;

/// Compile-time constants and compile-time evaluable functions.
/// > Constants, like statics, should always be in SCREAMING_SNAKE_CASE.
/// https://doc.rust-lang.org/std/keyword.const.html
pub const NO_TIMEOUT: Duration = Duration::from_secs(300);
pub const NO_BANDWIDTH_LIMIT: u64 = 1_000_000_000_000_u64;
const BUFFER_SIZE: usize = 16 * 1024;

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub enum RelayShutdownReasons {
    /// (Original comments)
    /// If a reader connection was gracefully closed
    GracefulShutdown,
    ReadError,
    WriteError,
    ReaderTimeout,
    WriterTimeout,
    TooSlow,
    TooFast,
}

#[derive(Builder, Deserialize, Clone)]
pub struct RelayPolicy {
    #[serde(with = "humantime_serde")]
    pub idle_timeout: Duration,
    // Date type u64 64bit integer
    // https://doc.rust-lang.org/book/ch03-02-data-types.html
    pub min_rate_bpm: u64, // bpm = bytes per minute
    pub max_rate_bpm: u64,
}

impl RelayPolicy {
    /// Future trait
    /// > A future is a value that may not have finished computing yet. 
    /// > This kind of "asynchronous value" makes it possible for a thread to continue doing useful work 
    /// > while it waits for the value to become available.
    pub async fn timed_operation<T: Future>(&self, f: T) -> Result<<T as Future>::Output, ()> {
        if self.idle_timeout >= NO_TIMEOUT {
            return Ok(f.await);
        }
        let result = timeout(self.idle_timeout, f).await;

        if let Ok(r) = result {
            Ok(r)
        } else {
            Err(())
        }
    }

    /// (Original comments)
    /// Basic rate limiting. Placeholder for more sophisticated policy handling.
    /// e.g. sliding windows, detecting heavy hitters, etc.
    pub fn check_transimission_rates(
        &self,
        start: &Instant,
        total_bytes: usize,
    ) -> Result<(), RelayShutdownReasons> {
        if self.min_rate_bpm == 0 && self.max_rate_bpm >= NO_BANDWIDTH_LIMIT {
            return Ok(());
        }

        let elapsed = Instant::now().duration_since(*start);
        if elapsed.as_secs_f32() > 5.
            && total_bytes as u64 / elapsed.as_secs() as u64 > self.max_rate_bpm
        {
            // prevent bandwidth abuse
            // https://patents.google.com/patent/US20140010082A1/en
            Err(RelayShutdownReasons::TooFast)
        } else if elapsed.as_secs_f32() >= 30.
            && total_bytes as f64 / elapsed.as_secs_f64() / 60. < self.min_rate_bpm as f64
        {
            // prevent slowloris https://en.wikipedia.org/wiki/Slowloris_(computer_security)
            Err(RelayShutdownReasons::TooSlow)
        } else {
            Ok(())
        }
    }
}

/// (Original comments)
/// Relays traffic from one stream to another in a single direction.
/// To relay two sockets in full-duplex mode you need to create two `Relays` in both directions.
/// It doesn't really matter what is the protocol, as it only requires `AsyncReadExt`
/// and `AsyncWriteExt` traits from the source and the target.
#[derive(Builder, Clone)]
pub struct Relay {
    name: &'static str,
    relay_policy: RelayPolicy,
    tunnel_ctx: TunnelCtx,
}

impl Relay {
    /// (Original comments)
    /// Relays data in a single directon. 
    /// 
    /// (My comments)
    /// AsyncReadExt, AsyncWriteExt 
    /// > Implemented as an extention trait, adding utility methods to all AsyncWrite types.
    /// > Callers will tend to import this trait instead of AsyncWrite.
    /// https://docs.rs/tokio/0.2.9/tokio/io/trait.AsyncWriteExt.html
    /// ReadHalf The readable half of a value returned from split.
    /// https://docs.rs/tokio/0.2.9/tokio/io/struct.ReadHalf.html
    pub async fn relay_data<R: AsyncReadExt + Sized, W: AsyncWriteExt + Sized>(
        self,
        mut source: ReadHalf<R>,
        mut dest: WriteHalf<W>,
    ) -> io::Result<RelayStats> {
        let mut buffer = [0; BUFFER_SIZE];

        let mut total_bytes = 0;
        let mut event_count = 0;
        let start_time = Instant::now();
        let shutdown_reason;

        loop {
            let read_result = self
                .relay_policy
                .timed_operation(source.read(&mut buffer))
                .await;
            
                if read_result.is_err() {
                    shutdown_reason = RelayShutdownReasons::ReaderTimeout;
                    break;
                }

                let n = match read_result.unwrap() {
                    Ok(n) if n == 0 => {
                        shutdown_reason = RelayShutdownReasons::GracefulShutdown;
                        break;
                    }
                    Ok(n) => n,
                    Err(e) => {
                        error!(
                            "{} failed to read, Err = {:?}, CTX={}",
                            self.name, e, self.tunnel_ctx
                        );
                        shutdown_reason = RelayShutdownReasons::ReadError;
                        break;
                    }
                };

                let write_result = self
                    .relay_policy
                    .timed_operation(dest.write_all(&buffer[..n]))
                    .await;
                
                if write_result.is_err() {
                    shutdown_reason = RelayShutdownReasons::WriterTimeout;
                    break;
                }

                if let Err(e) = write_result.unwrap() {
                    error!(
                        "{} failed to write {} bytes. Err = {:?}, CTX={}",
                        self.name, n, e, self.tunnel_ctx
                    );
                    shutdown_reason = RelayShutdownReasons::WriteError;
                    break;
                }

                total_bytes += n;
                event_count += 1;

                if let Err(rate_violation) = self
                    .relay_policy
                    .check_transimission_rates(&start_time, total_bytes)
                {
                    shutdown_reason = rate_violation;
                    break;
                }
        }

        self.shutdown(&mut dest, &shutdown_reason).await;

        let duration = Instant::now().duration_since(start_time);

        let stats = RelayStatsBuilder::default()
            .shutdown_reason(shutdown_reason)
            .total_bytes(total_bytes)
            .event_count(event_count)
            .duration(duration)
            .build()
            .expect("RelayStatsBuilder failed");


        info!("{} closed: {}, CTX={}", self.name, stats, self.tunnel_ctx);
        
        Ok(stats)
    }

    async fn shutdown<W: AsyncWriteExt + Sized>(
        &self,
        dest: &mut WriteHalf<W>,
        reason: &RelayShutdownReasons,
    ) {
        match dest.shutdown().await {
            Ok(_) => {
                debug!(
                    "{} shutdown due do {:?}, CTX={}",
                    self.name, reason, self.tunnel_ctx
                );
            }
            Err(e) => {
                error!(
                    "{} failed to shutdown. Err = {:?}, CTX={}",
                    self.name, e, self.tunnel_ctx
                );
            }
        }
    }
}

/// (Original comments)
/// Stats after the relay is closed. Can be used for telemetry/monitoring.
#[derive(Builder, Clone, Debug, Serialize)]
pub struct RelayStats {
    pub shutdown_reason: RelayShutdownReasons,
    pub total_bytes: usize,
    pub event_count: usize,
    pub duration: Duration,
}

impl fmt::Display for RelayStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "shutdown_reason={:?}, bytes={}, event_count={}, duration={:?}, rate_kbps={:.3}",
            self.shutdown_reason,
            self.total_bytes,
            self.event_count,
            self.duration,
            self.total_bytes as f64 / 1024. / self.duration.as_secs_f64()
        )
    }
}