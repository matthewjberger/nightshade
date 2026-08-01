#define_import_path nightshade_wgpu::sdf_cache

#import nightshade_wgpu::sdf_common::{
    SDF_BLEND_START,
    SDF_BRICK_CELLS,
    SDF_COVERED,
    SDF_DIRT_BIT,
    SDF_EDIT_BITS,
    SDF_FIRST_BRICK,
    SDF_HALF_DIAGONAL,
    SDF_NO_EDIT,
    SDF_RANGE_CELLS,
    SDF_WANTS_BRICK,
    SDF_WANTS_INSIDE,
    sdf,
    sdf_atlas_origin,
    sdf_atlas_texels,
    sdf_base_at,
    sdf_brick_at,
    sdf_empty_reach,
    sdf_stored_distance,
    sdf_level_cell,
    sdf_level_origin,
    sdf_level_reach,
    sdf_level_span,
    sdf_pointer_index,
    sdf_pointer_is_brick,
    sdf_pointer_is_empty,
    sdf_pointer_is_inside,
    sdf_tint_of,
}

@group(1) @binding(0)
var<storage, read> sdf_pointers: array<u32>;

@group(1) @binding(1)
var sdf_atlas: texture_3d<f32>;

@group(1) @binding(2)
var sdf_atlas_sampler: sampler;

@group(1) @binding(3)
var<storage, read> sdf_counts: array<u32>;

@group(1) @binding(4)
var<storage, read> sdf_stamps: array<u32>;

const STATE_FILLED: u32 = 0u;
const STATE_EMPTY: u32 = 1u;
const STATE_SOLID: u32 = 2u;
const STATE_OUTSIDE: u32 = 3u;
const STATE_NEAR: u32 = 4u;

const MARCH_STEPS: u32 = 160u;
const SDF_MISSING: f32 = 1.0e9;
const READING_TRUSTED: f32 = 0.98;
const LEAP_LEVELS: u32 = 3u;
const COARSE_SLACK: f32 = 0.87;
const VOXEL_NUDGE: f32 = 0.01;
const VOXEL_LAST: f32 = 6.0;
const ROOT_STEPS: u32 = 6u;
const ROOT_FLAT: f32 = 1.0e-7;
const SHADOW_STEPS: u32 = 32u;
const SHADOW_STRIDE_FLOOR: f32 = 0.5;
const SHADOW_TOUCH: f32 = 0.1;
const SHADOW_REACH: f32 = 60.0;
const SHADOW_LEVELS: u32 = 2u;
const SHADOW_DARK: f32 = 0.02;
const SHADOW_SUN_TANGENT: f32 = 0.08;
const OCCLUSION_TAPS: u32 = 4u;
const OCCLUSION_SHARES: array<f32, 4> = array<f32, 4>(0.12, 0.3, 0.6, 1.0);

struct Reading {
    value: f32,
    state: u32,
    slot: u32,
}

struct SdfWalk {
    hit: bool,
    exact: bool,
    point: vec3<f32>,
    traveled: f32,
    level: u32,
    slot: u32,
    steps: u32,
    gradient: vec3<f32>,
    tint: vec3<f32>,
    material: u32,
}

struct Corners {
    value: array<f32, 8>,
    material: array<u32, 8>,
}

struct Root {
    found: bool,
    along: f32,
}

fn safe_direction(direction: vec3<f32>) -> vec3<f32> {
    let size = max(abs(direction), vec3<f32>(1.0e-6));
    let way = select(vec3<f32>(-1.0), vec3<f32>(1.0), direction >= vec3<f32>(0.0));
    return size * way;
}

fn pointer_at(level: u32, span: f32, point: vec3<f32>) -> u32 {
    let side = i32(sdf.counts.z);
    let brick = sdf_brick_at(span, point);
    let within = brick - sdf_base_at(span);
    if any(within < vec3<i32>(0)) || any(within >= vec3<i32>(side)) {
        return SDF_COVERED;
    }
    return sdf_pointers[sdf_pointer_index(level, brick)];
}

