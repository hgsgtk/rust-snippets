use std::time::{Duration, Instant};

use crate::tunnel::TunnelCtx;

use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};

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

        // TODO: loop relaying

        let stats = RelayStatsBuilder::default()
            .shutdown_reason(RelayShutdownReasons::GracefulShutdown) // Fake it!
            .build()
            .expect("RelayStatsBuilder failed");
        Ok(stats)
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