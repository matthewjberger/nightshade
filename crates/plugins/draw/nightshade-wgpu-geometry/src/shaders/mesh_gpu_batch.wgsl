
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
    debug_view_exempt: u32,
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

struct BatchParams {
    object_count: u32,
    regular_object_count: u32,
    mesh_count: u32,
    material_count: u32,
    batches_per_class: u32,
    max_keys_per_lane: u32,
    table_word_count: u32,
    _pad0: u32,
};

const FLAG_TRANSPARENT: u32 = 1u;
const FLAG_MASK: u32 = 2u;
const FLAG_DOUBLE_SIDED: u32 = 4u;
const FLAG_TRANSMISSIVE: u32 = 8u;
const NUM_CLASSES: u32 = 10u;

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

@group(0) @binding(6)
var<storage, read_write> batch_meta: array<u32>;

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
    if (flags & FLAG_TRANSMISSIVE) != 0u && obj.is_overlay == 0u {
        return 9u;
    }
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
    if class_value == 2u || class_value == 8u || class_value == 9u {
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

const NUM_LANES: u32 = NUM_CLASSES * 2u;

fn dense_index(class_value: u32, is_mask: u32, mesh: u32, material: u32) -> u32 {
    let lane = class_value * 2u + is_mask;
    return (lane * params.mesh_count + mesh) * params.material_count + material;
}

fn lane_count_index(lane: u32) -> u32 {
    return NUM_LANES * params.mesh_count * params.material_count + lane;
}

fn lane_combo_base(lane: u32) -> u32 {
    return NUM_LANES * params.mesh_count * params.material_count
        + NUM_LANES
        + lane * params.max_keys_per_lane * 2u;
}

const CLEAR_GROUPS_PER_ROW: u32 = 32768u;

@compute @workgroup_size(64)
fn clear_dense(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.y * CLEAR_GROUPS_PER_ROW * 64u + global_id.x;
    if index >= params.table_word_count {
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
    let previous = atomicAdd(&dense_capacity[index], 1u);
    if previous != 0u {
        return;
    }

    let lane = obj.pipeline_class * 2u + is_mask;
    let slot = atomicAdd(&dense_capacity[lane_count_index(lane)], 1u);
    if slot >= params.max_keys_per_lane {
        return;
    }
    let entry = lane_combo_base(lane) + slot * 2u;
    atomicStore(&dense_capacity[entry], obj.mesh_id);
    atomicStore(&dense_capacity[entry + 1u], obj.material_id);
}

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
        let lane = class_value * 2u + is_mask;
        let combo_base = lane_combo_base(lane);
        let recorded = atomicLoad(&dense_capacity[lane_count_index(lane)]);
        let combo_count = min(recorded, params.max_keys_per_lane);
        atomicStore(&dense_capacity[lane_count_index(lane)], 0u);

        for (var combo = 0u; combo < combo_count; combo = combo + 1u) {
            let entry = combo_base + combo * 2u;
            let mesh = atomicLoad(&dense_capacity[entry]);
            let material = atomicLoad(&dense_capacity[entry + 1u]);
            atomicStore(&dense_capacity[entry], 0u);
            atomicStore(&dense_capacity[entry + 1u], 0u);

            let index = dense_index(class_value, is_mask, mesh, material);
            let capacity = atomicLoad(&dense_capacity[index]);
            atomicStore(&dense_capacity[index], 0u);
            if capacity == 0u {
                continue;
            }

            if slot + 1u > cap {
                continue;
            }

            let base_slot = region_base + slot;
            batch_keys[region_base + key_index] =
                BatchKey(class_value, mesh, material, base_slot);
            key_index = key_index + 1u;

            batch_descs[base_slot] = BatchDesc(mesh, capacity);

            if is_mask == 0u {
                prepass = prepass + 1u;
            }
            slot = slot + 1u;
        }
    }

    batch_meta[class_value] = slot;
    batch_meta[NUM_CLASSES + class_value] = key_index;
    batch_meta[2u * NUM_CLASSES + class_value] = prepass;
}