fn atlas_texel(slot: u32, span: f32, cell_size: f32, point: vec3<f32>) -> vec3<f32> {
    let brick = sdf_brick_at(span, point);
    let inside = (point - vec3<f32>(brick) * span) / cell_size;
    return sdf_atlas_origin(slot)
        + clamp(inside, vec3<f32>(0.0), vec3<f32>(SDF_BRICK_CELLS))
        + vec3<f32>(0.5);
}

fn read_level(level: u32, cell_size: f32, point: vec3<f32>) -> Reading {
    let span = cell_size * SDF_BRICK_CELLS;
    let range = SDF_RANGE_CELLS * cell_size;
    let pointer = pointer_at(level, span, point);
    if pointer == SDF_COVERED {
        return Reading(0.0, STATE_OUTSIDE, SDF_NO_EDIT);
    }
    if sdf_pointer_is_empty(pointer) {
        let center = (vec3<f32>(sdf_brick_at(span, point)) + vec3<f32>(0.5)) * span;
        let bound = max(sdf_empty_reach(pointer, cell_size) - distance(point, center), 0.0);
        if sdf_pointer_is_inside(pointer) {
            return Reading(-max(bound, range), STATE_SOLID, SDF_NO_EDIT);
        }
        return Reading(bound, STATE_EMPTY, SDF_NO_EDIT);
    }
    if pointer == SDF_WANTS_INSIDE {
        return Reading(-range, STATE_SOLID, SDF_NO_EDIT);
    }
    if pointer == SDF_WANTS_BRICK {
        return Reading(cell_size, STATE_NEAR, SDF_NO_EDIT);
    }
    let slot = pointer - SDF_FIRST_BRICK;
    let texel = atlas_texel(slot, span, cell_size, point);
    let stored = textureSampleLevel(sdf_atlas, sdf_atlas_sampler, texel / sdf_atlas_texels(), 0.0);
    return Reading(sdf_stored_distance(stored) * range, STATE_FILLED, slot);
}

fn sdf_material_at(level: u32, cell_size: f32, point: vec3<f32>) -> u32 {
    let span = cell_size * SDF_BRICK_CELLS;
    let pointer = pointer_at(level, span, point);
    if !sdf_pointer_is_brick(pointer) {
        return SDF_NO_EDIT;
    }
    let texel = atlas_texel(pointer - SDF_FIRST_BRICK, span, cell_size, point);
    let stored = textureLoad(sdf_atlas, vec3<i32>(floor(texel)), 0);
    return stored_material(stored);
}

fn sdf_baked_at(level: u32, cell_size: f32, point: vec3<f32>) -> bool {
    let pointer = pointer_at(level, cell_size * SDF_BRICK_CELLS, point);
    if !sdf_pointer_is_brick(pointer) {
        return false;
    }
    return sdf_stamps[pointer - SDF_FIRST_BRICK] == sdf.frame.x;
}

fn pick_level(point: vec3<f32>) -> u32 {
    let away = distance(point, sdf.camera.xyz);
    let first = sdf_level_reach(0u);
    if away <= first {
        return 0u;
    }
    let over = ceil(log2(away / first));
    if over >= f32(sdf.counts.y) {
        return sdf.counts.y;
    }
    return u32(over);
}

fn coarser_share(level: u32, point: vec3<f32>) -> f32 {
    if level + 1u >= sdf.counts.y {
        return 0.0;
    }
    let reach = sdf_level_reach(level);
    let opens = reach * SDF_BLEND_START;
    let away = distance(point, sdf.camera.xyz);
    return smoothstep(opens, reach, away);
}

fn sdf_blended_cell(level: u32, point: vec3<f32>) -> f32 {
    return sdf_level_cell(level) * (1.0 + coarser_share(level, point));
}

