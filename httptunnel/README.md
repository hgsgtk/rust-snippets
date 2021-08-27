# HTTP Tunnel in Rust

ref: https://medium.com/swlh/writing-a-modern-http-s-tunnel-in-rust-56e70d898700

> Simply put, it’s a lightweight VPN that you can set up with your browser so your Internet provider cannot block or track your activity, and web-servers won’t see your IP address.

## Install Rust on MacOS

https://doc.rust-lang.org/cargo/getting-started/installation.html

```
$ curl https://sh.rustup.rs -sSf | sh

info: downloading installer

Welcome to Rust!

This will download and install the official compiler for the Rust
programming language, and its package manager, Cargo.

Rustup metadata and toolchains will be installed into the Rustup
home directory, located at:

  /Users/{user}/.rustup

This can be modified with the RUSTUP_HOME environment variable.

The Cargo home directory located at:

  /Users/{user}/.cargo

This can be modified with the CARGO_HOME environment variable.

The cargo, rustc, rustup and other commands will be added to
Cargo's bin directory, located at:

  /Users/{user}/.cargo/bin

This path will then be added to your PATH environment variable by
modifying the profile files located at:

  /Users/{user}/.profile
  /Users/{user}/.bashrc
  /Users/{user}/.zshenv

You can uninstall at any time with rustup self uninstall and
these changes will be reverted.

Current installation options:


   default host triple: x86_64-apple-darwin
     default toolchain: stable (default)
               profile: default
  modify PATH variable: yes

1) Proceed with installation (default)
2) Customize installation
3) Cancel installation
>1

info: profile set to 'default'
info: default host triple is x86_64-apple-darwin
info: syncing channel updates for 'stable-x86_64-apple-darwin'
info: latest update on 2021-07-29, rust version 1.54.0 (a178d0322 2021-07-26)
info: downloading component 'cargo'
info: downloading component 'clippy'
info: downloading component 'rust-docs'
 16.8 MiB /  16.8 MiB (100 %)  15.8 MiB/s in  1s ETA:  0s
info: downloading component 'rust-std'
 20.6 MiB /  20.6 MiB (100 %)  14.9 MiB/s in  1s ETA:  0s
info: downloading component 'rustc'
 75.0 MiB /  75.0 MiB (100 %)  13.8 MiB/s in  5s ETA:  0s
info: downloading component 'rustfmt'
info: installing component 'cargo'
info: installing component 'clippy'
info: installing component 'rust-docs'
 16.8 MiB /  16.8 MiB (100 %)   5.2 MiB/s in  3s ETA:  0s
info: installing component 'rust-std'
 20.6 MiB /  20.6 MiB (100 %)  13.7 MiB/s in  1s ETA:  0s
info: installing component 'rustc'
 75.0 MiB /  75.0 MiB (100 %)  14.2 MiB/s in  5s ETA:  0s
info: installing component 'rustfmt'
info: default toolchain set to 'stable-x86_64-apple-darwin'

  stable-x86_64-apple-darwin installed - rustc 1.54.0 (a178d0322 2021-07-26)


Rust is installed now. Great!

To get started you may need to restart your current shell.
This would reload your PATH environment variable to include
Cargo's bin directory ($HOME/.cargo/bin).

To configure your current shell, run:
source $HOME/.cargo/env
```

## Install http-tunnel app

```
$ cargo install http-tunnel
```

## Start tunnel server

```
$ http-tunnel --bind 0.0.0.0:8080 http
Cannot initialize logger from ./config/log4rs.yaml, error=[No such file or directory (os error 2)]. Logging to the console.
2021-08-27T20:31:05.888398+09:00 INFO http_tunnel::configuration - Starting in HTTP mode: bind: 0.0.0.0:8080, configuration: None
2021-08-27T20:31:05.889516+09:00 INFO http_tunnel - Starting listener on: 0.0.0.0:8080
2021-08-27T20:31:05.889687+09:00 INFO http_tunnel - Serving requests on: 0.0.0.0:8080
```

Open browser with setting HTTP Proxy.

