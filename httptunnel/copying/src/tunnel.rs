use async_trait::async_trait;

use crate::configuration::TunnelConfig;
use crate::proxy_target::{Nugget, TargetConnector};
use crate::relay::{RelayStats, RelayPolicy, Relay, RelayBuilder};

use core::fmt;
use futures::{StreamExt, SinkExt};
use futures::stream::SplitStream;
use log::{debug, error};
use std::fmt::Display;
use std::time::Duration;
use tokio::io;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time::timeout;
use tokio_util::codec::{Decoder, Encoder, Framed};

/// trait std::default::Default https://doc.rust-lang.org/std/default/trait.Default.html
/// A trait for giving a type a useful default value
#[derive(Builder, Copy, Clone, Default, Serialize)]
pub struct TunnelCtx {
    id: u128,
}

/// (Original comments)
/// Statistics. No sensitive information
#[derive(Serialize, Builder)]
pub struct TunnelStats {
    tunnel_ctx: TunnelCtx,
    result: EstablishTunnelResult,
    upstream_stats: Option<RelayStats>,
    downstream_stats: Option<RelayStats>,
}

// https://doc.rust-lang.org/std/fmt/trait.Display.html#examples
impl fmt::Display for TunnelCtx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// Dead Code https://doc.rust-lang.org/rust-by-example/attribute/unused.html
#[derive(Eq, PartialEq, Debug, Clone, Serialize)]
#[allow(dead_code)]
pub enum EstablishTunnelResult {
    /// Successfully connnected to target.
    Ok,
    /// Successfully connected to target by has a nugget to send after connection establishment.
    /// TODO: What's nugget?
    OkWithNugget,
    /// Malformed request
    BadRequest,
    /// Target is not allowed
    Forbidden,
    /// Unsupported operation, however valid for the protocol
    OperationNotAllowed,
    /// The client failed to send a tunnel request timely
    RequestTimeout,
    /// Cannot connect to target
    BadGateway,
    /// Connection attempt timed out
    GatewayTimeout,
    /// Busy. Try again later
    TooManyRequests,
    /// Any other error. E.g. an abrupt I/O error.
    ServerError,
}


/// (Original comment)
/// A connection tunnel.
/// 
/// # Parameters
/// * `<H>` - proxy handshake codec for initiating a tunnel.
///   It extracts the request message which contains the target and potentially policies.
///   It also takes care of encoding a response.
/// * `<C>` - a connection from the client.
/// * `<T>` - target connector. It takes result produced by the codec 
///           and establishes a connection to a target.
/// 
/// Once the target connection is established, it relays data until any connection is closed or an error happens.
/// 
/// (My comment)
/// std::option::Option https://doc.rust-lang.org/std/option/enum.Option.html
/// > Type Option represents an optional value: every Option is either Some and contains a value,
/// > or None, and does not. Option types are very common in Rust code, as they have a number of uses:
/// https://doc.rust-lang.org/std/option/index.html
/// > Rust の Option<T> 型は、値が 存在しない 可能性を暗示する列挙型です
/// > Option 型は、他の言語では Maybe や Optional などと呼ばれることもあります。
/// https://qiita.com/tatsuya6502/items/cd41599291e2e5f38a4a
/// > Rust の Result<T, E> 型は エラーになる可能性 を暗示する列挙型です。定義は以下の通りです。
pub struct ConnectionTunnel<H, C, T> {
    tunnel_request_codec: Option<H>,
    tunnel_ctx: TunnelCtx,
    target_connector: T,
    client: Option<C>,
    tunnel_config: TunnelConfig
}

#[async_trait]
pub trait TunnelTarget {
    // Advanced Traits > Specifying Placeholder Types in Trait > Associated Types
    // https://doc.rust-lang.org/book/ch19-03-advanced-traits.html
    type Addr;
    fn target_addr(&self) -> Self::Addr;
    fn has_nugget(&self) -> bool;
    fn nugget(&self) -> &Nugget;
}

