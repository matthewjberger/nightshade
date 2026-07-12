// Fully GPU-driven classification + batch table build. Runs when
// `gpu_batching_enabled`. No CPU readback: the GPU emits the batch
// descriptors/keys and the per-class draw counts directly, into fixed per-class
// regions of the indirect buffers, and the draw submission reads the GPU-written
// counts via multi_draw_indexed_indirect_count.
//
//   clear_dense - zero the dense capacity table
//   classify    - derive pipeline_class / skip_occlusion per object
//   count       - accumulate per (class, mask, mesh, material) capacities
//   build       - one workgroup per draw class: emit batch descriptors (LOD
//                 expanded), keys, and per-class desc/key/prepass counts into
//                 the class's fixed region

struct ObjectData {
    transform_index: u32,
    mesh_id: u32,
    material_id: u32,
    batch_id: u32,
    morph_weights: array<f32, 8>,
    morph_target_count: u32,
    morph_displacement_offset: u32,
    mesh_vertex_offset: u32,
    mesh_vertex_count: u32,
    entity_id: u32,
    is_overlay: u32,
    skip_occlusion: u32,
    flip_winding: u32,
    culling_mask: u32,
    visible: u32,
    pipeline_class: u32,
    _pad_culling_2: u32,
};

struct BatchDesc {
    mesh_geo_id: u32,
    capacity: u32,
};

struct BatchKey {
    pipeline_class: u32,
    mesh_id: u32,
    material_id: u32,
    base_slot: u32,
};

struct MeshLodGeo {
    lod_count: u32,
    geo: array<u32, 4>,
};

