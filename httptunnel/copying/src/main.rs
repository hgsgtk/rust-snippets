// tokio: Tokio is an asynchronous runtime for the Rust programming language. It provides the building blocks needed for writing networking applications
// https://tokio.rs/tokio/tutorial/hello-tokio
use tokio::io;

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
    }
}