fn read_clipmap(level: u32, cell: f32, point: vec3<f32>) -> Reading {
    var reading = read_level(level, cell, point);
    if reading.state != STATE_FILLED {
        return reading;
    }
    let share = coarser_share(level, point);
    if share > 0.0 {
        let coarse = read_level(level + 1u, cell * 2.0, point);
        if coarse.state == STATE_FILLED {
            reading.value = mix(reading.value, coarse.value, share);
        }
    }
    return reading;
}

fn box_exit(low: vec3<f32>, high: vec3<f32>, point: vec3<f32>, direction: vec3<f32>) -> f32 {
    let slope = 1.0 / safe_direction(direction);
    let toward_low = (low - point) * slope;
    let toward_high = (high - point) * slope;
    let farthest = max(toward_low, toward_high);
    return max(min(farthest.x, min(farthest.y, farthest.z)), 0.0);
}

fn brick_exit(size: f32, point: vec3<f32>, direction: vec3<f32>) -> f32 {
    let low = vec3<f32>(sdf_brick_at(size, point)) * size;
    return box_exit(low, low + vec3<f32>(size), point, direction);
}

fn coarse_bound(level: u32, cell: f32, point: vec3<f32>, direction: vec3<f32>, levels: u32) -> f32 {
    var bound = 0.0;
    var over = levels;
    loop {
        if over == 0u {
            break;
        }
        if level + over < sdf.counts.y {
            let coarse_cell = cell * exp2(f32(over));
            let coarse = read_level(level + over, coarse_cell, point);
            if coarse.state == STATE_EMPTY {
                let span = coarse_cell * SDF_BRICK_CELLS;
                return max(bound, max(brick_exit(span, point, direction), coarse.value));
            }
            if coarse.state == STATE_FILLED {
                bound = max(bound, coarse.value - COARSE_SLACK * coarse_cell);
            }
        }
        over = over - 1u;
    }
    return bound;
}

fn leap_empty(level: u32, cell: f32, point: vec3<f32>, direction: vec3<f32>, bound: f32, levels: u32) -> f32 {
    let near = max(brick_exit(cell * SDF_BRICK_CELLS, point, direction), bound);
    return max(near, coarse_bound(level, cell, point, direction, levels)) + cell * 0.1;
}

fn stored_material(stored: vec4<f32>) -> u32 {
    let packed = u32(round(stored.g * 255.0)) | (u32(round(stored.b * 255.0)) << 8u);
    let index = packed & SDF_EDIT_BITS;
    if index == 0u {
        return SDF_NO_EDIT;
    }
    return (index - 1u) | (packed & SDF_DIRT_BIT);
}

fn coarse_corner(
    level: u32,
    coarse_cell: f32,
    coarse_brick: vec3<i32>,
    coarse_slot: u32,
    at: vec3<f32>,
) -> Reading {
    let coarse_span = coarse_cell * SDF_BRICK_CELLS;
    if coarse_slot == SDF_NO_EDIT || any(sdf_brick_at(coarse_span, at) != coarse_brick) {
        return read_level(level, coarse_cell, at);
    }
    let coarse_texel = atlas_texel(coarse_slot, coarse_span, coarse_cell, at);
    let stored = textureSampleLevel(sdf_atlas, sdf_atlas_sampler, coarse_texel / sdf_atlas_texels(), 0.0);
    return Reading(sdf_stored_distance(stored) * SDF_RANGE_CELLS * coarse_cell, STATE_FILLED, coarse_slot);
}

