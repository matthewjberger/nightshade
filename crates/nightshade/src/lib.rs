//! # Nightshade
//!
//! A data-oriented game engine written in Rust.
//!
//! ## Getting Started
//!
//! ### 1. Add to Cargo.toml
//!
//! ```toml
//! [dependencies]
//! nightshade = "0.50"
//! ```
//!
//! ### 2. Create Your Game
//!
//! Programs assemble from plugins: the engine capabilities are default
//! plugins, and the game is one more plugin added in the builder.
//!
//! ```ignore
//! use nightshade::prelude::*;
//!
//! #[derive(Default)]
//! struct MyGame {
//!     camera: Option<Entity>,
//! }
//!
//! struct MyGamePlugin;
//!
//! impl Plugin for MyGamePlugin {
//!     fn build(&self, app: &mut App) {
//!         app.world.resources.window.title = "My Game".to_string();
//!         app.insert_resource(MyGame::default());
//!         app.add_startup_system("initialize", |world| {
//!             game_scope::<MyGame, _>(world, initialize);
//!         });
//!         app.add_system("update", "tick", |world| {
//!             game_scope::<MyGame, _>(world, tick);
//!         });
//!     }
//! }
//!
//! fn initialize(game: &mut MyGame, world: &mut World) {
//!     world.resources.render_settings.atmosphere = Atmosphere::Sky;
//!     spawn_sun(world);
//!
//!     let camera = spawn_camera(world, Vec3::new(5.0, 3.0, 5.0), "Camera".to_string());
//!     world.resources.active_camera = Some(camera);
//!     game.camera = Some(camera);
//!
//!     spawn_cube_at(world, Vec3::new(0.0, 0.5, 0.0));
//! }
//!
//! fn tick(game: &mut MyGame, world: &mut World) {
//!     let delta_time = world.resources.window.timing.delta_time;
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(MyGamePlugin)
//!         .run()
//! }
//! ```
//!
//! ### 3. Loading 3D Models
//!
//! ```ignore
//! let model_data = include_bytes!("../assets/character.glb");
//! let result = import_gltf_from_bytes(model_data).unwrap();
//! let prefab = &result.prefabs[0];
//! let entity = spawn_prefab_with_animations(world, prefab, &result.animations, Vec3::zeros());
//!
//! if let Some(player) = world.get_mut::<crate::ecs::animation::components::AnimationPlayer>(entity) {
//!     player.play(0);
//!     player.looping = true;
//! }
//! ```
//!
//! ## Architecture
//!
//! ### World
//!
//! The [`World`](ecs::world::World) contains all game state:
//!
//! - **Entities**: Unsigned integer handles (`Entity`) identifying objects
//! - **Components**: Data attached to entities (transforms, meshes, physics, etc.)
//! - **Resources**: Global singletons (`world.resources.window.timing`, `.input`, `.graphics`, `.physics`)
//!
//! ### Component Flags
//!
//! Entities are created with bitflags specifying their components:
//!
//! ```ignore
//! let entity = spawn_entities(world,
//!     LOCAL_TRANSFORM | GLOBAL_TRANSFORM | RENDER_MESH,
//!     1
//! )[0];
//! ```
//!
//! | Flag | Description |
//! |------|-------------|
//! | `LOCAL_TRANSFORM` | Position/rotation/scale relative to parent |
//! | `GLOBAL_TRANSFORM` | World-space transform (computed) |
//! | `RENDER_MESH` | Visible 3D geometry |
//! | `MATERIAL_REF` | Material reference |
//! | `CAMERA` | Camera component |
//! | `PARENT` | Hierarchy parent link |
//! | `RIGID_BODY` | Physics body |
//! | `COLLIDER` | Collision shape |
//! | `ANIMATION_PLAYER` | Animation controller |
//! | `PARTICLE_EMITTER` | GPU particles |
//! | `CLOTH` | GPU cloth sheet |
//! | `AUDIO_SOURCE` | Sound playback |
//! | `DECAL` | Projected texture |
//!
//! ### Resources
//!
//! ```ignore
//! // Timing
//! let dt = world.resources.window.timing.delta_time;
//! let fps = world.resources.window.timing.frames_per_second;
//! let uptime_ms = world.resources.window.timing.uptime_milliseconds;
//!
//! // Input
//! let keyboard = &world.resources.input.keyboard;
//! let mouse = &world.resources.input.mouse;
//!
//! // Graphics
//! world.resources.render_settings.bloom_enabled = true;
//! world.resources.render_settings.atmosphere = Atmosphere::Sky;
//!
//! // Camera
//! world.resources.active_camera = Some(camera_entity);
//! ```
//!
//! ### Render Graph
//!
//! Rendering uses a pass-based graph. Override [`State::configure_render_graph`](run::State::configure_render_graph)
//! for bloom, SSAO, or custom effects. See the [`render`] module.
//!
//! ## Troubleshooting
//!
//! **Black Screen**
//! - Set `world.resources.active_camera` to a valid camera
//! - Ensure camera can see objects (position/orientation)
//! - Add lighting: `spawn_sun(world)`
//!
//! **No Audio (WASM)**
//! - Browsers require user interaction first
//! - Trigger audio from click/key handlers
//!
//! **Physics Falls Through**
//! - Floor needs `RigidBodyComponent::new_static()` AND `ColliderComponent`
//! - Check collider dimensions match mesh
//!
//! **Animations Not Playing**
//! - Use `spawn_prefab_with_animations()` not `spawn_prefab()`
//! - Call `player.play(index)` to start
//!
//! **Textures Missing**
//! - Textures acquired through materials are released on despawn; keep hand-loaded
//!   textures alive with `texture_cache_protect(&mut world.resources.texture_cache, name)`
//! - Names are case-sensitive
//!
//! **Entity Invisible**
//! - Needs `RENDER_MESH` flag
//! - Needs valid `MaterialRef`
//! - Check transform position
//!
//! ## Features
//!
//! ### Aggregate Features
//!
//! - **`engine`** (default) - Full engine functionality including asset loading, scene graphs,
//!   file dialogs, and more. This is the standard feature for building games.
//!
//! - **`runtime`** - Minimal rendering without asset loading. Use with `wgpu` for
//!   lightweight apps that don't need gltf/image loading.
//!
//! - **`full`** - Everything in `engine` plus `audio`, `physics`, `gamepad`, `navmesh`.
//!
//! - **`wgpu`** (default) - WebGPU-based rendering with DirectX 12, Metal, Vulkan, and WebGPU.
//!
//! ### Granular Features
//!
//! These allow fine-grained control over dependencies:
//!
//! - **`core`** - Foundation: ECS (freecs), math (nalgebra), windowing (winit), time, petgraph.
//!
//! - **`assets`** - Asset loading: gltf, image, half, bincode, serde_json. Requires `core`.
//!
//! - **`scene_graph`** - Scene hierarchy system. Requires `assets`.
//!
//! - **`file_dialog`** - Native file dialogs using rfd. Requires `core`.
//!
//! - **`screenshot`** - PNG screenshot saving using image. Standalone feature.
//!
//! ### Optional Features
//!
//! - **`egui`** - Opt-in immediate-mode UI. Set `world.resources.egui.enabled`, then draw from
//!   any system with `egui_context(world)`. Composites over the scene and retained UI.
//!
//! - **`shell`** - Developer console with command registration, rendered via the retained UI.
//!
//! - **`audio`** - Audio playback using Kira.
//!
//! - **`physics`** - 3D physics using Rapier.
//!
//! - **`navmesh`** - Navigation mesh generation using Recast.
//!
//! - **`grass`** - GPU-driven procedural grass rendering.
//!
//! - **`terrain`** - GPU-driven clipmap terrain with procedural height generation. Implies `grass`.
//!
//! - **`picking`** - Ray-based entity picking. Trimesh picking requires `physics`.
//!
//! - **`gamepad`** - Gamepad input using gilrs.
//!
//! ### Platform Features
//!
//! - **`steam`** - Steamworks integration.
//!
//! ### Profiling Features
//!
//! - **`tracing`** - Rolling log files and `RUST_LOG` support.
//!
//! - **`tracy`** - Real-time profiling with Tracy. Implies `tracing`.
//!
//! - **`chrome`** - Chrome tracing output. Implies `tracing`.
//!
//! ## Feature Quick Reference
//!
//! ### Audio (`audio` feature)
//!
//! ```ignore
//! // Decode and register a sound by name
//! let data = load_sound_from_bytes(include_bytes!("sound.ogg")).unwrap();
//! audio_engine_load_sound(&mut world.resources.audio, "shot", data);
//!
//! // 3D spatial audio: play it through an AudioSource component
//! let source = spawn_entities(world, AUDIO_SOURCE | LOCAL_TRANSFORM | GLOBAL_TRANSFORM, 1)[0];
//! world.set(source, AudioSource::new("shot").with_spatial(true).playing());
//! world.set(source, LocalTransform::from_translation(Vec3::new(5.0, 0.0, 0.0)));
//! ```
//!
//! ### Physics (`physics` feature)
//!
//! ```ignore
//! // Dynamic rigid body with collider
//! let entity = spawn_entities(world,
//!     RIGID_BODY | COLLIDER | LOCAL_TRANSFORM | GLOBAL_TRANSFORM,
//!     1
//! )[0];
//! world.set(entity, RigidBodyComponent::new_dynamic());
//! world.set(entity, ColliderComponent::cuboid(1.0, 1.0, 1.0));
//! world.set(entity, LocalTransform::from_translation(Vec3::new(0.0, 5.0, 0.0)));
//!
//! // Static floor
//! let floor = spawn_entities(world, RIGID_BODY | COLLIDER | LOCAL_TRANSFORM | GLOBAL_TRANSFORM, 1)[0];
//! world.set(floor, RigidBodyComponent::new_static());
//! world.set(floor, ColliderComponent::cuboid(10.0, 0.1, 10.0));
//!
//! // Step physics each frame
//! fn run_systems(&mut self, world: &mut World) {
//!     physics_world_step(&mut world.resources.physics, world.resources.window.timing.delta_time);
//!     sync_physics_transforms(world);
//! }
//! ```
//!
//! ### Navigation Mesh (`navmesh` feature)
//!
//! ```ignore
//! // Generate navmesh from world geometry
//! let config = RecastNavMeshConfig::default();
//! generate_navmesh_recast(world, &config);
//!
//! // Spawn an agent
//! let agent = spawn_navmesh_agent(world, Vec3::new(0.0, 0.0, 0.0), 0.5, 2.0);
//!
//! // Set destination
//! set_agent_destination(world, agent, Vec3::new(10.0, 0.0, 10.0));
//! set_agent_speed(world, agent, 3.5);
//!
//! // Update each frame
//! fn run_systems(&mut self, world: &mut World) {
//!     run_navmesh_systems(world, world.resources.window.timing.delta_time);
//! }
//! ```
//!
//! ### Entity Picking (`picking` feature)
//!
//! ```ignore
//! // Bounding-box picking under the cursor
//! let mouse_pos = world.resources.input.mouse.position;
//! if let Some(hit) = pick_closest_entity(world, mouse_pos) {
//!     println!("Hit {:?} at distance {}", hit.entity, hit.distance);
//! }
//!
//! // GPU picking (pixel-perfect)
//! world.resources.gpu_picking.request_pick(mouse_pos.x as u32, mouse_pos.y as u32);
//! // Next frame:
//! if let Some(result) = world.resources.gpu_picking.take_result() {
//!     let world_pos = result.world_position;
//!     let normal = result.world_normal;
//! }
//! ```
//!
//! ### Developer Console (`shell` feature)
//!
//! ```ignore
//! use nightshade::shell::{Command, ShellState};
//!
//! fn spawn_cubes(args: &[&str], world: &mut World, _ctx: &mut ()) -> String {
//!     let count: usize = args.first().and_then(|s| s.parse().ok()).unwrap_or(1);
//!     for _ in 0..count {
//!         spawn_cube_at(world, Vec3::new(rand::random(), rand::random(), rand::random()));
//!     }
//!     format!("Spawned {} cubes", count)
//! }
//!
//! fn initialize(shell: &mut ShellState<()>) {
//!     shell.register_builtin_commands();
//!     shell.registry.register(Command {
//!         name: "spawn",
//!         description: "Spawn random cubes",
//!         usage: "spawn [count]",
//!         execute: spawn_cubes,
//!     });
//! }
//! // Press Alt+C to open console, type "spawn 10"
//! ```
//!
//! ### Immediate Mode UI (`egui` feature)
//!
//! Opt in with the `egui` feature and set `world.resources.egui.enabled`. The pass opens
//! before the app stages each frame, so draw from any system by pulling the frame's context
//! with `egui_context`. The tessellated output composites over the scene and the retained UI.
//!
//! ```ignore
//! impl Plugin for DebugUiPlugin {
//!     fn build(&self, app: &mut App) {
//!         app.world.resources.egui.enabled = true;
//!         app.add_system("update", "debug_ui", |world| {
//!             if let Some(ctx) = egui_context(world) {
//!                 egui::Window::new("Debug").show(&ctx, |ui| {
//!                     ui.label(format!("FPS: {:.0}", world.resources.window.timing.frames_per_second));
//!                     if ui.button("Spawn Cube").clicked() {
//!                         spawn_cube_at(world, Vec3::zeros());
//!                     }
//!                 });
//!             }
//!         });
//!     }
//! }
//! ```
//!
//! ### Gamepad Input (`gamepad` feature)
//!
//! ```ignore
//! fn tick(world: &mut World) {
//!     if let Some(gamepad) = &world.resources.input.gamepad {
//!         // Analog sticks
//!         let move_x = gamepad.left_stick.x;
//!         let move_y = gamepad.left_stick.y;
//!         let look_x = gamepad.right_stick.x;
//!         let look_y = gamepad.right_stick.y;
//!
//!         // Triggers (0.0 - 1.0)
//!         let accelerate = gamepad.right_trigger;
//!         let brake = gamepad.left_trigger;
//!
//!         // Buttons
//!         if gamepad.pressed(GamepadButton::South) { /* A/Cross */ }
//!         if gamepad.just_pressed(GamepadButton::North) { /* Y/Triangle */ }
//!     }
//! }
//! ```
//!
//! ### Steam Integration (`steam` feature)
//!
//! The engine pumps Steam callbacks every frame; systems only initialize
//! and use the API.
//!
//! ```ignore
//! fn initialize(world: &mut World) {
//!     steam_initialize(&mut world.resources.steam).ok();
//!     steam_request_stats(&mut world.resources.steam);
//! }
//!
//! fn tick(game: &mut MyGame, world: &mut World) {
//!     // Unlock achievement
//!     if game.score >= 1000 {
//!         steam_unlock_achievement(&world.resources.steam, "ACH_HIGH_SCORE").ok();
//!         steam_store_stats(&world.resources.steam).ok();
//!     }
//!
//!     // Update stat
//!     steam_set_stat_int(&world.resources.steam, "total_score", game.score).ok();
//! }
//! ```
//!
//! ### Debug Lines
//!
//! ```ignore
//! // Create debug lines entity
//! let debug = spawn_entities(world, LINES, 1)[0];
//!
//! fn tick(game: &mut MyGame, world: &mut World) {
//!     if let Some(lines) = world.get_mut::<crate::ecs::lines::components::Lines>(game.debug) {
//!         lines.clear();
//!
//!         // Draw a line
//!         lines.push(Line {
//!             start: Vec3::zeros(),
//!             end: Vec3::new(1.0, 1.0, 1.0),
//!             color: Vec4::new(1.0, 0.0, 0.0, 1.0),
//!         });
//!
//!         // Draw coordinate axes
//!         lines.push(Line { start: Vec3::zeros(), end: Vec3::x(), color: Vec4::new(1.0, 0.0, 0.0, 1.0) });
//!         lines.push(Line { start: Vec3::zeros(), end: Vec3::y(), color: Vec4::new(0.0, 1.0, 0.0, 1.0) });
//!         lines.push(Line { start: Vec3::zeros(), end: Vec3::z(), color: Vec4::new(0.0, 0.0, 1.0, 1.0) });
//!     }
//! }
//! ```
//!
//! ### Particles
//!
//! ```ignore
//! let emitter = spawn_entities(world,
//!     PARTICLE_EMITTER | LOCAL_TRANSFORM | GLOBAL_TRANSFORM,
//!     1
//! )[0];
//!
//! world.set(emitter, ParticleEmitter {
//!     spawn_rate: 100.0,
//!     particle_lifetime_min: 1.5,
//!     particle_lifetime_max: 2.0,
//!     size_start: 0.2,
//!     size_end: 0.0,
//!     color_gradient: ColorGradient::fire(),
//!     shape: EmitterShape::Sphere { radius: 0.5 },
//!     ..Default::default()
//! });
//!
//! world.set(emitter, LocalTransform::from_translation(Vec3::new(0.0, 0.0, 0.0)));
//!
//! // Runs automatically via frame schedule
//! ```
//!
//! ### Decals
//!
//! ```ignore
//! let decal = spawn_entities(world,
//!     DECAL | LOCAL_TRANSFORM | GLOBAL_TRANSFORM,
//!     1
//! )[0];
//!
//! world.set(decal, Decal {
//!     texture: Some("decal_texture".to_string()),
//!     size: Vec2::new(2.0, 2.0),
//!     depth: 1.0,  // projection depth
//!     ..Default::default()
//! });
//!
//! // Position and orient decal (projects along -Y)
//! world.set(decal, LocalTransform {
//!     translation: hit_position + hit_normal * 0.01,
//!     rotation: UnitQuaternion::face_towards(&(-hit_normal), &Vec3::x()),
//!     ..Default::default()
//! });
//! ```
//!
//! ## Minimal Example
//!
//! For a lightweight app without asset loading:
//!
//! ```toml
//! nightshade = { default-features = false, features = ["runtime", "wgpu"] }
//! ```
//!
//! ## Example
//!
//! ```no_run
//! use nightshade::prelude::*;
//!
//! struct GamePlugin;
//!
//! impl Plugin for GamePlugin {
//!     fn build(&self, app: &mut App) {
//!         app.world.resources.window.title = "My Game".to_string();
//!     }
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(GamePlugin)
//!         .run()
//! }
//! ```
//!
//! ## Frame Lifecycle
//!
//! Each frame executes in this order:
//!
//! 1. Process window/input events
//! 2. Update keyboard/mouse/gamepad state
//! 3. Calculate delta time
//! 4. Run the app stages (`first`, `update`, `late_update`)
//! 5. Dispatch event bus messages
//! 6. Run [`FrameSchedule`](schedule::FrameSchedule) (engine systems):
//!    - Initialize and update audio (if `audio` enabled)
//!    - Update camera aspect ratios
//!    - Step physics simulation (if `physics` enabled)
//!    - Update animation players, apply to transforms
//!    - Propagate transform hierarchy
//!    - Update instanced mesh caches, sync text
//!    - Reset input state
//!    - Process deferred commands, cleanup
//! 7. Execute render graph passes
//!
//! The frame schedule is a resource at `world.resources.schedules.frame`, with
//! the retained UI sub-schedule at `world.resources.schedules.retained_ui`. Modify
//! either from a startup system to insert, remove, or
//! reorder engine systems. See [`system_names`](schedule::system_names) and
//! [`retained_ui_system_names`](schedule::retained_ui_system_names) for the
//! available system name constants.
//!
//! ## Threading Model
//!
//! The engine runs on a single thread with async GPU operations via wgpu
//! and fixed timestep physics with interpolation for smooth rendering.
//!
//! ## Platform Notes
//!
//! - **Native** (Windows/macOS/Linux): Full feature support with DirectX 12, Metal, or Vulkan.
//! - **WebAssembly**: WebGPU backend. Files via drag-and-drop or HTTP. No WASI plugins. Async initialization.

