pub mod contract;

#[cfg(feature = "plugin")]
pub mod commands;
#[cfg(feature = "plugin")]
mod data;
#[cfg(feature = "plugin")]
mod plugin;
#[cfg(feature = "plugin")]
pub mod prelude;
#[cfg(feature = "plugin")]
pub mod queries;
#[cfg(feature = "plugin")]
pub mod resources;
#[cfg(feature = "plugin")]
pub mod systems;

pub use contract::render_data::{SpriteAtlasSpec, SpriteAtlasUpload, SpriteFrame, SpriteInstance};
pub use contract::{
    Sprite, SpriteAnimationFrame, SpriteAnimator, SpriteFilter, SpriteLoop, SpriteRegion, TileData,
    TileTrim, Tilemap,
};
#[cfg(feature = "plugin")]
pub use plugin::SpritePlugin;
