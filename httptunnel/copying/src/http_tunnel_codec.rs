use regex::Regex;

use crate::tunnel::{TunnelCtx};

/// Codec to extract `HTTP/1.1 CONNECT` requests and build a corresponding `HTTP` response.
/// Codec means 符号化方式を使ってデータのエンコード（符号化）とデコード（復号）を双方向にできる装置やソフトウェア.
/// https://ja.wikipedia.org/wiki/%E3%82%B3%E3%83%BC%E3%83%87%E3%83%83%E3%82%AF
#[derive(Clone, Builder)]
pub struct HttpTunnelCodec {
    tunnel_ctx: TunnelCtx,
    enabled_targets: Regex,
}