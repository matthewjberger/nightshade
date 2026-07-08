//! Material component definitions.

use crate::ecs::asset_id::{MaterialId, TextureId};
use serde::{Deserialize, Serialize};

/// Component referencing a material by name in the [`super::MaterialRegistry`].
///
/// The `id` field is populated when the material is resolved from the registry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, enum2schema::Schema)]
pub struct MaterialRef {
    /// Name of the material in the registry.
    pub name: String,
    /// Resolved material identifier.
    #[serde(skip)]
    #[schema(skip)]
    pub id: Option<MaterialId>,
}

impl MaterialRef {
    /// Creates a new material reference by name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            id: None,
        }
    }

    /// Creates a material reference with a pre-resolved identifier.
    pub fn with_id(name: impl Into<String>, id: MaterialId) -> Self {
        Self {
            name: name.into(),
            id: Some(id),
        }
    }
}

impl Default for MaterialRef {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            id: None,
        }
    }
}

impl From<String> for MaterialRef {
    fn from(name: String) -> Self {
        Self { name, id: None }
    }
}

/// Per-primitive material variant table from the
/// [`KHR_materials_variants`] glTF extension. Each mapping says: when
/// any listed variant name is active, swap the entity's material to
/// the named one. Outside an active variant the entity uses
/// `default_material_name`. Variant names rather than document-scoped
/// indices are stored so multiple assets with different variant
/// orderings can coexist.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MaterialVariants {
    pub default_material_name: String,
    pub mappings: Vec<MaterialVariantMapping>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialVariantMapping {
    pub variant_names: Vec<String>,
    pub material_name: String,
}

impl MaterialVariants {
    /// Returns the material name to use for the supplied variant name,
    /// or `default_material_name` if no mapping covers it.
    pub fn material_for_variant(&self, variant_name: &str) -> &str {
        for mapping in &self.mappings {
            if mapping.variant_names.iter().any(|n| n == variant_name) {
                return &mapping.material_name;
            }
        }
        &self.default_material_name
    }
}

impl From<&str> for MaterialRef {
    fn from(name: &str) -> Self {
        Self {
            name: name.to_string(),
            id: None,
        }
    }
}

/// Per-texture UV transform from glTF KHR_texture_transform.
///
/// Applied as `T(offset) * R(rotation) * S(scale)` to homogeneous UVs.
/// When the extension is absent, the transform is identity and `uv_set`
/// inherits from the binding's tex_coord.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TextureTransform {
    /// UV offset (translation).
    pub offset: [f32; 2],
    /// UV rotation in radians (positive = counter-clockwise about origin in glTF).
    pub rotation: f32,
    /// UV scale.
    pub scale: [f32; 2],
    /// Which TEXCOORD_n attribute to sample (0 or 1).
    pub uv_set: u32,
}

impl Default for TextureTransform {
    fn default() -> Self {
        Self {
            offset: [0.0, 0.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
            uv_set: 0,
        }
    }
}

impl TextureTransform {
    pub const IDENTITY: Self = Self {
        offset: [0.0, 0.0],
        rotation: 0.0,
        scale: [1.0, 1.0],
        uv_set: 0,
    };

