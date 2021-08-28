use regex::Regex;

use crate::proxy_target::Nugget;
use crate::tunnel::{TunnelCtx};

/// Codec to extract `HTTP/1.1 CONNECT` requests and build a corresponding `HTTP` response.
/// Codec means 符号化方式を使ってデータのエンコード（符号化）とデコード（復号）を双方向にできる装置やソフトウェア.
/// https://ja.wikipedia.org/wiki/%E3%82%B3%E3%83%BC%E3%83%87%E3%83%83%E3%82%AF
#[derive(Clone, Builder)]
pub struct HttpTunnelCodec {
    tunnel_ctx: TunnelCtx,
    enabled_targets: Regex,
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