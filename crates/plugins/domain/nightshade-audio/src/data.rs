#[cfg(not(target_arch = "wasm32"))]
pub mod binaural;
#[cfg(not(target_arch = "wasm32"))]
pub mod capture;
#[cfg(not(target_arch = "wasm32"))]
pub mod room_reverb;
#[cfg(not(target_arch = "wasm32"))]
pub mod stream;

pub const LOAD_CREW: usize = 1;
pub const LOAD_LIMIT: usize = 2;

pub struct Sound;