    /// Compose offset/rotation/scale as `T * R * S` and pack into the two-row
    /// `mat3x2` form used by the shader. Returns `(row0, row1)` where
    /// row0 = (m00, m01, m02), row1 = (m10, m11, m12).
    ///
    /// Per KHR_texture_transform reference renderer (matches Khronos sample
    /// renderings of TextureTransformTest):
    ///
    /// | cos*sx   sin*sy  ox |
    /// | -sin*sx  cos*sy  oy |
    /// |       0       0   1 |
    ///
    /// applied as `M * (uv.x, uv.y, 1)^T`.
    pub fn to_packed(&self) -> ([f32; 3], [f32; 3]) {
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();
        let m00 = cos_r * self.scale[0];
        let m01 = sin_r * self.scale[1];
        let m02 = self.offset[0];
        let m10 = -sin_r * self.scale[0];
        let m11 = cos_r * self.scale[1];
        let m12 = self.offset[1];
        ([m00, m01, m02], [m10, m11, m12])
    }
}

/// How alpha values are interpreted for transparency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, Hash)]
pub enum AlphaMode {
    /// Fully opaque, alpha is ignored.
    #[default]
    Opaque,
    /// Binary transparency using alpha cutoff threshold.
    Mask,
    /// Full alpha blending with background.
    Blend,
    /// Stochastic (screen-door) transparency: alpha becomes a per-pixel dither
    /// probability resolved by temporal antialiasing. Renders opaque, writes
    /// depth, and needs no sorting, so it suits foliage and hair.
    Dither,
}

/// PBR material definition following glTF 2.0 conventions.
///
/// Supports the metallic-roughness workflow with optional textures for each parameter.
/// Includes glTF extensions: KHR_materials_transmission, KHR_materials_volume,
/// KHR_materials_specular, and KHR_materials_emissive_strength.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Material {
    /// Base color (albedo) as RGBA. Multiplied with base_texture if present.
    pub base_color: [f32; 4],
    /// Emissive color multiplier as RGB.
    pub emissive_factor: [f32; 3],
    /// Transparency handling mode.
    pub alpha_mode: AlphaMode,
    /// Alpha threshold for [`AlphaMode::Mask`].
    pub alpha_cutoff: f32,
    /// Alpha threshold above which a fragment of an [`AlphaMode::Blend`]
    /// material is treated as effectively opaque by the depth prepass:
    /// fragments at or above this value write depth so transparent
    /// fragments behind them are correctly occluded; fragments below
    /// the threshold skip the prepass and accumulate through OIT.
    /// Defaults to 0.99; lower it for materials with soft alpha edges
    /// like anti-aliased text or decals where a tighter threshold
    /// produces visible halos in OIT.
    #[serde(default = "default_blend_opaque_alpha_threshold")]
    pub blend_opaque_alpha_threshold: f32,
    /// Path to base color texture.
    pub base_texture: Option<String>,
    #[serde(default)]
    pub base_texture_transform: TextureTransform,
    /// Path to emissive texture.
    pub emissive_texture: Option<String>,
    #[serde(default)]
    pub emissive_texture_transform: TextureTransform,
    /// Path to normal map texture.
    pub normal_texture: Option<String>,
    #[serde(default)]
    pub normal_texture_transform: TextureTransform,
    /// Normal map intensity multiplier.
    #[serde(default = "default_normal_scale")]
    pub normal_scale: f32,
    /// Flip normal map Y (green) channel for DirectX-style maps.
    #[serde(default)]
    pub normal_map_flip_y: bool,
    /// Two-component normal map (RG only, B reconstructed).
    #[serde(default)]
    pub normal_map_two_component: bool,
    /// Detail albedo texture, tiled by `detail_uv_scale` and overlaid on the
    /// base color for close-up surface variation. Load with `TextureUsage::Color`
    /// so it lands in the sRGB texture array.
    #[serde(default)]
    pub detail_base_texture: Option<String>,
    /// Detail normal map, reoriented onto the base surface normal. Load with
    /// `TextureUsage::Linear`.
    #[serde(default)]
    pub detail_normal_texture: Option<String>,
    /// UV tiling multiplier applied to the detail textures.
    #[serde(default = "default_detail_uv_scale")]
    pub detail_uv_scale: f32,
    /// Height map for single-step parallax offset mapping. Load with
    /// `TextureUsage::Linear`.
    #[serde(default)]
    pub height_texture: Option<String>,
    /// Parallax height scale in UV units (0 disables parallax).
    #[serde(default)]
    pub parallax_scale: f32,
    /// World-space minimum corner of the reflection box used to reproject the
    /// environment reflection so it aligns with a finite room instead of an
    /// infinitely distant environment. Equal min and max disables box
    /// projection.
    #[serde(default)]
    pub reflection_box_min: [f32; 3],
    /// World-space maximum corner of the reflection box.
    #[serde(default)]
    pub reflection_box_max: [f32; 3],
    /// Path to metallic (B) / roughness (G) texture.
    pub metallic_roughness_texture: Option<String>,
    #[serde(default)]
    pub metallic_roughness_texture_transform: TextureTransform,
    /// Path to ambient occlusion texture (R channel).
    pub occlusion_texture: Option<String>,
    #[serde(default)]
    pub occlusion_texture_transform: TextureTransform,
    /// Occlusion effect strength (0 = none, 1 = full).
    #[serde(default = "default_occlusion_strength")]
    pub occlusion_strength: f32,
    /// Surface roughness (0 = smooth/mirror, 1 = rough/diffuse).
    pub roughness: f32,
    /// Metallic factor (0 = dielectric, 1 = metal).
    pub metallic: f32,
    /// Skip lighting calculations (flat shaded).
    pub unlit: bool,
    /// Render both sides of faces.
    #[serde(default)]
    pub double_sided: bool,
    /// Transmission factor for refractive materials (KHR_materials_transmission).
    #[serde(default)]
    pub transmission_factor: f32,
    #[serde(default)]
    pub transmission_texture: Option<String>,
    #[serde(default)]
    pub transmission_texture_transform: TextureTransform,
    /// Volume thickness for transmission (KHR_materials_volume).
    #[serde(default)]
    pub thickness: f32,
    #[serde(default)]
    pub thickness_texture: Option<String>,
    #[serde(default)]
    pub thickness_texture_transform: TextureTransform,
    /// Light absorption color inside the volume.
    #[serde(default = "default_attenuation_color")]
    pub attenuation_color: [f32; 3],
    /// Distance at which light is attenuated to attenuation_color.
    #[serde(default)]
    pub attenuation_distance: f32,
    /// Index of refraction (default 1.5 for glass).
    #[serde(default = "default_ior")]
    pub ior: f32,
    /// Specular intensity override (KHR_materials_specular).
    #[serde(default = "default_specular_factor")]
    pub specular_factor: f32,
    /// Specular color tint.
    #[serde(default = "default_specular_color_factor")]
    pub specular_color_factor: [f32; 3],
    #[serde(default)]
    pub specular_texture: Option<String>,
    #[serde(default)]
    pub specular_texture_transform: TextureTransform,
    #[serde(default)]
    pub specular_color_texture: Option<String>,
    #[serde(default)]
    pub specular_color_texture_transform: TextureTransform,
    /// Emissive intensity multiplier (KHR_materials_emissive_strength).
    #[serde(default = "default_emissive_strength")]
    pub emissive_strength: f32,
    /// Diffuse transmission factor (KHR_materials_diffuse_transmission).
    /// Fraction of base color light transmitted as Lambertian through the surface.
    #[serde(default)]
    pub diffuse_transmission_factor: f32,
    #[serde(default)]
    pub diffuse_transmission_texture: Option<String>,
    #[serde(default)]
    pub diffuse_transmission_texture_transform: TextureTransform,
    /// Color tint applied to the diffuse-transmitted light.
    #[serde(default = "default_diffuse_transmission_color")]
    pub diffuse_transmission_color_factor: [f32; 3],
    #[serde(default)]
    pub diffuse_transmission_color_texture: Option<String>,
    #[serde(default)]
    pub diffuse_transmission_color_texture_transform: TextureTransform,
    /// Chromatic dispersion strength (KHR_materials_dispersion).
    /// Splits the refraction angle per wavelength using Cauchy's approximation.
    #[serde(default)]
    pub dispersion: f32,
    /// Anisotropy strength (KHR_materials_anisotropy). 0 = isotropic, 1 = maximum.
    #[serde(default)]
    pub anisotropy_strength: f32,
    /// Rotation of anisotropic direction in radians around the surface normal.
    #[serde(default)]
    pub anisotropy_rotation: f32,
    #[serde(default)]
    pub anisotropy_texture: Option<String>,
    #[serde(default)]
    pub anisotropy_texture_transform: TextureTransform,
    /// Iridescence strength (KHR_materials_iridescence). 0 = none, 1 = full thin-film effect.
    #[serde(default)]
    pub iridescence_factor: f32,
    #[serde(default)]
    pub iridescence_texture: Option<String>,
    #[serde(default)]
    pub iridescence_texture_transform: TextureTransform,
    /// Index of refraction for the iridescent thin-film layer.
    #[serde(default = "default_iridescence_ior")]
    pub iridescence_ior: f32,
    /// Minimum film thickness in nanometers.
    #[serde(default = "default_iridescence_thickness_min")]
    pub iridescence_thickness_minimum: f32,
    /// Maximum film thickness in nanometers (modulated by iridescence_thickness_texture.g).
    #[serde(default = "default_iridescence_thickness_max")]
    pub iridescence_thickness_maximum: f32,
    #[serde(default)]
    pub iridescence_thickness_texture: Option<String>,
    #[serde(default)]
    pub iridescence_thickness_texture_transform: TextureTransform,
    /// Sheen color (KHR_materials_sheen). Multiplied with sheen_color_texture if present.
    #[serde(default)]
    pub sheen_color_factor: [f32; 3],
    #[serde(default)]
    pub sheen_color_texture: Option<String>,
    #[serde(default)]
    pub sheen_color_texture_transform: TextureTransform,
    /// Sheen roughness (0 = sharp, 1 = smooth velvet).
    #[serde(default)]
    pub sheen_roughness_factor: f32,
    #[serde(default)]
    pub sheen_roughness_texture: Option<String>,
    #[serde(default)]
    pub sheen_roughness_texture_transform: TextureTransform,
    /// Clearcoat layer strength (KHR_materials_clearcoat). 0 = none, 1 = full coat.
    #[serde(default)]
    pub clearcoat_factor: f32,
    #[serde(default)]
    pub clearcoat_texture: Option<String>,
    #[serde(default)]
    pub clearcoat_texture_transform: TextureTransform,
    /// Clearcoat layer roughness.
    #[serde(default)]
    pub clearcoat_roughness_factor: f32,
    #[serde(default)]
    pub clearcoat_roughness_texture: Option<String>,
    #[serde(default)]
    pub clearcoat_roughness_texture_transform: TextureTransform,
    /// Optional separate normal map for the clearcoat layer.
    #[serde(default)]
    pub clearcoat_normal_texture: Option<String>,
    #[serde(default)]
    pub clearcoat_normal_texture_transform: TextureTransform,
    #[serde(default = "default_normal_scale")]
    pub clearcoat_normal_scale: f32,
}

