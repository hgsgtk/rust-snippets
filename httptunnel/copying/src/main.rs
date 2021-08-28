#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate serde_derive;

/// > そして lib.rs の中で以下のようにmodで参照してあげれば使えます。
/// https://keens.github.io/blog/2018/12/08/rustnomoju_runotsukaikata_2018_editionhan/
mod configuration;
mod relay;
mod proxy_target;
mod tunnel;
mod http_tunnel_codec;

/// tokio: Tokio is an asynchronous runtime for the Rust programming language. It provides the building blocks needed for writing networking applications
/// https://tokio.rs/tokio/tutorial/hello-tokio
use tokio::io;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;

/// Without `mod {filename}`, we got an error: could not find `configuration` in the crate root
use crate::configuration::{ProxyConfiguration, ProxyMode};
use crate::proxy_target::SimpleCachingDnsResolver;
use crate::tunnel::{
    TunnelCtxBuilder
};
use crate::http_tunnel_codec::{HttpTunnelCodec, HttpTunnelCodecBuilder};

/// log: A lightweight logging facade for Rust
/// https://crates.io/crates/log
use log::{error, info, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};
use log4rs::Config;

use rand::{thread_rng, Rng};

/// async fn tunnel_stream<C: AsyncRead + AsyncWrite + Send + Unpin + 'static>(

type DnsResolver = SimpleCachingDnsResolver;

#[tokio::main]
pub async fn main() -> io::Result<()> {
    init_logger();

    let proxy_configuration = ProxyConfiguration::from_command_line().map_err(|e| {
        println!("Failed to process parameters. See ./log/application.log for details");
        e
    })?;
    // Tips for deadcode
    // > warning: unused variable: `proxy_configuration`
    // > help: if this is intentional, prefix it with an underscore: `_proxy_configuration`

    info!("Starting listener on : {}", proxy_configuration.bind_address);

    // TcpListener: An I/O object representing a TCP socket listening for incoming connections.
    // https://docs.rs/tokio/0.1.22/tokio/net/struct.TcpListener.html
    let mut tcp_listener = TcpListener::bind(&proxy_configuration.bind_address)
        .await
        .map_err(|e| {
            error!(
                "Error binding address {} {}",
                &proxy_configuration.bind_address, e
            );
            e
        })?;

    let dns_resolver = SimpleCachingDnsResolver::new(
        proxy_configuration
            .tunnel_config
            .target_connection
            .dns_cache_ttl,
    );

    match &proxy_configuration.mode {
        ProxyMode::HTTP => {
            // about .await https://rust-lang.github.io/async-book/01_getting_started/04_async_await_primer.html
            serve_plain_text(proxy_configuration, &mut tcp_listener, dns_resolver).await?;
        }
        // TODO: HOW to bind value to tls_identity?
        ProxyMode::HTTPS(tls_identity) => {
            // TODO: HTTPS        
        }
        ProxyMode::TCP(d) => {
            // TODO: TCP
        }
    }

    info!("Proxy stopped");

    // Contains the success value
    // https://doc.rust-lang.org/std/result/enum.Result.html#variant.Ok
    Ok(())
}

fn init_logger() {
    // let: bind a value to variable 
    // https://doc.rust-lang.org/std/keyword.let.html
    let logger_configuration = "./config/log4rs.yaml";
    //  Err(E) Contains the error value
    // https://doc.rust-lang.org/std/result/enum.Result.html#variant.Err
    // log4rs: log4rs is a highly configurable logging framework modeled after Java's Logback and log4j libraries.
    // https://docs.rs/log4rs/1.0.0/log4rs
    // Trait std::default::Default
    // https://doc.rust-lang.org/std/default/trait.Default.html
    if let Err(e) = log4rs::init_file(logger_configuration, Default::default()) {
        // https://doc.rust-lang.org/std/macro.println.html
        println!(
            "Cannot initialize logger from {}, error=[{}]. Logging to the console.",
            logger_configuration, e
        );

        let config = Config::builder()
            .appender(
                Appender::builder()
                    // use std::boxed::Box; A pointer type for heap allocation.
                    // https://doc.rust-lang.org/std/boxed/struct.Box.html
                    // log4rs::append::console::ConsoleAppender; An appender which logs to standard out.
                    // https://docs.rs/log4rs/0.8.3/log4rs/append/console/struct.ConsoleAppender.html
                    .build("application", Box::new(ConsoleAppender::builder().build()))
            )
            .build(
                Root::builder()
                    .appender("application")
                    // LevelFilter
                    // https://docs.rs/log/0.4.0/log/enum.LevelFilter.html
                    .build(LevelFilter::Info)
            )
            .unwrap();
        log4rs::init_config(config).expect("Bug: bad default config");
    }
}

