/// About Comments -> INNER_LINE_DOC -> //! ~[\n IsolatedCR]*
/// https://doc.rust-lang.org/reference/comments.html

use crate::tunnel::{TunnelCtx};

use async_trait::async_trait;
use log::{debug, error, info};
use rand::prelude::thread_rng;
use rand::Rng;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::io;
use tokio::io::{Error, ErrorKind};
use tokio::time::Duration;
use tokio::sync::RwLock;

/// Vec https://doc.rust-lang.org/std/vec/struct.Vec.html
/// A measurement of a monotonically increasing clock.
/// u128: The 128-bit unsigned integer type. https://doc.rust-lang.org/std/primitive.u128.html
//
/// SocketAddr is a enum which represents an internal socket address, either IPv4 or IPv6.
/// https://doc.rust-lang.org/std/net/enum.SocketAddr.html
type CachedSocketAddrs = (Vec<SocketAddr>, u128);

/// (Original comment)
/// Caching DNS resolution to minimize DNS look-ups.
/// The cache has relaxed consistency, it allows concurrncy DNS look-ups of the same key,
/// without any guarantees which result is going to be cached.
//
/// Given it's used for DNS look-ups this trade-off seems to be reasonable.
/// 
/// (My comment)
/// TODO: I didn't understand it.
#[derive(Clone)]
pub struct SimpleCachingDnsResolver {
    // Arc: A thread-safe reference-counting pointer. ‘Arc’ stands for ‘Atomically Reference Counted’.
    // https://doc.rust-lang.org/std/sync/struct.Arc.html
    // > Arc/Rc は参照カウントを使ったスマートポインタであり、データや状態を共有できる。
    // > よりコストの低い Rc で実装を開始して、必要になったら Arc に切り替えるというので問題ないでしょう。ただし、不特定多数が使うライブラリの場合ははじめから Arc でもいいかもしれません。
    // https://qiita.com/qnighy/items/4bbbb20e71cf4ae527b9
    //
    // RwLock: An asynchronous reader-writer lock
    // https://docs.rs/tokio/0.2.22/tokio/sync/struct.RwLock.html
    // professional comment about tokio::sync::RwLock https://github.com/ynaka81/in_memory_db_comparison/issues/1
    // 
    // HashMap: A hash map implemented with quadratic probing and SIMD lookup.
    // JP: quadratic probing 2次プロービング
    // - https://www.geeksforgeeks.org/quadratic-probing-in-hashing/
    // - > Quadratic probing is an open-addressing scheme where we look for i2‘th slot in i’th iteration 
    // - > if the given hash value x collides in the hash table. 
    //
    // JP: SIMD lookup SIMDルックアップ
    // https://doc.rust-lang.org/std/collections/struct.HashMap.html
    // > By default, HashMap uses a hashing algorithm selected to provide resistance against HashDoS attacks. 
    //
    // HashDoS attacks: https://www.f5.com/services/resources/glossary/hash-dos-attack
    // > By sending a single POST message filled with thousands of variables,
    // > the hashing function would overload and a server could be tied up processing this single request for as long as an hour. 
    // > This is a hash denial-of-service (DoS) attack.
    //
    // Rustのコレクション型まとめ (VecやHashMapなど) https://qiita.com/garkimasera/items/a6df4d1cd99bc5010a5e
    // - Vec 可変長配列
    // - VecDeque リングバッファによる両端キューです。
    // - LinkedList: 各要素が前後の要素へのポインタを持つ連結リストです。
    // - HashMap: キーと値をペアで記録してくれるもので、他の言語では連想配列や辞書型と呼ばれたりします。
    cache: Arc<RwLock<HashMap<String, CachedSocketAddrs>>>,
    ttl: Duration,
    // Instant
    // https://doc.rust-jp.rs/the-rust-programming-language-ja/1.6/std/time/struct.Instant.html
    start_time: Instant,
}

