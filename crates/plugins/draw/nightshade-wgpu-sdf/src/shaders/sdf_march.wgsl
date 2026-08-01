#import nightshade_wgpu::sdf_common::{
    SDF_CASCADES,
    SDF_EDIT_BITS,
    SDF_NO_EDIT,
    sdf,
    sdf_atlas_bricks,
    sdf_edits,
    sdf_brick_of,
    sdf_identity_of,
    sdf_level_cell,
    sdf_tint_of,
}
#import nightshade_wgpu::sdf_cache::{
    MARCH_STEPS,
    sdf_baked_at,
    sdf_blended_cell,
    sdf_cache_normal,
    sdf_cache_occlusion,
    sdf_cache_shadow,
    sdf_counts,
    sdf_material_at,
    sdf_walk,
}

@group(2) @binding(0)
var shadow_depth: texture_depth_2d;

@group(2) @binding(1)
var irradiance: texture_cube<f32>;

@group(2) @binding(2)
var irradiance_other: texture_cube<f32>;

@group(2) @binding(3)
var irradiance_sampler: sampler;

@group(2) @binding(4)
var cast_depth: texture_2d<f32>;

@group(2) @binding(5)
var tile_start: texture_2d<f32>;

@group(2) @binding(6)
var<storage, read> sdf_motions: array<mat4x4<f32>>;

@group(2) @binding(7)
var prefiltered: texture_cube<f32>;

@group(2) @binding(8)
var prefiltered_other: texture_cube<f32>;

const TILE: f32 = 8.0;

const OCCLUSION_CELLS: f32 = 3.0;
const SHADOW_LIFT: f32 = 1.5;
const OCCLUSION_FLOOR: f32 = 0.25;
const INV_PI: f32 = 0.31830989;
const ROUGHNESS_FLOOR: f32 = 0.04;
const DIELECTRIC_REFLECTANCE: f32 = 0.04;
const PLAIN_ROUGHNESS: f32 = 0.8;
const SHADOW_TAPS: u32 = 8u;
const SHADOW_SPREAD: f32 = 1.5;
const SHADOW_LEAN_FLOOR: f32 = 0.15;
const SHADOW_SLOPE_FLOOR: f32 = 0.35;
const SHADOW_FADE: f32 = 0.9;
const SHADOW_BLEND: f32 = 0.25;
const IRRADIANCE_CAP: f32 = 65000.0;
const BAKED_TINT: vec3<f32> = vec3<f32>(0.62, 0.18, 0.80);
const SETTLED_TINT: vec3<f32> = vec3<f32>(0.55, 0.55, 0.55);

const POISSON_8: array<vec2<f32>, 8> = array<vec2<f32>, 8>(
    vec2<f32>(-0.94201624, -0.39906216),
    vec2<f32>(0.94558609, -0.76890725),
    vec2<f32>(-0.09418410, -0.92938870),
    vec2<f32>(0.34495938, 0.29387760),
    vec2<f32>(-0.91588581, 0.45771432),
    vec2<f32>(-0.81544232, -0.87912464),
    vec2<f32>(-0.38277543, 0.27676845),
    vec2<f32>(0.97484398, 0.75648379)
);

struct Surface {
    @location(0) color: vec4<f32>,
    @location(1) entity_id: f32,
    @location(2) view_normal: vec4<f32>,
    @location(3) velocity: vec2<f32>,
    @builtin(frag_depth) depth: f32,
}

fn mesh_shadow_tap(texel: vec2<i32>, reference: f32) -> f32 {
    let stored = textureLoad(shadow_depth, texel, 0);
    let shrink = i32(max(sdf.flags.w, 1u));
    let written = textureLoad(cast_depth, texel / shrink, 0).r;
    if stored == written {
        return 1.0;
    }
    return select(0.0, 1.0, reference >= stored);
}

