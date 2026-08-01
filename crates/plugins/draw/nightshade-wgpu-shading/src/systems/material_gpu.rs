use nightshade_wgpu::backend::MATERIAL_SAMPLER_COUNT;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;
use std::collections::HashMap;

pub type MaterialSamplers = [wgpu::Sampler; MATERIAL_SAMPLER_COUNT];

pub fn material_sampler_entry(
    binding: u32,
    visibility: wgpu::ShaderStages,
) -> wgpu::BindGroupLayoutEntry {
    build::sampler_at(binding, wgpu::SamplerBindingType::Filtering, visibility)
}

pub fn material_sampler_entries(
    first_binding: u32,
    visibility: wgpu::ShaderStages,
) -> impl Iterator<Item = wgpu::BindGroupLayoutEntry> {
    (0..MATERIAL_SAMPLER_COUNT as u32)
        .map(move |offset| material_sampler_entry(first_binding + offset, visibility))
}

fn material_sampler_bind_entries(
    first_binding: u32,
    samplers: &MaterialSamplers,
) -> impl Iterator<Item = wgpu::BindGroupEntry<'_>> {
    samplers
        .iter()
        .enumerate()
        .map(move |(offset, sampler)| wgpu::BindGroupEntry {
            binding: first_binding + offset as u32,
            resource: wgpu::BindingResource::Sampler(sampler),
        })
}

pub fn material_texture_bind_group_layout_entries(
    bindless_max: Option<u32>,
) -> Vec<wgpu::BindGroupLayoutEntry> {
    let visibility = wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE;
    if let Some(max_textures) = bindless_max {
        let mut entries = vec![wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: std::num::NonZeroU32::new(max_textures),
        }];
        entries.extend(material_sampler_entries(1, visibility));
        return entries;
    }
    let mut entries = vec![
        build::texture_at(
            0,
            wgpu::TextureSampleType::Float { filterable: true },
            wgpu::TextureViewDimension::D2Array,
            visibility,
        ),
        build::texture_at(
            1,
            wgpu::TextureSampleType::Float { filterable: true },
            wgpu::TextureViewDimension::D2Array,
            visibility,
        ),
    ];
    entries.extend(material_sampler_entries(2, visibility));
    entries
}

pub fn bindless_material_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    views: &[&wgpu::TextureView],
    samplers: &MaterialSamplers,
    label: &str,
) -> wgpu::BindGroup {
    let mut entries = vec![wgpu::BindGroupEntry {
        binding: 0,
        resource: wgpu::BindingResource::TextureViewArray(views),
    }];
    entries.extend(material_sampler_bind_entries(1, samplers));
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &entries,
    })
}

pub fn array_material_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    srgb_view: &wgpu::TextureView,
    linear_view: &wgpu::TextureView,
    samplers: &MaterialSamplers,
    label: &str,
) -> wgpu::BindGroup {
    let mut entries = vec![
        wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(srgb_view),
        },
        wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::TextureView(linear_view),
        },
    ];
    entries.extend(material_sampler_bind_entries(2, samplers));
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &entries,
    })
}