fn voxel_corners(
    level: u32,
    cell: f32,
    voxel_low: vec3<f32>,
    texel: vec3<i32>,
) -> Corners {
    let range = SDF_RANGE_CELLS * cell;
    let middle = voxel_low + vec3<f32>(cell * 0.5);
    let farthest = coarser_share(
        level,
        middle + normalize(middle - sdf.camera.xyz) * cell * SDF_HALF_DIAGONAL,
    );
    let coarse_cell = cell * 2.0;
    let coarse_span = coarse_cell * SDF_BRICK_CELLS;
    let coarse_brick = sdf_brick_at(coarse_span, middle);
    var coarse_slot = SDF_NO_EDIT;
    if farthest > 0.0 {
        let pointer = pointer_at(level + 1u, coarse_span, middle);
        if sdf_pointer_is_brick(pointer) {
            coarse_slot = pointer - SDF_FIRST_BRICK;
        }
    }
    var corners: Corners;
    for (var corner = 0u; corner < 8u; corner = corner + 1u) {
        let offset = vec3<i32>(i32(corner & 1u), i32((corner >> 1u) & 1u), i32(corner >> 2u));
        let stored = textureLoad(sdf_atlas, texel + offset, 0);
        var value = sdf_stored_distance(stored) * range;
        if farthest > 0.0 {
            let at = voxel_low + vec3<f32>(offset) * cell;
            let share = coarser_share(level, at);
            let coarse = coarse_corner(level + 1u, coarse_cell, coarse_brick, coarse_slot, at);
            if share > 0.0 && coarse.state == STATE_FILLED {
                value = mix(value, coarse.value, share);
            }
        }
        corners.value[corner] = value;
        corners.material[corner] = stored_material(stored);
    }
    return corners;
}

fn cubic_at(coefficients: vec4<f32>, along: f32) -> f32 {
    return ((coefficients.w * along + coefficients.z) * along + coefficients.y) * along
        + coefficients.x;
}

fn ray_cubic(corners: array<f32, 8>, origin: vec3<f32>, direction: vec3<f32>) -> vec4<f32> {
    let k0 = corners[0];
    let k1 = corners[1] - corners[0];
    let k2 = corners[2] - corners[0];
    let k3 = corners[3] - corners[2] - k1;
    let a = corners[5] - corners[4];
    let k4 = k0 - corners[4];
    let k5 = k1 - a;
    let k6 = k2 - (corners[6] - corners[4]);
    let k7 = k3 - (corners[7] - corners[6] - a);
    let m0 = origin.x * origin.y;
    let m1 = direction.x * direction.y;
    let m2 = origin.x * direction.y + origin.y * direction.x;
    let m3 = k5 * origin.z - k1;
    let m4 = k6 * origin.z - k2;
    let m5 = k7 * origin.z - k3;
    let c0 = (k4 * origin.z - k0) + origin.x * m3 + origin.y * m4 + m0 * m5;
    let c1 = direction.x * m3 + direction.y * m4 + m2 * m5
        + direction.z * (k4 + k5 * origin.x + k6 * origin.y + k7 * m0);
    let c2 = m1 * m5 + direction.z * (k5 * direction.x + k6 * direction.y + k7 * m2);
    let c3 = k7 * m1 * direction.z;
    return -vec4<f32>(c0, c1, c2, c3);
}

fn stop_after(stops: array<f32, 4>, count: u32, candidate: f32, far: f32) -> array<f32, 4> {
    var placed = stops;
    if !(candidate > 0.0 && candidate < far) {
        return placed;
    }
    var at = count;
    loop {
        if at <= 1u || placed[at - 1u] <= candidate {
            break;
        }
        placed[at] = placed[at - 1u];
        at = at - 1u;
    }
    placed[at] = candidate;
    return placed;
}

