#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate serde_derive;

// > そして lib.rs の中で以下のようにmodで参照してあげれば使えます。
// https://keens.github.io/blog/2018/12/08/rustnomoju_runotsukaikata_2018_editionhan/
mod configuration;
mod relay;
mod proxy_target;

// tokio: Tokio is an asynchronous runtime for the Rust programming language. It provides the building blocks needed for writing networking applications
// https://tokio.rs/tokio/tutorial/hello-tokio
use tokio::io;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;

// Without `mod {filename}`, we got an error: could not find `configuration` in the crate root
use crate::configuration::{ProxyConfiguration, ProxyMode};
use crate::proxy_target::SimpleCachingDnsResolver;

// log: A lightweight logging facade for Rust
// https://crates.io/crates/log
use log::{error, info, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};
use log4rs::Config;

// async fn tunnel_stream<C: AsyncRead + AsyncWrite + Send + Unpin + 'static>(

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

// The () type called unit.
// > The () type has exactly one value (), and is used when there is no other meaningful value that could be returned. 
// https://doc.rust-lang.org/std/primitive.unit.html
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

// TODO: what's this?
// tokio::AsyncRead/AsyncWrite
//
// Send
//
// Unpin
//
// 'static
async fn tunnel_stream<C: AsyncRead + AsyncWrite + Send + Unpin + 'static>(
    config: &ProxyConfiguration,
    client: C,
    dns_resolver: DnsResolver,
) -> io::Result<()> {
    Ok(())
}