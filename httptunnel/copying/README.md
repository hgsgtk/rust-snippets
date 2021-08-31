# Copying of a sutra (http tunnel)

ref: https://github.com/xnuter/http-tunnel
ref: https://medium.com/swlh/writing-a-modern-http-s-tunnel-in-rust-56e70d898700

- http mode

```
./target/debug/copying --config ./config/config.yml --bind 0.0.0.0:8443 http
```

# Refs

## Initialize cargo app

- new package

https://doc.rust-lang.org/cargo/getting-started/first-steps.html

```
$ cargo new copying
     Created binary (application) `copying` package
```

- build it

```
$ cargo build
   Compiling copying v0.1.0 (/Users/{user}/rust-snippets/httptunnel/copying)
    Finished dev [unoptimized + debuginfo] target(s) in 0.99s
```

- execute it

```
$ ./target/debug/copying 
Hello, world!
```

- tun

```
$ cargo run
```