fn sample_cascade(world: vec3<f32>, normal: vec3<f32>, cascade: u32) -> f32 {
    let toward_sun = sdf.sun_direction.xyz;
    let texel_world = sdf.cascade_offsets[cascade].z;
    let facing = clamp(dot(normal, toward_sun), 0.0, 1.0);
    let slope = sqrt(max(1.0 - facing * facing, 0.0));
    let search = clamp(sdf.sizes.w * 2.0, 1.0, 4.0);
    let normal_offset = sdf.sizes.z * texel_world * normal * max(slope, SHADOW_SLOPE_FLOOR) * search;
    let depth_offset = sdf.sizes.y * texel_world * toward_sun;
    let offset_world = world + normal_offset + depth_offset;
    let clip = sdf.cascade_view_projections[cascade] * vec4<f32>(offset_world, 1.0);
    let ndc = clip.xyz / clip.w;
    let slot_scale = vec2<f32>(0.5, 0.5);
    let slot_offset = sdf.cascade_offsets[cascade].xy;
    let uv = (ndc.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5)) * slot_scale + slot_offset;
    let slot_low = slot_offset;
    let slot_high = slot_offset + slot_scale;
    let in_bounds = f32(
        uv.x >= slot_low.x && uv.x <= slot_high.x
            && uv.y >= slot_low.y && uv.y <= slot_high.y
            && ndc.z >= 0.0 && ndc.z <= 1.0
    );
    let size = f32(textureDimensions(shadow_depth).x);
    let texel = 1.0 / size;
    let margin = slot_scale * 0.01;
    let safe_low = slot_low + margin;
    let safe_high = slot_high - margin;
    let depth_row = vec3<f32>(
        sdf.cascade_view_projections[cascade][0][2],
        sdf.cascade_view_projections[cascade][1][2],
        sdf.cascade_view_projections[cascade][2][2]
    );
    let leaning = slope / max(facing, SHADOW_LEAN_FLOOR);
    let per_texel = texel_world * leaning * length(depth_row);
    let reference = ndc.z + per_texel * SHADOW_SPREAD;
    var visibility = 0.0;
    for (var tap = 0u; tap < SHADOW_TAPS; tap = tap + 1u) {
        let sample_uv = clamp(uv + POISSON_8[tap] * texel * SHADOW_SPREAD, safe_low, safe_high);
        visibility = visibility + mesh_shadow_tap(vec2<i32>(sample_uv * size), reference);
    }
    return mix(1.0, visibility / f32(SHADOW_TAPS), in_bounds);
}

fn mesh_shadow(world: vec3<f32>, normal: vec3<f32>) -> f32 {
    if sdf.flags.z == 0u {
        return 1.0;
    }
    let view_depth = -(sdf.view * vec4<f32>(world, 1.0)).z;
    let shadow_far = sdf.cascade_splits[SDF_CASCADES - 1u];
    if view_depth > shadow_far {
        return 1.0;
    }
    let far_fade = 1.0 - smoothstep(shadow_far * SHADOW_FADE, shadow_far, view_depth);
    var cascade = SDF_CASCADES - 1u;
    for (var which = 0u; which < SDF_CASCADES; which = which + 1u) {
        if view_depth < sdf.cascade_splits[which] {
            cascade = which;
            break;
        }
    }
    var factor = sample_cascade(world, normal, cascade);
    if cascade + 1u < SDF_CASCADES {
        let cascade_end = sdf.cascade_splits[cascade];
        let blend_range = cascade_end * SHADOW_BLEND;
        let blend_start = cascade_end - blend_range;
        if view_depth > blend_start {
            let share = (view_depth - blend_start) / blend_range;
            factor = mix(factor, sample_cascade(world, normal, cascade + 1u), share);
        }
    }
    return mix(1.0, factor, far_fade);
}

fn finish_of(material: u32) -> vec2<f32> {
    if material == SDF_NO_EDIT {
        return vec2<f32>(PLAIN_ROUGHNESS, 0.0);
    }
    return sdf_edits[material & SDF_EDIT_BITS].surface.xy;
}

fn sky_reflection(direction: vec3<f32>, roughness: f32) -> vec3<f32> {
    let deepest = f32(textureNumLevels(prefiltered)) - 1.0;
    let level = roughness * deepest;
    let first = clamp(
        textureSampleLevel(prefiltered, irradiance_sampler, direction, level).rgb,
        vec3<f32>(0.0),
        vec3<f32>(IRRADIANCE_CAP)
    );
    let second = clamp(
        textureSampleLevel(prefiltered_other, irradiance_sampler, direction, level).rgb,
        vec3<f32>(0.0),
        vec3<f32>(IRRADIANCE_CAP)
    );
    return mix(first, second, sdf.sun_color.w);
}

fn reflectance_at(base: vec3<f32>, angle: f32) -> vec3<f32> {
    return base + (vec3<f32>(1.0) - base) * pow(1.0 - angle, 5.0);
}

