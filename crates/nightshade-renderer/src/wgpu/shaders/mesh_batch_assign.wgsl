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

struct BatchKey {
    pipeline_class: u32,
    mesh_id: u32,
    material_id: u32,
    base_slot: u32,
};

struct AssignParams {
    object_count: u32,
    key_count: u32,
    invalid_batch: u32,
    mode: u32,
    cap: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

const NUM_CLASSES: u32 = 9u;

@group(0) @binding(0)
var<storage, read_write> objects: array<ObjectData>;

@group(0) @binding(1)
var<storage, read> keys: array<BatchKey>;

@group(0) @binding(2)
var<uniform> params: AssignParams;

@group(0) @binding(3)
var<storage, read> batch_meta: array<u32>;

fn key_less(object_class: u32, mesh: u32, material: u32, other: BatchKey) -> bool {
    if object_class != other.pipeline_class {
        return object_class < other.pipeline_class;
    }
    if mesh != other.mesh_id {
        return mesh < other.mesh_id;
    }
    return material < other.material_id;
}

fn key_equal(object_class: u32, mesh: u32, material: u32, other: BatchKey) -> bool {
    return object_class == other.pipeline_class && mesh == other.mesh_id && material == other.material_id;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let object_index = global_id.x;
    if object_index >= params.object_count {
        return;
    }

    let object = objects[object_index];
    if object.batch_id == params.invalid_batch {
        return;
    }

    let object_class = object.pipeline_class;
    let mesh = object.mesh_id;
    let material = object.material_id;

    // GPU-driven mode: keys live in a fixed per-class region; linear search it.
    if params.mode == 1u {
        let key_count = batch_meta[NUM_CLASSES + object_class];
        let region = object_class * params.cap;
        var found = params.invalid_batch;
        for (var k = 0u; k < key_count; k = k + 1u) {
            let key = keys[region + k];
            if key.mesh_id == mesh && key.material_id == material {
                found = key.base_slot;
                break;
            }
        }
        objects[object_index].batch_id = found;
        return;
    }

    var low = 0u;
    var high = params.key_count;
    while low < high {
        let mid = (low + high) >> 1u;
        if key_less(object_class, mesh, material, keys[mid]) {
            high = mid;
        } else {
            low = mid + 1u;
        }
    }

    if low == 0u {
        objects[object_index].batch_id = params.invalid_batch;
        return;
    }
    let found = keys[low - 1u];
    if key_equal(object_class, mesh, material, found) {
        objects[object_index].batch_id = found.base_slot;
    } else {
        objects[object_index].batch_id = params.invalid_batch;
    }
}
