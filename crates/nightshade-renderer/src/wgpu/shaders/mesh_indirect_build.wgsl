struct DrawIndexedIndirect {
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
};

struct BatchDesc {
    mesh_geo_id: u32,
    capacity: u32,
};

struct MeshGeo {
    index_count: u32,
    first_index: u32,
    base_vertex: i32,
    _pad: u32,
};

struct BuildParams {
    batch_count: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

@group(0) @binding(0)
var<storage, read_write> commands: array<DrawIndexedIndirect>;

@group(0) @binding(1)
var<storage, read> batch_descs: array<BatchDesc>;

@group(0) @binding(2)
var<storage, read> mesh_geo: array<MeshGeo>;

@group(0) @binding(3)
var<uniform> params: BuildParams;

const WORKGROUP_SIZE: u32 = 256u;

var<workgroup> thread_sums: array<u32, 256>;

@compute @workgroup_size(256)
fn main(@builtin(local_invocation_id) local_id: vec3<u32>) {
    let thread = local_id.x;
    let batch_count = params.batch_count;
    let per_thread = (batch_count + WORKGROUP_SIZE - 1u) / WORKGROUP_SIZE;
    let chunk_start = thread * per_thread;
    let chunk_end = min(chunk_start + per_thread, batch_count);

    var chunk_sum = 0u;
    for (var index = chunk_start; index < chunk_end; index++) {
        chunk_sum += batch_descs[index].capacity;
    }
    thread_sums[thread] = chunk_sum;
    workgroupBarrier();

    for (var offset = 1u; offset < WORKGROUP_SIZE; offset <<= 1u) {
        var addend = 0u;
        if thread >= offset {
            addend = thread_sums[thread - offset];
        }
        workgroupBarrier();
        thread_sums[thread] += addend;
        workgroupBarrier();
    }

    var first_instance = thread_sums[thread] - chunk_sum;
    for (var index = chunk_start; index < chunk_end; index++) {
        let desc = batch_descs[index];
        let geo = mesh_geo[desc.mesh_geo_id];
        commands[index] = DrawIndexedIndirect(
            geo.index_count,
            0u,
            geo.first_index,
            geo.base_vertex,
            first_instance,
        );
        first_instance += desc.capacity;
    }
}
