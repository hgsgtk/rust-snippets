use std::time::Duration;

#[derive(Builder, Deserialize, Clone)]
pub struct RelayPolicy {
    #[serde(with = "humantime_serde")]
    pub idle_timeout: Duration,
    // Date type u64 64bit integer
    // https://doc.rust-lang.org/book/ch03-02-data-types.html
    pub min_rate_bpm: u64, // bpm = bytes per minute
    pub max_rate_bpm: u64,
}