pub mod filesystem;

#[cfg(feature = "runtime")]
pub mod ecs;
pub mod logging;
#[cfg(feature = "runtime")]
pub use nightshade_renderer as render;
#[cfg(feature = "runtime")]
pub mod app;
pub mod render_driver;
pub mod run;
#[cfg(feature = "runtime")]
pub mod schedule;
#[cfg(feature = "runtime")]
pub mod state;
#[cfg(feature = "runtime")]
mod systems;

#[cfg(feature = "networking")]
pub use hearsay as networking;

#[cfg(feature = "shell")]
pub mod shell;

#[cfg(all(feature = "steam", not(target_arch = "wasm32")))]
pub mod steam;

#[cfg(all(windows, feature = "windows-app-icon"))]
pub mod build;

#[cfg(feature = "runtime")]
pub use systems::*;

/// Common re-exports for application code.
///
/// Most apps will want:
///
/// ```ignore
/// use nightshade::prelude::*;
/// ```
///
/// The prelude pulls in [`World`], [`Entity`], component bitflags, the
/// [`App`](crate::app::App) builder with its `Plugin` trait (plus the
/// `State` trait escape hatch), math types from `nalgebra_glm`, schedule helpers, command
/// queue free functions ([`queue_ecs_command`], [`queue_render_command`]),
/// spawn helpers ([`spawn_cube_at`], [`spawn_sphere_at`], [`spawn_sun`],
/// [`spawn_camera`], etc.), input types, transform helpers, material and
/// mesh registry helpers, and retained UI builders.
///
/// Items not in the prelude live in their submodules and can be imported
/// individually when needed.
#[cfg(feature = "runtime")]
pub mod prelude {
    #[cfg(feature = "physics")]
    pub use crate::app::PhysicsPlugin;
    pub use crate::app::{
        App, DefaultPlugins, Plugin, PluginGroup, RenderPlugin, UiPlugin, WindowPlugin, game_scope,
    };
    pub use crate::ecs::window::resources::{
        Monitor, Monitors, refresh_monitors_from_event_loop, window_scale_factor,
        window_viewport_size,
    };
    pub use crate::ecs::world::components::*;
    pub use crate::ecs::world::resources::*;
    pub use crate::ecs::world::*;
    pub use crate::logging::{LogConfig, LogRotation};
    pub use crate::run::*;
    pub use crate::schedule::{
        FrameSchedule, FrameScheduleEntry, Schedules, SystemFn, build_default_retained_ui_schedule,
        retained_ui_system_names, run_retained_ui_schedule, schedule_contains,
        schedule_insert_after, schedule_insert_before, schedule_push, schedule_remove,
        schedule_run, system_names,
    };
    pub use crate::state::{NextStateBuilder, RenderResources, State};
    pub use crate::systems::*;
    pub use web_time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    pub const LOGO_BYTES: &[u8] = include_bytes!("../images/logo.png");