impl SimpleCachingDnsResolver {
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl,
            start_time: Instant::now(),
        }
    }

    fn pick(&self, addrs: &[SocketAddr]) -> SocketAddr {
        addrs[thread_rng().gen::<usize>() % addrs.len()]
    }

    async fn try_find(&mut self, target: &str) -> Option<SocketAddr> {
        let map = self.cache.read().await;

        let addr = match map.get(target) {
            None => None,
            Some((cached, expiration)) => {
                // gen_range: Generate a random value in the range [low, high)
                // https://docs.rs/rand/0.5.0/rand/trait.Rng.html#method.gen_range
                let expiration_jitter = *expiration + thread_rng().gen_range(0..5_000);
                if Instant::now().duration_since(self.start_time).as_millis() < expiration_jitter {
                    Some(self.pick(cached))
                } else {
                    None
                }
            }
        };

        addr
    }

    async fn resolve_and_cache(&mut self, target: &str) -> io::Result<SocketAddr> {
        let resolved = SimpleCachingDnsResolver::resolve(target).await?;

        let mut map = self.cache.write().await;
        map.insert(
            target.to_string(),
            (
                resolved.clone(),
                Instant::now().duration_since(self.start_time).as_millis() + self.ttl.as_millis(),
            ),
        );

        Ok(self.pick(&resolved))
    }

    async fn resolve(target: &str) -> io::Result<Vec<SocketAddr>> {
        debug!("Resolving DNS {}", target,);
        let resolved: Vec<SocketAddr> = tokio::net::lookup_host(target).await?.collect();
        info!("Resolved DNS {} to {:?}", target, resolved);

        if resolved.is_empty() {
            error!("Cannot resolve DNS {}", target,);
            // A nonexistent interface was requested or the requested address was not local
            // https://doc.rust-lang.org/std/io/enum.ErrorKind.html#variant.AddrNotAvailable
            return Err(Error::from(ErrorKind::AddrNotAvailable));
        }

        Ok(resolved)
    }
}

/// Crate async_trait: This crate provides an attribute macro to make async fn in traits work.
/// https://docs.rs/async-trait/0.1.51/async_trait/
/// Without this trait, we may got an error: error[E0706]: trait fns cannot be declared `async`
#[async_trait]
pub trait DnsResolver {
    async fn resolve(&mut self, target: &str) -> io::Result<SocketAddr>;
}

/// Without this definition, we got an error:
/// > error[E0277]: the trait bound `SimpleCachingDnsResolver: proxy_target::DnsResolver` is not satisfied
#[async_trait]
impl DnsResolver for SimpleCachingDnsResolver {
    async fn  resolve(&mut self, target: &str) -> io::Result<SocketAddr> {
        match self.try_find(target).await {
            Some(a) => Ok(a), // if it found
            _ => Ok(self.resolve_and_cache(target).await?), // if it not found
        }
    }
}


#[derive(Clone, Builder)]
pub struct SimpleTcpConnector<D, R: DnsResolver> {
    connect_timeout: Duration,
    tunnel_ctx: TunnelCtx,
    dns_resolver: R,
    #[builder(setter(skip))]
    // Struct std::marker::PhantomData
    // https://doc.rust-lang.org/std/marker/struct.PhantomData.html
    // How to use it in JA: https://qnighy.hatenablog.com/entry/2018/01/14/220000
    _phantom_target: PhantomData<D>,
}

// Where statement
// useful scene ジェネリック型とジェネリック境界に別々に制限を加えたほうが明瞭になる場合
// https://doc.rust-jp.rs/rust-by-example-ja/generics/where.html
impl<D, R> SimpleTcpConnector<D, R>
where
    R: DnsResolver,
{
    pub fn new(dns_resolver: R, connect_timeout: Duration, tunnel_ctx: TunnelCtx) -> Self {
        Self {
            dns_resolver,
            connect_timeout,
            tunnel_ctx,
            _phantom_target: PhantomData,
        }
    }
}

// TODO: What's nugget?
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Nugget {
    data: Arc<Vec<u8>>,
}