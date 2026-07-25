//! Per-object render data for everything that is not a plain mesh: level
//! of detail chains, instancing, decals, water, cloth, text, bounds, and
//! meshlet placement.

/// One level-of-detail step: a mesh and the screen size below which it is used.
#[derive(Clone)]
pub struct MeshLodLevel {
    /// Registered name of the mesh this level draws.
    pub mesh_name: String,
    /// Minimum on-screen size in pixels for this level to remain active.
    pub min_screen_pixels: f32,
}

/// A base mesh and its ordered chain of lower-detail replacements.
#[derive(Clone)]
pub struct MeshLodChain {
    /// Registered name of the full-detail mesh.
    pub base_mesh: String,
    /// Lower-detail levels, ordered from highest to lowest detail.
    pub levels: Vec<MeshLodLevel>,
}

/// Expanded per-instance data for one instanced-mesh entity, snapshotted once
/// per frame so the renderer builds instanced draws without reading the
/// `InstancedMesh` component.
#[derive(Clone, Default)]
pub struct InstancedObjectData {
    /// Per-instance world matrices.
    pub world_models: Vec<[[f32; 4]; 4]>,
    /// Per-instance normal matrices.
    pub world_normals: Vec<[[f32; 4]; 3]>,
    /// Per-instance local transforms relative to the parent.
    pub local_matrices: Vec<nalgebra_glm::Mat4>,
    /// Per-instance color tints.
    pub custom_tints: Vec<[f32; 4]>,
    /// Registered name of the instanced mesh.
    pub mesh_name: String,
    /// Render layer index.
    pub render_layer: u32,
    /// Visibility flag, 1 visible and 0 hidden.
    pub visible: u32,
    /// World transform of the instancing entity's parent.
    pub parent_transform: nalgebra_glm::Mat4,
}

/// A projected decal's render parameters snapshotted once per frame from the
/// `Decal` component, so the decal pass reads them here instead of that
/// component.
#[derive(Clone, Default)]
pub struct RenderDecalData {
    /// Optional base color texture name.
    pub texture: Option<String>,
    /// Optional emissive texture name.
    pub emissive_texture: Option<String>,
    /// Emissive brightness multiplier.
    pub emissive_strength: f32,
    /// Tint color multiplied into the decal.
    pub color: [f32; 4],
    /// Decal footprint size in world units.
    pub size: nalgebra_glm::Vec2,
    /// Projection depth along the decal's forward axis in world units.
    pub depth: f32,
    /// Maximum angle between surface and decal normals before the decal fades.
    pub normal_threshold: f32,
    /// Distance from the projector where fading begins.
    pub fade_start: f32,
    /// Distance from the projector where the decal fully fades out.
    pub fade_end: f32,
}

/// A water body's render parameters snapshotted once per frame from the `Water`
/// component, so the water pass reads them here instead of that component.
#[derive(Clone, Debug, Default)]
pub struct RenderWaterData {
    /// Half the water plane's size in world units.
    pub half_extents: nalgebra_glm::Vec2,
    /// Mesh grid subdivision count.
    pub tessellation: u32,
    /// Gerstner wave height in world units.
    pub wave_amplitude: f32,
    /// Gerstner wave crest steepness.
    pub wave_steepness: f32,
    /// Distance between wave crests in world units.
    pub wave_length: f32,
    /// Wave travel speed.
    pub wave_speed: f32,
    /// Wave travel direction in radians.
    pub wave_direction_radians: f32,
    /// Linear color in shallow water.
    pub shallow_color: [f32; 3],
    /// Linear color in deep water.
    pub deep_color: [f32; 3],
    /// Depth over which color fades from shallow to deep, in world units.
    pub depth_fade_distance: f32,
    /// Distance from shoreline edges where foam appears, in world units.
    pub edge_foam_distance: f32,
    /// Overall foam coverage amount.
    pub foam_amount: f32,
    /// Linear foam color.
    pub foam_color: [f32; 3],
    /// Surface roughness.
    pub roughness: f32,
    /// Fresnel falloff exponent.
    pub fresnel_power: f32,
    /// Reflection blend strength.
    pub reflection_strength: f32,
    /// Refraction blend strength.
    pub refraction_strength: f32,
    /// Specular highlight strength.
    pub specular_strength: f32,
}

/// One water body with its world transform, snapshotted once per frame so the
/// water pass reads it here instead of walking `WATER | GLOBAL_TRANSFORM`.
#[derive(Clone)]
pub struct RenderWater {
    /// The scene entity this water body belongs to.
    pub entity: nightshade_ecs::Entity,
    /// The water body's render parameters.
    pub water: RenderWaterData,
    /// World transform.
    pub transform: nalgebra_glm::Mat4,
}

