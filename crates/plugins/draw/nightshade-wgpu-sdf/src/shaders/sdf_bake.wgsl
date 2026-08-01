#import nightshade_wgpu::sdf_common::{
    SDF_CONTROL_BAKING,
    SDF_DIRT_BIT,
    SDF_EDIT_BITS,
    SDF_NO_EDIT,
    SDF_RANGE_CELLS,
    sdf,
    sdf_atlas_origin,
    sdf_material_cut,
    sdf_pack_distance,
    sdf_probe,
    sdf_level_brick,
    sdf_level_cell,
    sdf_unpack_brick,
    sdf_unpack_level,
}

@group(1) @binding(0)
var<storage, read> sdf_slots: array<u32>;

@group(1) @binding(1)
var<storage, read> sdf_bricks: array<vec2<u32>>;

@group(1) @binding(2)
var sdf_atlas: texture_storage_3d<rgba8unorm, write>;

@group(1) @binding(3)
var<storage, read> sdf_control: array<u32>;

@group(1) @binding(4)
var<storage, read_write> sdf_stamps: array<u32>;

const GROUPS_PER_ROW: u32 = 65535u;

fn material_bytes(material: u32) -> vec2<f32> {
    if material == SDF_NO_EDIT {
        return vec2<f32>(0.0);
    }
    let packed = min((material & SDF_EDIT_BITS) + 1u, SDF_EDIT_BITS) | (material & SDF_DIRT_BIT);
    return vec2<f32>(f32(packed & 0xffu), f32(packed >> 8u)) / 255.0;
}

@compute @workgroup_size(8, 8, 4)
fn main(
    @builtin(workgroup_id) group: vec3<u32>,
    @builtin(local_invocation_id) local: vec3<u32>,
) {
    let which = group.x + group.y * GROUPS_PER_ROW;
    if which >= sdf_control[SDF_CONTROL_BAKING] {
        return;
    }
    let slot = sdf_slots[arrayLength(&sdf_slots) / 2u + which];
    let packed = sdf_bricks[slot];
    let level = sdf_unpack_level(packed);
    let brick = sdf_unpack_brick(packed);

    let cell_size = sdf_level_cell(level);
    let origin = vec3<f32>(brick) * sdf_level_brick(level);
    let atlas = vec3<i32>(sdf_atlas_origin(slot));
    let range = SDF_RANGE_CELLS * cell_size;

    if all(local == vec3<u32>(0u)) {
        sdf_stamps[slot] = sdf.frame.x;
    }

    for (var slab = 0u; slab < 2u; slab = slab + 1u) {
        let sample = vec3<u32>(local.x, local.y, local.z + slab * 4u);
        let world = origin + vec3<f32>(sample) * cell_size;
        let probe = sdf_probe(world, range, cell_size);
        let stored = sdf_pack_distance(probe.value / range);
        let material = material_bytes(sdf_material_cut(probe));
        textureStore(
            sdf_atlas,
            atlas + vec3<i32>(sample),
            vec4<f32>(stored.x, material.x, material.y, stored.y)
        );
    }
}
