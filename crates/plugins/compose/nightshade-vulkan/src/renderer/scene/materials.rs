use crate::renderer::context::Context;
use crate::renderer::memory::{Buffer, create_buffer, destroy_buffer, upload_to_buffer};
use ash::vk;
use gpu_allocator::MemoryLocation;
use nightshade_render::data::config::RenderMaterials;
use nightshade_render::data::material::alpha_mode_code;
use nightshade_render::data::material::{Material, MaterialTextureIds};
use nightshade_render::queries::texture::{NO_TEXTURE_SLOT, pack_texture_slot};

use crate::renderer::scene::textures::{SceneTextures, texture_sampler};

const INITIAL_MATERIAL_CAPACITY: u64 = 1 << 8;

pub const FLAG_DOUBLE_SIDED: u32 = 1;
pub const FLAG_UNLIT: u32 = 2;
pub const FLAG_NORMAL_FLIP_Y: u32 = 4;
pub const FLAG_NORMAL_TWO_COMPONENT: u32 = 8;
pub const FLAG_SKY_LIGHT: u32 = 16;
pub const FLAG_VERTEX_OCCLUSION: u32 = 32;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuMaterial {
    pub base_color: [f32; 4],
    pub emissive: [f32; 4],
    pub factors: [f32; 4],
    pub alpha: [f32; 4],
    pub transmission: [f32; 4],
    pub attenuation: [f32; 4],
    pub specular_color: [f32; 4],
    pub anisotropy: [f32; 4],
    pub sheen: [f32; 4],
    pub sheen_color: [f32; 4],
    pub detail: [f32; 4],
    pub diffuse_transmission_color: [f32; 4],
    pub reflection_box_min: [f32; 4],
    pub reflection_box_max: [f32; 4],
    pub reveal: [f32; 4],
    pub reveal_noise: [f32; 4],
    pub reveal_seam: [f32; 4],
    pub atlas: [f32; 4],
    pub textures: [u32; TEXTURE_SLOT_COUNT],
    pub transforms: [[f32; 4]; TEXTURE_SLOT_COUNT * 2],
}

pub const TEXTURE_SLOT_COUNT: usize = 24;

pub struct MaterialTable {
    pub buffer: Buffer,
    pub count: u32,
    pub generation: u64,
    capacity: u64,
}

pub fn create_table(context: &Context) -> Result<MaterialTable, Box<dyn std::error::Error>> {
    Ok(MaterialTable {
        buffer: create_storage(context, INITIAL_MATERIAL_CAPACITY)?,
        count: 0,
        generation: u64::MAX,
        capacity: INITIAL_MATERIAL_CAPACITY,
    })
}

pub fn destroy_table(context: &Context, table: &mut MaterialTable) {
    destroy_buffer(context, &mut table.buffer);
}

pub fn upload_materials(
    table: &mut MaterialTable,
    context: &Context,
    materials: &RenderMaterials,
    textures: &SceneTextures,
) -> Result<(), Box<dyn std::error::Error>> {
    if table.generation == materials.generation {
        return Ok(());
    }
    let records: Vec<GpuMaterial> = materials
        .entries
        .iter()
        .map(|entry| pack_material(&entry.material, &entry.texture_ids, textures))
        .collect();
    if records.len() as u64 > table.capacity {
        let capacity = (records.len() as u64).next_power_of_two();
        let grown = create_storage(context, capacity)?;
        destroy_buffer(context, &mut table.buffer);
        table.buffer = grown;
        table.capacity = capacity;
    }
    upload_to_buffer(context, &table.buffer, 0, bytemuck::cast_slice(&records))?;
    table.count = records.len() as u32;
    table.generation = materials.generation;
    Ok(())
}

