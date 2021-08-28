use crate::relay::{RelayPolicy, NO_BANDWIDTH_LIMIT, NO_TIMEOUT};

use clap::clap_app;
use log::{info, error};
/// rust-native-tls: An abstraction over platform-specific TLS implementations.
/// https://crates.io/crates/native-tls
use native_tls::Identity;
use regex::Regex;
/// A reference to an open file on the filesystem.
/// https://doc.rust-lang.org/std/fs/struct.File.html
use std::fs::File;
/// Withoud Read, we got an compile errors.
/// Error: file.read_to_end(&mut identity).map_err(|e| { -> ^^^^^^^^^^^ method not found in `std::fs::File`
/// https://users.rust-lang.org/t/why-call-read-read-to-end-gives-method-not-found-but-io-copy-works/40021/9
/// To solve it, we should use Read trait `use std::io::Read;`
/// https://qiita.com/fujitayy/items/12a80560a356607da637
use std::io::{Error, ErrorKind, Read};
/// A Duration type to represent a span of time, typically used for system timeouts.
/// https://doc.rust-lang.org/stable/std/time/struct.Duration.html
use std::time::Duration;
use tokio::io;

/// Attribute: #[attribute(value)]
/// https://doc.rust-lang.org/rust-by-example/attribute.html
/// Enum JA: 列挙型
/// https://doc.rust-jp.rs/rust-by-example-ja/custom_types/enum.html
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
    pub relay_policy: RelayPolicy,
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
    pub relay_policy: RelayPolicy,
}

/// serde::Deserialize
/// https://dev.classmethod.jp/articles/rust-serde-getting-started/
#[derive(Deserialize, Clone)]
pub struct TunnelConfig {
    pub client_connection: ClientConnectionConfig,
    pub target_connection: TargetConnectionConfig,
}

/// JA: コンパイラには、[#derive]アトリビュートを用いることで型に対して特定のトレイトの標準的な実装を提供する機能があります。
/// https://doc.rust-jp.rs/rust-by-example-ja/trait/derive.html
/// Clone: JA これはコピーによって&TからTを作成するトレイト
/// https://doc.rust-lang.org/std/clone/trait.Clone.html
/// create derive_builder;
/// builder setters automatically
/// https://docs.rs/derive_builder/0.10.2/derive_builder/
/// struct
/// JP: 構造体
/// - タプル。（ほとんどの場合は名前付きタプル）
/// - クラシックなC言語スタイルの構造体。 <- Y
/// - ユニット。これはフィールドを持たず、ジェネリック型を扱う際に有効です。
/// https://doc.rust-jp.rs/rust-by-example-ja/custom_types/structs.html
#[derive(Clone, Builder)]
pub struct ProxyConfiguration {
    pub mode: ProxyMode,
    pub bind_address: String,
    pub tunnel_config: TunnelConfig,
}

/// Implement some functionality for a type.
impl Default for TunnelConfig {
    fn default() -> Self {
        // Self is not same as self.
        // https://doc.rust-lang.org/std/keyword.self.html
        Self {
            client_connection: ClientConnectionConfig {
                initiation_timeout: NO_TIMEOUT,
                relay_policy: RelayPolicy {
                    idle_timeout: NO_TIMEOUT,
                    min_rate_bpm: 0,
                    max_rate_bpm: NO_BANDWIDTH_LIMIT,
                },
            },
            target_connection: TargetConnectionConfig {
                dns_cache_ttl: NO_TIMEOUT,
                allowed_targets: Regex::new(".*").expect("Bug: bad default regexp"),
                connect_timeout: NO_TIMEOUT,
                relay_policy: RelayPolicy {
                    idle_timeout: NO_TIMEOUT,
                    min_rate_bpm: 0,
                    max_rate_bpm: NO_BANDWIDTH_LIMIT,
                },
            },
        }
    }
}