fn sky_response(base: vec3<f32>, roughness: f32, facing: f32) -> vec3<f32> {
    let low = vec4<f32>(-1.0, -0.0275, -0.572, 0.022);
    let high = vec4<f32>(1.0, 0.0425, 1.04, -0.04);
    let shaped = roughness * low + high;
    let fall = min(shaped.x * shaped.x, exp2(-9.28 * facing)) * shaped.x + shaped.y;
    let scale_and_bias = vec2<f32>(-1.04, 1.04) * fall + shaped.zw;
    return base * scale_and_bias.x + vec3<f32>(scale_and_bias.y);
}

fn sun_highlight(
    normal: vec3<f32>,
    toward_eye: vec3<f32>,
    toward_sun: vec3<f32>,
    roughness: f32,
    base: vec3<f32>,
) -> vec3<f32> {
    let halfway = normalize(toward_eye + toward_sun);
    let facing_eye = max(dot(normal, toward_eye), 1.0e-4);
    let facing_sun = max(dot(normal, toward_sun), 1.0e-4);
    let facing_half = max(dot(normal, halfway), 0.0);
    let alpha = roughness * roughness;
    let alpha_squared = alpha * alpha;
    let lobe = facing_half * facing_half * (alpha_squared - 1.0) + 1.0;
    let distribution = alpha_squared / (3.14159265 * lobe * lobe);
    let reach = (roughness + 1.0) * (roughness + 1.0) / 8.0;
    let hidden = facing_eye / (facing_eye * (1.0 - reach) + reach)
        * facing_sun / (facing_sun * (1.0 - reach) + reach);
    let fresnel = reflectance_at(base, max(dot(halfway, toward_eye), 0.0));
    return distribution * hidden * fresnel / (4.0 * facing_eye * facing_sun);
}

fn ambient_light(normal: vec3<f32>) -> vec3<f32> {
    let first = clamp(
        textureSampleLevel(irradiance, irradiance_sampler, normal, 0.0).rgb,
        vec3<f32>(0.0),
        vec3<f32>(IRRADIANCE_CAP)
    );
    let second = clamp(
        textureSampleLevel(irradiance_other, irradiance_sampler, normal, 0.0).rgb,
        vec3<f32>(0.0),
        vec3<f32>(IRRADIANCE_CAP)
    );
    return mix(first, second, sdf.sun_color.w);
}

fn brick_color(level: u32, point: vec3<f32>) -> vec3<f32> {
    let brick = vec3<f32>(sdf_brick_of(level, point));
    let seed = brick.x * 73.1 + brick.y * 151.7 + brick.z * 311.3 + f32(level) * 57.0;
    return vec3<f32>(
        fract(sin(seed) * 43758.5453),
        fract(sin(seed + 1.7) * 24634.6345),
        fract(sin(seed + 3.3) * 18465.2357)
    ) * 0.8 + vec3<f32>(0.2);
}

fn level_color(level: u32) -> vec3<f32> {
    let bands = array<vec3<f32>, 8>(
        vec3<f32>(0.95, 0.35, 0.35),
        vec3<f32>(0.95, 0.70, 0.30),
        vec3<f32>(0.90, 0.92, 0.35),
        vec3<f32>(0.40, 0.90, 0.45),
        vec3<f32>(0.35, 0.80, 0.92),
        vec3<f32>(0.45, 0.50, 0.95),
        vec3<f32>(0.78, 0.45, 0.95),
        vec3<f32>(0.85, 0.85, 0.85)
    );
    return bands[level % 8u];
}

