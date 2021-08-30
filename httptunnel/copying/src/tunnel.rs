use async_trait::async_trait;

use crate::configuration::TunnelConfig;
use crate::proxy_target::{Nugget, TargetConnector};

use core::fmt;

use std::fmt::Display;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{Decoder, Encoder};

/// trait std::default::Default https://doc.rust-lang.org/std/default/trait.Default.html
/// A trait for giving a type a useful default value
#[derive(Builder, Copy, Clone, Default, Serialize)]
pub struct TunnelCtx {
    id: u128,
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
}