use std::collections::HashMap;

/// Builds the bindless material bind group (a texture-view array at binding 0 and
/// a shared sampler at binding 1). Shared by the mesh and skinned mesh passes,
/// which differ only in the debug label.
pub fn bindless_material_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    views: &[&wgpu::TextureView],
    sampler: &wgpu::Sampler,
    label: &str,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureViewArray(views),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}

/// Builds the array-emulated material bind group (sRGB and linear texture arrays
/// plus a shared sampler). Shared by the mesh and skinned mesh passes, which
/// differ only in the debug label.
pub fn array_material_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    srgb_view: &wgpu::TextureView,
    linear_view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
    label: &str,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(srgb_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(linear_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}

/// Rebuilds the array material bind group from the stored slots, or returns
/// `None` when any slot is empty. Shared by the mesh and skinned mesh passes.
pub fn rebuild_array_material_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    srgb: Option<&wgpu::TextureView>,
    linear: Option<&wgpu::TextureView>,
    sampler: Option<&wgpu::Sampler>,
    label: &str,
) -> Option<wgpu::BindGroup> {
    let (Some(srgb), Some(linear), Some(sampler)) = (srgb, linear, sampler) else {
        return None;
    };
    Some(array_material_bind_group(
        device, layout, srgb, linear, sampler, label,
    ))
}

/// Per-texture UV transform packed for std140-compatible upload.
///
/// Layout (32 bytes total):
/// row0 = (m00, m01, m02, uv_set)
/// row1 = (m10, m11, m12, _pad)
/// where m is the 2x3 affine UV matrix `T(offset) * R(rotation) * S(scale)`.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuTextureTransform {
    pub row0: [f32; 4],
    pub row1: [f32; 4],
}

impl GpuTextureTransform {
    pub const IDENTITY: Self = Self {
        row0: [1.0, 0.0, 0.0, 0.0],
        row1: [0.0, 1.0, 0.0, 0.0],
    };

