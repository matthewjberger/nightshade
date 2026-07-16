//! Material system for surface appearance.
//!
//! Materials define how surfaces look when rendered using PBR (Physically Based Rendering):
//!
//! - [`Material`]: PBR material definition following glTF conventions
//! - [`MaterialRef`]: Component referencing a material by name
//! - [`MaterialRegistry`]: Resource storing all named materials
//! - [`AlphaMode`]: Transparency handling mode (Opaque, Mask, Blend)
//!
//! Supports metallic-roughness workflow, normal mapping, emissive surfaces,
//! transmission/refraction (glTF extensions), and specular overrides.
//!
//! # Creating a Simple Material
//!
//! Materials are registered by name and applied to entities via [`MaterialRef`]:
//!
//! ```ignore
//! // Create and register a material
//! let material = Material {
//!     base_color: [1.0, 0.2, 0.2, 1.0],  // Red
//!     roughness: 0.3,
//!     metallic: 0.8,
//!     ..Default::default()
//! };
//!
//! material_registry_insert(
//!     &mut world.expect_resource_mut::<crate::ecs::asset_state::AssetState>().material_registry,
//!     "red_metal".to_string(),
//!     material,
//! );
//! if let Some(&index) =
//!     world.expect_resource_mut::<crate::ecs::asset_state::AssetState>().material_registry.registry.name_to_index.get("red_metal")
//! {
//!     registry_add_reference(&mut world.expect_resource_mut::<crate::ecs::asset_state::AssetState>().material_registry.registry, index);
//! }
//!
//! // Apply to an entity
//! world.set(entity, MaterialRef::new("red_metal"));
//! ```
//!
//! # Material Registration Pattern
//!
//! Materials use reference counting. After inserting, add a reference so the
//! material survives, and it is cleaned up when no entities reference it:
//!
//! ```ignore
//! // Full registration pattern
//! material_registry_insert(
//!     &mut world.expect_resource_mut::<crate::ecs::asset_state::AssetState>().material_registry,
//!     "my_material".to_string(),
//!     material,
//! );
//! if let Some(&index) =
//!     world.expect_resource_mut::<crate::ecs::asset_state::AssetState>().material_registry.registry.name_to_index.get("my_material")
//! {
//!     registry_add_reference(&mut world.expect_resource_mut::<crate::ecs::asset_state::AssetState>().material_registry.registry, index);
//! }
//! ```
//!
//! # Textured Materials
//!
//! For materials with textures, load textures first and reference by path:
//!
//! ```ignore
//! // Load textures into the cache
//! texture_cache_add_reference(&mut world.expect_resource_mut::<crate::render::wgpu::texture_cache::TextureCache>(), "assets/brick_albedo.png");
//! texture_cache_add_reference(&mut world.expect_resource_mut::<crate::render::wgpu::texture_cache::TextureCache>(), "assets/brick_normal.png");
//!
//! // Create material with texture paths
//! let material = Material {
//!     base_color: [1.0, 1.0, 1.0, 1.0],
//!     base_texture: Some("assets/brick_albedo.png".to_string()),
//!     normal_texture: Some("assets/brick_normal.png".to_string()),
//!     roughness: 0.8,
//!     metallic: 0.0,
//!     ..Default::default()
//! };
//!
//! material_registry_insert(
//!     &mut world.expect_resource_mut::<crate::ecs::asset_state::AssetState>().material_registry,
//!     "brick".to_string(),
//!     material,
//! );
//! if let Some(&index) =
//!     world.expect_resource_mut::<crate::ecs::asset_state::AssetState>().material_registry.registry.name_to_index.get("brick")
//! {
//!     registry_add_reference(&mut world.expect_resource_mut::<crate::ecs::asset_state::AssetState>().material_registry.registry, index);
//! }
//! ```
//!
//! # Emissive Materials (for Bloom)
//!
//! ```ignore
//! let glow_material = Material {
//!     base_color: [0.0, 0.5, 1.0, 1.0],
//!     emissive_factor: [0.0, 0.5, 1.0],  // Same as base for glow
//!     emissive_strength: 5.0,            // Bloom intensity multiplier
//!     unlit: true,                       // Skip lighting
//!     ..Default::default()
//! };
//! ```
//!
//! # Transparent Materials
//!
//! ```ignore
//! // Alpha mask (foliage, fences)
//! let foliage = Material {
//!     alpha_mode: AlphaMode::Mask,
//!     alpha_cutoff: 0.5,  // Pixels below this alpha are discarded
//!     double_sided: true,
//!     ..Default::default()
//! };
//!
//! // Alpha blend (glass)
//! let glass = Material {
//!     alpha_mode: AlphaMode::Blend,
//!     base_color: [1.0, 1.0, 1.0, 0.3],  // 30% opacity
//!     transmission_factor: 0.9,           // Refractive
//!     ior: 1.5,                           // Glass IOR
//!     ..Default::default()
//! };
//! ```
//!
//! # Material Properties
//!
//! | Property | Range | Description |
//! |----------|-------|-------------|
//! | `base_color` | [0-1, 0-1, 0-1, 0-1] | RGBA albedo color |
//! | `roughness` | 0-1 | 0 = mirror, 1 = diffuse |
//! | `metallic` | 0-1 | 0 = dielectric, 1 = metal |
//! | `normal_scale` | 0-2+ | Normal map intensity |
//! | `emissive_factor` | [0+, 0+, 0+] | Glow color (for bloom) |
//! | `emissive_strength` | 0-100+ | Bloom multiplier |
//! | `transmission_factor` | 0-1 | Refraction amount |
//! | `ior` | 1.0-2.5 | Index of refraction |
//!
//! [`Material`]: components::Material
//! [`MaterialRef`]: components::MaterialRef
//! [`MaterialRegistry`]: resources::MaterialRegistry
//! [`AlphaMode`]: components::AlphaMode

pub mod commands;
pub mod components;
pub mod resources;
pub mod spec_gloss;

pub use commands::*;
pub use components::*;
pub use resources::*;
