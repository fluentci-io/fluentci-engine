//! A library so clean rebuilds exercise sccache's Rust compilation cache.
pub fn greeting(name: &str) -> String {
    format!("hello, {name}, from a cached Rust library")
}