/// impl keyword
/// Implement some functionality for a type.
/// https://doc.rust-lang.org/std/keyword.impl.html
impl ProxyConfiguration {
    pub fn from_command_line() -> io::Result<ProxyConfiguration> {
        // Crate clap: clap is a simple-to-use, efficient, and full-featured library for parsing command line arguments and subcommands when writing console/terminal applications.
        // https://docs.rs/clap/2.22.2/clap/index.html
        // clap_app! macro https://github.com/clap-rs/clap/issues/1347
        //         $ ./target/debug/copying --help
        // 
        // ==================================
        // Copied simple HTTP(S) Tunnel 0.0.1
        //
        // Kazuki Higashiguchi
        //
        // A simple HTTP(S) tunnel
        //
        // USAGE:
        //      copying [OPTIONS] --bind <BIND> [SUBCOMMAND]
        //
        // FLAGS:
        //      -h, --help       Print help information
        //      -V, --version    Print version information
        //
        // OPTIONS:
        //      --bind <BIND>        Bind address, e.g. 0.0.0.0:8443
        //      --config <CONFIG>    Configuration file
        //
        // SUBCOMMANDS:
        //      help     Print this message or the help of the given subcommand(s)
        //      http     Run the tunnel in HTTP mode
        //      https    Run the tunnel in HTTPS mode
        //      tcp      Run the tunnel in TCP proxy mode
        // ==================================
        let matches = clap_app!(myapp => 
            (name: "Copied simple HTTP(S) Tunnel")
            (version: "0.0.1")
            (author: "Kazuki Higashiguchi")
            (about: "A simple HTTP(S) tunnel")
            // @: Pattern binding
            // https://doc.rust-lang.org/book/appendix-02-operators.html
            (@arg CONFIG: --config +takes_value "Configuration file")
            (@arg BIND: --bind +required +takes_value "Bind address, e.g. 0.0.0.0:8443")
            (@subcommand http =>
                (about: "Run the tunnel in HTTP mode")
                (version: "0.0.1")
            )
            (@subcommand https =>
                (about: "Run the tunnel in HTTPS mode")
                (version: "0.0.1")
                // PKCS12 https://en.wikipedia.org/wiki/PKCS_12
                // > PKCS #12 defines an archive file format for storing many cryptography objects as a single file
                // https://qiita.com/kunichiko/items/3e2ec27928a95630a73a
                (@arg PKCS12: --pk +required +takes_value "pkcs12 filename")
                (@arg PASSWORD: --password +required +takes_value "Password for the pkcs12 file")
            )
            (@subcommand tcp => 
                (about: "Run the tunnel in TCP proxy mode")
                (version: "0.0.1")
                (@arg DESTINATION: --destination -d +required +takes_value "Destination address, e.g. 10.0.0.2:8443")
            )
        )
        .get_matches();

        // We can get value by API value_of()
        // https://docs.rs/clap/2.33.3/clap/struct.ArgMatches.html
        let config = matches.value_of("CONFIG");

        let bind_address = matches
            .value_of("BIND")
            .expect("misconfiguration for bind")
            .to_string();

        // subcommand_matches()
        // > This method returns the ArgMatches for a particular subcommand or None if the subcommand wasn't present at runtime.
        // https://docs.rs/clap/2.33.3/clap/struct.ArgMatches.html#method.subcommand_matches
        // Argmatches is Option type, which has is_some().
        // > Returns true if the option is a Some value.
        // https://doc.rust-lang.org/beta/core/option/enum.Option.html
        let mode = if matches.subcommand_matches("http").is_some() {
            // If user has a subcommand that uses http mode
            // Crate info!
            // https://qiita.com/fujitayy/items/590145c0f4b4e7d06de7
            info!(
                "Starting in HTTP mode: bind: {}, configuration: {:?}",
                bind_address, config
            );
            ProxyMode::HTTP
        } else if let Some(https) = matches.subcommand_matches("https") {
            let pkcs12_file = https
                .value_of("PKCS12")
                .expect("misconfiguration for pkcs12");
            let password = https
                .value_of("PASSWORD")
                .expect("misconfiguration for password");
            
            // ?: Error propagation
            // https://doc.rust-lang.org/book/appendix-02-operators.html
            let identity = ProxyConfiguration::tls_identify_from_file(pkcs12_file, password)?;
            info!(
                "Starting in HTTPS mode: pkcs12: {}, password: {}, bind: {}, configuration: {:?}",
                pkcs12_file,
                // !: Bitwise or logical complement (in short, "NOT")
                !password.is_empty(),
                bind_address,
                config
            );
            ProxyMode::HTTPS(identity)
        } else if let Some(tcp) = matches.subcommand_matches("tcp") {
            let destination = tcp
                .value_of("DESTINATION")
                .expect("misconfiguration for destination")
                .to_string();
            info!(
                "Starting in TCP mode: destination: {}, configuration: {:?}",
                destination, config
            );
            ProxyMode::TCP(destination)
        } else {
            // Indicates unreachable code.
            // This will always panic!.
            // https://doc.rust-lang.org/std/macro.unreachable.html
            // I send nits improvement PR :) https://github.com/xnuter/http-tunnel/pull/8 .
            unreachable!("only http, https and tcp commands are supported")
        };

        // The match Control Flow Operator
        // https://doc.rust-lang.org/book/ch06-02-match.html
        let tunnel_config = match config {
            // TODO: add default configuration
            None => TunnelConfig::default(), 
            Some(config) => ProxyConfiguration::read_tunnel_config(config)?,
            // Without no None, the following error occured.
            // > error[E0004]: non-exhaustive patterns: `None` not covered
            // Just `None`, got following error
            // > error: struct literals are not allowed here
        };
        
        // derive_builder allows us to build structs Builder pattern.
        // https://docs.rs/derive_builder/0.10.2/derive_builder/#builder-patterns
        // Without calling build(), we got the following error.
        // > expected struct `ProxyConfiguration`, found struct `ProxyConfigurationBuilder`
        Ok(ProxyConfigurationBuilder::default()
            .bind_address(bind_address)
            .mode(mode)
            .tunnel_config(tunnel_config)
            // Withoug any binding, we got an error at runtime.
            // > thread 'main' panicked at 'ProxyConfigurationBuilder failed: "`mode` must be initialized"', src/configuration.rs:108:14
            .build()
            .expect("ProxyConfigurationBuilder failed"))
    }


