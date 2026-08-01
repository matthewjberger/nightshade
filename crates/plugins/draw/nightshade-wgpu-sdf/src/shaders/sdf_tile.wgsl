#import nightshade_wgpu::sdf_common::{
    sdf,
    sdf_level_cell,
}
#import nightshade_wgpu::sdf_cache::{
    sdf_clipmap_span,
    sdf_cone_bound,
    pick_level,
}

@group(2) @binding(0)
var tile_start: texture_storage_2d<r32float, write>;

const TILE: f32 = 8.0;
const TILE_STEPS: u32 = 48u;
const TILE_MARGIN: f32 = 1.05;
const TILE_SKY: f32 = -1.0;

fn ray(pixel: vec2<f32>) -> vec3<f32> {
    let screen = vec2<f32>(pixel.x * sdf.screen.z * 2.0 - 1.0, 1.0 - pixel.y * sdf.screen.w * 2.0);
    let near = sdf.inverse_view_projection * vec4<f32>(screen, 1.0, 1.0);
    let middle = sdf.inverse_view_projection * vec4<f32>(screen, 0.5, 1.0);
    return normalize(middle.xyz / middle.w - near.xyz / near.w);
}

fn tangent_between(center: vec3<f32>, other: vec3<f32>) -> f32 {
    let cosine = clamp(dot(center, other), 1.0e-4, 1.0);
    return sqrt(max(1.0 - cosine * cosine, 0.0)) / cosine;
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(tile_start);
    if id.x >= size.x || id.y >= size.y {
        return;
    }
    let low = vec2<f32>(id.xy) * TILE;
    let high = min(low + vec2<f32>(TILE), sdf.screen.xy);
    let direction = ray(0.5 * (low + high));
    let eye = sdf.camera.xyz;
    var spread = tangent_between(direction, ray(low));
    spread = max(spread, tangent_between(direction, ray(high)));
    spread = max(spread, tangent_between(direction, ray(vec2<f32>(low.x, high.y))));
    spread = max(spread, tangent_between(direction, ray(vec2<f32>(high.x, low.y))));
    spread = spread * TILE_MARGIN;

    let span = sdf_clipmap_span(eye, direction);
    var traveled = span.x + sdf.sizes.x;
    if span.x > span.y {
        textureStore(tile_start, vec2<i32>(id.xy), vec4<f32>(TILE_SKY, 0.0, 0.0, 0.0));
        return;
    }
    for (var step = 0u; step < TILE_STEPS; step = step + 1u) {
        let point = eye + direction * traveled;
        let level = pick_level(point);
        if level >= sdf.counts.y {
            break;
        }
        let cell = sdf_level_cell(level);
        let advance = sdf_cone_bound(level, cell, point) - traveled * spread;
        if advance <= cell * 0.5 {
            break;
        }
        traveled = traveled + advance;
        if traveled > span.y {
            traveled = TILE_SKY;
            break;
        }
    }
    textureStore(tile_start, vec2<i32>(id.xy), vec4<f32>(traveled, 0.0, 0.0, 0.0));
}
