#[cfg(feature = "physics")]
pub mod commands;
pub mod queries;
#[cfg(feature = "physics")]
pub mod resources;

#[cfg(feature = "physics")]
pub use commands::*;
pub use queries::*;
#[cfg(feature = "physics")]
pub use resources::*;
