/// About Comments -> INNER_LINE_DOC -> //! ~[\n IsolatedCR]*
/// https://doc.rust-lang.org/reference/comments.html

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
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
}