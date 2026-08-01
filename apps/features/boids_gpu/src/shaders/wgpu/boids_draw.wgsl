
struct DrawUniforms {
    view_projection: mat4x4<f32>,
    sun: vec4<f32>,
    scale: vec4<f32>,
};

@group(0) @binding(0) var<storage, read> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> velocities: array<vec4<f32>>;
@group(0) @binding(2) var<uniform> uniforms: DrawUniforms;

const SIDES: u32 = 6u;
const TAU: f32 = 6.283185307179586;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) tint: vec3<f32>,
};

fn shade_for(index: u32) -> vec3<f32> {
    let turn = f32(index + 1u) * 0.618033988749895;
    let hue = fract(turn);
    let saturation = 0.78 + fract(f32(index + 1u) * 0.276393202250021) * 0.22;
    let value = 0.55 + fract(f32(index + 1u) * 0.414213562373095) * 0.40;

    let sector = hue * 6.0;
    let offset = sector - floor(sector);
    let falling = value * (1.0 - saturation);
    let rising = value * (1.0 - saturation * offset);
    let peaking = value * (1.0 - saturation * (1.0 - offset));

    switch u32(sector) % 6u {
        case 0u: { return vec3<f32>(value, peaking, falling); }
        case 1u: { return vec3<f32>(rising, value, falling); }
        case 2u: { return vec3<f32>(falling, value, peaking); }
        case 3u: { return vec3<f32>(falling, rising, value); }
        case 4u: { return vec3<f32>(peaking, falling, value); }
        default: { return vec3<f32>(value, falling, rising); }
    }
}

fn heading_basis(velocity: vec3<f32>) -> mat3x3<f32> {
    let speed = length(velocity);
    if speed < 1.0e-6 {
        return mat3x3<f32>(
            vec3<f32>(1.0, 0.0, 0.0),
            vec3<f32>(0.0, 1.0, 0.0),
            vec3<f32>(0.0, 0.0, 1.0),
        );
    }

    let forward = velocity / speed;
    var reference = vec3<f32>(0.0, 1.0, 0.0);
    if abs(forward.y) > 0.99 {
        reference = vec3<f32>(0.0, 0.0, 1.0);
    }

    let right = normalize(cross(reference, forward));
    let up = cross(right, forward);
    return mat3x3<f32>(right, forward, up);
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    let triangle = vertex_index / 3u;
    let corner = vertex_index % 3u;

    let half_width = uniforms.scale.x;
    let half_length = uniforms.scale.y;

    var local = vec3<f32>(0.0, half_length, 0.0);
    var local_normal = vec3<f32>(0.0, 1.0, 0.0);
    if corner != 0u {
        var step = triangle;
        if corner == 2u {
            step = triangle + 1u;
        }
        let angle = f32(step % SIDES) * TAU / f32(SIDES);
        let around = vec3<f32>(cos(angle), 0.0, sin(angle));
        local = around * half_width - vec3<f32>(0.0, half_length, 0.0);
        local_normal = normalize(around + vec3<f32>(0.0, 0.5, 0.0));
    } else {
        let angle = (f32(triangle % SIDES) + 0.5) * TAU / f32(SIDES);
        local_normal = normalize(vec3<f32>(cos(angle), 0.5, sin(angle)));
    }

    let basis = heading_basis(velocities[instance_index].xyz);
    let world = positions[instance_index].xyz + basis * local;

    var out: VertexOutput;
    out.clip_position = uniforms.view_projection * vec4<f32>(world, 1.0);
    out.normal = basis * local_normal;
    out.tint = shade_for(instance_index);
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(input.normal);
    let sun = normalize(uniforms.sun.xyz);
    let lit = max(dot(normal, sun), 0.0);
    let ambient = 0.25;
    return vec4<f32>(input.tint * (ambient + lit * 0.9), 1.0);
}
