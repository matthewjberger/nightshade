struct LineData {
    start: vec4<f32>,
    end: vec4<f32>,
    color: vec4<f32>,
    entity_id: u32,
    visible: u32,
    depth_mode: u32,
    _padding: u32,
}

struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

struct Uniforms {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(0)
var<storage, read> lines: array<LineData>;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let line = lines[in.instance_index];

    let pos = mix(line.start.xyz, line.end.xyz, in.position.x);

    out.clip_position = uniforms.view_proj * vec4<f32>(pos, 1.0);
    if (line.depth_mode == 1u) {
        out.clip_position.z = out.clip_position.w;
    }
    out.color = line.color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