/// Resolved [`TextureId`]s for every texture role on a [`Material`].
///
/// Mirrors the `Option<String>` fields on [`Material`] one-for-one so that
/// hot-path code (per-frame material rebuild) can index a layer map by
/// [`TextureId`] without re-hashing texture names. Lives in a parallel `Vec`
/// alongside the [`crate::ecs::material::MaterialRegistry`] entries; populated
/// by `material_registry_resolve_uploaded_textures` from the renderer drain.
#[derive(Clone, Copy, Debug, Default)]
pub struct MaterialTextureIds {
    pub base: Option<TextureId>,
    pub emissive: Option<TextureId>,
    pub normal: Option<TextureId>,
    pub detail_base: Option<TextureId>,
    pub detail_normal: Option<TextureId>,
    pub height: Option<TextureId>,
    pub metallic_roughness: Option<TextureId>,
    pub occlusion: Option<TextureId>,
    pub transmission: Option<TextureId>,
    pub thickness: Option<TextureId>,
    pub specular: Option<TextureId>,
    pub specular_color: Option<TextureId>,
    pub diffuse_transmission: Option<TextureId>,
    pub diffuse_transmission_color: Option<TextureId>,
    pub anisotropy: Option<TextureId>,
    pub iridescence: Option<TextureId>,
    pub iridescence_thickness: Option<TextureId>,
    pub sheen_color: Option<TextureId>,
    pub sheen_roughness: Option<TextureId>,
    pub clearcoat: Option<TextureId>,
    pub clearcoat_roughness: Option<TextureId>,
    pub clearcoat_normal: Option<TextureId>,
}

