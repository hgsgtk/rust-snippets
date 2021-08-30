use async_trait::async_trait;
use bytes::BytesMut;
use core::fmt;
use regex::Regex;
use log::debug;
use std::fmt::Write;

use crate::proxy_target::Nugget;
use crate::tunnel::{TunnelCtx, EstablishTunnelResult, TunnelTarget};

use tokio::io::{Error, ErrorKind};
use tokio_util::codec::{Decoder, Encoder};

/// A reasonable value to limit possible header size.
/// usize: The pointer-sized unsigned integer type.
/// https://doc.rust-lang.org/std/primitive.usize.html
/// 16384 
/// > ETTINGS_MAX_FRAME_SIZE (0x5):  Indicates the size of the largest 
/// > frame payload that the sender is willing to receive, in octets.
/// https://datatracker.ietf.org/doc/html/rfc7540#section-6.5.2
const MAX_HTTP_REQUEST_SIZE: usize = 16384;

const REQUEST_END_MARKER: &[u8] = b"\r\n\r\n";

/// (Original comments)
/// HTTP/1.1 request representation
/// Supports only `CONNECT` method, unless the `plain_text` feature is enabled
struct HttpConnectRequest {
    uri: String,
    nugget: Option<Nugget>,
    // (Original comments)
    // out of scope of this demo, but let's put it here for extensibility
    // e.g. Authorization/Policies headers
    // headers: Vec<(String, String)>,
}

impl HttpConnectRequest {
    pub fn parse(http_request: &[u8]) -> Result<Self, EstablishTunnelResult> {
        // TODO: http_request parser
        Ok(Self {
            uri: String::from("aaa"),
            nugget: None,
        })
    }
}

/// Codec to extract `HTTP/1.1 CONNECT` requests and build a corresponding `HTTP` response.
/// Codec means 符号化方式を使ってデータのエンコード（符号化）とデコード（復号）を双方向にできる装置やソフトウェア.
/// https://ja.wikipedia.org/wiki/%E3%82%B3%E3%83%BC%E3%83%87%E3%83%83%E3%82%AF
#[derive(Clone, Builder)]
pub struct HttpTunnelCodec {
    tunnel_ctx: TunnelCtx,
    enabled_targets: Regex,
}

// Without this definition, we got an error: error[E0277]: the trait bound `HttpTunnelCodec: Decoder` is not satisfied
impl Decoder for HttpTunnelCodec {
    type Item = HttpTunnelTarget;
    type Error = EstablishTunnelResult;

    // bytes::BytesMut 
    // > BytesMut represents a unique view into a potentially shared memory region. 
    // > Given the uniqueness guarantee, owners of BytesMut handles are able to mutate the memory. 
    // > It is similar to a Vec<u8> but with less copies and allocations.
    // https://docs.rs/bytes/0.4.12/bytes/struct.BytesMut.html
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if !got_http_request(&src) {
            return Ok(None);
        }

        match HttpConnectRequest::parse(&src) {
            Ok(parsed_request) => {
                if !self.enabled_targets.is_match(&parsed_request.uri) {
                    debug!(
                        "Target `{}` is not allowed. Allowed: `{}`, CTX={}",
                        parsed_request.uri, self.enabled_targets, self.tunnel_ctx
                    );
                    Err(EstablishTunnelResult::Forbidden)
                } else {
                    Ok(Some(
                        HttpTunnelTargetBuilder::default()
                            .target(parsed_request.uri)
                            .nugget(parsed_request.nugget)
                            .build()
                            .expect("HttpTunnelTargetBuilder failed")
                    ))
                }
            }
            Err(e) => Err(e)
        }
    }
}

// Without this implementation, we got an error: error[E0277]: the trait bound `HttpTunnelCodec: Encoder<EstablishTunnelResult>` is not satisfied
impl Encoder<EstablishTunnelResult> for HttpTunnelCodec {
    type Error = std::io::Error;

