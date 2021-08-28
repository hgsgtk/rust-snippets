
/// trait std::default::Default https://doc.rust-lang.org/std/default/trait.Default.html
/// A trait for giving a type a useful default value
#[derive(Builder, Copy, Clone, Default, Serialize)]
pub struct TunnelCtx {
    id: u128,
}