fn voxel_root(coefficients: vec4<f32>, far: f32) -> Root {
    if coefficients.x <= 0.0 {
        return Root(true, 0.0);
    }
    var stops = array<f32, 4>(0.0, far, far, far);
    var count = 1u;
    let quadratic_a = 3.0 * coefficients.w;
    let quadratic_b = 2.0 * coefficients.z;
    let quadratic_c = coefficients.y;
    if abs(quadratic_a) > ROOT_FLAT {
        let discriminant = quadratic_b * quadratic_b - 4.0 * quadratic_a * quadratic_c;
        if discriminant >= 0.0 {
            let square = sqrt(discriminant);
            let first = (-quadratic_b - square) / (2.0 * quadratic_a);
            let second = (-quadratic_b + square) / (2.0 * quadratic_a);
            stops = stop_after(stops, count, min(first, second), far);
            count = count + u32(min(first, second) > 0.0 && min(first, second) < far);
            stops = stop_after(stops, count, max(first, second), far);
            count = count + u32(max(first, second) > 0.0 && max(first, second) < far);
        }
    } else if abs(quadratic_b) > ROOT_FLAT {
        let only = -quadratic_c / quadratic_b;
        stops = stop_after(stops, count, only, far);
        count = count + u32(only > 0.0 && only < far);
    }
    stops[count] = far;
    var low = 0.0;
    var low_value = coefficients.x;
    for (var interval = 0u; interval < count; interval = interval + 1u) {
        let high = stops[interval + 1u];
        let high_value = cubic_at(coefficients, high);
        if high_value > 0.0 {
            low = high;
            low_value = high_value;
            continue;
        }
        var near = low;
        var near_value = low_value;
        var beyond = high;
        var beyond_value = high_value;
        var along = high;
        for (var step = 0u; step < ROOT_STEPS; step = step + 1u) {
            let gap = near_value - beyond_value;
            if gap <= 0.0 {
                break;
            }
            along = (near_value * beyond - beyond_value * near) / gap;
            let value = cubic_at(coefficients, along);
            if value > 0.0 {
                near = along;
                near_value = value;
            } else {
                beyond = along;
                beyond_value = value;
            }
        }
        return Root(true, along);
    }
    return Root(false, far);
}

fn trilinear_gradient(corners: array<f32, 8>, at: vec3<f32>) -> vec3<f32> {
    let along_x = mix(
        mix(corners[1] - corners[0], corners[3] - corners[2], at.y),
        mix(corners[5] - corners[4], corners[7] - corners[6], at.y),
        at.z
    );
    let along_y = mix(
        mix(corners[2] - corners[0], corners[3] - corners[1], at.x),
        mix(corners[6] - corners[4], corners[7] - corners[5], at.x),
        at.z
    );
    let along_z = mix(
        mix(corners[4] - corners[0], corners[5] - corners[1], at.x),
        mix(corners[6] - corners[2], corners[7] - corners[3], at.x),
        at.y
    );
    return vec3<f32>(along_x, along_y, along_z);
}

struct VoxelTry {
    root: Root,
    exit: f32,
    exit_value: f32,
    gradient: vec3<f32>,
    tint: vec3<f32>,
    material: u32,
}

fn corner_weights(at: vec3<f32>) -> array<f32, 8> {
    var weights: array<f32, 8>;
    for (var corner = 0u; corner < 8u; corner = corner + 1u) {
        let offset = vec3<f32>(f32(corner & 1u), f32((corner >> 1u) & 1u), f32(corner >> 2u));
        let pull = mix(vec3<f32>(1.0) - at, at, offset);
        weights[corner] = pull.x * pull.y * pull.z;
    }
    return weights;
}

fn try_voxel(
    level: u32,
    cell: f32,
    slot: u32,
    point: vec3<f32>,
    direction: vec3<f32>,
) -> VoxelTry {
    let span = cell * SDF_BRICK_CELLS;
    let brick_low = vec3<f32>(sdf_brick_at(span, point)) * span;
    let voxel = clamp(floor((point - brick_low) / cell), vec3<f32>(0.0), vec3<f32>(VOXEL_LAST));
    let voxel_low = brick_low + voxel * cell;
    let texel = vec3<i32>(sdf_atlas_origin(slot)) + vec3<i32>(voxel);
    let corners = voxel_corners(level, cell, voxel_low, texel);
    let exit = box_exit(voxel_low, voxel_low + vec3<f32>(cell), point, direction);
    let origin = clamp((point - voxel_low) / cell, vec3<f32>(0.0), vec3<f32>(1.0));
    let coefficients = ray_cubic(corners.value, origin, direction / cell);
    let root = voxel_root(coefficients, exit);
    let exit_value = max(cubic_at(coefficients, exit), 0.0);
    let at = clamp(origin + direction / cell * root.along, vec3<f32>(0.0), vec3<f32>(1.0));
    var tint = vec3<f32>(0.0);
    var material = SDF_NO_EDIT;
    var heaviest = -1.0;
    if root.found {
        let weights = corner_weights(at);
        for (var corner = 0u; corner < 8u; corner = corner + 1u) {
            tint = tint + sdf_tint_of(corners.material[corner]) * weights[corner];
            if weights[corner] > heaviest {
                heaviest = weights[corner];
                material = corners.material[corner];
            }
        }
    }
    return VoxelTry(root, exit, exit_value, trilinear_gradient(corners.value, at), tint, material);
}