    pub use crate::ecs::animation::ik::{look_at_bone, solve_two_bone_ik};
    pub use crate::ecs::animation::systems::{apply_animations, update_animation_players};
    #[cfg(feature = "audio")]
    pub use crate::ecs::audio::resources::{
        audio_engine_cleanup_entity, audio_engine_fade_volume, audio_engine_get_sound,
        audio_engine_has_handle, audio_engine_initialize, audio_engine_is_initialized,
        audio_engine_load_sound, audio_engine_pause_sound, audio_engine_resume_sound,
        audio_engine_stop_sound,
    };
    #[cfg(feature = "audio")]
    pub use crate::ecs::audio::systems::{
        load_sound_from_bytes, set_audio_bus_volume, set_voice_ducking,
    };
    pub use crate::ecs::bounding_volume::systems::ensure_bounding_volumes;
    pub use crate::ecs::camera::commands::{
        spawn_camera, spawn_ortho_camera, spawn_pan_orbit_camera, spawn_third_person_camera,
    };
    pub use crate::ecs::camera::components::ThirdPersonCamera;
    pub use crate::ecs::camera::systems::{
        camera_controllers_system, first_person_camera_look_system, fly_camera_system,
        ortho_camera_system, pan_orbit_camera_system, third_person_camera_system,
    };
    pub use crate::ecs::cloth::commands::{reset_cloth, spawn_cloth};
    pub use crate::ecs::cloth::systems::sync_cloth_meshes_system;
    pub use crate::ecs::cutscene::{
        Cutscene, CutsceneAction, CutsceneDirector, CutsceneEvent, CutsceneShot,
        advance_cutscene_system, bind_cutscene_actor, camera_look_rotation, cutscene_finished,
        cutscene_playing, cutscene_progress, cutscene_time, pause_cutscene, play_cutscene,
        play_cutscene_reel, queue_cutscene, resume_cutscene, seek_cutscene, set_cutscene_camera,
        set_cutscene_localization, stop_cutscene, take_cutscene_markers,
    };
    #[cfg(feature = "scene_graph")]
    pub use crate::ecs::entity_registry::{assign_guid, ensure_guid, find_entity_by_guid};
    pub use crate::ecs::event::{
        Event, JournalEntry, cycle, drain_journal, emit_event, events, journal_enabled,
        set_journal_enabled,
    };
    pub use crate::ecs::event_bus::commands::{publish_app_event, publish_event};
    pub use crate::ecs::event_bus::resources::Message;
    pub use crate::ecs::gpu_picking::{
        GpuPickRequest, GpuPickResult, GpuPicking, surface_pick_coords,
    };
    #[cfg(feature = "gamepad")]
    pub use crate::ecs::input::queries::query_active_gamepad;
    pub use crate::ecs::input::resources::input_mode::InputMode;
    pub use crate::ecs::input::resources::mouse::MouseState;
    pub use crate::ecs::input::resources::touch::{TouchGesture, TouchPhase};
    #[cfg(feature = "gamepad")]
    pub use crate::ecs::input::systems::detect_input_mode_system;
    pub use crate::ecs::input::systems::{apply_ime_allowed_system, set_ime_allowed};
    pub use crate::ecs::loading::{
        LoadingPipeline, loading_pipeline_acquire_recipe_sources, loading_pipeline_acquire_source,
        loading_pipeline_allocate_source_image, loading_pipeline_completed,
        loading_pipeline_fraction, loading_pipeline_is_active, loading_pipeline_last_category,
        loading_pipeline_last_label, loading_pipeline_mark_completed, loading_pipeline_pending_len,
        loading_pipeline_pop, loading_pipeline_push, loading_pipeline_release_recipe_sources,
        loading_pipeline_release_source, loading_pipeline_reset_source_arena,
        loading_pipeline_total, queue_decoded_texture,
    };
    #[cfg(feature = "assets")]
    pub use crate::ecs::loading::{
        load_texture_from_image_bytes, load_texture_pack_from_image_bytes,
    };
    pub use crate::ecs::material::resources::{material_registry_insert, material_registry_mutate};
    pub use crate::ecs::mesh::commands::spawn_mesh;
    pub use crate::ecs::mesh::systems::update_instanced_mesh_caches_system;
    pub use crate::ecs::navmesh::{NavMeshAgent, NavMeshAgentState};
    #[cfg(feature = "navmesh")]
    pub use crate::ecs::navmesh::{
        NavMeshConnection, NavMeshEdge, NavMeshTriangle, NavMeshWorld, PathRequest, PathResult,
        PathStatus, PathfindingAlgorithm, SpatialHash, clear_navmesh,
        find_closest_point_on_navmesh, get_agent_path_length, get_agent_state, run_navmesh_systems,
        sample_navmesh_height, set_agent_destination, set_agent_speed, set_navmesh_debug_draw,
        spawn_navmesh_agent, stop_agent,
    };
    #[cfg(feature = "navmesh-bake")]
    pub use crate::ecs::navmesh::{RecastNavMeshConfig, generate_navmesh_recast};
    pub use crate::ecs::particles::systems::update_particle_emitters;
    #[cfg(feature = "physics")]
    pub use crate::ecs::physics::PhysicsWorld;
    #[cfg(feature = "physics")]
    pub use crate::ecs::physics::commands::{
        create_textured_material, reset_physics_interpolation,
        spawn_dynamic_physics_cube_with_material, spawn_dynamic_physics_cylinder_with_material,
        spawn_dynamic_physics_sphere_with_material, spawn_static_physics_cube_with_material,
    };
    #[cfg(all(feature = "physics", feature = "debug_render"))]
    pub use crate::ecs::physics::debug::physics_debug_draw_system;
    #[cfg(feature = "physics")]
    pub use crate::ecs::physics::events::{CollisionEvent, CollisionEventKind, ContactForceEvent};
    #[cfg(feature = "physics")]
    pub use crate::ecs::physics::resources::{
        physics_world_angular_velocity, physics_world_apply_force, physics_world_apply_impulse,
        physics_world_apply_impulse_at_point, physics_world_apply_torque,
        physics_world_body_rotation, physics_world_body_translation,
        physics_world_capsule_contacts, physics_world_cast_ray, physics_world_collision_events,
        physics_world_contact_force_events, physics_world_get_interpolation_alpha,
        physics_world_linear_velocity, physics_world_overlap_sphere,
        physics_world_set_additional_mass, physics_world_set_angular_damping,
        physics_world_set_angular_velocity, physics_world_set_friction,
        physics_world_set_gravity_scale, physics_world_set_linear_damping,
        physics_world_set_linear_velocity, physics_world_set_restitution, physics_world_step,
    };
    #[cfg(feature = "physics")]
    pub use crate::ecs::physics::types::{RaycastHit, ShapeContact};
    pub use crate::ecs::physics::{
        CharacterControllerComponent, ColliderComponent, ColliderShape, CollisionListener,
        PhysicsInterpolation, RigidBodyComponent,
    };
    #[cfg(feature = "picking")]
    pub use crate::ecs::picking::{
        PickingOptions, PickingRay, PickingResult, get_ground_position_from_screen,
        pick_closest_entity, pick_entities, pick_entities_in_frustum,
    };
    #[cfg(all(feature = "picking", feature = "physics"))]
    pub use crate::ecs::picking::{
        PickingWorld, pick_closest_entity_trimesh, pick_entities_trimesh,
        register_entity_for_trimesh_picking, register_entity_hierarchy_for_trimesh_picking,
        unregister_entity_from_picking, update_picking_transform,
    };
    #[cfg(feature = "assets")]
    pub use crate::ecs::prefab::commands::import_gltf_from_bytes;
    #[cfg(feature = "assets")]
    pub use crate::ecs::prefab::commands::import_gltf_with_resources;
    pub use crate::ecs::prefab::commands::spawn_prefab_with_animations;
    pub use crate::ecs::primitives::EasingFunction;
    pub use crate::ecs::primitives::Guid;
    #[cfg(feature = "scene_graph")]
    pub use crate::ecs::scene::SceneNavMesh;
    #[cfg(feature = "scene_graph")]
    pub use crate::ecs::scene::registry::{
        AssetRegistry, RuntimeAssetEntry, RuntimeAssetRegistry, asset_registry_add_manifest,
        asset_registry_all_assets, asset_registry_asset_count, asset_registry_assets_of_type,
        asset_registry_clear, asset_registry_get_base_path, asset_registry_get_entry,
        asset_registry_manifest_count, asset_registry_name_exists, asset_registry_path_exists,
        asset_registry_resolve_name, asset_registry_resolve_path, asset_registry_resolve_uuid,
        asset_registry_uuid_exists, runtime_asset_registry_clear, runtime_asset_registry_get,
        runtime_asset_registry_get_by_name, runtime_asset_registry_is_loaded,
        runtime_asset_registry_mark_loaded, runtime_asset_registry_mark_unloaded,
        runtime_asset_registry_register, runtime_asset_registry_resolve_name,
        runtime_asset_registry_unregister,
    };
    #[cfg(feature = "scene_graph")]
    pub use crate::ecs::scene::{
        AssetMode, AssetUuid, ComponentDescriptor, ComponentTypeRegistry, FieldDescriptor,
        FieldKind, Persistence, Scene, SceneAlphaMode, SceneAudioSource, SceneBodyType,
        SceneCamera, SceneCollider, SceneComponents, SceneDeserializer, SceneEntity, SceneError,
        SceneHdrSkybox, SceneInstancedMesh, SceneLight, SceneMaterial, SceneMesh,
        SceneMeshInstance, ScenePhysics, ScenePrefabInstance, SceneSerializer, SpawnSceneResult,
        TypedComponent, TypedPayload, TypedResource, apply_field_overrides, component_descriptor,
        load_scene, register_component_descriptor, save_scene, spawn_scene,
        spawn_scene_with_deserializer, world_to_scene, world_to_scene_with_serializer,
    };
    pub use crate::ecs::snapshot::{WorldSnapshot, restore_world, snapshot_world};
    #[cfg(feature = "assets")]
    pub use crate::ecs::snapshot::{load_world_snapshot_from_bytes, save_world_snapshot_to_bytes};
    #[cfg(all(feature = "terrain", feature = "physics"))]
    pub use crate::ecs::terrain::terrain_collider_system;
    pub use crate::ecs::text::commands::{
        load_font_from_bytes, spawn_ui_text, spawn_ui_text_with_properties,
    };
    pub use crate::ecs::text::components::TextProperties;
    #[cfg(feature = "assets")]
    pub use crate::ecs::texture_loader::{
        AssetLoadingState, AssetLoadingStatus, LoadedTexture, SharedTextureQueue, TextureLoadQueue,
        TextureLoadRequest, create_shared_queue, get_asset_search_paths, process_and_load_textures,
        process_texture_queue, queue_texture_from_path, set_asset_search_paths,
    };
    pub use crate::ecs::timer::{Stopwatch, Timer, TimerMode};
    pub use crate::ecs::tween::{Shake, Tween, TweenTarget, Tweens, update_tweens_system};
    #[cfg(feature = "egui")]
    pub use crate::ecs::ui::egui_state::egui_context;
    pub use crate::ecs::ui::icons::{self, IconGlyph, IconLookup, IconSet};
    pub use crate::ecs::ui::layout_types::GridLayout;
    pub use crate::ecs::ui::layout_types::{
        BoundaryLayout, FlowAlignment, FlowDirection, FlowLayout, ResponsiveFlowOverride,
        SolidLayout, WindowLayout,
    };
    pub use crate::ecs::ui::resources::{
        BubbledUiEvent, DrawRectParams, NavDirection, OverlayText, ReservedAreas,
        RetainedUiGamepadNav, ToastCorner, ToastSeverity, UiBreakpoint, UiEvent, UiViewport,
        WindowChrome, WindowShortcut, blend_color,
    };
    pub use crate::ecs::ui::testing::{
        UiTestDriver, UiTestResult, create_test_root, setup_test_scene,
    };
    pub use crate::ecs::ui::theme::{ThemeState, UiTheme};
    #[cfg(feature = "assets")]
    pub use crate::ecs::ui::widgets::world_layout::UiImageUpload;
    pub use crate::ecs::ui::widgets::world_layout::open_url_in_browser;
    pub use crate::ecs::ui::{
        Ab, AutoSizeMode, CanvasCommand, CharStyle, ColorPickerMode, ContextMenuBuilder,
        DataGridColumn, DragAcceptFilter, DragValueConfig, Em, InputMask, Length, LengthExt,
        ResizeEdge, Rh, Rl, Rw, ScalingMode, SliderConfig, SplitDirection, StateTransition,
        TextOverflow, TextSpan, ThemeColor, TileBuilder, TileId, TileLayout, TileLayoutNode,
        TileNode, Ui, UiAnimationPhase, UiAnimationType, UiBase, UiBreadcrumbData, UiButtonData,
        UiCanvasData, UiCheckboxData, UiCollapsingHeaderData, UiColorPickerData,
        UiCommandPaletteData, UiContextMenuData, UiDataGridData, UiDatePickerData, UiDepthMode,
        UiDisabled, UiDragValueData, UiDropdownData, UiFocused, UiHover, UiLayoutNode,
        UiLayoutRoot, UiLayoutType, UiMenuData, UiModalDialogData, UiMultiSelectData,
        UiNodeAnimation, UiNodeColor, UiNodeContent, UiNodeInteraction, UiPanelData, UiPanelKind,
        UiPressed, UiProgressBarData, UiPropertyGridData, UiRadioData, UiRangeSliderData,
        UiRichTextData, UiRichTextEditorData, UiScrollAreaData, UiSelectableLabelData, UiSelected,
        UiSliderData, UiSplitterData, UiStateTrait, UiStateWeights, UiTabBarData, UiTextAreaData,
        UiTextInputData, UiTileContainerData, UiToggleData, UiTreeBuilder, UiTreeNodeData,
        UiTreeViewData, UiValue, UiVirtualListData, ValidationRule, Vh, Vp, Vw,
    };
    pub use crate::ecs::vfx::components::{Beam, LightningBolt, Trail, VfxAnimator, VfxHandle};
    pub use crate::ecs::vfx::presets::{VfxFamily, VfxPreset, despawn_vfx, spawn_vfx};
    pub use crate::ecs::vfx::systems::update_vfx_system;
    pub use crate::ecs::vfx::textures::{FLARE_SLOT, GLOW_SLOT, SMOKE_SLOT, upload_vfx_textures};
    pub use crate::ecs::world::commands::{
        EcsCommand, RenderCommand, despawn_entities_with_cache_cleanup,
        despawn_recursive_immediate, is_paused, load_hdr_skybox, load_hdr_skybox_from_path,
        load_procedural_textures, pause, set_material_with_textures, set_paused, set_time_scale,
        spawn_3d_billboard_text_with_properties, spawn_3d_text_with_properties, spawn_cone_at,
        spawn_cube_at, spawn_cylinder_at, spawn_instanced_mesh, spawn_instanced_mesh_with_material,
        spawn_light_entity, spawn_material, spawn_mesh_at, spawn_plane_at, spawn_sphere_at,
        spawn_sun, spawn_sun_without_shadows, spawn_torus_at, time_scale, unpause,
    };
    pub use crate::render::bounding_volume::BoundingVolume;
    pub use crate::render::entity::RenderEntity;
    pub use crate::render::generational_registry::{
        GenerationalRegistry, registry_add_reference, registry_clear, registry_entry,
        registry_entry_by_name, registry_entry_by_name_mut, registry_insert, registry_is_empty,
        registry_is_filled, registry_is_valid, registry_len, registry_lookup_index,
        registry_name_for, registry_reference_count, registry_remove, registry_remove_reference,
        registry_remove_unused, registry_reserve_name,
    };
    #[cfg(feature = "grass")]
    pub use crate::render::grass_config::{
        GrassDomain, GrassHeightMap, GrassPusher, GrassSettings, GrassTypeParams,
    };
    pub use crate::render::mesh_cache::{mesh_cache_insert, mesh_cache_remove_unused};
    #[cfg(feature = "terrain")]
    pub use crate::render::terrain_config::{TERRAIN_PICK_ID, TerrainRenderState, TerrainSettings};
    pub use crate::render::text_data::FontKind;
    pub use crate::render::text_data::{TextAlignment, VerticalAlignment};
    pub use crate::render::ui_data::{Anchor, Rect, UiImage, UiLayer, UiRect, UiTextInstance};
    pub use crate::render::wgpu::texture_cache::{
        TextureOwner, texture_cache_acquire, texture_cache_protect, texture_cache_release,
        texture_cache_release_entity,
    };
    pub use crate::render_driver::{render_entity, scene_entity};
    #[cfg(feature = "assets")]
    pub use bincode;
    #[cfg(feature = "egui")]
    pub use egui;
    #[cfg(feature = "engine")]
    pub use ehttp;
    pub use freecs;
    #[cfg(feature = "assets")]
    pub use image;
    #[cfg(feature = "engine")]
    pub use rand::{self, Rng};
    pub use serde;
    #[cfg(feature = "assets")]
    pub use serde_json;
    pub use tracing;
    pub use uuid;
    pub use winit;
    pub use winit::{
        event::{ElementState, ElementState as KeyState, MouseButton, WindowEvent},
        keyboard::KeyCode,
        window,
    };