```
-> % http-tunnel --bind 0.0.0.0:8080 http
Cannot initialize logger from ./config/log4rs.yaml, error=[No such file or directory (os error 2)]. Logging to the console.
2021-08-27T20:31:05.888398+09:00 INFO http_tunnel::configuration - Starting in HTTP mode: bind: 0.0.0.0:8080, configuration: None
2021-08-27T20:31:05.889516+09:00 INFO http_tunnel - Starting listener on: 0.0.0.0:8080
2021-08-27T20:31:05.889687+09:00 INFO http_tunnel - Serving requests on: 0.0.0.0:8080
2021-08-27T20:31:46.418313+09:00 INFO metrics - {"tunnel_ctx":{"id":144068183384961038366744567532903369765},"result":"OperationNotAllowed","upstream_stats":null,"downstream_stats":null}
2021-08-27T20:35:05.190557+09:00 INFO http_tunnel::proxy_target - Resolved DNS www.mozilla.org:443 to [[2606:4700::6812:a522]:443, [2606:4700::6812:a422]:443, 104.18.165.34:443, 104.18.164.34:443]
2021-08-27T20:35:05.335393+09:00 INFO http_tunnel::proxy_target - Resolved DNS incoming.telemetry.mozilla.org:443 to [34.209.161.201:443, 44.235.28.153:443, 35.155.6.125:443, 35.164.22.70:443, 44.225.87.131:443, 54.148.159.250:443, 52.88.2.59:443, 35.163.9.121:443]
2021-08-27T20:35:05.409305+09:00 INFO http_tunnel::proxy_target - Resolved DNS www.firefox.com:443 to [143.204.129.132:443]
2021-08-27T20:35:05.431721+09:00 INFO http_tunnel::proxy_target - Resolved DNS www.googletagmanager.com:443 to [[2404:6800:4004:80b::2008]:443, 172.217.31.136:443]
2021-08-27T20:35:05.520109+09:00 ERROR http_tunnel::relay - Downstream failed to read. Err = Os { code: 54, kind: ConnectionReset, message: "Connection reset by peer" }, CTX=44257257998140248710077493675484364060
2021-08-27T20:35:05.520190+09:00 INFO http_tunnel::relay - Downstream closed: shutdown_reason=ReadError, bytes=0, event_count=0, duration=91.67µs, rate_kbps=0.000, CTX=44257257998140248710077493675484364060
2021-08-27T20:35:05.527521+09:00 INFO http_tunnel::proxy_target - Resolved DNS accounts.firefox.com:443 to [34.212.194.7:443, 35.166.84.75:443, 52.88.96.248:443]
2021-08-27T20:35:05.634070+09:00 ERROR http_tunnel::relay - Upstream failed to shutdown. Err = Os { code: 57, kind: NotConnected, message: "Socket is not connected" }, CTX=44257257998140248710077493675484364060
2021-08-27T20:35:05.634105+09:00 INFO http_tunnel::relay - Upstream closed: shutdown_reason=GracefulShutdown, bytes=0, event_count=0, duration=114.017684ms, rate_kbps=0.000, CTX=44257257998140248710077493675484364060
2021-08-27T20:35:05.634166+09:00 INFO metrics - {"tunnel_ctx":{"id":44257257998140248710077493675484364060},"result":"Ok","upstream_stats":{"shutdown_reason":"ReadError","total_bytes":0,"event_count":0,"duration":{"secs":0,"nanos":91670}},"downstream_stats":{"shutdown_reason":"GracefulShutdown","total_bytes":0,"event_count":0,"duration":{"secs":0,"nanos":114017684}}}
2021-08-27T20:35:07.626514+09:00 INFO http_tunnel::proxy_target - Resolved DNS www.google-analytics.com:443 to [[2404:6800:4004:822::200e]:443, 172.217.25.78:443]
2021-08-27T20:35:11.178819+09:00 INFO metrics - {"tunnel_ctx":{"id":198209908819004384893508996367801288229},"result":"BadRequest","upstream_stats":null,"downstream_stats":null}
2021-08-27T20:35:18.823374+09:00 INFO http_tunnel::proxy_target - Resolved DNS firefox.com:443 to [44.235.246.155:443, 44.236.48.31:443, 44.236.72.93:443]
2021-08-27T20:35:18.955764+09:00 ERROR http_tunnel::relay - Downstream failed to read. Err = Os { code: 54, kind: ConnectionReset, message: "Connection reset by peer" }, CTX=79951975676022357181275712586094362924
2021-08-27T20:35:18.955842+09:00 INFO http_tunnel::relay - Downstream closed: shutdown_reason=ReadError, bytes=0, event_count=0, duration=89.825µs, rate_kbps=0.000, CTX=79951975676022357181275712586094362924
2021-08-27T20:35:18.962252+09:00 ERROR http_tunnel::relay - Downstream failed to read. Err = Os { code: 54, kind: ConnectionReset, message: "Connection reset by peer" }, CTX=239558971666665646158165953248947345548
2021-08-27T20:35:18.962311+09:00 INFO http_tunnel::relay - Downstream closed: shutdown_reason=ReadError, bytes=0, event_count=0, duration=63.513µs, rate_kbps=0.000, CTX=239558971666665646158165953248947345548
2021-08-27T20:35:19.074452+09:00 ERROR http_tunnel::relay - Upstream failed to shutdown. Err = Os { code: 57, kind: NotConnected, message: "Socket is not connected" }, CTX=239558971666665646158165953248947345548
2021-08-27T20:35:19.074500+09:00 INFO http_tunnel::relay - Upstream closed: shutdown_reason=GracefulShutdown, bytes=0, event_count=0, duration=112.261725ms, rate_kbps=0.000, CTX=239558971666665646158165953248947345548
2021-08-27T20:35:19.074595+09:00 INFO metrics - {"tunnel_ctx":{"id":239558971666665646158165953248947345548},"result":"Ok","upstream_stats":{"shutdown_reason":"ReadError","total_bytes":0,"event_count":0,"duration":{"secs":0,"nanos":63513}},"downstream_stats":{"shutdown_reason":"GracefulShutdown","total_bytes":0,"event_count":0,"duration":{"secs":0,"nanos":112261725}}}
2021-08-27T20:35:19.091049+09:00 ERROR http_tunnel::relay - Upstream failed to shutdown. Err = Os { code: 57, kind: NotConnected, message: "Socket is not connected" }, CTX=79951975676022357181275712586094362924
2021-08-27T20:35:19.091096+09:00 INFO http_tunnel::relay - Upstream closed: shutdown_reason=GracefulShutdown, bytes=0, event_count=0, duration=135.350406ms, rate_kbps=0.000, CTX=79951975676022357181275712586094362924
2021-08-27T20:35:19.091169+09:00 INFO metrics - {"tunnel_ctx":{"id":79951975676022357181275712586094362924},"result":"Ok","upstream_stats":{"shutdown_reason":"ReadError","total_bytes":0,"event_count":0,"duration":{"secs":0,"nanos":89825}},"downstream_stats":{"shutdown_reason":"GracefulShutdown","total_bytes":0,"event_count":0,"duration":{"secs":0,"nanos":135350406}}}


2021-08-27T20:36:19.868186+09:00 INFO http_tunnel::relay - Downstream closed: shutdown_reason=GracefulShutdown, bytes=1811, event_count=5, duration=74.212156973s, rate_kbps=0.024, CTX=45055479826366269461028916866561002968
2021-08-27T20:36:19.998529+09:00 INFO http_tunnel::relay - Upstream closed: shutdown_reason=GracefulShutdown, bytes=4906, event_count=5, duration=74.342510044s, rate_kbps=0.064, CTX=45055479826366269461028916866561002968
2021-08-27T20:36:19.998821+09:00 INFO metrics - {"tunnel_ctx":{"id":45055479826366269461028916866561002968},"result":"Ok","upstream_stats":{"shutdown_reason":"GracefulShutdown","total_bytes":1811,"event_count":5,"duration":{"secs":74,"nanos":212156973}},"downstream_stats":{"shutdown_reason":"GracefulShutdown","total_bytes":4906,"event_count":5,"duration":{"secs":74,"nanos":342510044}}}
```

Looking at the log, it looks like it's polling over a perios of time.

## Design details

After this point, just relay TCP traffic both ways until one of the sides closes it, or an I/O error happens.

- Functional requirements
    - should work for both HTTP and HTTPS
    - should be able to manage access/block targets
- Non-Functional requirements
    - shouldn't log any information that identifies users
    - should have high throughput and low-latency
    - want it to be resilent to traffic spikes
    - resist basic DDoS attacks


