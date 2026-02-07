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
//! nightshade = "0.6"
//! ```
//!
//! ### 2. Create Your Game
//!
//! ```ignore
//! use nightshade::prelude::*;
//!
//! struct Game {
//!     camera: EntityHandle,
//! }
//!
//! impl Default for Game {
//!     fn default() -> Self {
//!         Self { camera: EntityHandle::dangling() }
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
//!         self.camera = spawn_camera(world, Vec3::new(5.0, 3.0, 5.0), "Camera".to_string());
//!         world.resources.active_camera = Some(self.camera);
//!
//!         // Spawn objects
//!         spawn_cube_at(world, Vec3::new(0.0, 0.5, 0.0));
//!     }
//!
//!     fn run_systems(&mut self, world: &mut World) {
//!         let _dt = world.resources.time.delta_seconds;
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
//! let prefab = import_gltf_from_bytes(world, model_data, "character").unwrap();
//! let entity = spawn_prefab_with_animations(world, &prefab, Vec3::zeros());
//!
//! // Play animation
//! if let Some(player) = world.get_animation_player_mut(entity) {
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
//! - **Entities**: Handles (`EntityHandle`) identifying objects
//! - **Components**: Data attached to entities (transforms, meshes, physics, etc.)
//! - **Resources**: Global singletons (`world.resources.time`, `.input`, `.graphics`, `.physics`)
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
//! // Time
//! let dt = world.resources.time.delta_seconds;
//! let elapsed = world.resources.time.elapsed_seconds;
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
//! - **`text`** - 3D text rendering using fontdue. Requires `core`.
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
//!   services, themes, and built-in viewport widgets. Re-exports `nightshade_mosaic` as `mosaic`.
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
//! world.set_audio_source(source, AudioSource::new(sound).with_spatial(true));
//! world.set_local_transform(source, LocalTransform::from_translation(Vec3::new(5.0, 0.0, 0.0)));
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
//! world.set_rigid_body(entity, RigidBodyComponent::new_dynamic());
//! world.set_collider(entity, ColliderComponent::cuboid(1.0, 1.0, 1.0));
//! world.set_local_transform(entity, LocalTransform::from_translation(Vec3::new(0.0, 5.0, 0.0)));
//!
//! // Static floor
//! let floor = world.spawn_entities(RIGID_BODY | COLLIDER | LOCAL_TRANSFORM | GLOBAL_TRANSFORM, 1)[0];
//! world.set_rigid_body(floor, RigidBodyComponent::new_static());
//! world.set_collider(floor, ColliderComponent::cuboid(10.0, 0.1, 10.0));
//!
//! // Step physics each frame
//! fn run_systems(&mut self, world: &mut World) {
//!     world.resources.physics.step(world.resources.time.delta_seconds);
//!     sync_physics_transforms(world);  // Copy physics transforms to ECS
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
//!     run_navmesh_systems(world, world.resources.time.delta_seconds);
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
//!     fn ui(&mut self, ctx: &egui::Context, world: &mut World) {
//!         egui::Window::new("Debug").show(ctx, |ui| {
//!             ui.label(format!("FPS: {:.0}", 1.0 / world.resources.time.delta_seconds));
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
//! if let Some(lat) = world.get_lattice_mut(lattice) {
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
//! Build multi-pane desktop applications with dockable widgets:
//!
//! ```ignore
//! use nightshade::mosaic::prelude::*;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     launch_mosaic(MyApp)
//! }
//!
//! #[derive(Default)]
//! struct MyApp;
//!
//! impl MosaicApp for MyApp {
//!     type Widget = AppWidget;
//!     type WidgetKind = AppWidgetKind;
//!
//!     fn title(&self) -> &str { "My App" }
//!
//!     fn initialize(&mut self, world: &mut World, _ctx: &mut MosaicContext<Self>) {
//!         let camera = spawn_pan_orbit_camera(world, Vec3::ZERO, 10.0, 0.0, 0.5, "Camera".into());
//!         world.resources.active_camera = Some(camera);
//!     }
//!
//!     fn default_tile_tree(&self) -> egui_tiles::Tree<Pane<Self::Widget>> {
//!         let mut tiles = egui_tiles::Tiles::default();
//!         let pane = tiles.insert_pane(Pane::new(AppWidget::Viewport(ViewportWidget::new("Scene"))));
//!         egui_tiles::Tree::new("tiles", tiles.insert_tab_tile(vec![pane]), tiles)
//!     }
//!
//!     fn run_systems(&mut self, world: &mut World) {
//!         pan_orbit_camera_system(world);
//!     }
//! }
//!
//! define_widgets! {
//!     AppWidget, AppWidgetKind {
//!         Viewport(ViewportWidget),
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
//!     if let Some(lines) = world.get_lines_mut(self.debug) {
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
//! world.set_particle_emitter(emitter, ParticleEmitter {
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
//! world.set_local_transform(emitter, LocalTransform::from_translation(Vec3::new(0.0, 0.0, 0.0)));
//!
//! // Update each frame
//! update_particle_emitters(world, world.resources.time.delta_seconds);
//! ```
//!
//! ### Sprites
//!
//! ```ignore
//! let sprite_entity = world.spawn_entities(SPRITE, 1)[0];
//! world.set_sprite(sprite_entity, Sprite::new(
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
//! world.set_water(water, Water {
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
//! world.set_decal(decal, Decal {
//!     texture_index: world.texture_cache_lookup("decal_texture"),
//!     size: Vec2::new(2.0, 2.0),
//!     depth: 1.0,  // Projection depth
//!     opacity: 1.0,
//! });
//!
//! // Position and orient decal (projects along -Y)
//! world.set_local_transform(decal, LocalTransform {
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
//! 7. Update animation players, apply to transforms
//! 8. Propagate transform hierarchy
//! 9. Step physics simulation, sync transforms (if enabled)
//! 10. Execute render graph passes
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

