// tokio: Tokio is an asynchronous runtime for the Rust programming language. It provides the building blocks needed for writing networking applications
// https://tokio.rs/tokio/tutorial/hello-tokio
use tokio::io;

#[tokio::main]
pub async fn main() -> io::Result<()> {
    println!("Hello, world!");
    // Contains the success value
    // https://doc.rust-lang.org/std/result/enum.Result.html#variant.Ok
    Ok(())
}
