#import nightshade_wgpu::sdf_common::{
    sdf,
}

@group(1) @binding(0)
var low_color: texture_2d<f32>;

@group(1) @binding(1)
var low_identity: texture_2d<f32>;

@group(1) @binding(2)
var low_normals: texture_2d<f32>;

@group(1) @binding(3)
var low_velocity: texture_2d<f32>;

@group(1) @binding(4)
var low_depth: texture_depth_2d;

const DEPTH_TOLERANCE: f32 = 0.03;

struct Surface {
    @location(0) color: vec4<f32>,
    @location(1) entity_id: f32,
    @location(2) view_normal: vec4<f32>,
    @location(3) velocity: vec2<f32>,
    @builtin(frag_depth) depth: f32,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) @invariant vec4<f32> {
    let half = i32(vertex_index) / 2;
    let odd = i32(vertex_index) & 1;
    return vec4<f32>(f32(half) * 4.0 - 1.0, f32(odd) * 4.0 - 1.0, 0.0, 1.0);
}

fn clamped(texel: vec2<i32>, size: vec2<i32>) -> vec2<i32> {
    return clamp(texel, vec2<i32>(0), size - vec2<i32>(1));
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> Surface {
    let size = vec2<i32>(textureDimensions(low_color));
    let uv = position.xy * sdf.viewport.zw;
    let at = uv * vec2<f32>(size);
    let nearest = clamped(vec2<i32>(floor(at)), size);
    let depth = textureLoad(low_depth, nearest, 0);
    if depth <= 0.0 {
        discard;
    }
    let corner = at - vec2<f32>(0.5);
    let base = vec2<i32>(floor(corner));
    let share = corner - floor(corner);
    var color = vec3<f32>(0.0);
    var weight = 0.0;
    for (var tap = 0u; tap < 4u; tap = tap + 1u) {
        let offset = vec2<i32>(i32(tap & 1u), i32(tap >> 1u));
        let texel = clamped(base + offset, size);
        let tap_depth = textureLoad(low_depth, texel, 0);
        if tap_depth <= 0.0 || abs(tap_depth - depth) > depth * DEPTH_TOLERANCE {
            continue;
        }
        let pull = mix(vec2<f32>(1.0) - share, share, vec2<f32>(offset));
        let tap_weight = pull.x * pull.y;
        color = color + textureLoad(low_color, texel, 0).rgb * tap_weight;
        weight = weight + tap_weight;
    }
    if weight > 0.0 {
        color = color / weight;
    } else {
        color = textureLoad(low_color, nearest, 0).rgb;
    }
    return Surface(
        vec4<f32>(color, 1.0),
        textureLoad(low_identity, nearest, 0).r,
        textureLoad(low_normals, nearest, 0),
        textureLoad(low_velocity, nearest, 0).xy,
        depth
    );
}