    pub fn from_material(t: &crate::material::TextureTransform) -> Self {
        let (r0, r1) = t.to_packed();
        Self {
            row0: [r0[0], r0[1], r0[2], t.uv_set as f32],
            row1: [r1[0], r1[1], r1[2], 0.0],
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialData {
    pub base_color: [f32; 4],
    pub emissive_factor: [f32; 3],
    pub alpha_mode: u32,
    pub alpha_cutoff: f32,
    pub base_layer: u32,
    pub emissive_layer: u32,
    pub normal_layer: u32,
    pub metallic_roughness_layer: u32,
    pub occlusion_layer: u32,
    pub normal_scale: f32,
    pub occlusion_strength: f32,
    pub roughness: f32,
    pub metallic: f32,
    pub unlit: u32,
    pub normal_map_flags: u32,
    pub transmission_factor: f32,
    pub transmission_layer: u32,
    pub thickness: f32,
    pub thickness_layer: u32,
    pub attenuation_color: [f32; 3],
    pub attenuation_distance: f32,
    pub ior: f32,
    pub specular_factor: f32,
    pub _align_specular: [u32; 2],
    pub specular_color_factor: [f32; 3],
    pub specular_layer: u32,
    pub specular_color_layer: u32,
    pub emissive_strength: f32,
    pub _pad_before_transforms: [u32; 2],
    pub base_transform: GpuTextureTransform,
    pub emissive_transform: GpuTextureTransform,
    pub normal_transform: GpuTextureTransform,
    pub metallic_roughness_transform: GpuTextureTransform,
    pub occlusion_transform: GpuTextureTransform,
    pub transmission_transform: GpuTextureTransform,
    pub thickness_transform: GpuTextureTransform,
    pub specular_transform: GpuTextureTransform,
    pub specular_color_transform: GpuTextureTransform,
    pub clearcoat_factor: f32,
    pub clearcoat_roughness_factor: f32,
    pub clearcoat_normal_scale: f32,
    pub clearcoat_layer: u32,
    pub clearcoat_roughness_layer: u32,
    pub clearcoat_normal_layer: u32,
    pub _pad_clearcoat: [u32; 2],
    pub clearcoat_transform: GpuTextureTransform,
    pub clearcoat_roughness_transform: GpuTextureTransform,
    pub clearcoat_normal_transform: GpuTextureTransform,
    pub sheen_color_factor: [f32; 3],
    pub sheen_roughness_factor: f32,
    pub sheen_color_layer: u32,
    pub _pad_sheen0: u32,
    pub _pad_sheen: [u32; 2],
    pub sheen_color_transform: GpuTextureTransform,
    pub sheen_roughness_transform: GpuTextureTransform,
    pub iridescence_factor: f32,
    pub iridescence_ior: f32,
    pub iridescence_thickness_minimum: f32,
    pub iridescence_thickness_maximum: f32,
    pub iridescence_layer: u32,
    pub _pad_iridescence0: u32,
    pub _pad_iridescence: [u32; 2],
    pub iridescence_transform: GpuTextureTransform,
    pub iridescence_thickness_transform: GpuTextureTransform,
    pub anisotropy_strength: f32,
    pub anisotropy_rotation_cos: f32,
    pub anisotropy_rotation_sin: f32,
    pub anisotropy_layer: u32,
    pub anisotropy_transform: GpuTextureTransform,
    pub dispersion: f32,
    pub diffuse_transmission_factor: f32,
    pub _pad_diffuse_transmission0: u32,
    pub diffuse_transmission_color_layer: u32,
    pub diffuse_transmission_color_factor: [f32; 3],
    pub blend_opaque_alpha_threshold: f32,
    pub diffuse_transmission_transform: GpuTextureTransform,
    pub diffuse_transmission_color_transform: GpuTextureTransform,
    pub detail_base_layer: u32,
    pub detail_normal_layer: u32,
    pub height_layer: u32,
    pub detail_uv_scale: f32,
    pub parallax_scale: f32,
    pub _pad_detail: [u32; 3],
    pub reflection_box_min: [f32; 3],
    pub _pad_box_min: f32,
    pub reflection_box_max: [f32; 3],
    pub _pad_box_max: f32,
}

pub const NO_TEXTURE_LAYER: u32 = 0xFFFFFFFFu32;

pub const NORMAL_MAP_FLIP_Y: u32 = 1;
pub const NORMAL_MAP_TWO_COMPONENT: u32 = 2;

pub fn material_layer_for(
    layer_map: &HashMap<
        crate::asset_id::TextureId,
        crate::wgpu::material_texture_arrays::MaterialTextureLayer,
    >,
    id: Option<crate::asset_id::TextureId>,
) -> u32 {
    id.and_then(|i| layer_map.get(&i))
        .map(|l| l.packed())
        .unwrap_or(NO_TEXTURE_LAYER)
}

pub fn convert_material_to_gpu_data(
    material: &crate::material::Material,
    texture_ids: &crate::material::MaterialTextureIds,
    layer_map: &HashMap<
        crate::asset_id::TextureId,
        crate::wgpu::material_texture_arrays::MaterialTextureLayer,
    >,
) -> MaterialData {
    let alpha_mode = match material.alpha_mode {
        crate::material::AlphaMode::Opaque => 0,
        crate::material::AlphaMode::Mask => 1,
        crate::material::AlphaMode::Blend => 2,
        crate::material::AlphaMode::Dither => 3,
    };
    let normal_map_flags = if material.normal_map_flip_y {
        NORMAL_MAP_FLIP_Y
    } else {
        0
    } | if material.normal_map_two_component {
        NORMAL_MAP_TWO_COMPONENT
    } else {
        0
    };
    MaterialData {
        base_color: material.base_color,
        emissive_factor: material.emissive_factor,
        alpha_mode,
        alpha_cutoff: material.alpha_cutoff,
        base_layer: material_layer_for(layer_map, texture_ids.base),
        emissive_layer: material_layer_for(layer_map, texture_ids.emissive),
        normal_layer: material_layer_for(layer_map, texture_ids.normal),
        metallic_roughness_layer: material_layer_for(layer_map, texture_ids.metallic_roughness),
        occlusion_layer: material_layer_for(layer_map, texture_ids.occlusion),
        normal_scale: material.normal_scale,
        occlusion_strength: material.occlusion_strength,
        roughness: material.roughness,
        metallic: material.metallic,
        unlit: material.unlit as u32,
        normal_map_flags,
        transmission_factor: material.transmission_factor,
        transmission_layer: material_layer_for(layer_map, texture_ids.transmission),
        thickness: material.thickness,
        thickness_layer: material_layer_for(layer_map, texture_ids.thickness),
        attenuation_color: material.attenuation_color,
        attenuation_distance: material.attenuation_distance,
        ior: material.ior,
        specular_factor: material.specular_factor,
        _align_specular: [0, 0],
        specular_color_factor: material.specular_color_factor,
        specular_layer: material_layer_for(layer_map, texture_ids.specular),
        specular_color_layer: material_layer_for(layer_map, texture_ids.specular_color),
        emissive_strength: material.emissive_strength,
        _pad_before_transforms: [0, 0],
        base_transform: GpuTextureTransform::from_material(&material.base_texture_transform),
        emissive_transform: GpuTextureTransform::from_material(
            &material.emissive_texture_transform,
        ),
        normal_transform: GpuTextureTransform::from_material(&material.normal_texture_transform),
        metallic_roughness_transform: GpuTextureTransform::from_material(
            &material.metallic_roughness_texture_transform,
        ),
        occlusion_transform: GpuTextureTransform::from_material(
            &material.occlusion_texture_transform,
        ),
        transmission_transform: GpuTextureTransform::from_material(
            &material.transmission_texture_transform,
        ),
        thickness_transform: GpuTextureTransform::from_material(
            &material.thickness_texture_transform,
        ),
        specular_transform: GpuTextureTransform::from_material(
            &material.specular_texture_transform,
        ),
        specular_color_transform: GpuTextureTransform::from_material(
            &material.specular_color_texture_transform,
        ),
        clearcoat_factor: material.clearcoat_factor,
        clearcoat_roughness_factor: material.clearcoat_roughness_factor,
        clearcoat_normal_scale: material.clearcoat_normal_scale,
        clearcoat_layer: material_layer_for(layer_map, texture_ids.clearcoat),
        clearcoat_roughness_layer: material_layer_for(layer_map, texture_ids.clearcoat_roughness),
        clearcoat_normal_layer: material_layer_for(layer_map, texture_ids.clearcoat_normal),
        _pad_clearcoat: [0, 0],
        clearcoat_transform: GpuTextureTransform::from_material(
            &material.clearcoat_texture_transform,
        ),
        clearcoat_roughness_transform: GpuTextureTransform::from_material(
            &material.clearcoat_roughness_texture_transform,
        ),
        clearcoat_normal_transform: GpuTextureTransform::from_material(
            &material.clearcoat_normal_texture_transform,
        ),
        sheen_color_factor: material.sheen_color_factor,
        sheen_roughness_factor: material.sheen_roughness_factor,
        sheen_color_layer: material_layer_for(layer_map, texture_ids.sheen_color),
        _pad_sheen0: 0,
        _pad_sheen: [0, 0],
        sheen_color_transform: GpuTextureTransform::from_material(
            &material.sheen_color_texture_transform,
        ),
        sheen_roughness_transform: GpuTextureTransform::from_material(
            &material.sheen_roughness_texture_transform,
        ),
        iridescence_factor: material.iridescence_factor,
        iridescence_ior: material.iridescence_ior,
        iridescence_thickness_minimum: material.iridescence_thickness_minimum,
        iridescence_thickness_maximum: material.iridescence_thickness_maximum,
        iridescence_layer: material_layer_for(layer_map, texture_ids.iridescence),
        _pad_iridescence0: 0,
        _pad_iridescence: [0, 0],
        iridescence_transform: GpuTextureTransform::from_material(
            &material.iridescence_texture_transform,
        ),
        iridescence_thickness_transform: GpuTextureTransform::from_material(
            &material.iridescence_thickness_texture_transform,
        ),
        anisotropy_strength: material.anisotropy_strength,
        anisotropy_rotation_cos: material.anisotropy_rotation.cos(),
        anisotropy_rotation_sin: material.anisotropy_rotation.sin(),
        anisotropy_layer: material_layer_for(layer_map, texture_ids.anisotropy),
        anisotropy_transform: GpuTextureTransform::from_material(
            &material.anisotropy_texture_transform,
        ),
        dispersion: material.dispersion,
        diffuse_transmission_factor: material.diffuse_transmission_factor,
        _pad_diffuse_transmission0: 0,
        diffuse_transmission_color_layer: material_layer_for(
            layer_map,
            texture_ids.diffuse_transmission_color,
        ),
        diffuse_transmission_color_factor: material.diffuse_transmission_color_factor,
        blend_opaque_alpha_threshold: material.blend_opaque_alpha_threshold,
        diffuse_transmission_transform: GpuTextureTransform::from_material(
            &material.diffuse_transmission_texture_transform,
        ),
        diffuse_transmission_color_transform: GpuTextureTransform::from_material(
            &material.diffuse_transmission_color_texture_transform,
        ),
        detail_base_layer: material_layer_for(layer_map, texture_ids.detail_base),
        detail_normal_layer: material_layer_for(layer_map, texture_ids.detail_normal),
        height_layer: material_layer_for(layer_map, texture_ids.height),
        detail_uv_scale: material.detail_uv_scale,
        parallax_scale: material.parallax_scale,
        _pad_detail: [0, 0, 0],
        reflection_box_min: material.reflection_box_min,
        _pad_box_min: 0.0,
        reflection_box_max: material.reflection_box_max,
        _pad_box_max: 0.0,
    }
}

pub fn default_material_data(base_color: [f32; 4]) -> MaterialData {
    MaterialData {
        base_color,
        emissive_factor: [0.0, 0.0, 0.0],
        alpha_mode: 0,
        alpha_cutoff: 0.5,
        base_layer: NO_TEXTURE_LAYER,
        emissive_layer: NO_TEXTURE_LAYER,
        normal_layer: NO_TEXTURE_LAYER,
        metallic_roughness_layer: NO_TEXTURE_LAYER,
        occlusion_layer: NO_TEXTURE_LAYER,
        normal_scale: 1.0,
        occlusion_strength: 1.0,
        roughness: 0.5,
        metallic: 0.0,
        unlit: 0,
        normal_map_flags: 0,
        transmission_factor: 0.0,
        transmission_layer: NO_TEXTURE_LAYER,
        thickness: 0.0,
        thickness_layer: NO_TEXTURE_LAYER,
        attenuation_color: [1.0, 1.0, 1.0],
        attenuation_distance: f32::INFINITY,
        ior: 1.5,
        specular_factor: 1.0,
        _align_specular: [0, 0],
        specular_color_factor: [1.0, 1.0, 1.0],
        specular_layer: NO_TEXTURE_LAYER,
        specular_color_layer: NO_TEXTURE_LAYER,
        emissive_strength: 1.0,
        _pad_before_transforms: [0, 0],
        base_transform: GpuTextureTransform::IDENTITY,
        emissive_transform: GpuTextureTransform::IDENTITY,
        normal_transform: GpuTextureTransform::IDENTITY,
        metallic_roughness_transform: GpuTextureTransform::IDENTITY,
        occlusion_transform: GpuTextureTransform::IDENTITY,
        transmission_transform: GpuTextureTransform::IDENTITY,
        thickness_transform: GpuTextureTransform::IDENTITY,
        specular_transform: GpuTextureTransform::IDENTITY,
        specular_color_transform: GpuTextureTransform::IDENTITY,
        clearcoat_factor: 0.0,
        clearcoat_roughness_factor: 0.0,
        clearcoat_normal_scale: 1.0,
        clearcoat_layer: NO_TEXTURE_LAYER,
        clearcoat_roughness_layer: NO_TEXTURE_LAYER,
        clearcoat_normal_layer: NO_TEXTURE_LAYER,
        _pad_clearcoat: [0, 0],
        clearcoat_transform: GpuTextureTransform::IDENTITY,
        clearcoat_roughness_transform: GpuTextureTransform::IDENTITY,
        clearcoat_normal_transform: GpuTextureTransform::IDENTITY,
        sheen_color_factor: [0.0, 0.0, 0.0],
        sheen_roughness_factor: 0.0,
        sheen_color_layer: NO_TEXTURE_LAYER,
        _pad_sheen0: 0,
        _pad_sheen: [0, 0],
        sheen_color_transform: GpuTextureTransform::IDENTITY,
        sheen_roughness_transform: GpuTextureTransform::IDENTITY,
        iridescence_factor: 0.0,
        iridescence_ior: 1.3,
        iridescence_thickness_minimum: 100.0,
        iridescence_thickness_maximum: 400.0,
        iridescence_layer: NO_TEXTURE_LAYER,
        _pad_iridescence0: 0,
        _pad_iridescence: [0, 0],
        iridescence_transform: GpuTextureTransform::IDENTITY,
        iridescence_thickness_transform: GpuTextureTransform::IDENTITY,
        anisotropy_strength: 0.0,
        anisotropy_rotation_cos: 1.0,
        anisotropy_rotation_sin: 0.0,
        anisotropy_layer: NO_TEXTURE_LAYER,
        anisotropy_transform: GpuTextureTransform::IDENTITY,
        dispersion: 0.0,
        diffuse_transmission_factor: 0.0,
        _pad_diffuse_transmission0: 0,
        diffuse_transmission_color_layer: NO_TEXTURE_LAYER,
        diffuse_transmission_color_factor: [1.0, 1.0, 1.0],
        blend_opaque_alpha_threshold: 0.99,
        diffuse_transmission_transform: GpuTextureTransform::IDENTITY,
        diffuse_transmission_color_transform: GpuTextureTransform::IDENTITY,
        detail_base_layer: NO_TEXTURE_LAYER,
        detail_normal_layer: NO_TEXTURE_LAYER,
        height_layer: NO_TEXTURE_LAYER,
        detail_uv_scale: 4.0,
        parallax_scale: 0.0,
        _pad_detail: [0, 0, 0],
        reflection_box_min: [0.0, 0.0, 0.0],
        _pad_box_min: 0.0,
        reflection_box_max: [0.0, 0.0, 0.0],
        _pad_box_max: 0.0,
    }
}
