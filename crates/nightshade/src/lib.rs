//! # Nightshade
//!
//! A data-oriented game engine written in Rust.
//!
//! ## Features
//!
//! - **`wgpu`** (default) - WebGPU-based rendering with support for DirectX 12, Metal, Vulkan, and WebGPU backends.
//!
//! - **`openxr`** - OpenXR support for VR headsets. Enables the [`xr`] module for VR development.
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
//! See [PROFILING.md](https://github.com/matthewjberger/nightshade/blob/main/PROFILING.md) for
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

pub mod ecs;
pub mod render;
pub mod run;
mod systems;

#[cfg(feature = "openxr")]
pub mod xr;

#[cfg(all(feature = "steam", not(target_arch = "wasm32")))]
pub mod steam;

pub use systems::*;

pub mod prelude {
    pub use crate::ecs::world::components::*;
    pub use crate::ecs::world::resources::*;
    pub use crate::ecs::world::*;
    pub use crate::run::*;
    pub use crate::systems::*;
    pub use web_time::{Duration, Instant, SystemTime};

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
        PickingOptions, PickingRay, PickingResult, get_ground_position_from_screen,
        pick_closest_entity, pick_entities, pick_entities_in_frustum,
    };
    pub use crate::ecs::prefab::commands::spawn_prefab_with_animations;
    pub use crate::ecs::text::commands::{
        load_font_from_bytes, spawn_hud_text, spawn_hud_text_with_properties,
    };
    pub use crate::ecs::text::systems::sync_text_meshes_system;
    pub use crate::ecs::world::commands::{
        load_hdr_skybox, load_hdr_skybox_from_path, load_procedural_textures,
        set_material_with_textures, spawn_3d_billboard_text_with_properties,
        spawn_3d_text_with_properties, spawn_cone_at, spawn_cube_at, spawn_cylinder_at,
        spawn_instanced_mesh, spawn_instanced_mesh_with_material, spawn_mesh_at, spawn_plane_at,
        spawn_sphere_at, spawn_sun, spawn_sun_without_shadows, spawn_torus_at,
    };
    pub use egui;
    pub use freecs;
    pub use rand::{self, Rng};
    pub use serde;
    pub use tracing;
    pub use winit::{
        event::{ElementState as KeyState, WindowEvent},
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