    /// Result<T,E> is for handling recoverable error
    /// https://doc.rust-lang.org/std/result/enum.Result.html
    /// https://speakerdeck.com/tanden/phpdethrowsinaili-wai-handoringu?slide=29
    /// std::io::Result; A specialized Result type for I/O operations.
    /// https://doc.rust-lang.org/std/io/type.Result.html
    fn tls_identify_from_file(filename: &str, password: &str) -> io::Result<Identity> {
        // open -> Result<File>
        // https://doc.rust-lang.org/std/fs/struct.File.html#method.open
        // map_err: 
        // https://doc.rust-lang.org/std/result/enum.Result.html#method.map_err
        // mut keyword: JP 変数宣言のmutable
        // https://qiita.com/hiro4669/items/1eea8c6443e7b533ea03
        // ?; A Shortcut for Propagating Errors: the ? Operator
        // The ? operator eliminates a lot of boilerplate and makes this function’s implementation simpler.
        // https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#a-shortcut-for-propagating-errors-the--operator
        let mut file = File::open(filename).map_err(|e| {
            error!("Error opening PKSC12 file {}: {}", filename, e);
            e
        })?;

        // The vec! macro is provided to make initialization more convenient:
        // https://doc.rust-lang.org/std/vec/struct.Vec.html
        let mut identity = vec![];

        // read_to_end: Read all bytes until EOF in this source, placing them into buf
        // https://doc.rust-lang.org/std/io/trait.Read.html#method.read_to_end
        //
        // &mut: it becomes a mutable reference and can be written.
        // https://qiita.com/cactaceae/items/2c70a9947364c60ec100
        file.read_to_end(&mut identity).map_err(|e| {
            error!("Error reading file {}: {}", filename, e);
            e
        })?;

        Identity::from_pkcs12(&identity, &password).map_err(|e| {
            error!("Cannot process PKCS12 file {}: {}", filename, e);
            // ErrorKind: A list specifying general categories of I/O error.
            // InvalidInput: A parameter was incorrect.
            // https://doc.rust-lang.org/std/io/enum.ErrorKind.html
            Error::from(ErrorKind::InvalidInput)
        })
    }

    fn read_tunnel_config(filename: &str) -> io::Result<TunnelConfig> {
        let mut file = File::open(filename).map_err(|e| {
            error!("Error opening config file {}: {}", filename, e);
            e
        })?;

        let mut yaml = vec![];

        file.read_to_end(&mut yaml).map_err(|e| {
            error!("Error reading file {}: {}", filename, e);
            e
        })?;

        let result: TunnelConfig = serde_yaml::from_slice(&yaml).map_err(|e| {
            error!("Error parsing yaml {}: {}", filename, e);
            Error::from(ErrorKind::InvalidInput)
        })?;

        // Fake it
        // https://doc.rust-jp.rs/book-ja/ch05-01-defining-structs.html
        // let tunnel_config = TunnelConfig {
        //     client_connection: ClientConnectionConfig {
        //         // ex. Duration::new(10, 0) means 10 seconds
        //         // https://doc.rust-lang.org/stable/std/time/struct.Duration.html
        //         initiation_timeout: Duration::new(10, 0)
        //         relay_policy: RelayPolicy {
        //             idle_timeout: Duration::new(30, 0),
        //             min_rate_bpm: 1000,
        //             max_rate_bpm: 10000,
        //         }
        //     },
        //     target_connection: TargetConnectionConfig {
        //         dns_cache_ttl: Duration::new(60, 0),
        //         allowed_targets: "^(?i)([a-z]+)\\.(wikipedia|rust-lang)\\.org:443$",
        //         connect_timeout: Duration::new(10, 0)
        //         relay_policy: RelayPolicy {
        //             idle_timeout: Duration::new(30, 0),
        //             min_rate_bpm: 1000,
        //             max_rate_bpm: 10000,
        //         }
        //     }
        // }
        Ok(result)
    }
}