fn sdf_clipmap_span(eye: vec3<f32>, direction: vec3<f32>) -> vec2<f32> {
    let coarsest = sdf.counts.y - 1u;
    let low = sdf_level_origin(coarsest);
    let high = low + vec3<f32>(sdf_level_span(coarsest));
    let slope = 1.0 / safe_direction(direction);
    let toward_low = (low - eye) * slope;
    let toward_high = (high - eye) * slope;
    let entry = max(
        max(min(toward_low.x, toward_high.x), min(toward_low.y, toward_high.y)),
        max(min(toward_low.z, toward_high.z), 0.0)
    );
    let leaving = min(
        min(max(toward_low.x, toward_high.x), max(toward_low.y, toward_high.y)),
        max(toward_low.z, toward_high.z)
    );
    return vec2<f32>(entry, leaving);
}

fn sdf_cone_bound(level: u32, cell: f32, point: vec3<f32>) -> f32 {
    let reading = read_level(level, cell, point);
    var bound = 0.0;
    if reading.state == STATE_EMPTY {
        bound = reading.value;
    } else if reading.state == STATE_FILLED {
        bound = reading.value - COARSE_SLACK * cell;
    } else {
        return 0.0;
    }
    for (var over = 1u; over <= LEAP_LEVELS; over = over + 1u) {
        if level + over >= sdf.counts.y {
            break;
        }
        let coarse_cell = cell * exp2(f32(over));
        let coarse = read_level(level + over, coarse_cell, point);
        if coarse.state == STATE_EMPTY {
            bound = max(bound, coarse.value);
        }
        if coarse.state == STATE_FILLED {
            bound = max(bound, coarse.value - COARSE_SLACK * coarse_cell);
        }
    }
    return max(bound, 0.0);
}

