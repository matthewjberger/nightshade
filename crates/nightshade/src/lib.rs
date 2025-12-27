//! # Nightshade
//!
//! A data-oriented game engine written in Rust.
//!
//! ## Features
//!
//! - **`engine`** (default) - Full engine functionality including ECS, rendering, physics, and audio.
//!
//! - **`plugins`** - Minimal guest-side plugin API for WASM plugins. Only includes serialization types.
//!
//! - **`wgpu`** (default) - WebGPU-based rendering with support for DirectX 12, Metal, Vulkan, and WebGPU backends.
//!
//! - **`openxr`** - OpenXR support for VR headsets. Enables the `xr` module for VR development.
//!
//! - **`steam`** - Steamworks integration for achievements, stats, friends, and multiplayer. Native only.
//!
//! - **`tracing`** - Enables rolling daily log files written to the `logs/` directory. Also enables
//!   `RUST_LOG` environment variable filtering (e.g., `RUST_LOG=debug,nightshade::render=trace`).
//!
//! - **`tracy`** - Real-time profiling with [Tracy](https://github.com/wolfpld/tracy). Requires the
//!   Tracy profiler to be running. Shows frame timings, span hierarchy, and performance data live.
//!   Implies `tracing`.
//!
//! - **`chrome`** - Chrome tracing output. Writes `logs/trace.json` that can be loaded in
//!   `chrome://tracing` for offline performance analysis. Implies `tracing`.
//!
//! - **`editor-plugin-host`** - Editor plugin host API. Re-exports egui and egui_tiles for building
//!   editor UIs that can host WASM plugins.
//!
//! - **`editor-plugin-guest`** - Editor plugin guest API for WASM plugins. Provides functions to
//!   register inspectors, menu items, and communicate with the editor host.
//!
//! - **`scripting`** - Rhai scripting support for runtime script execution. Enables the script
//!   runtime, systems, and API. Script data types (`Script`, `ScriptSource`) are always available
//!   for map/prefab serialization, but script execution requires this feature. Used by the editor
//!   for play mode and any app that needs to run embedded or file-based scripts.
//!
//! See [PROFILING.md](https://github.com/matthewjberger/nightshade/blob/main/docs/PROFILING.md) for
//! detailed profiling documentation including Tracy setup and adding custom instrumentation.
//!
//! ## Example
//!
//! ```no_run
//! use nightshade::prelude::*;
//!
//! struct Game;
//!
//! impl State for Game {
//!     fn title(&self) -> &str { "My Game" }
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     launch(Game)
//! }
//! ```

pub mod plugin;

#[cfg(any(feature = "editor-plugin-host", feature = "editor-plugin-guest"))]
pub mod editor_plugin;

#[cfg(feature = "engine")]
pub mod ecs;
#[cfg(feature = "engine")]
pub mod render;
#[cfg(feature = "engine")]
pub mod run;
#[cfg(feature = "engine")]
mod systems;

#[cfg(feature = "openxr")]
pub mod xr;

#[cfg(all(feature = "steam", not(target_arch = "wasm32")))]
pub mod steam;

#[cfg(all(feature = "plugin_runtime", not(target_arch = "wasm32")))]
pub mod plugin_runtime;

#[cfg(feature = "engine")]
pub use systems::*;

#[cfg(feature = "engine")]
pub mod prelude {
    pub use crate::ecs::world::components::*;
    pub use crate::ecs::world::resources::*;
    pub use crate::ecs::world::*;
    pub use crate::run::*;
    pub use crate::systems::*;
    pub use web_time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    pub const LOGO_BYTES: &[u8] = include_bytes!("../images/logo.png");

    pub use crate::ecs::animation::systems::{apply_animations, update_animation_players};
    pub use crate::ecs::event_bus::commands::{publish_app_event, publish_event};
    pub use crate::ecs::map::{
        Map, MapCamera, MapError, MapLight, MapNode, MapPrefab, NodeIndex, load_map,
        load_map_binary, load_map_json, save_map, save_map_binary, save_map_json, spawn_map,
    };
    pub use crate::ecs::mesh::commands::spawn_mesh;
    pub use crate::ecs::mesh::systems::update_instanced_mesh_caches_system;
    pub use crate::ecs::particles::systems::update_particle_emitters;
    pub use crate::ecs::physics::{
        CharacterControllerComponent, ColliderComponent, ColliderShape, PhysicsInterpolation,
        PhysicsMaterial, PhysicsMaterialComponent, PhysicsWorld, RigidBodyComponent,
    };
    pub use crate::ecs::picking::{
        PickingOptions, PickingRay, PickingResult, PickingWorld, get_ground_position_from_screen,
        pick_closest_entity, pick_closest_entity_trimesh, pick_entities, pick_entities_in_frustum,
        pick_entities_trimesh, register_entity_for_trimesh_picking,
        register_entity_hierarchy_for_trimesh_picking, unregister_entity_from_picking,
        update_picking_transform,
    };
    pub use crate::ecs::prefab::commands::spawn_prefab_with_animations;
    pub use crate::ecs::terrain::{
        NoiseConfig, NoiseType, TerrainConfig, TerrainGenerationResult, generate_terrain_mesh,
        sample_terrain_height, spawn_terrain, spawn_terrain_with_material,
    };
    pub use crate::ecs::text::commands::{
        load_font_from_bytes, spawn_hud_text, spawn_hud_text_with_properties, spawn_ui_text,
        spawn_ui_text_with_properties,
    };
    pub use crate::ecs::text::systems::sync_text_meshes_system;
    pub use crate::ecs::ui::{
        Anchor, ImmediateUi, Layout, LayoutAlignment, LayoutStack, Rect, UiId, UiResponse, UiStyle,
        UiWidget, WidgetState,
    };
    pub use crate::ecs::world::commands::{
        despawn_recursive_immediate, load_hdr_skybox, load_hdr_skybox_from_path,
        load_procedural_textures, set_material_with_textures,
        spawn_3d_billboard_text_with_properties, spawn_3d_text_with_properties, spawn_cone_at,
        spawn_cube_at, spawn_cylinder_at, spawn_instanced_mesh, spawn_instanced_mesh_with_material,
        spawn_mesh_at, spawn_plane_at, spawn_sphere_at, spawn_sun, spawn_sun_without_shadows,
        spawn_torus_at,
    };
    pub use egui;
    pub use freecs;
    pub use rand::{self, Rng};
    pub use serde;
    pub use tracing;
    pub use winit::{
        event::{ElementState, ElementState as KeyState, MouseButton, WindowEvent},
        keyboard::KeyCode,
    };

    #[cfg(feature = "wgpu")]
    pub use crate::render::wgpu::passes;
    #[cfg(feature = "wgpu")]
    pub use crate::render::wgpu::rendergraph::{
        PassExecutionContext, PassNode, RenderGraph, ResourceId,
    };
    pub use gilrs;
    pub use nalgebra_glm;
    pub use rapier3d;
    pub use wgpu;

    #[cfg(feature = "openxr")]
    pub use crate::xr::{XrContext, XrInput, launch_xr};

    #[cfg(all(feature = "steam", not(target_arch = "wasm32")))]
    pub use crate::steam::{
        AchievementInfo, ActiveSession, FriendInfo, NetworkMessage, PendingSessionRequest,
        PersonaState, SessionState, StatInfo, StatValue, SteamResources,
    };
    #[cfg(all(feature = "steam", not(target_arch = "wasm32")))]
    pub use steamworks::SteamId;
}