impl<H, C, T> ConnectionTunnel<H, C, T>
where
    // Decoder: A Decoder is used together with FramedRead or Framed to turn an AsyncRead into a Stream.
    // > The main method on the Decoder trait is the decode method
    // https://docs.rs/tokio-util/0.6.7/tokio_util/codec/index.html
    H: Decoder<Error = EstablishTunnelResult> + Encoder<EstablishTunnelResult>,
    // std::maker::Sized: Types with a constant size known at compile time.
    // https://doc.rust-lang.org/std/marker/trait.Sized.html
    // std::fmt::Display: Format trait for an empty format, {}.
    // https://doc.rust-lang.org/std/fmt/trait.Display.html
    H::Item: TunnelTarget + Sized + Display + Send + Sync,
    // tokio::io::AsyncWrite: Writes bytes asynchronously.
    // https://docs.rs/tokio/0.2.6/tokio/io/trait.AsyncWrite.html
    C: AsyncRead + AsyncWrite + Sized + Send + Unpin + 'static,
    T: TargetConnector<Target = H::Item>,
{
    pub fn new(
        handshake_codec: H,
        target_connector: T,
        client: C,
        tunnel_config: TunnelConfig,
        tunnel_ctx: TunnelCtx,
    ) -> Self {
        Self {
            // Some: Some value T
            // https://doc.rust-lang.org/std/option/enum.Option.html#variant.Some
            tunnel_request_codec: Some(handshake_codec),
            target_connector,
            tunnel_ctx,
            client: Some(client),
            tunnel_config,
        }
    }

    /// (Original comments)
    /// Once the client connected we wait for a tunnel establishment handshake.
    /// For instance, an `HTTP/1.1 CONNECT` for HTTP tunnels.
    /// 
    /// During handshake we obtained the target, and if we were able to connect to it,
    /// a message indicating success is sent back to client (or an error response otherwise).
    /// 
    /// At that point we start relaying data in full-deplex mode.
    /// 
    /// This method consumes `self` and thus can be called only once.
    /// 
    /// (My comments)
    /// full-deplex: 全二重
    /// stackoverflow: Is HTTP 1.1 Full duplex?
    /// https://stackoverflow.com/questions/23419469/is-http-1-1-full-duplex/27164848
    /// A formal discussion of full duplex https://datatracker.ietf.org/doc/html/draft-zhu-http-fullduplex
    pub async fn start(mut self) -> io::Result<TunnelStats> {
        // Option Enum provides take().
        // https://doc.rust-lang.org/std/option/enum.Option.html#method.take
        let stream = self.client.take().expect("downstream can be taken once");

        let tunnel_result = self.establish_tunnel(stream, self.tunnel_config.clone()).await;

        if let Err(error) = tunnel_result {
            return Ok(TunnelStats {
                tunnel_ctx: self.tunnel_ctx,
                result: error,
                upstream_stats: None,
                downstream_stats: None,
            });
        }

        // upwrap 
        let (client, target) = tunnel_result.unwrap();
        relay_connections(
            client,
            target,
            self.tunnel_ctx,
            self.tunnel_config.client_connection.relay_policy,
            self.tunnel_config.target_connection.relay_policy,
        )
        .await
    }

    async fn establish_tunnel(
        &mut self,
        stream: C,
        configuration: TunnelConfig,
    ) -> Result<(C, T::Stream), EstablishTunnelResult> {
        debug!("Accepting HTTP tunnel requests: CTX={}", self.tunnel_ctx);

        let (mut write, mut read) = self
            .tunnel_request_codec
            .take()
            .expect("estalish_tunnel can be called only once")
            .framed(stream)
            // futures::StreanExt trait provides split() method.
            // > This can be useful when you want to split ownership between tasks, or allow direct interaction between the two objects (e.g. via Sink::send_all).
            // https://docs.rs/futures/0.2.1/futures/trait.StreamExt.html#method.split
            .split();
        
        let (response, target) = self.process_tunnel_request(&configuration, &mut read).await;

        let response_sent = match response {
            EstablishTunnelResult::OkWithNugget => true,
            _ => timeout(
                configuration.client_connection.initiation_timeout,
                // futures::SinkExt trait provides send() method
                // > A future that completes after the given item has been fully processed into the sink, including flushing.
                // https://docs.rs/futures/0.2.1/futures/trait.SinkExt.html#method.send
                write.send(response.clone()),
            )
            .await
            .is_ok(),
        };

        if response_sent {
            match target {
                None => Err(response),
                Some(u) => {
                    // lets take the original stream to either relay data, or to drop it on error
                    let framed = write.reunite(read).expect("Uniting previously split parts");
                    let original_stream = framed.into_inner();

                    Ok((original_stream, u))
                }
            }
        } else {
            Err(EstablishTunnelResult::RequestTimeout)
        }
    }

    async fn process_tunnel_request(
        &mut self,
        configuration: &TunnelConfig,
        // tokio_util::codec::Framed
        // > A unified Stream and Sink interface to an underlying I/O object,
        // >  using the Encoder and Decoder traits to encode and decode frames.
        // https://docs.rs/tokio-util/0.6.7/tokio_util/codec/struct.Framed.html
        // futures::stream::SplitStream A Stream part of the split pair
        // https://docs.rs/futures/0.2.1/futures/stream/struct.SplitStream.html
        read: &mut SplitStream<Framed<C, H>>,
    ) -> (
        EstablishTunnelResult,
        Option<<T as TargetConnector>::Stream>,
    ) {
        let connect_request = timeout(
            configuration.client_connection.initiation_timeout,
            read.next(),
        )
        .await;

        let response;
        let mut target = None;

        if connect_request.is_err() {
            error!("Client established TLS connection but failed an HTTP request within {:?}, CTX={}",
                configuration.client_connection.initiation_timeout,
                self.tunnel_ctx);
            response = EstablishTunnelResult::RequestTimeout;
        } else if let Some(event) = connect_request.unwrap() {
            match event {
                Ok(decoded_target) => {
                    let has_nugget = decoded_target.has_nugget();
                    response = match self
                        .connect_to_target(
                            decoded_target,
                            configuration.target_connection.connect_timeout,
                        )
                        .await
                    {
                        Ok(t) => {
                            target = Some(t);
                            if has_nugget {
                                EstablishTunnelResult::OkWithNugget
                            } else {
                                EstablishTunnelResult::Ok
                            }
                        }
                        Err(e) => e,
                    }
                }
                Err(e) => {
                    response = e;
                }
            }
        } else {
            response = EstablishTunnelResult::BadRequest;
        }

        (response, target)
    }

    async fn connect_to_target(
        &mut self,
        target: T::Target,
        connect_timeout: Duration,
    ) -> Result<T::Stream, EstablishTunnelResult> {
        debug!(
            "Establishing HTTP tunnel target connection: {}, CTX={}",
            target, self.tunnel_ctx,
        );

        let timed_connection_result = timeout(
            connect_timeout, 
            self.target_connector.connect(&target)
        ).await;

        if timed_connection_result.is_err() {
            Err(EstablishTunnelResult::GatewayTimeout)
        } else {
            match timed_connection_result.unwrap() {
                Ok(tcp_stream) => Ok(tcp_stream),
                Err(e) => Err(EstablishTunnelResult::from(e)),
            }
        }
    }
}