fn pack_material(
    material: &Material,
    ids: &MaterialTextureIds,
    textures: &SceneTextures,
) -> GpuMaterial {
    let mut flags = 0;
    if material.double_sided {
        flags |= FLAG_DOUBLE_SIDED;
    }
    if material.unlit {
        flags |= FLAG_UNLIT;
    }
    if material.normal_map_flip_y {
        flags |= FLAG_NORMAL_FLIP_Y;
    }
    if material.normal_map_two_component {
        flags |= FLAG_NORMAL_TWO_COMPONENT;
    }
    if material.sky_light {
        flags |= FLAG_SKY_LIGHT;
    }
    if material.vertex_occlusion {
        flags |= FLAG_VERTEX_OCCLUSION;
    }
    GpuMaterial {
        base_color: material.base_color,
        emissive: [
            material.emissive_factor[0],
            material.emissive_factor[1],
            material.emissive_factor[2],
            material.emissive_strength,
        ],
        factors: [
            material.metallic,
            material.roughness,
            material.normal_scale,
            material.occlusion_strength,
        ],
        alpha: [
            material.alpha_cutoff,
            alpha_mode_code(material.alpha_mode) as f32,
            flags as f32,
            material.ior,
        ],
        transmission: [
            material.transmission_factor,
            material.thickness,
            material.attenuation_distance,
            material.dispersion,
        ],
        attenuation: [
            material.attenuation_color[0],
            material.attenuation_color[1],
            material.attenuation_color[2],
            material.specular_factor,
        ],
        specular_color: [
            material.specular_color_factor[0],
            material.specular_color_factor[1],
            material.specular_color_factor[2],
            material.anisotropy_rotation,
        ],
        anisotropy: [
            material.anisotropy_strength,
            material.iridescence_factor,
            material.iridescence_ior,
            material.iridescence_thickness_minimum,
        ],
        sheen: [
            material.iridescence_thickness_maximum,
            material.sheen_roughness_factor,
            material.clearcoat_factor,
            material.clearcoat_roughness_factor,
        ],
        sheen_color: [
            material.sheen_color_factor[0],
            material.sheen_color_factor[1],
            material.sheen_color_factor[2],
            material.clearcoat_normal_scale,
        ],
        detail: [
            material.diffuse_transmission_factor,
            material.detail_uv_scale,
            material.parallax_scale,
            if reflects_into_a_box(material) {
                1.0
            } else {
                0.0
            },
        ],
        diffuse_transmission_color: [
            material.diffuse_transmission_color_factor[0],
            material.diffuse_transmission_color_factor[1],
            material.diffuse_transmission_color_factor[2],
            0.0,
        ],
        reflection_box_min: [
            material.reflection_box_min[0],
            material.reflection_box_min[1],
            material.reflection_box_min[2],
            material.reveal_seam_strength,
        ],
        reflection_box_max: [
            material.reflection_box_max[0],
            material.reflection_box_max[1],
            material.reflection_box_max[2],
            material.blend_opaque_alpha_threshold,
        ],
        reveal: [
            material.reveal_normal[0],
            material.reveal_normal[1],
            material.reveal_normal[2],
            material.reveal_front,
        ],
        reveal_noise: [
            material.reveal_noise_scale[0],
            material.reveal_noise_scale[1],
            material.reveal_noise_scale[2],
            material.reveal_noise_amplitude,
        ],
        reveal_seam: [
            material.reveal_seam_color[0],
            material.reveal_seam_color[1],
            material.reveal_seam_color[2],
            material.reveal_seam_width,
        ],
        atlas: [material.atlas_cells[0], material.atlas_cells[1], 0.0, 0.0],
        textures: texture_slots(ids, textures),
        transforms: transforms(material),
    }
}

fn reflects_into_a_box(material: &Material) -> bool {
    (0..3).any(|axis| material.reflection_box_max[axis] > material.reflection_box_min[axis])
}

fn texture_slots(ids: &MaterialTextureIds, textures: &SceneTextures) -> [u32; TEXTURE_SLOT_COUNT] {
    let named = [
        ids.base,
        ids.metallic_roughness,
        ids.normal,
        ids.occlusion,
        ids.emissive,
        ids.transmission,
        ids.thickness,
        ids.specular,
        ids.specular_color,
        ids.iridescence,
        ids.iridescence_thickness,
        ids.sheen_color,
        ids.sheen_roughness,
        ids.clearcoat,
        ids.clearcoat_roughness,
        ids.clearcoat_normal,
        ids.anisotropy,
        ids.diffuse_transmission,
        ids.diffuse_transmission_color,
        ids.detail_base,
        ids.detail_normal,
        ids.height,
    ];
    let mut slots = [NO_TEXTURE_SLOT; TEXTURE_SLOT_COUNT];
    for (index, id) in named.into_iter().enumerate() {
        slots[index] = slot(id, textures);
    }
    slots
}

fn transforms(material: &Material) -> [[f32; 4]; TEXTURE_SLOT_COUNT * 2] {
    let sources = [
        &material.base_texture_transform,
        &material.metallic_roughness_texture_transform,
        &material.normal_texture_transform,
        &material.occlusion_texture_transform,
        &material.emissive_texture_transform,
        &material.transmission_texture_transform,
        &material.thickness_texture_transform,
        &material.specular_texture_transform,
        &material.specular_color_texture_transform,
        &material.iridescence_texture_transform,
        &material.iridescence_thickness_texture_transform,
        &material.sheen_color_texture_transform,
        &material.sheen_roughness_texture_transform,
        &material.clearcoat_texture_transform,
        &material.clearcoat_roughness_texture_transform,
        &material.clearcoat_normal_texture_transform,
        &material.anisotropy_texture_transform,
        &material.diffuse_transmission_texture_transform,
        &material.diffuse_transmission_color_texture_transform,
    ];
    let mut rows = [[0.0f32; 4]; TEXTURE_SLOT_COUNT * 2];
    for row in rows.as_chunks_mut::<2>().0 {
        row[0] = [1.0, 0.0, 0.0, 0.0];
        row[1] = [0.0, 1.0, 0.0, 0.0];
    }
    for (index, source) in sources.iter().enumerate() {
        let (row_u, row_v) = source.to_packed();
        rows[index * 2] = [row_u[0], row_u[1], row_u[2], source.uv_set as f32];
        rows[index * 2 + 1] = [row_v[0], row_v[1], row_v[2], 0.0];
    }
    rows
}

fn slot(
    texture: Option<nightshade_render::data::asset_id::TextureId>,
    textures: &SceneTextures,
) -> u32 {
    let Some(id) = texture else {
        return NO_TEXTURE_SLOT;
    };
    let sampler = texture_sampler(textures, id.index);
    pack_texture_slot(id.index, sampler.wrap_u, sampler.wrap_v, sampler.mag_filter)
}

fn create_storage(context: &Context, capacity: u64) -> Result<Buffer, Box<dyn std::error::Error>> {
    create_buffer(
        context,
        "scene materials",
        capacity * std::mem::size_of::<GpuMaterial>() as u64,
        vk::BufferUsageFlags::STORAGE_BUFFER
            | vk::BufferUsageFlags::TRANSFER_DST
            | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        MemoryLocation::GpuOnly,
    )
}
