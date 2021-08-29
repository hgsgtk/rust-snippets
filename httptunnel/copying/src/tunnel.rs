use crate::configuration::TunnelConfig;

/// trait std::default::Default https://doc.rust-lang.org/std/default/trait.Default.html
/// A trait for giving a type a useful default value
#[derive(Builder, Copy, Clone, Default, Serialize)]
pub struct TunnelCtx {
    id: u128,
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

impl<H, C, T> ConnectionTunnel<H, C, T>
where
    // TODO: What's this?
    H: Decoder<Error = EstablishTunnelResult> + Encoder<EstablishTunnelResult>,
    H::Item: TunnelTarget + Sized + Display + Send + Sync,
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
            // TODO: Some
            tunnel_request_codec: Some(handshake_codec),
            target_connector,
            tunnel_ctx,
            client: Some(client),
            tunnel_config,
        }
    }
}