fn default_blend_opaque_alpha_threshold() -> f32 {
    0.99
}

fn default_normal_scale() -> f32 {
    1.0
}

fn default_detail_uv_scale() -> f32 {
    4.0
}

fn default_occlusion_strength() -> f32 {
    1.0
}

fn default_attenuation_color() -> [f32; 3] {
    [1.0, 1.0, 1.0]
}

fn default_ior() -> f32 {
    1.5
}

fn default_specular_factor() -> f32 {
    1.0
}

fn default_specular_color_factor() -> [f32; 3] {
    [1.0, 1.0, 1.0]
}

fn default_emissive_strength() -> f32 {
    1.0
}

fn default_diffuse_transmission_color() -> [f32; 3] {
    [1.0, 1.0, 1.0]
}

fn default_iridescence_ior() -> f32 {
    1.3
}

fn default_iridescence_thickness_min() -> f32 {
    100.0
}

fn default_iridescence_thickness_max() -> f32 {
    400.0
}

impl Material {
    /// Returns `true` if this material requires transparency handling.
    pub fn is_transparent(&self) -> bool {
        matches!(self.alpha_mode, AlphaMode::Mask | AlphaMode::Blend)
    }

    /// Yields every texture name referenced by this material across every
    /// PBR slot (base color, normal map, metallic-roughness, all glTF
    /// extension textures). Used by the texture cache to bump and drop
    /// reference counts without missing any slots.
    pub fn texture_names(&self) -> impl Iterator<Item = &str> {
        [
            self.base_texture.as_deref(),
            self.emissive_texture.as_deref(),
            self.normal_texture.as_deref(),
            self.detail_base_texture.as_deref(),
            self.detail_normal_texture.as_deref(),
            self.height_texture.as_deref(),
            self.metallic_roughness_texture.as_deref(),
            self.occlusion_texture.as_deref(),
            self.transmission_texture.as_deref(),
            self.thickness_texture.as_deref(),
            self.specular_texture.as_deref(),
            self.specular_color_texture.as_deref(),
            self.diffuse_transmission_texture.as_deref(),
            self.diffuse_transmission_color_texture.as_deref(),
            self.anisotropy_texture.as_deref(),
            self.iridescence_texture.as_deref(),
            self.iridescence_thickness_texture.as_deref(),
            self.sheen_color_texture.as_deref(),
            self.sheen_roughness_texture.as_deref(),
            self.clearcoat_texture.as_deref(),
            self.clearcoat_roughness_texture.as_deref(),
            self.clearcoat_normal_texture.as_deref(),
        ]
        .into_iter()
        .flatten()
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            base_color: [0.7, 0.7, 0.7, 1.0],
            emissive_factor: [0.0, 0.0, 0.0],
            alpha_mode: AlphaMode::Opaque,
            alpha_cutoff: 0.5,
            blend_opaque_alpha_threshold: 0.99,
            base_texture: None,
            base_texture_transform: TextureTransform::IDENTITY,
            emissive_texture: None,
            emissive_texture_transform: TextureTransform::IDENTITY,
            normal_texture: None,
            normal_texture_transform: TextureTransform::IDENTITY,
            normal_scale: 1.0,
            normal_map_flip_y: false,
            normal_map_two_component: false,
            detail_base_texture: None,
            detail_normal_texture: None,
            detail_uv_scale: default_detail_uv_scale(),
            height_texture: None,
            parallax_scale: 0.0,
            reflection_box_min: [0.0, 0.0, 0.0],
            reflection_box_max: [0.0, 0.0, 0.0],
            metallic_roughness_texture: None,
            metallic_roughness_texture_transform: TextureTransform::IDENTITY,
            occlusion_texture: None,
            occlusion_texture_transform: TextureTransform::IDENTITY,
            occlusion_strength: 1.0,
            roughness: 0.5,
            metallic: 0.0,
            unlit: false,
            double_sided: false,
            transmission_factor: 0.0,
            transmission_texture: None,
            transmission_texture_transform: TextureTransform::IDENTITY,
            thickness: 0.0,
            thickness_texture: None,
            thickness_texture_transform: TextureTransform::IDENTITY,
            attenuation_color: [1.0, 1.0, 1.0],
            attenuation_distance: 0.0,
            ior: 1.5,
            specular_factor: 1.0,
            specular_color_factor: [1.0, 1.0, 1.0],
            specular_texture: None,
            specular_texture_transform: TextureTransform::IDENTITY,
            specular_color_texture: None,
            specular_color_texture_transform: TextureTransform::IDENTITY,
            emissive_strength: 1.0,
            diffuse_transmission_factor: 0.0,
            diffuse_transmission_texture: None,
            diffuse_transmission_texture_transform: TextureTransform::IDENTITY,
            diffuse_transmission_color_factor: [1.0, 1.0, 1.0],
            diffuse_transmission_color_texture: None,
            diffuse_transmission_color_texture_transform: TextureTransform::IDENTITY,
            dispersion: 0.0,
            anisotropy_strength: 0.0,
            anisotropy_rotation: 0.0,
            anisotropy_texture: None,
            anisotropy_texture_transform: TextureTransform::IDENTITY,
            iridescence_factor: 0.0,
            iridescence_texture: None,
            iridescence_texture_transform: TextureTransform::IDENTITY,
            iridescence_ior: 1.3,
            iridescence_thickness_minimum: 100.0,
            iridescence_thickness_maximum: 400.0,
            iridescence_thickness_texture: None,
            iridescence_thickness_texture_transform: TextureTransform::IDENTITY,
            sheen_color_factor: [0.0, 0.0, 0.0],
            sheen_color_texture: None,
            sheen_color_texture_transform: TextureTransform::IDENTITY,
            sheen_roughness_factor: 0.0,
            sheen_roughness_texture: None,
            sheen_roughness_texture_transform: TextureTransform::IDENTITY,
            clearcoat_factor: 0.0,
            clearcoat_texture: None,
            clearcoat_texture_transform: TextureTransform::IDENTITY,
            clearcoat_roughness_factor: 0.0,
            clearcoat_roughness_texture: None,
            clearcoat_roughness_texture_transform: TextureTransform::IDENTITY,
            clearcoat_normal_texture: None,
            clearcoat_normal_texture_transform: TextureTransform::IDENTITY,
            clearcoat_normal_scale: 1.0,
        }
    }
}
