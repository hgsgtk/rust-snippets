
// rust-native-tls: An abstraction over platform-specific TLS implementations.
// https://crates.io/crates/native-tls
use native_tls::Identiy;

// Enum JA: 列挙型
// https://doc.rust-jp.rs/rust-by-example-ja/custom_types/enum.html
#[derive(Clone)]
pub enum ProxyMode {
    HTTP,
    HTTPS(Identiy),
    // use std::string::String;
    // You can create a String from a literal string with String::from:
    // https://doc.rust-lang.org/std/string/struct.String.html
    TCP(String),
}

#[derive(Deserialize, Clone)]
pub struct ClientConnectionConfig {
    // TODO: I'll resume from here.
    #[serde(with = "humantime_serde")]
    pub initiation_timeout: Duration,
    pub relay_policy: RelayPolicy,
}

// serde::Deserialize
// https://dev.classmethod.jp/articles/rust-serde-getting-started/
#[derive(Deserialize, Clone)]
pub struct TunnelConfig {
    pub client_connection: ClientConnectionConfig,
    pub target_connection: TargetConnectionConfig,
}

// JA: コンパイラには、[#derive]アトリビュートを用いることで型に対して特定のトレイトの標準的な実装を提供する機能があります。
// https://doc.rust-jp.rs/rust-by-example-ja/trait/derive.html
// Clone: JA これはコピーによって&TからTを作成するトレイト
// https://doc.rust-lang.org/std/clone/trait.Clone.html
// create derive_builder;
// builder setters automatically
// https://docs.rs/derive_builder/0.10.2/derive_builder/
// struct
// JP: 構造体
// - タプル。（ほとんどの場合は名前付きタプル）
// - クラシックなC言語スタイルの構造体。 <- Y
// - ユニット。これはフィールドを持たず、ジェネリック型を扱う際に有効です。
// https://doc.rust-jp.rs/rust-by-example-ja/custom_types/structs.html
#[derive(Clone, Builder)]
pub struct ProxyConfiguration {
    pub mode: ProxyMode,
    pub bind_address: String,
    pub tunnel_config: TunnelConfig,
}