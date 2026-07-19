use std::collections::HashMap;

/// Builds the bindless material bind group (a texture-view array at binding 0 and
/// a shared sampler at binding 1). Shared by the mesh and skinned mesh passes,
/// which differ only in the debug label.
fn material_sampler_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    }
}

/// The layout `material_sampling` expects, which every pass that shades from a
/// material binds. Bindless collapses the two arrays into one indexed binding;
/// otherwise the srgb and linear arrays are bound separately, and either way
/// the three samplers are the wrap modes the shader picks between.
pub fn material_texture_bind_group_layout_entries(
    bindless_max: Option<u32>,
) -> Vec<wgpu::BindGroupLayoutEntry> {
    if let Some(max_textures) = bindless_max {
        return vec![
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: std::num::NonZeroU32::new(max_textures),
            },
            material_sampler_entry(1),
            material_sampler_entry(2),
            material_sampler_entry(3),
        ];
    }
    vec![
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2Array,
                multisampled: false,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2Array,
                multisampled: false,
            },
            count: None,
        },
        material_sampler_entry(2),
        material_sampler_entry(3),
        material_sampler_entry(4),
    ]
}

/// Builds the bindless material bind group (a texture-view array at binding 0 and
/// the three wrap-mode samplers at bindings 1 through 3). Shared by the mesh and
/// skinned mesh passes, which differ only in the debug label.
pub fn bindless_material_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    views: &[&wgpu::TextureView],
    samplers: &[wgpu::Sampler; 3],
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
                resource: wgpu::BindingResource::Sampler(&samplers[0]),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&samplers[1]),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(&samplers[2]),
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
    samplers: &[wgpu::Sampler; 3],
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
                resource: wgpu::BindingResource::Sampler(&samplers[0]),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(&samplers[1]),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::Sampler(&samplers[2]),
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
    samplers: Option<&[wgpu::Sampler; 3]>,
    label: &str,
) -> Option<wgpu::BindGroup> {
    let (Some(srgb), Some(linear), Some(samplers)) = (srgb, linear, samplers) else {
        return None;
    };
    Some(array_material_bind_group(
        device, layout, srgb, linear, samplers, label,
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
    /// First affine row `(m00, m01, m02, uv_set)`, where `uv_set` selects the UV channel.
    pub row0: [f32; 4],
    /// Second affine row `(m10, m11, m12, _pad)`.
    pub row1: [f32; 4],
}

impl GpuTextureTransform {
    /// Identity UV transform on UV set 0.
    pub const IDENTITY: Self = Self {
        row0: [1.0, 0.0, 0.0, 0.0],
        row1: [0.0, 1.0, 0.0, 0.0],
    };

    /// Packs a material [`crate::material::TextureTransform`] into the GPU layout.
    pub fn from_material(t: &crate::material::TextureTransform) -> Self {
        let (r0, r1) = t.to_packed();
        Self {
            row0: [r0[0], r0[1], r0[2], t.uv_set as f32],
            row1: [r1[0], r1[1], r1[2], 0.0],
        }
    }
}

/// GPU-side material parameters uploaded per material, matching the shader's
/// `MaterialData` struct for PBR shading and the glTF PBR extensions. Texture
/// slots are array-layer indices, or [`NO_TEXTURE_LAYER`] when absent.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialData {
    /// Base color factor, linear RGBA.
    pub base_color: [f32; 4],
    /// Emissive color factor, linear RGB.
    pub emissive_factor: [f32; 3],
    /// Alpha mode: 0 opaque, 1 mask, 2 blend, 3 dither.
    pub alpha_mode: u32,
    /// Alpha threshold below which mask-mode fragments are discarded.
    pub alpha_cutoff: f32,
    /// Array-layer index of the base color texture.
    pub base_layer: u32,
    /// Array-layer index of the emissive texture.
    pub emissive_layer: u32,
    /// Array-layer index of the normal map.
    pub normal_layer: u32,
    /// Array-layer index of the metallic-roughness texture.
    pub metallic_roughness_layer: u32,
    /// Array-layer index of the occlusion texture.
    pub occlusion_layer: u32,
    /// Normal map strength.
    pub normal_scale: f32,
    /// Occlusion map strength.
    pub occlusion_strength: f32,
    /// Roughness factor.
    pub roughness: f32,
    /// Metallic factor.
    pub metallic: f32,
    /// Nonzero disables lighting and outputs the base color directly.
    pub unlit: u32,
    /// Bitmask of [`NORMAL_MAP_FLIP_Y`] and [`NORMAL_MAP_TWO_COMPONENT`].
    pub normal_map_flags: u32,
    /// `KHR_materials_transmission` transmission factor.
    pub transmission_factor: f32,
    /// Array-layer index of the transmission texture.
    pub transmission_layer: u32,
    /// `KHR_materials_volume` thickness, in world units.
    pub thickness: f32,
    /// Array-layer index of the thickness texture.
    pub thickness_layer: u32,
    /// Volume attenuation color, linear RGB.
    pub attenuation_color: [f32; 3],
    /// Distance over which light is attenuated inside the volume, in world units.
    pub attenuation_distance: f32,
    /// Index of refraction.
    pub ior: f32,
    /// `KHR_materials_specular` specular strength factor.
    pub specular_factor: f32,
    /// Padding for 16-byte alignment.
    pub _align_specular: [u32; 2],
    /// Specular color factor, linear RGB.
    pub specular_color_factor: [f32; 3],
    /// Array-layer index of the specular texture.
    pub specular_layer: u32,
    /// Array-layer index of the specular color texture.
    pub specular_color_layer: u32,
    /// `KHR_materials_emissive_strength` multiplier on the emissive factor.
    pub emissive_strength: f32,
    /// Padding before the UV-transform block.
    pub _pad_before_transforms: [u32; 2],
    /// UV transform for the base color texture.
    pub base_transform: GpuTextureTransform,
    /// UV transform for the emissive texture.
    pub emissive_transform: GpuTextureTransform,
    /// UV transform for the normal map.
    pub normal_transform: GpuTextureTransform,
    /// UV transform for the metallic-roughness texture.
    pub metallic_roughness_transform: GpuTextureTransform,
    /// UV transform for the occlusion texture.
    pub occlusion_transform: GpuTextureTransform,
    /// UV transform for the transmission texture.
    pub transmission_transform: GpuTextureTransform,
    /// UV transform for the thickness texture.
    pub thickness_transform: GpuTextureTransform,
    /// UV transform for the specular texture.
    pub specular_transform: GpuTextureTransform,
    /// UV transform for the specular color texture.
    pub specular_color_transform: GpuTextureTransform,
    /// `KHR_materials_clearcoat` clearcoat strength factor.
    pub clearcoat_factor: f32,
    /// Clearcoat roughness factor.
    pub clearcoat_roughness_factor: f32,
    /// Clearcoat normal map strength.
    pub clearcoat_normal_scale: f32,
    /// Array-layer index of the clearcoat texture.
    pub clearcoat_layer: u32,
    /// Array-layer index of the clearcoat roughness texture.
    pub clearcoat_roughness_layer: u32,
    /// Array-layer index of the clearcoat normal map.
    pub clearcoat_normal_layer: u32,
    /// Padding for 16-byte alignment.
    pub _pad_clearcoat: [u32; 2],
    /// UV transform for the clearcoat texture.
    pub clearcoat_transform: GpuTextureTransform,
    /// UV transform for the clearcoat roughness texture.
    pub clearcoat_roughness_transform: GpuTextureTransform,
    /// UV transform for the clearcoat normal map.
    pub clearcoat_normal_transform: GpuTextureTransform,
    /// `KHR_materials_sheen` sheen color factor, linear RGB.
    pub sheen_color_factor: [f32; 3],
    /// Sheen roughness factor.
    pub sheen_roughness_factor: f32,
    /// Array-layer index of the sheen color texture.
    pub sheen_color_layer: u32,
    /// Padding for 16-byte alignment.
    pub _pad_sheen0: u32,
    /// Padding for 16-byte alignment.
    pub _pad_sheen: [u32; 2],
    /// UV transform for the sheen color texture.
    pub sheen_color_transform: GpuTextureTransform,
    /// UV transform for the sheen roughness texture.
    pub sheen_roughness_transform: GpuTextureTransform,
    /// `KHR_materials_iridescence` iridescence strength factor.
    pub iridescence_factor: f32,
    /// Iridescence thin-film index of refraction.
    pub iridescence_ior: f32,
    /// Minimum thin-film thickness, in nanometers.
    pub iridescence_thickness_minimum: f32,
    /// Maximum thin-film thickness, in nanometers.
    pub iridescence_thickness_maximum: f32,
    /// Array-layer index of the iridescence texture.
    pub iridescence_layer: u32,
    /// Padding for 16-byte alignment.
    pub _pad_iridescence0: u32,
    /// Padding for 16-byte alignment.
    pub _pad_iridescence: [u32; 2],
    /// UV transform for the iridescence texture.
    pub iridescence_transform: GpuTextureTransform,
    /// UV transform for the iridescence thickness texture.
    pub iridescence_thickness_transform: GpuTextureTransform,
    /// `KHR_materials_anisotropy` anisotropy strength.
    pub anisotropy_strength: f32,
    /// Cosine of the anisotropy rotation angle.
    pub anisotropy_rotation_cos: f32,
    /// Sine of the anisotropy rotation angle.
    pub anisotropy_rotation_sin: f32,
    /// Array-layer index of the anisotropy texture.
    pub anisotropy_layer: u32,
    /// UV transform for the anisotropy texture.
    pub anisotropy_transform: GpuTextureTransform,
    /// `KHR_materials_dispersion` dispersion strength.
    pub dispersion: f32,
    /// `KHR_materials_diffuse_transmission` diffuse transmission factor.
    pub diffuse_transmission_factor: f32,
    /// Padding for 16-byte alignment.
    pub _pad_diffuse_transmission0: u32,
    /// Array-layer index of the diffuse transmission color texture.
    pub diffuse_transmission_color_layer: u32,
    /// Diffuse transmission color factor, linear RGB.
    pub diffuse_transmission_color_factor: [f32; 3],
    /// Alpha above which a blend-mode fragment is treated as opaque.
    pub blend_opaque_alpha_threshold: f32,
    /// UV transform for the diffuse transmission texture.
    pub diffuse_transmission_transform: GpuTextureTransform,
    /// UV transform for the diffuse transmission color texture.
    pub diffuse_transmission_color_transform: GpuTextureTransform,
    /// Array-layer index of the detail base color texture.
    pub detail_base_layer: u32,
    /// Array-layer index of the detail normal map.
    pub detail_normal_layer: u32,
    /// Array-layer index of the height map used for parallax.
    pub height_layer: u32,
    /// UV tiling scale applied to the detail textures.
    pub detail_uv_scale: f32,
    /// Parallax occlusion displacement scale.
    pub parallax_scale: f32,
    /// Padding for 16-byte alignment.
    pub _pad_detail: [u32; 3],
    /// Minimum corner of the box-projection reflection bounds, world space.
    pub reflection_box_min: [f32; 3],
    /// Padding for 16-byte alignment.
    pub _pad_box_min: f32,
    /// Maximum corner of the box-projection reflection bounds, world space.
    pub reflection_box_max: [f32; 3],
    /// Padding for 16-byte alignment.
    pub _pad_box_max: f32,
}

/// Sentinel layer index marking a material texture slot as absent.
pub const NO_TEXTURE_LAYER: u32 = 0xFFFFFFFFu32;

/// [`MaterialData::normal_map_flags`] bit: reconstruct the normal with the green
/// channel flipped.
pub const NORMAL_MAP_FLIP_Y: u32 = 1;
/// [`MaterialData::normal_map_flags`] bit: the normal map stores two channels and
/// the shader reconstructs Z.
pub const NORMAL_MAP_TWO_COMPONENT: u32 = 2;

/// Resolves a texture id to its packed array-layer index, or [`NO_TEXTURE_LAYER`]
/// when the id is `None` or unmapped.
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

/// Converts a CPU [`crate::material::Material`] and its texture ids into the
/// packed [`MaterialData`], resolving each texture slot through `layer_map`.
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

/// Builds the built-in default [`MaterialData`] with the given base color and no
/// textures, matching material id 0.
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
