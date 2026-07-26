//! Assets: what a program loads from disk and what it writes back out.
//!
//! The glTF import and export path, the scene format and its capability hooks,
//! prefabs, the texture and mesh upload queue, and the file watcher. These sit
//! above the runtime domains they fill out and below the plugins, so a domain
//! never reaches in here.

#[cfg(all(
    feature = "assets",
    feature = "file_watcher",
    not(target_arch = "wasm32")
))]
pub mod asset_watcher;
pub mod texture_source;

pub mod asset;
pub mod loading;
pub mod prefab;
#[cfg(feature = "scene_graph")]
pub mod scene;
#[cfg(feature = "assets")]
pub mod texture_loader;
