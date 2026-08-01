struct DrawIndexedIndirect {
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
};

struct CompactParams {
    class_count: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

@group(0) @binding(0)
var<storage, read_write> commands: array<DrawIndexedIndirect>;

@group(0) @binding(1)
var<storage, read_write> counts: array<u32>;

@group(0) @binding(2)
var<storage, read> class_ranges: array<vec2<u32>>;

@group(0) @binding(3)
var<uniform> params: CompactParams;

const WORKGROUP_SIZE: u32 = 256u;

var<workgroup> prefix: array<u32, 256>;
var<workgroup> running_base: u32;

@compute @workgroup_size(256)
fn main(
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
) {
    let class_index = workgroup_id.x;
    if class_index >= params.class_count {
        return;
    }

    let range = class_ranges[class_index];
    let start = range.x;
    let len = range.y;
    let thread = local_id.x;

    if thread == 0u {
        running_base = 0u;
    }
    workgroupBarrier();

    let chunk_count = (len + WORKGROUP_SIZE - 1u) / WORKGROUP_SIZE;
    for (var chunk = 0u; chunk < chunk_count; chunk = chunk + 1u) {
        let index = chunk * WORKGROUP_SIZE + thread;

        var command: DrawIndexedIndirect;
        var keep = 0u;
        if index < len {
            command = commands[start + index];
            if command.instance_count > 0u {
                keep = 1u;
            }
        }
        prefix[thread] = keep;
        workgroupBarrier();

        for (var offset = 1u; offset < WORKGROUP_SIZE; offset = offset << 1u) {
            var addend = 0u;
            if thread >= offset {
                addend = prefix[thread - offset];
            }
            workgroupBarrier();
            prefix[thread] = prefix[thread] + addend;
            workgroupBarrier();
        }

        let inclusive = prefix[thread];
        let chunk_total = prefix[WORKGROUP_SIZE - 1u];
        if keep == 1u {
            let dense_index = running_base + inclusive - 1u;
            commands[start + dense_index] = command;
        }
        workgroupBarrier();
        if thread == 0u {
            running_base = running_base + chunk_total;
        }
        workgroupBarrier();
    }

    if thread == 0u {
        counts[class_index] = running_base;
    }
}
