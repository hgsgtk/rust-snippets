#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate serde_derive;

// > そして lib.rs の中で以下のようにmodで参照してあげれば使えます。
// https://keens.github.io/blog/2018/12/08/rustnomoju_runotsukaikata_2018_editionhan/
mod configuration;
mod relay;

// tokio: Tokio is an asynchronous runtime for the Rust programming language. It provides the building blocks needed for writing networking applications
// https://tokio.rs/tokio/tutorial/hello-tokio
use tokio::io;

// TODO: solve compile error
// could not find `configuration` in the crate root
use crate::configuration::{ProxyConfiguration};


// log: A lightweight logging facade for Rust
// https://crates.io/crates/log
use log::{LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};
use log4rs::Config;

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