pub async fn relay_connections<
    D: AsyncRead + AsyncWrite + Sized + Send + Unpin + 'static,
    U: AsyncRead + AsyncWrite + Sized + Send + 'static,
>(
    client: D,
    target: U,
    ctx: TunnelCtx,
    downstream_relay_policy: RelayPolicy,
    upstream_relay_policy: RelayPolicy,
) -> io::Result<TunnelStats> {
    let (client_recv, client_send) = io::split(client);
    let (target_recv, target_send) = io::split(target);

    let downstream_relay: Relay = RelayBuilder::default()
        .name("Downstream")
        .tunnel_ctx(ctx)
        .relay_policy(downstream_relay_policy)
        .build()
        .expect("RepayBuilder failed");
    
    let upstream_relay: Relay = RelayBuilder::default()
        .name("Upstream")
        .tunnel_ctx(ctx)
        .relay_policy(upstream_relay_policy)
        .build()
        .expect("RelayBuilder failed");
    
    let upstream_task = 
        tokio::spawn(async move { downstream_relay.relay_data(client_recv, target_send).await });
    
    let downstream_task = 
        tokio::spawn(async move { upstream_relay.relay_data(target_recv, client_send).await });
    
    // TODO: What's ??;
    let downstream_stats = downstream_task.await??;
    let upstream_stats = upstream_task.await??;

    Ok(TunnelStats {
        tunnel_ctx: ctx,
        result: EstablishTunnelResult::Ok,
        upstream_stats: Some(upstream_stats),
        downstream_stats: Some(downstream_stats),
    })
}
