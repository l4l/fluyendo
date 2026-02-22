#[cfg(all(not(target_arch = "wasm32"), not(target_env = "musl")))]
mod native;
#[cfg(all(not(target_arch = "wasm32"), not(target_env = "musl")))]
pub use native::*;

// libasound is a shared object and cannot be linked staticly
// so for musl we don't have any other option
#[cfg(target_env = "musl")]
mod stub;
#[cfg(target_env = "musl")]
pub use stub::*;

#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
pub use web::*;