    fn encode(
        &mut self,
        item: EstablishTunnelResult,
        dst: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        let (code, message) = match item {
            EstablishTunnelResult::Ok => (200, "OK"),
            EstablishTunnelResult::OkWithNugget => {
                // (Original comments)
                // do nothing, the upstream should respond isntead
                return Ok(());
            }
            EstablishTunnelResult::BadRequest => (400, "BAD_REQUEST"),
            EstablishTunnelResult::Forbidden => (403, "FORBIDDEN"),
            EstablishTunnelResult::OperationNotAllowed => (405, "NOT_ALLOWED"),
            EstablishTunnelResult::RequestTimeout => (408, "TIMEOUT"),
            EstablishTunnelResult::TooManyRequests => (429, "TOO_MANY_REQUESTS"),
            EstablishTunnelResult::ServerError => (500, "SERVER_ERROR"),
            EstablishTunnelResult::BadGateway => (502, "BAD_GATEWAY"),
            EstablishTunnelResult::GatewayTimeout => (504, "GATEWAY_TIMEOUT"),
        };

        // use std::fmt::Write; 
        // This trait provides method write_fmt().
        // https://doc.rust-jp.rs/the-rust-programming-language-ja/1.6/std/fmt/trait.Write.html#method.write_fmt
        dst.write_fmt(format_args!("HTTP/1.1 {} {}\r\n\r\n", code as u32, message))
            .map_err(|_| std::io::Error::from(std::io::ErrorKind::Other))
    }
}

#[cfg(not(feature = "plain_text"))]
fn got_http_request(buffer: &BytesMut) -> bool {
    buffer.len() >= MAX_HTTP_REQUEST_SIZE || buffer.ends_with(REQUEST_END_MARKER)
}

// cfg: Configuration conditional checks are possible through two different operators
// https://doc.rust-lang.org/rust-by-example/attribute/cfg.html
#[cfg(feature = "plain_text")]
fn got_http_request(buffer: &BytesMut) -> bool {
    buffer.len() >= MAX_HTTP_REQUEST_SIZE
        || buffer  
            .windows(REQUEST_END_MARKER.len())
            .find(|w| *w == REQUEST_END_MARKER)
            .is_some()
}

/// Trait std::cmp::Eq 
/// > Trait for equality comparisons which are equivalence relations.
/// > `==`
/// https://doc.rust-lang.org/std/cmp/trait.Eq.html
/// 
/// Trait std::cmp::PartialEq
/// > This trait allows for partial equality, for types that do not have a full equivalence relation.
/// https://doc.rust-lang.org/std/cmp/trait.PartialEq.html
///
/// Trait std::fmt::Debug
/// > Debug should format the output in a programmer-facing, debugging context.
/// https://doc.rust-lang.org/std/fmt/trait.Debug.html
#[derive(Builder, Eq, PartialEq, Debug, Clone)]
pub struct HttpTunnelTarget {
    pub target: String,
    pub nugget: Option<Nugget>,
}


/// Without this implmentation, we got an error: error[E0277]: the trait bound `HttpTunnelTarget: TunnelTarget` is not satisfied
#[async_trait]
impl TunnelTarget for HttpTunnelTarget {
    type Addr = String;

    fn target_addr(&self) -> Self::Addr {
        self.target.clone()
    }

    fn has_nugget(&self) -> bool {
        self.nugget.is_some()
    }

    fn nugget(&self) -> &Nugget {
        self.nugget
            .as_ref()
            .expect("Cannot use this method without checking `has_nugget`")
    }
}

// Without this implementation, we got an error: error[E0277]: `HttpTunnelTarget` doesn't implement `std::fmt::Display`
impl fmt::Display for HttpTunnelTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.target)
    }
}

// Without this implementatin, we got an error: error[E0277]: the trait bound `EstablishTunnelResult: From<std::io::Error>` is not satisfied
impl From<Error> for EstablishTunnelResult {
    fn from(e: Error) -> Self {
        match e.kind() {
            ErrorKind::TimedOut => EstablishTunnelResult::GatewayTimeout,
            _ => EstablishTunnelResult::BadGateway,
        }
    }
}