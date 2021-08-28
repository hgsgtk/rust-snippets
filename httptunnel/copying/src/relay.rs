use std::time::Duration;

/// Compile-time constants and compile-time evaluable functions.
/// > Constants, like statics, should always be in SCREAMING_SNAKE_CASE.
/// https://doc.rust-lang.org/std/keyword.const.html
pub const NO_TIMEOUT: Duration = Duration::from_secs(300);
pub const NO_BANDWIDTH_LIMIT: u64 = 1_000_000_000_000_u64;

#[derive(Builder, Deserialize, Clone)]
pub struct RelayPolicy {
    #[serde(with = "humantime_serde")]
    pub idle_timeout: Duration,
    // Date type u64 64bit integer
    // https://doc.rust-lang.org/book/ch03-02-data-types.html
    pub min_rate_bpm: u64, // bpm = bytes per minute
    pub max_rate_bpm: u64,
}