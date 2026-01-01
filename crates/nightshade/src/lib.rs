//! # Nightshade
//!
//! A data-oriented game engine written in Rust.
//!
//! ## Features
//!
//! ### Core Features
//!
//! - **`engine`** (default) - Full engine functionality including ECS, windowing, and all sub-features
//!   below. This is the standard feature for building games.
//!
//! - **`full`** - Everything in `engine` plus `audio`, `navmesh`, `trenchbroom`, and `scripting`.
//!   Use this when you want all optional functionality.
//!
//! - **`wgpu`** (default) - WebGPU-based rendering with support for DirectX 12, Metal, Vulkan, and
//!   WebGPU backends.
//!
//! ### Optional Features
//!
//! These are NOT included in `engine` and must be enabled explicitly:
//!
//! - **`egui`** (default) - Immediate mode GUI using egui. Enables the `fn ui()` method on the State
//!   trait. Games that only use the built-in immediate mode UI can disable this feature.
//!
//! - **`shell`** - Drop-down developer console with command registration. Requires `egui`. Apps
//!   define a context struct and register commands that can modify both the ECS world and the
//!   app-specific context.
//!
//! - **`audio`** - Audio playback and spatial audio using the Kira library.
//!
//! - **`navmesh`** - Navigation mesh generation using Recast. The navmesh pathfinding and agent
//!   systems are always available; this feature enables procedural mesh generation.
//!
//! - **`trenchbroom`** - TrenchBroom/Quake `.map` file import with brush geometry and entity parsing.
//!
//! - **`scripting`** - Rhai scripting for runtime script execution. Script data types are always
//!   available for serialization; this enables the runtime.
//!
//! ### Platform Features
//!
//! - **`openxr`** - OpenXR support for VR headsets. Enables the `xr` module.
//!
//! - **`steam`** - Steamworks integration for achievements, stats, friends, and multiplayer.
//!
//! ### Profiling Features
//!
//! - **`tracing`** - Rolling log files in `logs/` and `RUST_LOG` environment variable support.
//!
//! - **`tracy`** - Real-time profiling with Tracy. Implies `tracing`.
//!
//! - **`chrome`** - Chrome tracing output to `logs/trace.json`. Implies `tracing`.
//!
//! ### Plugin Features
//!
//! - **`plugins`** - Minimal guest-side plugin API for WASM plugins. Only includes serialization types.
//!
//! - **`plugin_runtime`** - WASM plugin runtime using Wasmtime. Enables hosting and executing plugins.
//!
//! - **`editor-plugin-host`** - Editor plugin host API with egui and egui_tiles.
//!
//! - **`editor-plugin-guest`** - Editor plugin guest API for WASM plugins.
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

#[cfg(feature = "shell")]
pub mod shell;

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
    pub use crate::ecs::navmesh::{
        NavMeshAgent, NavMeshAgentState, NavMeshConnection, NavMeshEdge, NavMeshTriangle,
        NavMeshWorld, PathRequest, PathResult, PathStatus, PathfindingAlgorithm, SpatialHash,
        clear_navmesh, get_agent_path_length, get_agent_state, run_navmesh_systems,
        set_agent_destination, set_agent_speed, set_navmesh_debug_draw, spawn_navmesh_agent,
        stop_agent,
    };
    #[cfg(feature = "navmesh")]
    pub use crate::ecs::navmesh::{RecastNavMeshConfig, generate_navmesh_recast};
    pub use crate::ecs::particles::systems::update_particle_emitters;
    #[cfg(feature = "physics")]
    pub use crate::ecs::physics::PhysicsWorld;
    pub use crate::ecs::physics::{
        CharacterControllerComponent, ColliderComponent, ColliderShape, PhysicsInterpolation,
        PhysicsMaterial, PhysicsMaterialComponent, RigidBodyComponent,
    };
    #[cfg(feature = "physics")]
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
    pub use crate::ecs::texture_loader::{
        AssetLoadingState, AssetLoadingStatus, LoadedTexture, SharedTextureQueue, TextureLoadQueue,
        TextureLoadRequest, create_shared_queue, get_asset_search_paths, process_and_load_textures,
        process_texture_queue, queue_texture_from_path, set_asset_search_paths,
    };
    pub use crate::ecs::ui::{
        Anchor, ImmediateUi, Layout, LayoutAlignment, LayoutStack, Rect, UiId, UiResponse, UiStyle,
        UiWidget, WidgetState,
    };
    pub use crate::ecs::world::commands::{
        despawn_recursive_immediate, load_hdr_skybox, load_hdr_skybox_from_path,
        load_procedural_textures, set_material_with_textures,
        spawn_3d_billboard_text_with_properties, spawn_3d_text_with_properties, spawn_cone_at,
        spawn_cube_at, spawn_cylinder_at, spawn_instanced_mesh, spawn_instanced_mesh_with_material,
        spawn_material, spawn_mesh_at, spawn_plane_at, spawn_sphere_at, spawn_sun,
        spawn_sun_without_shadows, spawn_torus_at,
    };
    pub use crate::render::wgpu::passes::geometry::UiLayer;
    #[cfg(feature = "egui")]
    pub use egui;
    pub use freecs;
    pub use image;
    pub use rand::{self, Rng};
    pub use serde;
    pub use tracing;
    pub use winit::{
        event::{ElementState, ElementState as KeyState, MouseButton, WindowEvent},
        keyboard::KeyCode,
        window,
    };

    #[cfg(feature = "wgpu")]
    pub use crate::render::wgpu::passes;
    #[cfg(feature = "wgpu")]
    pub use crate::render::wgpu::rendergraph::{
        PassExecutionContext, PassNode, RenderGraph, ResourceId,
    };
    #[cfg(feature = "gamepad")]
    pub use gilrs;
    pub use nalgebra_glm;
    #[cfg(feature = "physics")]
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

    #[cfg(feature = "shell")]
    pub use crate::shell::{Command, CommandRegistry, OutputLine, ShellState, shell_ui};
}