fn sdf_walk(eye: vec3<f32>, direction: vec3<f32>, limit: f32, start: f32) -> SdfWalk {
    var walk = SdfWalk(false, false, eye, 0.0, 0u, SDF_NO_EDIT, 0u, -direction, vec3<f32>(0.0), SDF_NO_EDIT);

    let span = sdf_clipmap_span(eye, direction);
    let entry = span.x;
    let leaving = min(span.y, limit);
    if entry > leaving {
        return walk;
    }

    var traveled = max(entry + sdf.sizes.x, start);
    var behind = traveled;
    var behind_value = SDF_MISSING;
    var steps = 0u;
    var hit = false;
    var exact = false;
    var level = 0u;
    var slot = SDF_NO_EDIT;
    var point = eye;
    var gradient = -direction;
    var tint = vec3<f32>(0.0);
    var material = SDF_NO_EDIT;
    loop {
        if steps >= MARCH_STEPS || traveled > leaving {
            break;
        }
        steps = steps + 1u;
        point = eye + direction * traveled;
        level = pick_level(point);
        if level >= sdf.counts.y {
            break;
        }
        let cell = sdf_level_cell(level);
        let reading = read_clipmap(level, cell, point);
        if reading.state == STATE_SOLID || (reading.state == STATE_FILLED && reading.value <= -cell) {
            if reading.state == STATE_FILLED
                && behind_value >= SDF_MISSING
                && traveled - behind > cell * VOXEL_NUDGE {
                traveled = 0.5 * (behind + traveled);
                continue;
            }
            hit = true;
            if reading.state == STATE_FILLED {
                slot = reading.slot;
            }
            if behind_value > 0.0
                && behind_value < SDF_MISSING
                && behind_value > reading.value {
                let share = behind_value / (behind_value - reading.value);
                traveled = mix(behind, traveled, clamp(share, 0.0, 1.0));
                point = eye + direction * traveled;
            }
            break;
        }
        if reading.state == STATE_OUTSIDE {
            behind_value = SDF_MISSING;
            traveled = traveled + cell;
            continue;
        }
        if reading.state == STATE_EMPTY {
            behind = traveled;
            behind_value = SDF_MISSING;
            traveled = traveled + leap_empty(level, cell, point, direction, reading.value, LEAP_LEVELS);
            continue;
        }
        if reading.state == STATE_NEAR {
            behind = traveled;
            behind_value = SDF_MISSING;
            traveled = traveled + cell;
            continue;
        }
        if reading.value < cell {
            let tried = try_voxel(level, cell, reading.slot, point, direction);
            if tried.root.found {
                hit = true;
                exact = true;
                slot = reading.slot;
                traveled = traveled + tried.root.along;
                point = eye + direction * traveled;
                gradient = tried.gradient;
                tint = tried.tint;
                material = tried.material;
                break;
            }
            behind = traveled + tried.exit;
            behind_value = tried.exit_value;
            traveled = behind + max(tried.exit_value, cell * VOXEL_NUDGE);
            continue;
        }
        behind = traveled;
        let range = SDF_RANGE_CELLS * cell;
        var stride = reading.value;
        if reading.value < range * READING_TRUSTED {
            behind_value = reading.value;
        } else {
            behind_value = SDF_MISSING;
            stride = max(stride, coarse_bound(level, cell, point, direction, LEAP_LEVELS));
        }
        traveled = traveled + stride;
    }

    walk.hit = hit;
    walk.exact = exact;
    walk.point = point;
    walk.traveled = traveled;
    walk.level = level;
    walk.slot = slot;
    walk.steps = steps;
    walk.gradient = gradient;
    walk.tint = tint;
    walk.material = material;
    return walk;
}

fn brick_value(slot: u32, cell: f32, inside: vec3<f32>) -> f32 {
    let texel = sdf_atlas_origin(slot) + inside + vec3<f32>(0.5);
    let stored = textureSampleLevel(sdf_atlas, sdf_atlas_sampler, texel / sdf_atlas_texels(), 0.0);
    return sdf_stored_distance(stored) * SDF_RANGE_CELLS * cell;
}

struct Tap {
    value: f32,
    at: vec3<f32>,
}

fn fine_tap(level: u32, cell: f32, slot: u32, brick_low: vec3<f32>, at: vec3<f32>) -> Tap {
    let inside = (at - brick_low) / cell;
    if all(inside >= vec3<f32>(0.0)) && all(inside <= vec3<f32>(SDF_BRICK_CELLS)) {
        return Tap(brick_value(slot, cell, inside), at);
    }
    let neighbor = read_level(level, cell, at);
    if neighbor.state == STATE_FILLED {
        return Tap(neighbor.value, at);
    }
    let held = clamp(inside, vec3<f32>(0.0), vec3<f32>(SDF_BRICK_CELLS));
    return Tap(brick_value(slot, cell, held), brick_low + held * cell);
}

fn normal_tap(level: u32, cell: f32, slot: u32, brick_low: vec3<f32>, at: vec3<f32>, share: f32) -> Tap {
    var tap = fine_tap(level, cell, slot, brick_low, at);
    if share > 0.0 {
        let coarse = read_level(level + 1u, cell * 2.0, tap.at);
        if coarse.state == STATE_FILLED {
            tap.value = mix(tap.value, coarse.value, share);
        }
    }
    return tap;
}

