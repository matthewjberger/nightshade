use crate::data::asset_id::TextureId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TextureTransform {
    pub offset: [f32; 2],
    pub rotation: f32,
    pub scale: [f32; 2],
    pub uv_set: u32,
    pub drift: [f32; 2],
}

impl Default for TextureTransform {
    fn default() -> Self {
        Self {
            offset: [0.0, 0.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
            uv_set: 0,
            drift: [0.0, 0.0],
        }
    }
}

impl TextureTransform {
    pub const IDENTITY: Self = Self {
        offset: [0.0, 0.0],
        rotation: 0.0,
        scale: [1.0, 1.0],
        uv_set: 0,
        drift: [0.0, 0.0],
    };

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, Hash)]
pub enum AlphaMode {
    #[default]
    Opaque,
    Mask,
    Blend,
    Dither,
}

pub fn solid_material(color: [f32; 4], roughness: f32, metallic: f32) -> Material {
    Material {
        base_color: color,
        roughness,
        metallic,
        ..Default::default()
    }
}

pub fn alpha_mode_code(mode: AlphaMode) -> u32 {
    match mode {
        AlphaMode::Opaque => 0,
        AlphaMode::Mask => 1,
        AlphaMode::Blend => 2,
        AlphaMode::Dither => 3,
    }
}

pub const REVEAL_FRONT_DISABLED: f32 = 1.0e30;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Material {
    pub base_color: [f32; 4],
    pub emissive_factor: [f32; 3],
    pub alpha_mode: AlphaMode,
    pub alpha_cutoff: f32,
    pub blend_opaque_alpha_threshold: f32,
    pub base_texture: Option<String>,
    pub base_texture_transform: TextureTransform,
    pub atlas_cells: [f32; 2],
    pub sky_light: bool,
    pub vertex_occlusion: bool,
    pub emissive_texture: Option<String>,
    pub emissive_texture_transform: TextureTransform,
    pub normal_texture: Option<String>,
    pub normal_texture_transform: TextureTransform,
    pub normal_scale: f32,
    pub normal_map_flip_y: bool,
    pub normal_map_two_component: bool,
    pub detail_base_texture: Option<String>,
    pub detail_normal_texture: Option<String>,
    pub detail_uv_scale: f32,
    pub height_texture: Option<String>,
    pub parallax_scale: f32,
    pub reflection_box_min: [f32; 3],
    pub reflection_box_max: [f32; 3],
    pub metallic_roughness_texture: Option<String>,
    pub metallic_roughness_texture_transform: TextureTransform,
    pub occlusion_texture: Option<String>,
    pub occlusion_texture_transform: TextureTransform,
    pub occlusion_strength: f32,
    pub roughness: f32,
    pub metallic: f32,
    pub unlit: bool,
    pub double_sided: bool,
    pub transmission_factor: f32,
    pub transmission_texture: Option<String>,
    pub transmission_texture_transform: TextureTransform,
    pub thickness: f32,
    pub thickness_texture: Option<String>,
    pub thickness_texture_transform: TextureTransform,
    pub attenuation_color: [f32; 3],
    pub attenuation_distance: f32,
    pub ior: f32,
    pub specular_factor: f32,
    pub specular_color_factor: [f32; 3],
    pub specular_texture: Option<String>,
    pub specular_texture_transform: TextureTransform,
    pub specular_color_texture: Option<String>,
    pub specular_color_texture_transform: TextureTransform,
    pub emissive_strength: f32,
    pub diffuse_transmission_factor: f32,
    pub diffuse_transmission_texture: Option<String>,
    pub diffuse_transmission_texture_transform: TextureTransform,
    pub diffuse_transmission_color_factor: [f32; 3],
    pub diffuse_transmission_color_texture: Option<String>,
    pub diffuse_transmission_color_texture_transform: TextureTransform,
    pub dispersion: f32,
    pub anisotropy_strength: f32,
    pub anisotropy_rotation: f32,
    pub anisotropy_texture: Option<String>,
    pub anisotropy_texture_transform: TextureTransform,
    pub iridescence_factor: f32,
    pub iridescence_texture: Option<String>,
    pub iridescence_texture_transform: TextureTransform,
    pub iridescence_ior: f32,
    pub iridescence_thickness_minimum: f32,
    pub iridescence_thickness_maximum: f32,
    pub iridescence_thickness_texture: Option<String>,
    pub iridescence_thickness_texture_transform: TextureTransform,
    pub sheen_color_factor: [f32; 3],
    pub sheen_color_texture: Option<String>,
    pub sheen_color_texture_transform: TextureTransform,
    pub sheen_roughness_factor: f32,
    pub sheen_roughness_texture: Option<String>,
    pub sheen_roughness_texture_transform: TextureTransform,
    pub clearcoat_factor: f32,
    pub clearcoat_texture: Option<String>,
    pub clearcoat_texture_transform: TextureTransform,
    pub clearcoat_roughness_factor: f32,
    pub clearcoat_roughness_texture: Option<String>,
    pub clearcoat_roughness_texture_transform: TextureTransform,
    pub clearcoat_normal_texture: Option<String>,
    pub clearcoat_normal_texture_transform: TextureTransform,
    pub clearcoat_normal_scale: f32,
    pub reveal_normal: [f32; 3],
    pub reveal_front: f32,
    pub reveal_noise_scale: [f32; 3],
    pub reveal_noise_amplitude: f32,
    pub reveal_seam_width: f32,
    pub reveal_seam_color: [f32; 3],
    pub reveal_seam_strength: f32,
}

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

impl Material {
    pub fn is_transparent(&self) -> bool {
        matches!(self.alpha_mode, AlphaMode::Mask | AlphaMode::Blend)
    }

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
            atlas_cells: [0.0, 0.0],
            sky_light: false,
            vertex_occlusion: false,
            emissive_texture: None,
            emissive_texture_transform: TextureTransform::IDENTITY,
            normal_texture: None,
            normal_texture_transform: TextureTransform::IDENTITY,
            normal_scale: 1.0,
            normal_map_flip_y: false,
            normal_map_two_component: false,
            detail_base_texture: None,
            detail_normal_texture: None,
            detail_uv_scale: 4.0,
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
            reveal_normal: [0.0, 1.0, 0.0],
            reveal_front: REVEAL_FRONT_DISABLED,
            reveal_noise_scale: [8.0, 8.0, 8.0],
            reveal_noise_amplitude: 0.0,
            reveal_seam_width: 0.0,
            reveal_seam_color: [1.0, 1.0, 1.0],
            reveal_seam_strength: 0.0,
        }
    }
}