    #[cfg(all(not(target_arch = "wasm32"), feature = "file_dialog"))]
    pub use dirs;
    #[cfg(all(not(target_arch = "wasm32"), feature = "file_dialog"))]
    pub use rfd;

    #[cfg(target_arch = "wasm32")]
    pub use js_sys;
    #[cfg(target_arch = "wasm32")]
    pub use wasm_bindgen;
    #[cfg(target_arch = "wasm32")]
    pub use wasm_bindgen_futures;
    #[cfg(target_arch = "wasm32")]
    pub use web_sys;

    #[cfg(feature = "wgpu")]
    pub use crate::render::wgpu::WgpuRenderer;
    #[cfg(feature = "wgpu")]
    pub use crate::render::wgpu::passes;
    #[cfg(feature = "wgpu")]
    pub use crate::render::wgpu::render_configs::RenderInputs;
    #[cfg(feature = "wgpu")]
    pub use crate::render::wgpu::rendergraph::{
        PassExecutionContext, PassNode, RenderGraph, ResourceId, render_graph_add_buffer,
        render_graph_add_color_texture, render_graph_add_dependency,
        render_graph_add_depth_texture, render_graph_add_pass, render_graph_add_pass_with_slots,
        render_graph_add_sub_graph, render_graph_compile, render_graph_execute,
        render_graph_external_color, render_graph_get_pass_mut, render_graph_get_sub_graph,
        render_graph_get_sub_graph_mut, render_graph_get_texture, render_graph_get_texture_view,
        render_graph_new, render_graph_pass, render_graph_resize_all_transient_proportional,
        render_graph_resize_all_transient_textures, render_graph_resize_transient_resource,
        render_graph_resource_pool, render_graph_resources_mut,
        render_graph_restore_pass_enabled_states, render_graph_save_pass_enabled_states,
        render_graph_set_external_buffer, render_graph_set_external_texture,
        render_graph_set_pass_enabled, render_graph_transient_color_from_template,
        render_graph_transient_color_from_template_with_clear,
    };
    #[cfg(feature = "gamepad")]
    pub use gilrs;
    pub use nalgebra_glm;
    pub use nalgebra_glm::{vec2, vec3, vec4};
    pub use wgpu;

