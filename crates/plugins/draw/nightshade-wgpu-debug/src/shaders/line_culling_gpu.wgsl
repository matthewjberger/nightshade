struct LineData {
    start: vec4<f32>,
    end: vec4<f32>,
    color: vec4<f32>,
    entity_id: u32,
    visible: u32,
    depth_mode: u32,
    _padding: u32,
}

struct DrawIndexedIndirectCommand {
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    vertex_offset: i32,
    first_instance: u32,
}

struct CullingData {
    view_proj: mat4x4<f32>,
    frustum_planes: array<vec4<f32>, 6>,
    line_count: u32,
    _padding0: u32,
    _padding1: u32,
    _padding2: u32,
    _padding3: vec4<f32>,
    _padding4: vec4<f32>,
    _padding5: vec4<f32>,
}

@group(0) @binding(0)
var<storage, read> lines: array<LineData>;

@group(0) @binding(1)
var<storage, read_write> draw_commands: array<DrawIndexedIndirectCommand>;

@group(0) @binding(2)
var<storage, read_write> draw_count: atomic<u32>;

@group(0) @binding(3)
var<uniform> culling_data: CullingData;

fn point_in_frustum(point: vec3<f32>) -> bool {
    for (var i = 0u; i < 6u; i++) {
        let plane = culling_data.frustum_planes[i];
        let distance = dot(vec4<f32>(point, 1.0), plane);
        if (distance < 0.0) {
            return false;
        }
    }
    return true;
}

fn line_in_frustum(start: vec3<f32>, end: vec3<f32>) -> bool {
    if (point_in_frustum(start) || point_in_frustum(end)) {
        return true;
    }

    let samples = 8u;
    for (var i = 1u; i < samples; i++) {
        let t = f32(i) / f32(samples);
        let sample_point = mix(start, end, t);
        if (point_in_frustum(sample_point)) {
            return true;
        }
    }

    return false;
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let line_index = global_id.x;

    if (line_index >= culling_data.line_count) {
        return;
    }

    let line = lines[line_index];

    if (line.visible == 0u) {
        return;
    }

    let start = line.start.xyz;
    let end = line.end.xyz;

    if (!line_in_frustum(start, end)) {
        return;
    }

    let command_index = atomicAdd(&draw_count, 1u);

    var command: DrawIndexedIndirectCommand;
    command.index_count = 2u;
    command.instance_count = 1u;
    command.first_index = 0u;
    command.vertex_offset = 0;
    command.first_instance = line_index;

    draw_commands[command_index] = command;
}
