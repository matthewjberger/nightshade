struct NormalData {
    position: vec3<f32>,
    _pad0: f32,
    normal: vec3<f32>,
    _pad1: f32,
    transform: mat4x4<f32>,
    color: vec4<f32>,
    length: f32,
    _pad2_0: f32,
    _pad2_1: f32,
    _pad2_2: f32,
}

struct LineData {
    start: vec4<f32>,
    end: vec4<f32>,
    color: vec4<f32>,
    entity_id: u32,
    visible: u32,
    depth_mode: u32,
    _padding: u32,
}

struct Params {
    normal_count: u32,
    line_offset: u32,
    _padding: vec2<u32>,
}

@group(0) @binding(0)
var<storage, read> normals: array<NormalData>;

@group(0) @binding(1)
var<storage, read_write> lines: array<LineData>;

@group(0) @binding(2)
var<uniform> params: Params;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= params.normal_count) {
        return;
    }

    let data = normals[index];
    let transform = data.transform;

    let world_pos = transform * vec4<f32>(data.position, 1.0);

    let normal_matrix = mat3x3<f32>(
        transform[0].xyz,
        transform[1].xyz,
        transform[2].xyz
    );
    let world_normal = normalize(normal_matrix * data.normal);

    let end_pos = world_pos.xyz + world_normal * data.length;

    let line_index = params.line_offset + index;
    lines[line_index].start = vec4<f32>(world_pos.xyz, 1.0);
    lines[line_index].end = vec4<f32>(end_pos, 1.0);
    lines[line_index].color = data.color;
    lines[line_index].entity_id = index;
    lines[line_index].visible = 1u;
    lines[line_index].depth_mode = 0u;
    lines[line_index]._padding = 0u;
}