/// The () type called unit.
/// > The () type has exactly one value (), and is used when there is no other meaningful value that could be returned. 
/// https://doc.rust-lang.org/std/primitive.unit.html
async fn serve_plain_text(
    config: ProxyConfiguration,
    listener: &mut TcpListener,
    dns_resolver: DnsResolver,
) -> io::Result<()> {
    info!("Serving requests on: {}", config.bind_address);
    loop {
        // pub async fn accept(&self) -> Result<(TcpStream, SocketAddr)>
        // > Accepts a new incoming connection from this listener.
        // TCP Handshake here.
        // https://docs.rs/tokio/1.10.1/tokio/net/struct.TcpListener.html
        let socket = listener.accept().await;

        // Clone trait defines clone().
        // A common trait for the ability to explicitly duplicate an object
        // https://doc.rust-lang.org/std/clone/trait.Clone.html
        let dns_resolver_ref = dns_resolver.clone();

        match socket {
            Ok((stream, _)) => {
                // pub fn nodelay(&self) -> Result<bool>
                // Gets the value of the TCP_NODELAY option on this socket.
                // nodelay => disables the Nagle algorithm.
                // https://doc.rust-lang.org/std/net/struct.TcpStream.html#method.nodelay
                // Detail about Nagle algorithm at https://doc.rust-lang.org/std/net/struct.TcpStream.html#method.set_nodelay
                //
                // unwrap_or_default: Returns the contained Some value or a default
                // https://doc.rust-lang.org/std/option/enum.Option.html#method.unwrap_or_default
                stream.nodelay().unwrap_or_default();
                let config = config.clone();
                // handle accepted connnections asynchronously
                //
                // Function tokio::spawn: Spawns a new asynchronous task, returning a JoinHandle for it
                // > Spawning a task enables the task to execute concurrently to other tasks
                // https://docs.rs/tokio/0.2.2/tokio/fn.spawn.html
                //
                // Keyword `move` 
                // https://doc.rust-lang.org/std/keyword.move.html
                // > move converts any variables captured by reference or mutable reference to variables captured by value.
                tokio::spawn(async move { tunnel_stream(&config, stream, dns_resolver_ref).await });
            }
            Err(e) => error!("Failed TCP handshake{}", e)
        }
    }
}

/// tokio::AsyncRead/AsyncWrite https://docs.rs/tokio/1.10.1/tokio/io/trait.AsyncWrite.html
/// Writes bytes asynchronously.
/// > The trait inherits from std::io::Write and indicates that an I/O object is nonblocking. 
/// > All non-blocking I/O objects must return an error when bytes cannot be written instead of blocking the current thread.
//
/// Send https://doc.rust-lang.org/std/marker/trait.Send.html
/// > This trait is automatically implemented when the compiler determines it’s appropriate.
/// https://doc.rust-lang.org/nomicon/send-and-sync.html
/// > A type is Send if it is safe to send it to another thread.
/// > A type is Sync if it is safe to share between threads (T is Sync if and only if &T is Send).
//
/// Thread safe
/// https://ja.wikipedia.org/wiki/%E3%82%B9%E3%83%AC%E3%83%83%E3%83%89%E3%82%BB%E3%83%BC%E3%83%95
/// > - マルチスレッドプログラミングにおける概念
/// > - あるコードがスレッドセーフであるという場合、そのコードを複数のスレッドが同時並行的に実行しても問題が発生しないことを意味する
/// > - 特に、ある共有データへの複数のスレッドによるアクセスがあるとき、一度に1つのスレッドのみがその共有データにアクセスするようにして安全性を確保しなければならない
/// Related pricipal 驚き最小の原則 Principle of least astonishment / Rule of least surprise
/// > ユーザインタフェースやプログラミング言語の設計および人間工学において、
/// > インタフェースの2つの要素が互いに矛盾あるいは不明瞭だったときに、
/// > その動作としては人間のユーザやプログラマが最も自然に思える（驚きが少ない）ものを選択すべきだ
/// https://ja.wikipedia.org/wiki/%E9%A9%9A%E3%81%8D%E6%9C%80%E5%B0%8F%E3%81%AE%E5%8E%9F%E5%89%87
/// 
/// Unpin https://doc.rust-lang.org/std/marker/trait.Unpin.html
/// Types that can be safely moved after being pinned.
/// The Pin type is used instead to prevent moves through the type system. 
/// Related module: std::pin: Types that pin data to its location in memory
/// https://doc.rust-lang.org/std/pin/index.html
/// JA RustのPinチョットワカル https://tech-blog.optim.co.jp/entry/2020/03/05/160000
/// > このUnpinトレイトは自動トレイト*1として宣言されており、基本的にはあらゆる型に実装されます。 
/// > それもそのはず、普通にコードを書いていて「ムーブしたら絶対アカン😡型」なんてものは出てこないからです。
/// 
/// 'static
/// > As a reference lifetime 'static indicates that the data pointed to by the reference lives for the entire lifetime of the running program.
/// > It can still be coerced to a shorter lifetime.
/// https://doc.rust-lang.org/rust-by-example/scope/lifetime/static_lifetime.html
/// In this case, it's trait bound.
/// > As a trait bound, it means the type does not contain any non-static references.
//
/// Trait object
/// > A trait object is an opaque value of another type that implements a set of traits.
/// > Trait objects implement the base trait, its auto traits, and any super traits of the base trait.
/// > Trait objects are written as the path to the base trait followed by the list of auto traits
/// >  followed optionally by a lifetime bound all separated by +.
/// ex. Trait, Trait + Send, Trait + Send + Sync, Trait + 'static ...etc
/// http://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/reference/types.html#trait-objects
async fn tunnel_stream<C: AsyncRead + AsyncWrite + Send + Unpin + 'static>(
    config: &ProxyConfiguration,
    client: C,
    dns_resolver: DnsResolver,
) -> io::Result<()> {
    let ctx = TunnelCtxBuilder::default()
        // thread_rng https://docs.rs/rand/0.6.2/rand/fn.thread_rng.html
        // > Retrieve the lazily-initialized thread-local random number generator, seeded by the system
        // performance benchmark https://qiita.com/hhatto/items/c1f311eb80280c26b7e8
        // We got an error withdout Rng trait, because Rng trait defined get()
        // > https://docs.rs/rand/0.5.0/rand/trait.Rng.html
        .id(thread_rng().gen::<u128>())
        .build()
        .expect("TunnelCtxBuilder failed");

    let codec: HttpTunnelCodec = HttpTunnelCodecBuilder::default()
        .tunnel_ctx(ctx)
        .enabled_targets(
            config 
                .tunnel_config
                .target_connection
                .allowed_targets
                .clone(),
        )
        .build()
        .expect("HttpTunnelCodecBuilder failed");

    Ok(())
}