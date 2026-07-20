// GPU-driven batch build for the skinned mesh pass. Replaces the CPU sort +
// draw-group + indirect-command construction. Two draw classes (opaque,
// transparent), keyed by (class, mesh_id); material is addressed per object, so
// it is not part of the key. Runs every frame (skinned entity counts are low):
//
//   clear - zero the dense capacity table
//   count - per object, accumulate per (class, mesh) capacity
//   build - one workgroup: assign each non-empty (class, mesh) a command in the
//           class's fixed region with a prefix-summed first_instance, fill the
//           indirect command, the (class,mesh) -> command map, and per-class
//           command counts
//   assign - per object, write its command index into the cull object so the
//            cull pass gathers it into the right command's visible range

struct SkinnedObjectData {
    transform_index: u32,
    mesh_id: u32,
    material_id: u32,
    joint_offset: u32,
    morph_weights: array<f32, 8>,
    morph_target_count: u32,
    morph_displacement_offset: u32,
    mesh_vertex_offset: u32,
    mesh_vertex_count: u32,
    entity_id: u32,
    is_transparent: u32,
    _padding: u32,
};

struct SkinnedCullObject {
    bounds: vec4<f32>,
    command_index: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

struct DrawIndexedIndirect {
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
};

struct MeshGeo {
    index_count: u32,
    first_index: u32,
    base_vertex: i32,
    _pad: u32,
};

struct BatchParams {
    object_count: u32,
    mesh_count: u32,
    commands_per_class: u32,
    _pad: u32,
};

@group(0) @binding(0)
var<storage, read> objects: array<SkinnedObjectData>;

@group(0) @binding(1)
var<storage, read_write> cull_objects: array<SkinnedCullObject>;

@group(0) @binding(2)
var<storage, read_write> dense_capacity: array<atomic<u32>>;

@group(0) @binding(3)
var<storage, read_write> command_map: array<u32>;

@group(0) @binding(4)
var<storage, read_write> indirect_commands: array<DrawIndexedIndirect>;

// [0] opaque command count, [1] transparent command count.
@group(0) @binding(5)
var<storage, read_write> batch_meta: array<u32>;

@group(0) @binding(6)
var<storage, read> mesh_geo: array<MeshGeo>;

@group(0) @binding(7)
var<uniform> params: BatchParams;

fn dense_index(class_value: u32, mesh: u32) -> u32 {
    return class_value * params.mesh_count + mesh;
}

@compute @workgroup_size(64)
fn clear(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if index >= 2u * params.mesh_count {
        return;
    }
    atomicStore(&dense_capacity[index], 0u);
    // The draw issues `commands_per_class` (== mesh_count) commands per class, so
    // the dense index and the command slot share the same flat layout. Zero every
    // command slot; the build overwrites the ones it fills, the rest draw nothing.
    indirect_commands[index] = DrawIndexedIndirect(0u, 0u, 0u, 0, 0u);
}

@compute @workgroup_size(64)
fn count(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let object_index = global_id.x;
    if object_index >= params.object_count {
        return;
    }
    let object = objects[object_index];
    if object.mesh_id >= params.mesh_count {
        return;
    }
    let class_value = select(0u, 1u, object.is_transparent != 0u);
    atomicAdd(&dense_capacity[dense_index(class_value, object.mesh_id)], 1u);
}

@compute @workgroup_size(1)
fn build() {
    var first_instance = 0u;
    for (var class_value = 0u; class_value < 2u; class_value = class_value + 1u) {
        let region_base = class_value * params.commands_per_class;
        var slot = 0u;
        for (var mesh = 0u; mesh < params.mesh_count; mesh = mesh + 1u) {
            let capacity = atomicLoad(&dense_capacity[dense_index(class_value, mesh)]);
            if capacity == 0u {
                command_map[dense_index(class_value, mesh)] = 0xFFFFFFFFu;
                continue;
            }
            if slot >= params.commands_per_class {
                command_map[dense_index(class_value, mesh)] = 0xFFFFFFFFu;
                continue;
            }
            let command_index = region_base + slot;
            let geo = mesh_geo[mesh];
            indirect_commands[command_index] = DrawIndexedIndirect(
                geo.index_count,
                0u,
                geo.first_index,
                geo.base_vertex,
                first_instance,
            );
            command_map[dense_index(class_value, mesh)] = command_index;
            first_instance = first_instance + capacity;
            slot = slot + 1u;
        }
        batch_meta[class_value] = slot;
    }
}

@compute @workgroup_size(64)
fn assign(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let object_index = global_id.x;
    if object_index >= params.object_count {
        return;
    }
    let object = objects[object_index];
    if object.mesh_id >= params.mesh_count {
        cull_objects[object_index].command_index = 0xFFFFFFFFu;
        return;
    }
    let class_value = select(0u, 1u, object.is_transparent != 0u);
    cull_objects[object_index].command_index = command_map[dense_index(class_value, object.mesh_id)];
}