pub mod plugin;

#[cfg(any(feature = "tui", feature = "terminal"))]
pub mod tui;

#[cfg(feature = "webview")]
pub mod webview;

#[cfg(feature = "runtime")]
pub mod ecs;
#[cfg(feature = "runtime")]
pub mod render;
#[cfg(feature = "runtime")]
pub mod run;
#[cfg(feature = "runtime")]
mod systems;

#[cfg(feature = "openxr")]
pub mod xr;

#[cfg(feature = "shell")]
pub mod shell;

#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
pub mod mcp;

#[cfg(all(feature = "steam", not(target_arch = "wasm32")))]
pub mod steam;

#[cfg(all(feature = "plugin_runtime", not(target_arch = "wasm32")))]
pub mod plugin_runtime;

#[cfg(all(windows, feature = "windows-app-icon"))]
pub mod build;

#[cfg(feature = "runtime")]
pub use systems::*;

#[cfg(feature = "runtime")]
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
    #[cfg(feature = "lattice")]
    pub use crate::ecs::lattice::{
        debug::{LatticeDebugState, get_lattice_point_at_screen_position, lattice_debug_draw},
        systems::{
            create_lattice_entity, lattice_deformation_system,
            register_entity_for_lattice_deformation,
        },
    };
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
    pub use crate::ecs::physics::{
        CharacterControllerComponent, ColliderComponent, ColliderShape, PhysicsInterpolation,
        PhysicsMaterial, PhysicsMaterialComponent, RigidBodyComponent,
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
    pub use crate::ecs::prefab::commands::spawn_prefab_with_animations;
    #[cfg(feature = "scene_graph")]
    pub use crate::ecs::scene::{
        AssetMode, AssetUuid, Scene, SceneAlphaMode, SceneAudioSource, SceneBodyType, SceneCamera,
        SceneCollider, SceneComponents, SceneEntity, SceneError, SceneHdrSkybox,
        SceneInstancedMesh, SceneLight, SceneMaterial, SceneMesh, SceneMeshInstance, SceneNavMesh,
        ScenePhysics, ScenePrefabInstance, SpawnSceneResult, load_scene, save_scene, spawn_scene,
        world_to_scene,
    };
    #[cfg(feature = "terrain")]
    pub use crate::ecs::terrain::{
        NoiseConfig, NoiseType, TerrainConfig, TerrainGenerationResult, generate_terrain_mesh,
        sample_terrain_height, spawn_terrain, spawn_terrain_with_material,
    };
    pub use crate::ecs::text::commands::{
        load_font_from_bytes, spawn_hud_text, spawn_hud_text_with_properties, spawn_ui_text,
        spawn_ui_text_with_properties,
    };
    pub use crate::ecs::text::systems::sync_text_meshes_system;
    #[cfg(feature = "assets")]
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
        capture_procedural_atmosphere_ibl, despawn_recursive_immediate, load_hdr_skybox,
        load_hdr_skybox_from_path, load_procedural_textures, set_material_with_textures,
        spawn_3d_billboard_text_with_properties, spawn_3d_text_with_properties, spawn_cone_at,
        spawn_cube_at, spawn_cylinder_at, spawn_instanced_mesh, spawn_instanced_mesh_with_material,
        spawn_material, spawn_mesh_at, spawn_plane_at, spawn_sphere_at, spawn_sun,
        spawn_sun_without_shadows, spawn_torus_at, spawn_water_plane_at,
    };
    pub use crate::render::wgpu::passes::geometry::UiLayer;
    #[cfg(feature = "assets")]
    pub use bincode;
    #[cfg(feature = "egui")]
    pub use egui;
    #[cfg(feature = "egui")]
    pub use egui_tiles;
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

    #[cfg(feature = "fft")]
    pub use crate::ecs::audio::AudioAnalyzer;
}