/// Which cloth particles anchor to the entity transform, mirrored from the
/// `ClothPinning` component enum so the cloth pass does not read that type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RenderClothPinning {
    /// Pin the entire top row of particles.
    #[default]
    TopRow,
    /// Pin only the two top corner particles.
    TopCorners,
    /// Pin nothing, the cloth falls freely.
    None,
}

/// A cloth body's simulation parameters, held as a scene-world component on
/// the cloth entity so the cloth pass queries them alongside the entity's
/// [`crate::render_world::Transform`] and [`crate::render_world::MeshName`].
/// Carries every field so the pass's config-change comparison still detects
/// rebuilds and resets.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct RenderClothData {
    /// Particle grid column count.
    pub columns: u32,
    /// Particle grid row count.
    pub rows: u32,
    /// Cloth width in world units.
    pub width: f32,
    /// Cloth height in world units.
    pub height: f32,
    /// Which particles anchor to the entity transform.
    pub pinning: RenderClothPinning,
    /// Constraint stiffness.
    pub stiffness: f32,
    /// Velocity damping factor.
    pub damping: f32,
    /// Simulation substeps per frame.
    pub substeps: u32,
    /// Constraint solver iterations per substep.
    pub solver_iterations: u32,
    /// Gravity acceleration applied to particles.
    pub gravity: nalgebra_glm::Vec3,
    /// Response strength to scene wind.
    pub wind_response: f32,
    /// Ground collision plane height, `None` to disable.
    pub ground_height: Option<f32>,
    /// Texture repeat count across the cloth.
    pub texture_tiling: nalgebra_glm::Vec2,
    /// Counter bumped to force a simulation reset.
    pub reset_epoch: u32,
}

/// One visible 3D text entity, held as a scene-world component with its world
/// transform, resolved character count, and per-character colors, so the text
/// pass queries it instead of reading the `Text`, `GlobalTransform`, and
/// `TextCharacterColors` components. Only visible text with a cached mesh
/// carries this component.
#[derive(Clone, Default)]
pub struct RenderText {
    /// Cached glyph mesh for the text.
    pub mesh: nightshade_text::text_data::TextMesh,
    /// Base fill color.
    pub color: nalgebra_glm::Vec4,
    /// Outline color.
    pub outline_color: nalgebra_glm::Vec4,
    /// Outline width in signed-distance-field units.
    pub outline_width: f32,
    /// Edge smoothing width for the signed distance field.
    pub smoothing: f32,
    /// Whether the text faces the camera.
    pub billboard: bool,
    /// World transform.
    pub transform: nalgebra_glm::Mat4,
    /// Number of drawn characters.
    pub char_count: usize,
    /// Optional per-character color overrides.
    pub character_colors: Option<Vec<Option<nalgebra_glm::Vec4>>>,
}

/// A renderable's bounding volume reduced to the data the renderer needs:
/// the local-space sphere center and radius. Snapshotted once per frame from
/// the `BoundingVolume` component so the renderer never reads that component.
#[derive(Clone, Copy, Debug, Default)]
pub struct RenderBounds {
    /// Bounding sphere center in local space.
    pub center: nalgebra_glm::Vec3,
    /// Bounding sphere radius in local space.
    pub sphere_radius: f32,
}

/// One placement of a baked meshlet mesh in the scene, held as a scene-world
/// component on the meshlet entity. `asset_id` keys the baked asset in the
/// renderer's meshlet asset cache; the mesh's root bvh node is resolved by the
/// meshlet pass from its own upload ledger, so it is not carried here.
#[derive(Clone, Copy, Debug, Default)]
pub struct MeshletPlacement {
    /// World transform for this placement.
    pub transform: nalgebra_glm::Mat4,
    /// Key of the baked asset in the renderer's meshlet asset cache.
    pub asset_id: u64,
    /// The material this places, as the registry index plus one, with zero the
    /// built-in default grey. Resolved from the registry where the placement is
    /// written rather than looked up in the pass against a material table: the
    /// registry is the source of truth and is already current there, so the id
    /// never trails the table's separate rebuild, which is what left every mesh
    /// grey the frame the table lagged the collection.
    pub material_id: u32,
}

/// The renderer's cache of baked meshlet assets, keyed by asset id. Persists
/// across frames like the mesh and texture caches: the engine inserts an asset
/// the first time a [`MeshletPlacement`] references it, and the meshlet pass
/// uploads any asset it has not seen before.
#[cfg(feature = "meshlet")]
pub type MeshletAssetCache =
    std::collections::HashMap<u64, std::sync::Arc<crate::meshlet::asset::MeshletMesh>>;