fn sdf_cache_normal(level: u32, cell: f32, slot: u32, point: vec3<f32>, fallback: vec3<f32>) -> vec3<f32> {
    if slot == SDF_NO_EDIT {
        if length(fallback) < 1.0e-9 {
            return vec3<f32>(0.0, 1.0, 0.0);
        }
        return normalize(fallback);
    }
    let span = cell * SDF_BRICK_CELLS;
    let brick_low = vec3<f32>(sdf_brick_at(span, point)) * span;
    let share = coarser_share(level, point);
    let apart = 0.5 * cell;
    var gradient = vec3<f32>(0.0);
    for (var axis = 0u; axis < 3u; axis = axis + 1u) {
        var step = vec3<f32>(0.0);
        step[axis] = apart;
        let ahead = normal_tap(level, cell, slot, brick_low, point + step, share);
        let behind = normal_tap(level, cell, slot, brick_low, point - step, share);
        let width = max(ahead.at[axis] - behind.at[axis], 1.0e-4 * cell);
        gradient[axis] = (ahead.value - behind.value) / width;
    }
    if length(gradient) < 1.0e-9 {
        if length(fallback) < 1.0e-9 {
            return vec3<f32>(0.0, 1.0, 0.0);
        }
        return normalize(fallback);
    }
    return normalize(gradient);
}

fn sdf_cache_occlusion(level: u32, cell: f32, point: vec3<f32>, normal: vec3<f32>, reach: f32) -> f32 {
    var closed = 0.0;
    var weight = 1.0;
    var total = 0.0;
    for (var tap = 0u; tap < OCCLUSION_TAPS; tap = tap + 1u) {
        let away = reach * OCCLUSION_SHARES[tap];
        let reading = read_clipmap(level, cell, point + normal * away);
        var found = away;
        if reading.state == STATE_FILLED || reading.state == STATE_SOLID {
            found = clamp(reading.value, -away, away);
        }
        closed = closed + weight * (away - found) / away;
        total = total + weight;
        weight = weight * 0.7;
    }
    return clamp(1.0 - closed / total, 0.0, 1.0);
}

fn sdf_cache_shadow(point: vec3<f32>, toward: vec3<f32>, start: f32) -> f32 {
    var shade = 1.0;
    var traveled = start;
    var previous = SDF_MISSING;
    for (var step = 0u; step < SHADOW_STEPS; step = step + 1u) {
        if traveled > SHADOW_REACH {
            break;
        }
        let at = point + toward * traveled;
        let level = pick_level(at);
        if level >= sdf.counts.y {
            break;
        }
        let cell = sdf_level_cell(level);
        let reading = read_clipmap(level, cell, at);
        if reading.state == STATE_OUTSIDE {
            previous = SDF_MISSING;
            traveled = traveled + cell;
            continue;
        }
        if reading.state == STATE_SOLID || (reading.state == STATE_FILLED && reading.value <= -cell) {
            return 0.0;
        }
        if reading.state == STATE_EMPTY {
            previous = SDF_MISSING;
            traveled = traveled + leap_empty(level, cell, at, toward, reading.value, SHADOW_LEVELS);
            continue;
        }
        if reading.state == STATE_NEAR {
            previous = SDF_MISSING;
            traveled = traveled + cell;
            continue;
        }
        var stride = max(reading.value, cell * SHADOW_STRIDE_FLOOR);
        if reading.value >= SDF_RANGE_CELLS * cell * READING_TRUSTED {
            stride = max(stride, coarse_bound(level, cell, at, toward, SHADOW_LEVELS));
            previous = SDF_MISSING;
        } else {
            if reading.value < cell * SHADOW_TOUCH {
                return 0.0;
            }
            let value = reading.value;
            let bend = value * value / (2.0 * previous);
            let apart = sqrt(max(value * value - bend * bend, 0.0));
            shade = min(shade, apart / (max(traveled - bend, cell) * SHADOW_SUN_TANGENT));
            previous = value;
            if shade <= SHADOW_DARK {
                return 0.0;
            }
        }
        traveled = traveled + stride;
    }
    let soft = clamp(shade, 0.0, 1.0);
    return soft * soft * (3.0 - 2.0 * soft);
}