fn screen_uv(clip: vec4<f32>) -> vec2<f32> {
    let ndc = clip.xy / clip.w;
    return vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) @invariant vec4<f32> {
    let half = i32(vertex_index) / 2;
    let odd = i32(vertex_index) & 1;
    return vec4<f32>(f32(half) * 4.0 - 1.0, f32(odd) * 4.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> Surface {
    let screen = vec2<f32>(
        position.x * sdf.screen.z * 2.0 - 1.0,
        1.0 - position.y * sdf.screen.w * 2.0
    );
    let near = sdf.inverse_view_projection * vec4<f32>(screen, 1.0, 1.0);
    let middle = sdf.inverse_view_projection * vec4<f32>(screen, 0.5, 1.0);
    let eye = sdf.camera.xyz;
    let lens = near.xyz / near.w;
    let direction = normalize(middle.xyz / middle.w - lens);

    let start = textureLoad(tile_start, vec2<i32>(position.xy / TILE), 0).r;
    if start < 0.0 {
        discard;
    }
    let walk = sdf_walk(eye, direction, 1.0e9, max(start, distance(lens, eye)));
    if !walk.hit {
        discard;
    }
    let point = walk.point;
    let cell = sdf_level_cell(walk.level);

    let clip = sdf.view_projection * vec4<f32>(point, 1.0);
    let depth = clamp(clip.z / clip.w, 0.0, 1.0);

    let normal = sdf_cache_normal(walk.level, cell, walk.slot, point, walk.gradient);
    let view_normal = normalize((sdf.view * vec4<f32>(normal, 0.0)).xyz);

    var material = walk.material;
    var tint = walk.tint;
    if !walk.exact {
        material = sdf_material_at(walk.level, cell, point);
        tint = sdf_tint_of(material);
    }
    let identity = bitcast<f32>(sdf_identity_of(material));

    var earlier = vec4<f32>(point, 1.0);
    if material != SDF_NO_EDIT {
        earlier = sdf_motions[material & SDF_EDIT_BITS] * earlier;
    }
    let previous = sdf.previous_view_projection * earlier;
    let velocity = screen_uv(previous) - screen_uv(clip);

    let debug = sdf.counts.w;
    if debug == 1u {
        return Surface(
            vec4<f32>(brick_color(walk.level, point), 1.0),
            identity,
            vec4<f32>(view_normal, 0.0),
            velocity,
            depth
        );
    }
    if debug == 2u {
        return Surface(
            vec4<f32>(level_color(walk.level), 1.0),
            identity,
            vec4<f32>(view_normal, 0.0),
            velocity,
            depth
        );
    }
    if debug == 3u {
        let load = f32(walk.steps) / f32(MARCH_STEPS);
        let bricks = sdf_atlas_bricks();
        let slots = bricks * bricks * bricks;
        let spare = f32(sdf_counts[0]) / f32(slots);
        return Surface(
            vec4<f32>(load, 1.0 - load, spare, 1.0),
            identity,
            vec4<f32>(view_normal, 0.0),
            velocity,
            depth
        );
    }
    if debug == 4u {
        let shade = 0.4 + 0.6 * max(normal.y, 0.0);
        let color = select(SETTLED_TINT, BAKED_TINT, sdf_baked_at(walk.level, cell, point)) * shade;
        return Surface(vec4<f32>(color, 1.0), identity, vec4<f32>(view_normal, 0.0), velocity, depth);
    }

    let smooth_cell = sdf_blended_cell(walk.level, point);
    let occlusion = sdf_cache_occlusion(walk.level, cell, point, normal, smooth_cell * OCCLUSION_CELLS);
    let shaded = mix(OCCLUSION_FLOOR, 1.0, occlusion);
    let finish = finish_of(material);
    let roughness = clamp(finish.x, ROUGHNESS_FLOOR, 1.0);
    let metal = clamp(finish.y, 0.0, 1.0);
    let toward_eye = -direction;
    let facing_eye = max(dot(normal, toward_eye), 1.0e-4);
    let base = mix(vec3<f32>(DIELECTRIC_REFLECTANCE), tint, metal);
    let body = tint * (1.0 - metal);
    let mirrored = reflect(direction, normal);
    let sheen = sky_reflection(mirrored, roughness) * sky_response(base, roughness, facing_eye);
    var lit = (body * ambient_light(normal) + sheen) * shaded;
    if sdf.sun_direction.w > 0.5 {
        let toward_sun = sdf.sun_direction.xyz;
        let facing = max(dot(normal, toward_sun), 0.0);
        if facing > 0.0 {
            let meshed = mesh_shadow(point, normal);
            var clear = 0.0;
            if meshed > 0.0 {
                let lifted = point + normal * smooth_cell * SHADOW_LIFT;
                clear = sdf_cache_shadow(lifted, toward_sun, 0.0) * meshed;
            }
            let glint = sun_highlight(normal, toward_eye, toward_sun, roughness, base);
            lit = lit + (body * INV_PI + glint) * sdf.sun_color.rgb * facing * clear;
        }
    }
    return Surface(vec4<f32>(lit, 1.0), identity, vec4<f32>(view_normal, 0.0), velocity, depth);
}
