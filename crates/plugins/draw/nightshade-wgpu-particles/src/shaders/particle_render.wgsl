struct Particle {
    position: vec4<f32>,
    velocity: vec4<f32>,
    color: vec4<f32>,
    size_lifetime: vec4<f32>,
    emitter_data: vec4<f32>,
    physics: vec4<f32>,
    color_start: vec4<f32>,
    color_end: vec4<f32>,
}

struct CameraUniforms {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    view_projection: mat4x4<f32>,
    camera_position: vec4<f32>,
    camera_right: vec4<f32>,
    camera_up: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) emissive: f32,
    @location(3) world_position: vec3<f32>,
    @location(4) @interpolate(flat) emitter_type: u32,
    @location(5) @interpolate(flat) texture_index: u32,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniforms;

@group(0) @binding(1)
var<storage, read> particles: array<Particle>;

@group(0) @binding(2)
var<storage, read> alive_indices: array<u32>;

@group(1) @binding(0)
var particle_texture_array: texture_2d_array<f32>;

@group(1) @binding(1)
var particle_sampler: sampler;

const QUAD_VERTICES: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-0.5, -0.5),
    vec2<f32>(0.5, -0.5),
    vec2<f32>(0.5, 0.5),
    vec2<f32>(-0.5, -0.5),
    vec2<f32>(0.5, 0.5),
    vec2<f32>(-0.5, 0.5),
);

const QUAD_UVS: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(0.0, 0.0),
);

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    let particle_index = alive_indices[instance_index];
    let particle = particles[particle_index];

    let quad_vertex = QUAD_VERTICES[vertex_index];
    let quad_uv = QUAD_UVS[vertex_index];

    let size = particle.emitter_data.y;

    let right = camera.camera_right.xyz;
    let up = camera.camera_up.xyz;

    let world_offset = right * quad_vertex.x * size + up * quad_vertex.y * size;
    let world_position = particle.position.xyz + world_offset;

    let clip_position = camera.view_projection * vec4<f32>(world_position, 1.0);

    let emissive_strength = particle.emitter_data.z;
    let emitter_type = u32(particle.emitter_data.w);
    let tex_index = u32(particle.emitter_data.x);

    var output: VertexOutput;
    output.position = clip_position;
    output.uv = quad_uv;
    output.color = particle.color;
    output.emissive = emissive_strength;
    output.world_position = world_position;
    output.emitter_type = emitter_type;
    output.texture_index = tex_index;

    return output;
}

fn firework_glow(uv: vec2<f32>) -> f32 {
    let center = vec2<f32>(0.5, 0.5);
    let dist = length(uv - center);

    let core = exp(-dist * dist * 120.0);
    let bright_core = exp(-dist * dist * 40.0) * 0.9;
    let inner_glow = exp(-dist * dist * 15.0) * 0.6;
    let mid_glow = exp(-dist * dist * 6.0) * 0.35;
    let outer_glow = exp(-dist * dist * 2.5) * 0.15;

    return core + bright_core + inner_glow + mid_glow + outer_glow;
}

fn fire_shape(uv: vec2<f32>) -> f32 {
    let center = vec2<f32>(0.5, 0.5);
    var adjusted_uv = uv - center;
    adjusted_uv.y *= 0.65;
    let dist = length(adjusted_uv);

    let core = exp(-dist * dist * 60.0);
    let flame = exp(-dist * dist * 15.0) * 0.75;
    let outer = exp(-dist * dist * 5.0) * 0.35;

    return core + flame + outer;
}

fn smoke_shape(uv: vec2<f32>) -> f32 {
    let center = vec2<f32>(0.5, 0.5);
    let dist = length(uv - center);
    return exp(-dist * dist * 4.0) * 0.85;
}

fn spark_shape(uv: vec2<f32>) -> f32 {
    let center = vec2<f32>(0.5, 0.5);
    let dist = length(uv - center);

    let core = exp(-dist * dist * 200.0);
    let bright = exp(-dist * dist * 60.0) * 0.9;
    let glow = exp(-dist * dist * 15.0) * 0.5;

    return core + bright + glow;
}

fn get_shape_for_type(uv: vec2<f32>, emitter_type: u32) -> f32 {
    switch emitter_type {
        case 0u: { return firework_glow(uv); }
        case 1u: { return fire_shape(uv); }
        case 2u: { return smoke_shape(uv); }
        case 3u: { return spark_shape(uv); }
        case 4u: { return firework_glow(uv); }
        default: { return firework_glow(uv); }
    }
}

fn sample_particle_texture(uv: vec2<f32>, tex_index: u32) -> vec4<f32> {
    let layer = i32(tex_index - 1u);
    return textureSampleLevel(particle_texture_array, particle_sampler, uv, layer, 0.0);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    if (input.texture_index > 0u) {
        let tex_color = sample_particle_texture(input.uv, input.texture_index);
        if (tex_color.a < 0.01) {
            discard;
        }
        var color = input.color * tex_color;
        let intensity = color.a;
        let emissive_boost = max(input.emissive - 1.0, 0.0);
        let brightness = 1.0 + emissive_boost * 1.2;
        let hdr_color = color.rgb * intensity * brightness;
        let boosted = hdr_color + hdr_color * hdr_color * 0.3;
        return vec4<f32>(boosted, intensity);
    }

    let shape = get_shape_for_type(input.uv, input.emitter_type);

    if (shape < 0.005) {
        discard;
    }

    var color = input.color;
    let intensity = shape * color.a;

    if (input.emitter_type == 2u) {
        let smoke_color = color.rgb * intensity * 1.5;
        return vec4<f32>(smoke_color, intensity * 0.8);
    }

    let emissive_boost = max(input.emissive - 1.0, 0.0);
    let brightness = 1.0 + emissive_boost * 1.2;

    let hdr_color = color.rgb * intensity * brightness;
    let boosted = hdr_color + hdr_color * hdr_color * 0.3;

    return vec4<f32>(boosted, intensity);
}

