
// rust-native-tls: An abstraction over platform-specific TLS implementations.
// https://crates.io/crates/native-tls
use native_tls::Identity;
use regex::Regex;
// A Duration type to represent a span of time, typically used for system timeouts.
// https://doc.rust-lang.org/stable/std/time/struct.Duration.html
use std::time::Duration;
use tokio::io;

// Attribute: #[attribute(value)]
// https://doc.rust-lang.org/rust-by-example/attribute.html
// Enum JA: 列挙型
// https://doc.rust-jp.rs/rust-by-example-ja/custom_types/enum.html
#[derive(Clone)]
pub enum ProxyMode {
    HTTP,
    // HTTPS(Identity) says that is will have associated `Identity` value.
    // https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html
    // Identity:
    // > An identity is an X509 certificate along with its corresponding private key and chain of certificates to a trusted root.
    // https://docs.rs/native-tls/0.2.8/native_tls/struct.Identity.html
    HTTPS(Identity),
    // use std::string::String;
    // You can create a String from a literal string with String::from:
    // https://doc.rust-lang.org/std/string/struct.String.html
    TCP(String),
}

#[derive(Deserialize, Clone)]
pub struct ClientConnectionConfig {
    // When you want to serialize with specific serializaion loginc, you should use #[serde(with = "xx")] quotes.
    // https://github.com/serde-rs/serde
    // Serde support for the humantime crate.
    // https://docs.rs/humantime-serde/1.0.1/humantime_serde/
    // Currently std::time::{Duration, SystemTime} are supported.
    // That's why humantime_serde crates is used to selialize.
    //
    // Other references
    // https://qiita.com/legokichi/items/2a72ae89ef76f6252166
    // about serde_with quates
    // https://qiita.com/garkimasera/items/9f70967f58a2c1f0886d
    #[serde(with = "humantime_serde")]
    pub initiation_timeout: Duration,
    // TODO: add configuration to set relay policy
    // pub relay_policy: RelayPolicy,
}

#[derive(Deserialize, Clone)]
pub struct TargetConnectionConfig {
    #[serde(with = "humantime_serde")]
    pub dns_cache_ttl: Duration,
    // Crate serde_regex: A (de)serializer for regex::Regex
    // https://docs.rs/serde_regex/0.2.0/serde_regex/index.html
    // Crate regex: This crate provides a library for parsing, compiling, and executing regular expressions. 
    // https://docs.rs/regex/1.5.4/regex/
    #[serde(with = "serde_regex")]
    pub allowed_targets: Regex,
    #[serde(with = "humantime_serde")]
    pub connect_timeout: Duration,
    // TODO: add configuration to set relay policy
    // pub relay_policy: RelayPolicy,
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

// impl keyword
// Implement some functionality for a type.
// https://doc.rust-lang.org/std/keyword.impl.html
impl ProxyConfiguration {
    pub fn from_command_line() -> io::Result<ProxyConfiguration> {
        // TODO: command line
        
        // derive_builder allows us to build structs Builder pattern.
        // https://docs.rs/derive_builder/0.10.2/derive_builder/#builder-patterns
        // Without calling build(), we got the following error.
        // > expected struct `ProxyConfiguration`, found struct `ProxyConfigurationBuilder`
        Ok(ProxyConfigurationBuilder::default()
            // TODO set some values
            .build()
            .expect("ProxyConfigurationBuilder failed"))
    }
}