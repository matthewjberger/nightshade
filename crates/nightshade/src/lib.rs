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
//! nightshade = "0.13"
//! ```
//!
//! ### 2. Create Your Game
//!
//! ```ignore
//! use nightshade::prelude::*;
//!
//! struct Game {
//!     camera: Option<Entity>,
//! }
//!
//! impl Default for Game {
//!     fn default() -> Self {
//!         Self { camera: None }
//!     }
//! }
//!
//! impl State for Game {
//!     fn title(&self) -> &str { "My Game" }
//!
//!     fn initialize(&mut self, world: &mut World) {
//!         // Graphics setup
//!         world.resources.graphics.atmosphere = Atmosphere::Sky;
//!         spawn_sun(world);
//!
//!         // Camera
//!         let camera = spawn_camera(world, Vec3::new(5.0, 3.0, 5.0), "Camera".to_string());
//!         world.resources.active_camera = Some(camera);
//!         self.camera = Some(camera);
//!
//!         // Spawn objects
//!         spawn_cube_at(world, Vec3::new(0.0, 0.5, 0.0));
//!     }
//!
//!     fn run_systems(&mut self, world: &mut World) {
//!         let _dt = world.resources.window.timing.delta_time;
//!     }
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     nightshade::launch(Game::default())
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
//! if let Some(player) = world.core.get_animation_player_mut(entity) {
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
//! let entity = world.spawn_entities(
//!     LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | RENDER_MESH,
//!     1
//! )[0];
//! ```
//!
//! | Flag | Description |
//! |------|-------------|
//! | `LOCAL_TRANSFORM` | Position/rotation/scale relative to parent |
//! | `GLOBAL_TRANSFORM` | World-space transform (computed) |
//! | `LOCAL_TRANSFORM_DIRTY` | Marks transform for hierarchy update |
//! | `RENDER_MESH` | Visible 3D geometry |
//! | `MATERIAL_REF` | Material reference |
//! | `CAMERA` | Camera component |
//! | `PARENT` / `CHILDREN` | Hierarchy relationships |
//! | `RIGID_BODY` | Physics body |
//! | `COLLIDER` | Collision shape |
//! | `ANIMATION_PLAYER` | Animation controller |
//! | `PARTICLE_EMITTER` | GPU particles |
//! | `AUDIO_SOURCE` | Sound playback |
//! | `WATER` | Water surface/volume |
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
//! world.resources.graphics.bloom_enabled = true;
//! world.resources.graphics.atmosphere = Atmosphere::Sky;
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
//! - Call `world.texture_cache_add_reference(path)` before material creation
//! - Paths are case-sensitive
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
//!   terrain, file dialogs, and more. This is the standard feature for building games.
//!
//! - **`runtime`** - Minimal rendering without asset loading. Use with `wgpu` and `egui` for
//!   lightweight apps that don't need gltf/image loading.
//!
//! - **`full`** - Everything in `engine` plus `audio`, `navmesh`, `scripting`, `fbx`, etc.
//!
//! - **`wgpu`** (default) - WebGPU-based rendering with DirectX 12, Metal, Vulkan, and WebGPU.
//!
//! ### Granular Features
//!
//! These allow fine-grained control over dependencies:
//!
//! - **`core`** - Foundation: ECS (freecs), math (nalgebra), windowing (winit), time, petgraph.
//!
//! - **`text`** - MSDF text rendering using ttf-parser. Requires `core`.
//!
//! - **`assets`** - Asset loading: gltf, image, half, bincode, serde_json. Requires `core`.
//!
//! - **`scene_graph`** - Scene hierarchy system. Requires `assets`.
//!
//! - **`terrain`** - Procedural terrain generation using noise. Requires `core`.
//!
//! - **`file_dialog`** - Native file dialogs using rfd. Requires `core`.
//!
//! - **`async_runtime`** - Tokio async runtime. Requires `core`. Falls back to pollster if disabled.
//!
//! - **`screenshot`** - PNG screenshot saving using image. Standalone feature.
//!
//! ### Optional Features
//!
//! - **`egui`** - Immediate mode GUI. Enables `fn ui()` on State trait.
//!
//! - **`shell`** - Developer console with command registration. Requires `egui`.
//!
//! - **`audio`** - Audio playback using Kira.
//!
//! - **`physics`** - 3D physics using Rapier.
//!
//! - **`navmesh`** - Navigation mesh generation using Recast.
//!
//! - **`scripting`** - Rhai scripting runtime.
//!
//! - **`picking`** - Ray-based entity picking. Trimesh picking requires `physics`.
//!
//! - **`gamepad`** - Gamepad input using gilrs.
//!
//! - **`fbx`** - FBX model loading using ufbx. Requires `assets`.
//!
//! - **`lattice`** - Lattice deformation system.
//!
//! - **`sdf_sculpt`** - SDF sculpting system.
//!
//! ### Platform Features
//!
//! - **`openxr`** - OpenXR VR support.
//!
//! - **`steam`** - Steamworks integration.
//!
//! - **`webview`** - Bidirectional IPC for hosting web frontends (Leptos, Yew, etc.).
//!   On native (with `runtime` + `egui`), provides `WebviewContext` and `serve_embedded_dir`.
//!   On WASM, provides `connect` and `send` for the frontend bridge.
//!
//! ### Profiling Features
//!
//! - **`tracing`** - Rolling log files and `RUST_LOG` support.
//!
//! - **`tracy`** - Real-time profiling with Tracy. Implies `tracing`.
//!
//! - **`chrome`** - Chrome tracing output. Implies `tracing`.
//!
//! ### Plugin Features
//!
//! - **`plugins`** - Guest-side WASM plugin API.
//!
//! - **`plugin_runtime`** - WASM plugin hosting via Wasmtime. Requires `assets`.
//!
//! - **`mosaic`** - Multi-pane desktop application framework with tile-based layouts,
//!   themes, modals, notifications, and built-in viewport widgets. Requires `egui`.
//!
//! - **`editor`** - Scene editor infrastructure with gizmos, undo/redo, component inspectors,
//!   entity picking, selection management, and scene tree UI. Requires `mosaic` and `picking`.
//!
//! - **`mcp`** - HTTP-based Model Context Protocol server on `http://127.0.0.1:3333/mcp`.
//!   Exposes 50+ tools for AI clients to manipulate the running scene. Native only.
//!   Requires `async_runtime` and `behaviors`.
//!
//! - **`claude`** - Background worker for spawning Claude Code CLI as a subprocess and
//!   streaming its JSON output. Lets applications embed an AI chat interface. Native only.
//!
//! ## Feature Quick Reference
//!
//! ### Audio (`audio` feature)
//!
//! ```ignore
//! // Load and play a sound
//! let sound = world.resources.audio.load_sound(include_bytes!("sound.ogg"));
//! world.resources.audio.play(sound);
//!
//! // 3D spatial audio
//! let source = world.spawn_entities(AUDIO_SOURCE | LOCAL_TRANSFORM | GLOBAL_TRANSFORM, 1)[0];
//! world.core.set_audio_source(source, AudioSource::new(sound).with_spatial(true));
//! world.core.set_local_transform(source, LocalTransform::from_translation(Vec3::new(5.0, 0.0, 0.0)));
//! ```
//!
//! ### Physics (`physics` feature)
//!
//! ```ignore
//! // Dynamic rigid body with collider
//! let entity = world.spawn_entities(
//!     RIGID_BODY | COLLIDER | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY,
//!     1
//! )[0];
//! world.core.set_rigid_body(entity, RigidBodyComponent::new_dynamic());
//! world.core.set_collider(entity, ColliderComponent::cuboid(1.0, 1.0, 1.0));
//! world.core.set_local_transform(entity, LocalTransform::from_translation(Vec3::new(0.0, 5.0, 0.0)));
//!
//! // Static floor
//! let floor = world.spawn_entities(RIGID_BODY | COLLIDER | LOCAL_TRANSFORM | GLOBAL_TRANSFORM, 1)[0];
//! world.core.set_rigid_body(floor, RigidBodyComponent::new_static());
//! world.core.set_collider(floor, ColliderComponent::cuboid(10.0, 0.1, 10.0));
//!
//! // Step physics each frame
//! fn run_systems(&mut self, world: &mut World) {
//!     world.resources.physics.step(world.resources.window.timing.delta_time);
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
//! // Ray picking (bounding box)
//! let mouse_pos = world.resources.input.mouse.position;
//! let ray = PickingRay::from_screen_position(world, mouse_pos);
//! if let Some(hit) = pick_closest_entity(world, &ray, &PickingOptions::default()) {
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
//! ### Immediate Mode GUI (`egui` feature)
//!
//! ```ignore
//! impl State for Game {
//!     fn ui(&mut self, world: &mut World, ctx: &egui::Context) {
//!         egui::Window::new("Debug").show(ctx, |ui| {
//!             ui.label(format!("FPS: {:.0}", world.resources.window.timing.frames_per_second));
//!             if ui.button("Spawn Cube").clicked() {
//!                 spawn_cube_at(world, Vec3::zeros());
//!             }
//!             ui.add(egui::Slider::new(&mut self.speed, 0.0..=10.0).text("Speed"));
//!         });
//!     }
//! }
//! ```
//!
//! ### Developer Console (`shell` feature)
//!
//! ```ignore
//! fn initialize(&mut self, world: &mut World) {
//!     // Register custom command
//!     world.resources.shell.register_command(
//!         "spawn",
//!         "Spawn entities",
//!         |args, world| {
//!             let count: usize = args.get(0).and_then(|s| s.parse().ok()).unwrap_or(1);
//!             for _ in 0..count {
//!                 spawn_cube_at(world, Vec3::new(rand::random(), rand::random(), rand::random()));
//!             }
//!             Ok(format!("Spawned {} cubes", count))
//!         },
//!     );
//! }
//! // Press ` (backtick) to open console, type "spawn 10"
//! ```
//!
//! ### Procedural Terrain (`terrain` feature)
//!
//! ```ignore
//! let config = TerrainConfig {
//!     size: 100.0,
//!     resolution: 128,
//!     height_scale: 10.0,
//!     noise: NoiseConfig {
//!         noise_type: NoiseType::Perlin,
//!         frequency: 0.02,
//!         octaves: 4,
//!         ..Default::default()
//!     },
//! };
//! let terrain = spawn_terrain_with_material(world, &config, material_id);
//! ```
//!
//! ### Lattice Deformation (`lattice` feature)
//!
//! ```ignore
//! // Create lattice covering a region
//! let lattice = create_lattice_entity(
//!     world,
//!     Vec3::new(-2.0, 0.0, -2.0),  // Min bounds
//!     Vec3::new(2.0, 4.0, 2.0),    // Max bounds
//!     [3, 4, 3],                    // Grid dimensions
//! );
//!
//! // Bind mesh to lattice
//! register_entity_for_lattice_deformation(world, mesh_entity, lattice);
//!
//! // Deform by moving control points
//! if let Some(lat) = world.core.get_lattice_mut(lattice) {
//!     lat.set_displacement(1, 3, 1, Vec3::new(0.5, 0.0, 0.0));
//! }
//!
//! // Apply deformations each frame
//! lattice_deformation_system(world);
//! ```
//!
//! ### Gamepad Input (`gamepad` feature)
//!
//! ```ignore
//! fn run_systems(&mut self, world: &mut World) {
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
//! ### OpenXR VR (`openxr` feature)
//!
//! ```ignore
//! // Launch in VR mode
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     launch_xr(Game::default())
//! }
//!
//! // Read VR input
//! fn run_systems(&mut self, world: &mut World) {
//!     if let Some(input) = &world.resources.xr.input {
//!         let head_pos = input.head_position;
//!         let left_hand = input.left_hand_position();
//!         let right_trigger = input.right_trigger;
//!
//!         if input.a_button_pressed() {
//!             // A button action
//!         }
//!     }
//!
//!     // Enable locomotion
//!     world.resources.xr.locomotion_enabled = true;
//!     world.resources.xr.locomotion_speed = 3.0;
//! }
//! ```
//!
//! ### Steam Integration (`steam` feature)
//!
//! ```ignore
//! fn initialize(&mut self, world: &mut World) {
//!     world.resources.steam.initialize().ok();
//!     world.resources.steam.request_stats();
//! }
//!
//! fn run_systems(&mut self, world: &mut World) {
//!     world.resources.steam.run_callbacks();
//!
//!     // Unlock achievement
//!     if self.score >= 1000 {
//!         world.resources.steam.unlock_achievement("ACH_HIGH_SCORE").ok();
//!         world.resources.steam.store_stats().ok();
//!     }
//!
//!     // Update stat
//!     world.resources.steam.set_stat_int("total_score", self.score).ok();
//! }
//! ```
//!
//! ### Webview (`webview` feature)
//!
//! Host a web frontend (Leptos, Yew, etc.) inside a nightshade window with bidirectional IPC.
//!
//! **Native side** (use `webview` with `runtime` and `egui`):
//!
//! ```ignore
//! use include_dir::{Dir, include_dir};
//! use nightshade::prelude::*;
//! use nightshade::webview::{WebviewContext, serve_embedded_dir};
//!
//! static DIST: Dir = include_dir!("$CARGO_MANIFEST_DIR/site/dist");
//!
//! struct App {
//!     port: u16,
//!     ctx: WebviewContext<FrontendCommand, BackendEvent>,
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     launch(App {
//!         port: serve_embedded_dir(&DIST),
//!         ctx: WebviewContext::default(),
//!     })
//! }
//!
//! impl State for App {
//!     fn ui(&mut self, world: &mut World, ctx: &egui::Context) {
//!         // Handle messages from frontend
//!         for cmd in self.ctx.drain_messages() {
//!             match cmd {
//!                 FrontendCommand::Ready => self.ctx.send(BackendEvent::Connected),
//!                 // ...
//!             }
//!         }
//!
//!         // Render webview in a panel
//!         egui::CentralPanel::default().frame(egui::Frame::NONE).show(ctx, |ui| {
//!             if let Some(handle) = &world.resources.window.handle {
//!                 self.ctx.ensure_webview(handle.clone(), self.port, ui.available_rect_before_wrap());
//!             }
//!         });
//!     }
//! }
//! ```
//!
//! **WASM frontend** (use `webview` with `default-features = false`):
//!
//! ```ignore
//! use nightshade::webview::{connect, send};
//!
//! fn main() {
//!     connect(FrontendCommand::Ready, |event: BackendEvent| {
//!         // Handle events from native app
//!     });
//!
//!     // Send commands to native app
//!     send(&FrontendCommand::DoSomething);
//! }
//! ```
//!
//! ### Mosaic Framework (`mosaic` feature)
//!
//! Build multi-pane desktop applications with dockable, serializable widgets:
//!
//! ```ignore
//! use nightshade::prelude::*;
//! use nightshade::mosaic::{Mosaic, ViewportWidget, Widget, WidgetContext, WidgetEntry, Pane};
//!
//! #[derive(Clone, serde::Serialize, serde::Deserialize)]
//! enum AppWidget {
//!     Viewport(ViewportWidget),
//! }
//!
//! impl Widget for AppWidget {
//!     fn title(&self) -> String {
//!         match self {
//!             AppWidget::Viewport(v) => v.title(),
//!         }
//!     }
//!
//!     fn ui(&mut self, ui: &mut egui::Ui, context: &mut WidgetContext) {
//!         match self {
//!             AppWidget::Viewport(v) => v.ui(ui, context),
//!         }
//!     }
//!
//!     fn catalog() -> Vec<WidgetEntry<Self>> {
//!         vec![WidgetEntry {
//!             name: "Viewport".to_string(),
//!             create: || AppWidget::Viewport(ViewportWidget::default()),
//!         }]
//!     }
//! }
//!
//! struct MyApp {
//!     mosaic: Mosaic<AppWidget>,
//! }
//!
//! impl State for MyApp {
//!     fn title(&self) -> &str { "My App" }
//!
//!     fn initialize(&mut self, world: &mut World) {
//!         self.mosaic = Mosaic::with_panes(vec![
//!             AppWidget::Viewport(ViewportWidget::default()),
//!         ]);
//!     }
//!
//!     fn ui(&mut self, world: &mut World, ctx: &egui::Context) {
//!         self.mosaic.show(world, ctx, &mut ());
//!     }
//! }
//! ```
//!
//! ### Debug Lines
//!
//! ```ignore
//! // Create debug lines entity
//! let debug = world.spawn_entities(LINES, 1)[0];
//!
//! fn run_systems(&mut self, world: &mut World) {
//!     if let Some(lines) = world.core.get_lines_mut(self.debug) {
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
//! let emitter = world.spawn_entities(
//!     PARTICLE_EMITTER | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY,
//!     1
//! )[0];
//!
//! world.core.set_particle_emitter(emitter, ParticleEmitter {
//!     max_particles: 1000,
//!     emission_rate: 100.0,
//!     lifetime: 2.0,
//!     initial_velocity: Vec3::new(0.0, 5.0, 0.0),
//!     velocity_randomness: 1.0,
//!     size: 0.1,
//!     color: ColorGradient::new(
//!         Vec4::new(1.0, 0.5, 0.0, 1.0),  // Start: orange
//!         Vec4::new(1.0, 0.0, 0.0, 0.0),  // End: red, faded
//!     ),
//!     shape: EmitterShape::Sphere { radius: 0.5 },
//!     ..Default::default()
//! });
//!
//! world.core.set_local_transform(emitter, LocalTransform::from_translation(Vec3::new(0.0, 0.0, 0.0)));
//!
//! // Runs automatically via frame schedule
//! ```
//!
//! ### Sprites
//!
//! ```ignore
//! let sprite_entity = world.spawn();
//! world.sprite2d.add_components(sprite_entity, SPRITE);
//! world.sprite2d.set_sprite(sprite_entity, Sprite::new(
//!     Vec2::new(100.0, 100.0),  // Screen position
//!     Vec2::new(64.0, 64.0),    // Size
//! )
//!     .with_texture(texture_index)
//!     .with_color([1.0, 1.0, 1.0, 1.0])
//!     .with_depth(0.0)  // Lower = in front
//! );
//! ```
//!
//! ### Water
//!
//! ```ignore
//! // Simple water plane
//! spawn_water_plane_at(world, Vec3::new(0.0, 0.0, 0.0), 100.0);
//!
//! // Or with full control
//! let water = world.spawn_entities(WATER | LOCAL_TRANSFORM | GLOBAL_TRANSFORM, 1)[0];
//! world.core.set_water(water, Water {
//!     color: Vec3::new(0.0, 0.3, 0.5),
//!     opacity: 0.8,
//!     wave_speed: 1.0,
//!     wave_amplitude: 0.2,
//!     ..Default::default()
//! });
//! ```
//!
//! ### Decals
//!
//! ```ignore
//! let decal = world.spawn_entities(
//!     DECAL | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY,
//!     1
//! )[0];
//!
//! world.core.set_decal(decal, Decal {
//!     texture_index: world.texture_cache_lookup("decal_texture"),
//!     size: Vec2::new(2.0, 2.0),
//!     depth: 1.0,  // Projection depth
//!     opacity: 1.0,
//! });
//!
//! // Position and orient decal (projects along -Y)
//! world.core.set_local_transform(decal, LocalTransform {
//!     translation: hit_position + hit_normal * 0.01,
//!     rotation: UnitQuaternion::face_towards(&(-hit_normal), &Vec3::x()),
//!     ..Default::default()
//! });
//! ```
//!
//! ## Minimal Example
//!
//! For a lightweight egui app without asset loading:
//!
//! ```toml
//! nightshade = { default-features = false, features = ["runtime", "wgpu", "egui"] }
//! ```
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
//!
//! ## Frame Lifecycle
//!
//! Each frame executes in this order:
//!
//! 1. Process window/input events
//! 2. Update keyboard/mouse/gamepad state
//! 3. Calculate delta time
//! 4. Run egui frame (if enabled)
//! 5. Call [`State::run_systems()`](run::State::run_systems)
//! 6. Dispatch event bus messages
//! 7. Process MCP commands (if `mcp` feature enabled)
//! 8. Run [`FrameSchedule`](schedule::FrameSchedule) (engine systems):
//!    - Initialize and update audio (if `audio` enabled)
//!    - Update camera aspect ratios
//!    - Step physics simulation (if `physics` enabled)
//!    - Run scripts (if `scripting` enabled)
//!    - Update animation players, apply to transforms
//!    - Propagate transform hierarchy
//!    - Update instanced mesh caches, sync text
//!    - Reset input state
//!    - Process deferred commands, cleanup
//! 9. Execute render graph passes
//!
//! The frame schedule is a resource at `world.resources.frame_schedule`. Modify it
//! in [`State::initialize()`](run::State::initialize) to insert, remove, or reorder
//! engine systems. See [`system_names`](schedule::system_names) for the available
//! system name constants.
//!
//! ## Threading Model
//!
//! The engine runs on a single thread with async GPU operations via wgpu,
//! optional async asset loading (with `async_runtime` feature), and fixed
//! timestep physics with interpolation for smooth rendering.
//!
//! ## Platform Notes
//!
//! - **Native** (Windows/macOS/Linux): Full feature support with DirectX 12, Metal, or Vulkan.
//! - **WebAssembly**: WebGPU backend. Files via drag-and-drop or HTTP. No WASI plugins. Async initialization.
//! - **XR** (with `openxr`): Vulkan backend required. Use `launch_xr()` entry point.

pub mod filesystem;
pub mod plugin;

#[cfg(any(feature = "tui", feature = "terminal"))]
pub mod tui;

#[cfg(feature = "webview")]
pub mod webview;

#[cfg(feature = "runtime")]
pub mod ecs;
#[cfg(feature = "runtime")]
pub mod multi_window;
#[cfg(feature = "runtime")]
pub mod render;
#[cfg(feature = "runtime")]
pub mod run;
#[cfg(feature = "runtime")]
pub mod schedule;
#[cfg(feature = "runtime")]
mod systems;

#[cfg(feature = "openxr")]
pub mod xr;

#[cfg(feature = "shell")]
pub mod shell;

#[cfg(all(feature = "claude", not(target_arch = "wasm32")))]
pub mod claude;

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
pub mod mcp;

#[cfg(all(feature = "steam", not(target_arch = "wasm32")))]
pub mod steam;

#[cfg(all(feature = "plugin_runtime", not(target_arch = "wasm32")))]
pub mod plugin_runtime;

#[cfg(all(windows, feature = "windows-app-icon"))]
pub mod build;

#[cfg(feature = "interactive_fiction")]
pub mod interactive_fiction;

#[cfg(feature = "runtime")]
pub use systems::*;

#[cfg(feature = "runtime")]
pub mod prelude {
    pub use crate::ecs::window::resources::{
        SecondaryWindowInput, SecondaryWindowState, SecondaryWindows, WindowSpawnRequest,
    };
    pub use crate::ecs::world::components::*;
    pub use crate::ecs::world::resources::*;
    pub use crate::ecs::world::*;
    pub use crate::run::*;
    pub use crate::schedule::{
        FrameSchedule, FrameScheduleEntry, SystemFn, run_retained_ui_systems, system_names,
    };
    pub use crate::systems::*;
    pub use web_time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    pub const LOGO_BYTES: &[u8] = include_bytes!("../images/logo.png");

    pub use crate::ecs::animation::systems::{apply_animations, update_animation_players};
    #[cfg(feature = "audio")]
    pub use crate::ecs::audio::systems::load_sound_from_bytes;
    pub use crate::ecs::bounding_volume::components::BoundingVolume;
    pub use crate::ecs::camera::commands::{
        spawn_ortho_camera, spawn_pan_orbit_camera, spawn_third_person_camera,
    };
    pub use crate::ecs::camera::components::ThirdPersonCamera;
    pub use crate::ecs::camera::systems::{
        camera_controllers_system, first_person_camera_look_system, fly_camera_system,
        ortho_camera_system, pan_orbit_camera_system, third_person_camera_system,
    };
    pub use crate::ecs::event_bus::commands::{publish_app_event, publish_event};
    pub use crate::ecs::event_bus::resources::Message;
    #[cfg(feature = "gamepad")]
    pub use crate::ecs::input::queries::query_active_gamepad;
    pub use crate::ecs::input::resources::input_mode::InputMode;
    pub use crate::ecs::input::resources::mouse::MouseState;
    #[cfg(feature = "gamepad")]
    pub use crate::ecs::input::systems::detect_input_mode_system;
    #[cfg(feature = "lattice")]
    pub use crate::ecs::lattice::{
        debug::{LatticeDebugState, get_lattice_point_at_screen_position, lattice_debug_draw},
        systems::{
            create_lattice_entity, lattice_deformation_system,
            register_entity_for_lattice_deformation,
        },
    };
    pub use crate::ecs::material::resources::material_registry_insert;
    pub use crate::ecs::mesh::commands::spawn_mesh;
    pub use crate::ecs::mesh::systems::update_instanced_mesh_caches_system;
    pub use crate::ecs::navmesh::{
        NavMeshAgent, NavMeshAgentState, NavMeshConnection, NavMeshEdge, NavMeshTriangle,
        NavMeshWorld, PathRequest, PathResult, PathStatus, PathfindingAlgorithm, SpatialHash,
        clear_navmesh, find_closest_point_on_navmesh, get_agent_path_length, get_agent_state,
        run_navmesh_systems, sample_navmesh_height, set_agent_destination, set_agent_speed,
        set_navmesh_debug_draw, spawn_navmesh_agent, stop_agent,
    };
    #[cfg(feature = "navmesh")]
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
    #[cfg(feature = "physics")]
    pub use crate::ecs::physics::debug::physics_debug_draw_system;
    #[cfg(feature = "physics")]
    pub use crate::ecs::physics::events::{CollisionEvent, CollisionEventKind, ContactForceEvent};
    #[cfg(feature = "physics")]
    pub use crate::ecs::physics::types::RaycastHit;
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
    pub use crate::ecs::prefab::resources::mesh_cache_insert;
    #[cfg(feature = "scene_graph")]
    pub use crate::ecs::scene::{
        AssetMode, AssetUuid, Scene, SceneAlphaMode, SceneAudioSource, SceneBodyType, SceneCamera,
        SceneCollider, SceneComponents, SceneEntity, SceneError, SceneHdrSkybox,
        SceneInstancedMesh, SceneLight, SceneMaterial, SceneMesh, SceneMeshInstance, SceneNavMesh,
        ScenePhysics, ScenePrefabInstance, SpawnSceneResult, load_scene, save_scene, spawn_scene,
        world_to_scene,
    };
    pub use crate::ecs::scene_2d::{
        Scene2D, Scene2DError, SceneEntity2D, load_scene_2d, save_scene_2d, spawn_scene_2d,
        world_to_scene_2d,
    };
    pub use crate::ecs::sprite::commands::spawn_sprite;
    pub use crate::ecs::sprite::components::{SpriteBlendMode, SpriteGradient, SpriteStencilMode};
    #[cfg(feature = "assets")]
    pub use crate::ecs::sprite::shapes::spawn_sprite_from_image_bytes;
    pub use crate::ecs::sprite::shapes::{
        BlurSource, CubicBezier, SpriteShape, screen_pixels_to_world_size, shape_texture_info,
        spawn_arrow, spawn_capsule, spawn_circle, spawn_cubic_bezier, spawn_dashed_line,
        spawn_ellipse, spawn_filled_and_stroked_circle, spawn_filled_and_stroked_ellipse,
        spawn_filled_and_stroked_rect, spawn_filled_path, spawn_filled_polygon, spawn_glow,
        spawn_line, spawn_outlined_rect, spawn_path, spawn_quadratic_bezier, spawn_rect,
        spawn_ring, spawn_rounded_rect, spawn_shadow, spawn_shape, spawn_shape_with_aa,
        spawn_triangle, spawn_variable_width_path, tessellate_cubic_bezier,
        tessellate_quadratic_bezier,
    };
    pub use crate::ecs::sprite::slot_allocator::{allocate_sprite_slot, sized_slot_uv};
    pub use crate::ecs::sprite_animator::{
        LoopMode, SpriteAnimator, SpriteFrame, systems::sprite_animation_system,
    };
    pub use crate::ecs::sprite_particles::{ColorRange2D, EmitterShape2D, SpriteParticleEmitter};
    #[cfg(feature = "terrain")]
    pub use crate::ecs::terrain::{
        NoiseConfig, NoiseType, TerrainConfig, TerrainGenerationResult, generate_terrain_mesh,
        sample_terrain_height, spawn_terrain, spawn_terrain_with_material,
    };
    pub use crate::ecs::text::commands::{
        load_font_from_bytes, spawn_ui_text, spawn_ui_text_with_properties,
    };
    pub use crate::ecs::text::components::{TextAlignment, TextProperties, VerticalAlignment};
    pub use crate::ecs::text::systems::sync_text_meshes_system;
    #[cfg(feature = "assets")]
    pub use crate::ecs::texture_loader::{
        AssetLoadingState, AssetLoadingStatus, LoadedTexture, SharedTextureQueue, TextureLoadQueue,
        TextureLoadRequest, create_shared_queue, get_asset_search_paths, process_and_load_textures,
        process_texture_queue, queue_texture_from_path, set_asset_search_paths,
    };
    pub use crate::ecs::tilemap::commands::spawn_tilemap;
    pub use crate::ecs::transform::commands::mark_local_transform_dirty;
    pub use crate::ecs::tween::{
        EasingFunction, Tween, TweenLoopMode, TweenState, TweenTrack, TweenValue,
    };
    pub use crate::ecs::ui::composite::{CompositeWidget, UiInteractionContext};
    pub use crate::ecs::ui::layout_types::GridLayout;
    pub use crate::ecs::ui::layout_types::{
        BoundaryLayout, FlowAlignment, FlowDirection, FlowLayout, ResponsiveFlowOverride,
        SolidLayout, WindowLayout,
    };
    pub use crate::ecs::ui::resources::{
        BubbledUiEvent, DrawRectParams, OverlayText, ReservedAreas, ToastSeverity, UiEvent,
        blend_color,
    };
    pub use crate::ecs::ui::testing::{
        UiTestDriver, UiTestResult, create_test_root, setup_test_scene,
    };
    pub use crate::ecs::ui::theme::{ThemeState, UiTheme};
    pub use crate::ecs::ui::{
        Ab, AutoSizeMode, CanvasCommand, CharStyle, ColorPickerMode, ContextMenuBuilder,
        DataGridColumn, DragAcceptFilter, DragValueConfig, Em, FromPropertyValue, FromWidgetState,
        InputMask, IntoPropertyValue, ResizeEdge, Rh, Rl, Rw, ScalingMode, SliderConfig,
        SplitDirection, StateTransition, TextSpan, ThemeColor, TileBuilder, TileId, TileLayout,
        TileLayoutNode, TileNode, Ui, UiAnimationPhase, UiAnimationType, UiBase, UiBreadcrumbData,
        UiButtonData, UiCanvasData, UiCheckboxData, UiCollapsingHeaderData, UiColorPickerData,
        UiCommandPaletteData, UiContextMenuData, UiDataGridData, UiDatePickerData, UiDepthMode,
        UiDisabled, UiDragValueData, UiDropdownData, UiFocused, UiHover, UiLayoutNode,
        UiLayoutRoot, UiLayoutType, UiMenuData, UiModalDialogData, UiMultiSelectData,
        UiNodeAnimation, UiNodeColor, UiNodeContent, UiNodeInteraction, UiPanelData, UiPanelKind,
        UiPressed, UiProgressBarData, UiPropertyGridData, UiRadioData, UiRangeSliderData,
        UiRichTextData, UiRichTextEditorData, UiScrollAreaData, UiSelectableLabelData, UiSelected,
        UiSliderData, UiSplitterData, UiStateWeights, UiTabBarData, UiTextAreaData,
        UiTextInputData, UiTileContainerData, UiToggleData, UiTreeBuilder, UiTreeNodeData,
        UiTreeViewData, UiValue, UiVirtualListData, UiWidgetState, ValidationRule, Vh, Vp, Vw,
    };
    pub use crate::ecs::ui::{Anchor, Rect, UiTextInstance};
    pub use crate::ecs::world::commands::{
        WorldCommand, capture_ibl_snapshots, capture_procedural_atmosphere_ibl,
        despawn_entities_with_cache_cleanup, despawn_recursive_immediate, load_hdr_skybox,
        load_hdr_skybox_from_path, load_procedural_textures, set_material_with_textures,
        spawn_3d_billboard_text_with_properties, spawn_3d_text_with_properties, spawn_cone_at,
        spawn_cube_at, spawn_cylinder_at, spawn_instanced_mesh, spawn_instanced_mesh_with_material,
        spawn_material, spawn_mesh_at, spawn_plane_at, spawn_sphere_at, spawn_sun,
        spawn_sun_without_shadows, spawn_torus_at, spawn_water_plane_at,
    };
    pub use crate::render::wgpu::passes::geometry::{UiImage, UiLayer};
    pub use crate::render::wgpu::texture_cache::texture_cache_add_reference;
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
    pub use strum;
    pub use tracing;
    pub use uuid;
    pub use winit;
    pub use winit::{
        event::{ElementState, ElementState as KeyState, MouseButton, WindowEvent},
        keyboard::KeyCode,
        window,
    };

    #[cfg(all(not(target_arch = "wasm32"), feature = "plugin_runtime"))]
    pub use anyhow;
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_os = "android"),
        feature = "file_dialog"
    ))]
    pub use dirs;
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_os = "android"),
        feature = "file_dialog"
    ))]
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
    pub use crate::shell::{
        Command, CommandRegistry, OutputLine, ShellState, shell_retained_ui, shell_ui,
    };

    #[cfg(all(feature = "file_watcher", not(target_arch = "wasm32")))]
    pub use crate::ecs::file_watcher::FileWatcher;

    #[cfg(feature = "fft")]
    pub use crate::ecs::audio::AudioAnalyzer;

    #[cfg(all(target_os = "android", feature = "android"))]
    pub use crate::filesystem::{get_android_app, read_android_asset};
    #[cfg(all(target_os = "android", feature = "android"))]
    pub use crate::run::launch_android;
    #[cfg(all(target_os = "android", feature = "android"))]
    pub use android_activity::AndroidApp;
}
