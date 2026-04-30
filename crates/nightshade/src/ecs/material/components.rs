//! Material component definitions.

use crate::ecs::asset_id::MaterialId;
use serde::{Deserialize, Serialize};

/// Component referencing a material by name in the [`super::MaterialRegistry`].
///
/// The `id` field is populated when the material is resolved from the registry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialRef {
    /// Name of the material in the registry.
    pub name: String,
    /// Resolved material identifier.
    #[serde(skip)]
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

fn default_normal_scale() -> f32 {
    1.0
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

impl Material {
    /// Returns `true` if this material requires transparency handling.
    pub fn is_transparent(&self) -> bool {
        matches!(self.alpha_mode, AlphaMode::Mask | AlphaMode::Blend)
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            base_color: [0.7, 0.7, 0.7, 1.0],
            emissive_factor: [0.0, 0.0, 0.0],
            alpha_mode: AlphaMode::Opaque,
            alpha_cutoff: 0.5,
            base_texture: None,
            base_texture_transform: TextureTransform::IDENTITY,
            emissive_texture: None,
            emissive_texture_transform: TextureTransform::IDENTITY,
            normal_texture: None,
            normal_texture_transform: TextureTransform::IDENTITY,
            normal_scale: 1.0,
            normal_map_flip_y: false,
            normal_map_two_component: false,
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
