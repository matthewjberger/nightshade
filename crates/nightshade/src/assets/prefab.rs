//! Prefab system for glTF/GLB model loading.
//!
//! Load 3D models with meshes, materials, textures, skeletons, and animations:
//!
//! - [`import_gltf_from_bytes`]: Parse glTF/GLB data into prefabs, meshes, and animations
//! - [`spawn_prefab_with_animations`]: Instantiate a prefab with animation support
//! - [`Prefab`]: Loaded model data (meshes, materials, skeleton, animations)
//!
//! # Loading a glTF Model
//!
//! ```ignore
//! // Load from embedded bytes
//! let model_data = include_bytes!("../assets/character.glb");
//! let result = import_gltf_from_bytes(model_data).unwrap();
//! let prefab = &result.prefabs[0];
//!
//! // Spawn with its animations
//! let entity = spawn_prefab_with_animations(world, prefab, &result.animations, Vec3::zeros());
//! ```
//!
//! # Spawning at a Transform
//!
//! ```ignore
//! let entity = spawn_prefab_with_animations(world, prefab, animations, Vec3::new(10.0, 0.0, 5.0));
//!
//! // Optionally adjust transform after spawning
//! if let Some(transform) = world.get_mut::<crate::ecs::transform::components::LocalTransform>(entity) {
//!     transform.scale = Vec3::new(2.0, 2.0, 2.0);  // Double size
//!     transform.rotation = UnitQuaternion::from_euler_angles(0.0, PI, 0.0);  // Rotate 180Â°
//! }
//! ```
//!
//! # Playing Animations
//!
//! Models with animations get an [`AnimationPlayer`](crate::ecs::animation::AnimationPlayer):
//!
//! ```ignore
//! if let Some(player) = world.get_mut::<crate::ecs::animation::components::AnimationPlayer>(entity) {
//!     // List available animations
//!     for (i, clip) in player.clips.iter().enumerate() {
//!         println!("{}: {}", i, clip.name);
//!     }
//!
//!     // Play animation
//!     player.play(0);
//!     player.looping = true;
//! }
//! ```
//!
//! # Accessing Prefab Data
//!
//! - Materials are registered in `world.res::<crate::ecs::material::resources::MaterialRegistry>()`
//! - Textures are loaded into `world.res::<crate::render::wgpu::texture_cache::TextureCache>()`
//! - Meshes are stored in the prefab and referenced by spawned entities
//!
//! # Runtime Loading (Async)
//!
//! For runtime file loading (not embedded):
//!
//! ```ignore
//! // Native: Read from filesystem
//! let data = std::fs::read("assets/model.glb").unwrap();
//!
//! // WASM: Fetch via HTTP or handle dropped files
//! fn on_dropped_file_data(&mut self, world: &mut World, name: &str, data: &[u8]) {
//!     if name.ends_with(".glb") || name.ends_with(".gltf") {
//!         let result = import_gltf_from_bytes(data).unwrap();
//!         spawn_prefab_with_animations(world, &result.prefabs[0], &result.animations, Vec3::zeros());
//!     }
//! }
//! ```
//!
//! # Supported glTF Features
//!
//! | Feature | Support |
//! |---------|---------|
//! | Meshes | Triangles with positions, normals, tangents, UVs |
//! | Materials | PBR metallic-roughness, textures, alpha modes |
//! | Textures | Embedded and external (PNG, JPEG) |
//! | Skeletons | Joint hierarchies, inverse bind matrices |
//! | Animations | Translation, rotation, scale, morph targets |
//! | Morph Targets | Blend shapes with weights |
//! | Multiple UV Sets | UV0 and UV1 |
//!
//! [`import_gltf_from_bytes`]: commands::import_gltf_from_bytes
//! [`spawn_prefab_with_animations`]: commands::spawn_prefab_with_animations
//! [`Prefab`]: components::Prefab

pub mod commands;
pub mod components;
pub mod hooks;
pub mod resources;

pub use commands::*;
pub use components::*;
pub use resources::*;
