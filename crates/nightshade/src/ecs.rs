//! Entity Component System module.
//!
//! Contains all ECS-related functionality:
//! - Components: Data attached to entities (transforms, meshes, materials, etc.)
//! - Resources: Global singletons (input, graphics settings, caches)
//! - Systems: Logic operating on components
//! - Commands: Deferred operations for spawning and loading
//!
//! The [`World`] struct is the central container built on `nightshade_ecs::dynamic`.
//! Components use struct-of-arrays storage accessed via:
//! - `world.ecs.worlds[CORE].get_<component>(entity)` - Immutable reference
//! - `world.ecs.worlds[CORE].get_<component>_mut(entity)` - Mutable reference
//! - `world.ecs.worlds[CORE].add_components(entity, MASK)` then `world.ecs.worlds[CORE].set_<component>(entity, value)` - Add and set a component
//! - `world.ecs.worlds[CORE].remove_components(entity, MASK)` - Remove a component

pub mod animation;
pub mod asset_state;
#[cfg(all(
    feature = "assets",
    feature = "file_watcher",
    not(target_arch = "wasm32")
))]
pub mod asset_watcher;
pub mod bounding_volume;
pub mod bundles;
pub mod camera;
pub mod cloth;
pub mod cutscene;
pub mod entity_registry;

pub mod decal;
pub mod event;
pub mod event_bus;
pub mod events;
pub mod gpu_picking;
pub mod graphics;
pub mod input;
pub mod light;
pub mod lines;
pub mod loading;
pub mod material;
pub mod mesh;
pub mod meshlet;
pub mod morph;
pub mod particles;
#[cfg(feature = "picking")]
pub mod picking;
pub mod prefab;
pub mod primitives;
#[cfg(feature = "scene_graph")]
pub mod scene;
pub mod script;
pub mod skin;
pub mod snapshot;
pub mod text;
#[cfg(feature = "assets")]
pub mod texture_loader;
pub mod time;
pub mod timer;
pub mod transform;
pub mod tween;
pub mod ui;
pub mod vfx;
pub mod water;
pub mod window;
pub mod world;

pub use crate::ecs::world::*;