    #[cfg(all(feature = "steam", not(target_arch = "wasm32")))]
    pub use crate::steam::{
        AchievementInfo, ActiveSession, FriendInfo, NetworkMessage, PendingSessionRequest,
        PersonaState, SessionState, StatInfo, StatValue, SteamResources,
    };
    #[cfg(all(feature = "steam", not(target_arch = "wasm32")))]
    pub use steamworks::SteamId;

    #[cfg(feature = "shell")]
    pub use crate::shell::{
        Command, CommandRegistry, OutputLine, ShellState, builtin_commands, shell_retained_ui,
    };

    #[cfg(all(feature = "file_watcher", not(target_arch = "wasm32")))]
    pub use crate::ecs::file_watcher::FileWatcher;

    // === decap re-exports (generated) ===
    pub use crate::ecs::transform::systems::{sync_hierarchy_index, update_parent};
    pub use crate::ecs::ui::widgets::world_canvas::{
        ui_canvas_arc, ui_canvas_circle, ui_canvas_circle_stroke, ui_canvas_clear,
        ui_canvas_cubic_bezier, ui_canvas_hit_test, ui_canvas_line, ui_canvas_polyline,
        ui_canvas_quadratic_bezier, ui_canvas_rect, ui_canvas_rect_stroke, ui_canvas_text,
    };
    pub use crate::ecs::ui::widgets::world_data::{
        ui_data_grid_clear_filter, ui_data_grid_populate, ui_data_grid_populate_fn,
        ui_data_grid_selection_changed, ui_data_grid_set_cell, ui_data_grid_set_filter,
        ui_data_grid_set_row_count, ui_data_grid_sort_changed, ui_data_grid_start_edit,
        ui_data_grid_stop_edit, ui_rich_text_editor_set_color, ui_rich_text_editor_set_value,
        ui_rich_text_editor_toggle_bold, ui_rich_text_editor_toggle_italic,
        ui_rich_text_editor_toggle_style, ui_rich_text_editor_toggle_underline,
        ui_text_area_set_value,
    };
    pub use crate::ecs::ui::widgets::world_interaction::{
        ui_add_validation_rule, ui_bubbled_events_for, ui_clear_error, ui_close_context_menus,
        ui_command_palette_clear, ui_command_palette_rebuild_results, ui_command_palette_register,
        ui_context_menu_get_checked, ui_context_menu_set_checked, ui_context_menu_widget_content,
        ui_drag_value_set_value, ui_has_error, ui_is_valid, ui_mark_children_dirty,
        ui_mark_layout_dirty, ui_mark_render_dirty, ui_rich_text_set_span_color,
        ui_rich_text_set_span_text, ui_set_error, ui_set_selected, ui_set_toast_corner,
        ui_set_validation_rules, ui_show_command_palette, ui_show_context_menu, ui_show_modal,
        ui_show_toast, ui_tick_toasts, ui_tree_node_mark_loaded, ui_tree_node_set_expanded,
        ui_tree_view_clear_filter, ui_tree_view_clear_selection, ui_tree_view_scroll_to_node,
        ui_tree_view_select_node, ui_tree_view_set_filter, ui_validate, ui_widget_content,
    };
    pub use crate::ecs::ui::widgets::world_layout::{
        ui_button_clicks, ui_clear_children, ui_clipboard_text, ui_despawn_node, ui_events,
        ui_events_for, ui_focus, ui_is_disabled, ui_label_text, ui_named_entity,
        ui_node_effectively_visible, ui_node_visible, ui_read_system_clipboard,
        ui_rebuild_children, ui_rect, ui_release_image_layer, ui_set_clipboard_text,
        ui_set_disabled, ui_set_disabled_recursive, ui_set_icon, ui_set_label_text, ui_set_named,
        ui_set_test_id, ui_set_text, ui_set_tooltip_entity, ui_set_tooltip_text, ui_set_visible,
        ui_set_visible_exclusive, ui_size, ui_text_slot_for_entity, ui_upload_image,
    };
    pub use crate::ecs::ui::widgets::world_reactive::{
        ui_announce, ui_announcements, ui_clicked, ui_data_grid_enable_filters,
        ui_data_grid_set_filtered_indices, ui_date_picker_set_value, ui_multi_select_set_selected,
        ui_set_accessible_label, ui_set_reduced_motion, ui_set_state_active,
    };
    pub use crate::ecs::ui::widgets::world_state::{
        measure_ui_text_width, ui_apply_dock_state, ui_breadcrumb_set_segments, ui_button_set_text,
        ui_capture_dock_state, ui_changed, ui_checkbox_changed, ui_checkbox_value,
        ui_clear_window_chrome, ui_load_dock_state_from_str, ui_register_window_shortcut,
        ui_save_dock_state_to_str, ui_set_window_menu_bar, ui_set_window_status_bar,
        ui_set_window_toolbar, ui_window_menu_bar, ui_window_status_bar, ui_window_toolbar,
    };
    pub use crate::ecs::ui::widgets::world_state::{
        ui_collapsing_header_open, ui_color_picker_changed, ui_color_picker_color,
        ui_color_picker_set_value, ui_dock_layout, ui_dock_state, ui_drag_value,
        ui_drag_value_changed, ui_dropdown_selected, ui_dropdown_selected_changed,
        ui_dropdown_set_value, ui_panel_set_header_visible, ui_panel_set_pinned,
        ui_pointer_over_widget, ui_progress_bar_set_value, ui_radio_group_value, ui_radio_selected,
        ui_range_slider_set_values, ui_reserved_areas, ui_scroll_area_set_offset,
        ui_scroll_area_set_snap, ui_slider_set_value, ui_slider_value, ui_slider_value_changed,
        ui_tab_bar_selected, ui_tab_bar_set_value, ui_text_input_changed, ui_text_input_set_mask,
        ui_text_input_set_password, ui_text_input_set_value, ui_text_input_submitted,
        ui_text_input_text, ui_toggle_changed, ui_toggle_set_value, ui_toggle_value,
        ui_tree_view_selected, ui_virtual_list_populate, ui_virtual_list_populate_fn,
        ui_virtual_list_selection_changed, ui_virtual_list_set_count, widget,
    };
    #[cfg(not(target_arch = "wasm32"))]
    pub use crate::ecs::ui::widgets::world_state::{
        ui_load_dock_state_from_path, ui_save_dock_state_to_path,
    };
    pub use crate::ecs::ui::widgets::world_tiles::{
        build_tiles, spawn_tile_pane_entity, ui_tile_active_pane, ui_tile_add_pane,
        ui_tile_add_pane_to, ui_tile_load_layout, ui_tile_pane_content, ui_tile_pane_title,
        ui_tile_remove, ui_tile_reset_to_panes, ui_tile_save_layout, ui_tile_set_active,
        ui_tile_split, ui_tile_splitter_moved, ui_tile_tab_activated, ui_tile_tab_closed,
        ui_tile_wrap_pane_in_tabs,
    };
    pub use crate::ecs::world::commands::{
        queue_ecs_command, queue_render_command, register_material, set_cursor_locked,
        set_cursor_visible, spawn_entities, spawn_entities_ui,
    };
    // === decap re-exports end ===
}