struct BatchParams {
    object_count: u32,
    regular_object_count: u32,
    mesh_count: u32,
    material_count: u32,
    batches_per_class: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

const FLAG_TRANSPARENT: u32 = 1u;
const FLAG_MASK: u32 = 2u;
const FLAG_DOUBLE_SIDED: u32 = 4u;
const NUM_CLASSES: u32 = 9u;

@group(0) @binding(0)
var<storage, read_write> objects: array<ObjectData>;

@group(0) @binding(1)
var<storage, read> material_flags: array<u32>;

@group(0) @binding(2)
var<storage, read_write> dense_capacity: array<atomic<u32>>;

@group(0) @binding(3)
var<uniform> params: BatchParams;

@group(0) @binding(4)
var<storage, read_write> batch_descs: array<BatchDesc>;

@group(0) @binding(5)
var<storage, read_write> batch_keys: array<BatchKey>;

// Per class: [0..9) desc count, [9..18) key (combo) count, [18..27) prepass
// (non-mask) desc count.
@group(0) @binding(6)
var<storage, read_write> batch_meta: array<u32>;

@group(0) @binding(7)
var<storage, read> mesh_lod_geo: array<MeshLodGeo>;

fn flags_for(material_id: u32) -> u32 {
    if material_id >= params.material_count {
        return 0u;
    }
    return material_flags[material_id];
}

fn class_for(object_index: u32, obj: ObjectData) -> u32 {
    let flags = flags_for(obj.material_id);
    let is_transparent = (flags & FLAG_TRANSPARENT) != 0u;
    let material_double_sided = (flags & FLAG_DOUBLE_SIDED) != 0u;

    if object_index >= params.regular_object_count {
        if is_transparent {
            return 8u;
        } else if material_double_sided {
            return 7u;
        }
        return 6u;
    }

    let double_sided = material_double_sided || obj.flip_winding != 0u;
    if obj.is_overlay != 0u {
        if is_transparent {
            return 5u;
        } else if double_sided {
            return 4u;
        }
        return 3u;
    }
    if is_transparent {
        return 2u;
    } else if double_sided {
        return 1u;
    }
    return 0u;
}

fn skip_for(class_value: u32, material_id: u32) -> u32 {
    let flags = flags_for(material_id);
    let is_mask = (flags & FLAG_MASK) != 0u;
    var skip = 0u;
    if class_value == 2u || class_value == 8u {
        skip = 1u;
    }
    if is_mask {
        skip = 1u;
    }
    return skip;
}

@compute @workgroup_size(64)
fn classify(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let object_index = global_id.x;
    if object_index >= params.object_count {
        return;
    }
    let obj = objects[object_index];
    if obj.batch_id == 0xFFFFFFFFu {
        return;
    }
    let class_value = class_for(object_index, obj);
    objects[object_index].pipeline_class = class_value;
    objects[object_index].skip_occlusion = skip_for(class_value, obj.material_id);
}

fn dense_index(class_value: u32, is_mask: u32, mesh: u32, material: u32) -> u32 {
    let lane = class_value * 2u + is_mask;
    return (lane * params.mesh_count + mesh) * params.material_count + material;
}

@compute @workgroup_size(64)
fn clear_dense(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    let total = NUM_CLASSES * 2u * params.mesh_count * params.material_count;
    if index >= total {
        return;
    }
    atomicStore(&dense_capacity[index], 0u);
}

@compute @workgroup_size(64)
fn count(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let object_index = global_id.x;
    if object_index >= params.object_count {
        return;
    }
    let obj = objects[object_index];
    if obj.batch_id == 0xFFFFFFFFu {
        return;
    }
    if obj.mesh_id >= params.mesh_count || obj.material_id >= params.material_count {
        return;
    }
    let flags = flags_for(obj.material_id);
    let is_mask = select(0u, 1u, (flags & FLAG_MASK) != 0u);
    let index = dense_index(obj.pipeline_class, is_mask, obj.mesh_id, obj.material_id);
    atomicAdd(&dense_capacity[index], 1u);
}

// One workgroup (single producer) per draw class. The class owns the fixed
// region [class * batches_per_class, ...). Non-mask combos are emitted first
// (the depth prepass draws that prefix), then mask combos. Each combo expands to
// `lod_count` consecutive descriptor slots.
@compute @workgroup_size(1)
fn build(@builtin(workgroup_id) workgroup_id: vec3<u32>) {
    let class_value = workgroup_id.x;
    if class_value >= NUM_CLASSES {
        return;
    }

    let cap = params.batches_per_class;
    let region_base = class_value * cap;

    for (var slot = 0u; slot < cap; slot = slot + 1u) {
        batch_descs[region_base + slot] = BatchDesc(0u, 0u);
    }

    var slot = 0u;
    var key_index = 0u;
    var prepass = 0u;

    for (var is_mask = 0u; is_mask < 2u; is_mask = is_mask + 1u) {
        for (var mesh = 0u; mesh < params.mesh_count; mesh = mesh + 1u) {
            for (var material = 0u; material < params.material_count; material = material + 1u) {
                let index = dense_index(class_value, is_mask, mesh, material);
                let capacity = atomicLoad(&dense_capacity[index]);
                if capacity == 0u {
                    continue;
                }

                let lod = mesh_lod_geo[mesh];
                let lod_count = max(lod.lod_count, 1u);
                if slot + lod_count > cap {
                    continue;
                }

                let base_slot = region_base + slot;
                batch_keys[region_base + key_index] =
                    BatchKey(class_value, mesh, material, base_slot);
                key_index = key_index + 1u;

                for (var level = 0u; level < lod_count; level = level + 1u) {
                    var geo = mesh;
                    if level < 4u {
                        geo = lod.geo[level];
                    }
                    batch_descs[base_slot + level] = BatchDesc(geo, capacity);
                }

                if is_mask == 0u {
                    prepass = prepass + lod_count;
                }
                slot = slot + lod_count;
            }
        }
    }

    batch_meta[class_value] = slot;
    batch_meta[NUM_CLASSES + class_value] = key_index;
    batch_meta[2u * NUM_CLASSES + class_value] = prepass;
}
