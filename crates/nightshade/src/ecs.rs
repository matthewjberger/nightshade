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
/// The retained UI, re-exported module by module. The list is the engine's
/// supported surface: the widget vocabulary, the state the host drains, and
/// the entry points a program calls. The UI's internal pipeline modules
/// (`render_sync`, `systems`, `widget_systems`, `scope`, `taffy_layout`) stay
/// out of it because `UiPlugin` is what runs them.
pub mod ui {
    pub use nightshade_ui::{
        MAX_POPUP_HEIGHT, builder, color, components, compose_ui, default_ui_systems, icons, layer,
        layout_types, prelude, register_ui_components, resources, scene, schema, state, testing,
        theme, ui_world_indices, units, user_interface, widgets,
    };

    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_os = "android"),
        feature = "core"
    ))]
    pub use nightshade_ui::accessibility;

    /// The widget vocabulary, mirroring what `nightshade-ui` re-exports at its
    /// own root: the builders, the components, the layout and unit types, the
    /// state resources, and the theme.
    pub use nightshade_ui::builder::*;
    pub use nightshade_ui::components::*;
    pub use nightshade_ui::layout_types::*;
    pub use nightshade_ui::resources::*;
    pub use nightshade_ui::scope::*;
    pub use nightshade_ui::state::*;
    pub use nightshade_ui::theme::*;
    pub use nightshade_ui::units::*;
    pub use nightshade_ui::user_interface::*;
    pub use nightshade_ui::widgets::{ContextMenuBuilder, TileBuilder};
}

pub mod vfx;
pub mod water;
pub mod world;

pub use crate::ecs::world::*;