pub fn rebuild_array_material_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    srgb: Option<&wgpu::TextureView>,
    linear: Option<&wgpu::TextureView>,
    samplers: Option<&MaterialSamplers>,
    label: &str,
) -> Option<wgpu::BindGroup> {
    let (Some(srgb), Some(linear), Some(samplers)) = (srgb, linear, samplers) else {
        return None;
    };
    Some(array_material_bind_group(
        device, layout, srgb, linear, samplers, label,
    ))
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuTextureTransform {
    pub row0: [f32; 4],
    pub row1: [f32; 4],
    pub drift: [f32; 4],
}

impl GpuTextureTransform {
    pub const IDENTITY: Self = Self {
        row0: [1.0, 0.0, 0.0, 0.0],
        row1: [0.0, 1.0, 0.0, 0.0],
        drift: [0.0; 4],
    };

    pub fn from_material(t: &nightshade_render::data::material::TextureTransform) -> Self {
        let (r0, r1) = t.to_packed();
        Self {
            row0: [r0[0], r0[1], r0[2], t.uv_set as f32],
            row1: [r1[0], r1[1], r1[2], 0.0],
            drift: [t.drift[0], t.drift[1], 0.0, 0.0],
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
    pub sky_light: u32,
    pub vertex_occlusion: u32,
    pub specular_color_factor: [f32; 3],
    pub specular_layer: u32,
    pub specular_color_layer: u32,
    pub emissive_strength: f32,
    pub atlas_cells: [f32; 2],
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
    pub pad_clearcoat: [u32; 2],
    pub clearcoat_transform: GpuTextureTransform,
    pub clearcoat_roughness_transform: GpuTextureTransform,
    pub clearcoat_normal_transform: GpuTextureTransform,
    pub sheen_color_factor: [f32; 3],
    pub sheen_roughness_factor: f32,
    pub sheen_color_layer: u32,
    pub pad_sheen_0: u32,
    pub pad_sheen: [u32; 2],
    pub sheen_color_transform: GpuTextureTransform,
    pub sheen_roughness_transform: GpuTextureTransform,
    pub iridescence_factor: f32,
    pub iridescence_ior: f32,
    pub iridescence_thickness_minimum: f32,
    pub iridescence_thickness_maximum: f32,
    pub iridescence_layer: u32,
    pub pad_iridescence_0: u32,
    pub pad_iridescence: [u32; 2],
    pub iridescence_transform: GpuTextureTransform,
    pub iridescence_thickness_transform: GpuTextureTransform,
    pub anisotropy_strength: f32,
    pub anisotropy_rotation_cos: f32,
    pub anisotropy_rotation_sin: f32,
    pub anisotropy_layer: u32,
    pub anisotropy_transform: GpuTextureTransform,
    pub dispersion: f32,
    pub diffuse_transmission_factor: f32,
    pub pad_diffuse_transmission: u32,
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
    pub pad_detail: [u32; 3],
    pub reflection_box_min: [f32; 3],
    pub pad_box_min: f32,
    pub reflection_box_max: [f32; 3],
    pub pad_box_max: f32,
    pub reveal_normal: [f32; 3],
    pub reveal_front: f32,
    pub reveal_noise_scale: [f32; 3],
    pub reveal_noise_amplitude: f32,
    pub reveal_seam_color: [f32; 3],
    pub reveal_seam_width: f32,
    pub reveal_seam_strength: f32,
    pub pad_reveal: [f32; 3],
}

pub const NO_TEXTURE_LAYER: u32 = 0xFFFFFFFFu32;

pub const NORMAL_MAP_FLIP_Y: u32 = 1;
pub const NORMAL_MAP_TWO_COMPONENT: u32 = 2;

pub fn material_layer_for(
    layer_map: &HashMap<
        nightshade_render::data::asset_id::TextureId,
        nightshade_wgpu::backend::MaterialTextureLayer,
    >,
    id: Option<nightshade_render::data::asset_id::TextureId>,
) -> u32 {
    id.and_then(|i| layer_map.get(&i))
        .map(|l| l.packed())
        .unwrap_or(NO_TEXTURE_LAYER)
}

pub fn convert_material_to_gpu_data(
    material: &nightshade_render::data::material::Material,
    texture_ids: &nightshade_render::data::material::MaterialTextureIds,
    layer_map: &HashMap<
        nightshade_render::data::asset_id::TextureId,
        nightshade_wgpu::backend::MaterialTextureLayer,
    >,
) -> MaterialData {
    let alpha_mode = nightshade_render::data::material::alpha_mode_code(material.alpha_mode);
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
        sky_light: u32::from(material.sky_light),
        vertex_occlusion: u32::from(material.vertex_occlusion),
        specular_color_factor: material.specular_color_factor,
        specular_layer: material_layer_for(layer_map, texture_ids.specular),
        specular_color_layer: material_layer_for(layer_map, texture_ids.specular_color),
        emissive_strength: material.emissive_strength,
        atlas_cells: material.atlas_cells,
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
        pad_clearcoat: [0, 0],
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
        pad_sheen_0: 0,
        pad_sheen: [0, 0],
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
        pad_iridescence_0: 0,
        pad_iridescence: [0, 0],
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
        pad_diffuse_transmission: 0,
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
        pad_detail: [0, 0, 0],
        reflection_box_min: material.reflection_box_min,
        pad_box_min: 0.0,
        reflection_box_max: material.reflection_box_max,
        pad_box_max: 0.0,
        reveal_normal: material.reveal_normal,
        reveal_front: material.reveal_front,
        reveal_noise_scale: material.reveal_noise_scale,
        reveal_noise_amplitude: material.reveal_noise_amplitude,
        reveal_seam_color: material.reveal_seam_color,
        reveal_seam_width: material.reveal_seam_width,
        reveal_seam_strength: material.reveal_seam_strength,
        pad_reveal: [0.0, 0.0, 0.0],
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
        sky_light: 0,
        vertex_occlusion: 0,
        specular_color_factor: [1.0, 1.0, 1.0],
        specular_layer: NO_TEXTURE_LAYER,
        specular_color_layer: NO_TEXTURE_LAYER,
        emissive_strength: 1.0,
        atlas_cells: [0.0, 0.0],
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
        pad_clearcoat: [0, 0],
        clearcoat_transform: GpuTextureTransform::IDENTITY,
        clearcoat_roughness_transform: GpuTextureTransform::IDENTITY,
        clearcoat_normal_transform: GpuTextureTransform::IDENTITY,
        sheen_color_factor: [0.0, 0.0, 0.0],
        sheen_roughness_factor: 0.0,
        sheen_color_layer: NO_TEXTURE_LAYER,
        pad_sheen_0: 0,
        pad_sheen: [0, 0],
        sheen_color_transform: GpuTextureTransform::IDENTITY,
        sheen_roughness_transform: GpuTextureTransform::IDENTITY,
        iridescence_factor: 0.0,
        iridescence_ior: 1.3,
        iridescence_thickness_minimum: 100.0,
        iridescence_thickness_maximum: 400.0,
        iridescence_layer: NO_TEXTURE_LAYER,
        pad_iridescence_0: 0,
        pad_iridescence: [0, 0],
        iridescence_transform: GpuTextureTransform::IDENTITY,
        iridescence_thickness_transform: GpuTextureTransform::IDENTITY,
        anisotropy_strength: 0.0,
        anisotropy_rotation_cos: 1.0,
        anisotropy_rotation_sin: 0.0,
        anisotropy_layer: NO_TEXTURE_LAYER,
        anisotropy_transform: GpuTextureTransform::IDENTITY,
        dispersion: 0.0,
        diffuse_transmission_factor: 0.0,
        pad_diffuse_transmission: 0,
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
        pad_detail: [0, 0, 0],
        reflection_box_min: [0.0, 0.0, 0.0],
        pad_box_min: 0.0,
        reflection_box_max: [0.0, 0.0, 0.0],
        pad_box_max: 0.0,
        reveal_normal: [0.0, 1.0, 0.0],
        reveal_front: nightshade_render::data::material::REVEAL_FRONT_DISABLED,
        reveal_noise_scale: [8.0, 8.0, 8.0],
        reveal_noise_amplitude: 0.0,
        reveal_seam_color: [1.0, 1.0, 1.0],
        reveal_seam_width: 0.0,
        reveal_seam_strength: 0.0,
        pad_reveal: [0.0, 0.0, 0.0],
    }
}
