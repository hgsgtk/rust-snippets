// tokio: Tokio is an asynchronous runtime for the Rust programming language. It provides the building blocks needed for writing networking applications
// https://tokio.rs/tokio/tutorial/hello-tokio
use tokio::io;

// log: A lightweight logging facade for Rust
// https://crates.io/crates/log
use log::{LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};
use log4rs::Config;

#[tokio::main]
pub async fn main() -> io::Result<()> {
    init_logger();

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