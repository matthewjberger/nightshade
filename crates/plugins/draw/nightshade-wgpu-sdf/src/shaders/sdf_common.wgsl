#define_import_path nightshade_wgpu::sdf_common

const SDF_BRICK_SAMPLES: u32 = 8u;
const SDF_BRICK_CELLS: f32 = 7.0;
const SDF_RANGE_CELLS: f32 = 6.0;
const SDF_CASCADES: u32 = 4u;

const SDF_KIND_TERRAIN: u32 = 6u;
const SDF_OCTAHEDRON_FACE: f32 = 0.57735027;

const SDF_UNION: u32 = 0u;
const SDF_SUBTRACT: u32 = 1u;
const SDF_INTERSECT: u32 = 2u;

const SDF_WANTS_BRICK: u32 = 0u;
const SDF_EMPTY_TAG: u32 = 0x80000000u;
const SDF_INSIDE_TAG: u32 = 0x40000000u;
const SDF_EMPTY_UNITS: u32 = 0x3fffffffu;
const SDF_EMPTY_STEPS_PER_CELL: f32 = 8.0;
const SDF_COVERED: u32 = 3u;
const SDF_WANTS_INSIDE: u32 = 4u;
const SDF_FIRST_BRICK: u32 = 5u;

const SDF_BRICK_BIAS: i32 = 32768;
const SDF_CONTROL_FREE: u32 = 0u;
const SDF_CONTROL_BAKE: u32 = 1u;
const SDF_CONTROL_BOOKED: u32 = 2u;
const SDF_CONTROL_WORK: u32 = 3u;
const SDF_CONTROL_QUEUED: u32 = 4u;
const SDF_CONTROL_BAKING: u32 = 5u;
const SDF_BAKE_ARGUMENTS: u32 = 4u;
const SDF_WORK_GROUP: u32 = 64u;
const SDF_GROUPS_PER_ROW: u32 = 65535u;
const SDF_NO_IDENTITY: u32 = 0xffffffffu;

const SDF_FAR: f32 = 1.0e6;
const SDF_NO_EDIT: u32 = 0xffffffffu;
const SDF_DIRT_BIT: u32 = 0x8000u;
const SDF_EDIT_BITS: u32 = 0x7fffu;
const SDF_OCTAVE_SWAY: f32 = 0.25;
const SDF_RESOLVE_CELLS: f32 = 2.5;
const SDF_NO_EARLY_EXIT: f32 = 1.0e9;
const SDF_BLEND_START: f32 = 0.8;
const SDF_LACUNARITY: f32 = 2.0;
const SDF_LATTICE_RADIUS: f32 = 0.5;
const SDF_HALF_DIAGONAL: f32 = 0.8660254;
const SDF_DEFAULT_TINT: vec3<f32> = vec3<f32>(0.72, 0.70, 0.66);
const SDF_TURN_AXIS: vec3<f32> = vec3<f32>(0.36, 0.48, 0.80);
const SDF_GOLDEN_ANGLE: f32 = 2.399963;

struct SdfEdit {
    inverse_transform: mat4x4<f32>,
    params: vec4<f32>,
    tint_blend: vec4<f32>,
    kind_operation_scale: vec4<f32>,
    bounds: vec4<f32>,
    identity: vec4<u32>,
    extra: vec4<f32>,
    surface: vec4<f32>,
}

struct SdfNode {
    low: vec4<f32>,
    high: vec4<f32>,
}

struct SdfUniforms {
    view_projection: mat4x4<f32>,
    inverse_view_projection: mat4x4<f32>,
    previous_view_projection: mat4x4<f32>,
    view: mat4x4<f32>,
    cascade_view_projections: array<mat4x4<f32>, 4>,
    cascade_inverse_view_projections: array<mat4x4<f32>, 4>,
    cascade_offsets: array<vec4<f32>, 4>,
    cascade_splits: vec4<f32>,
    camera: vec4<f32>,
    sun_direction: vec4<f32>,
    sun_color: vec4<f32>,
    screen: vec4<f32>,
    viewport: vec4<f32>,
    counts: vec4<u32>,
    flags: vec4<u32>,
    sizes: vec4<f32>,
    terrain: vec4<f32>,
    frame: vec4<u32>,
}

@group(0) @binding(0)
var<uniform> sdf: SdfUniforms;

@group(0) @binding(1)
var<storage, read> sdf_edits: array<SdfEdit>;

@group(0) @binding(2)
var<storage, read> sdf_tree: array<SdfNode>;

fn sdf_sphere(point: vec3<f32>, radius: f32) -> f32 {
    return length(point) - radius;
}

fn sdf_cube(point: vec3<f32>, half_extents: vec3<f32>, rounding: f32) -> f32 {
    let outside = abs(point) - half_extents;
    let corner = length(max(outside, vec3<f32>(0.0)));
    let inside = min(max(outside.x, max(outside.y, outside.z)), 0.0);
    return corner + inside - rounding;
}

fn sdf_capsule(point: vec3<f32>, half_height: f32, radius: f32) -> f32 {
    var along = point;
    along.y = along.y - clamp(along.y, -half_height, half_height);
    return length(along) - radius;
}

fn sdf_torus(point: vec3<f32>, major_radius: f32, minor_radius: f32) -> f32 {
    let ring = vec2<f32>(length(point.xz) - major_radius, point.y);
    return length(ring) - minor_radius;
}

fn sdf_cylinder(point: vec3<f32>, half_height: f32, radius: f32) -> f32 {
    let outside = abs(vec2<f32>(length(point.xz), point.y)) - vec2<f32>(radius, half_height);
    return min(max(outside.x, outside.y), 0.0) + length(max(outside, vec2<f32>(0.0)));
}

fn sdf_octahedron(point: vec3<f32>, radius: f32) -> f32 {
    let folded = abs(point);
    let plane = folded.x + folded.y + folded.z - radius;
    var turned = folded;
    if 3.0 * folded.x < plane {
        turned = folded;
    } else if 3.0 * folded.y < plane {
        turned = folded.yzx;
    } else if 3.0 * folded.z < plane {
        turned = folded.zxy;
    } else {
        return plane * SDF_OCTAHEDRON_FACE;
    }
    let along = clamp(0.5 * (turned.z - turned.y + radius), 0.0, radius);
    return length(vec3<f32>(turned.x, turned.y - radius + along, turned.z - along));
}

fn sdf_hash(cell: vec3<f32>) -> f32 {
    let scattered = fract(cell * vec3<f32>(0.1031, 0.1030, 0.0973));
    let mixed = scattered + vec3<f32>(dot(scattered, scattered.yzx + vec3<f32>(33.33)));
    return fract((mixed.x + mixed.y) * mixed.z);
}

fn sdf_turn(point: vec3<f32>, octave: u32) -> vec3<f32> {
    let angle = f32(octave) * SDF_GOLDEN_ANGLE;
    let cosine = cos(angle);
    let sine = sin(angle);
    return point * cosine
        + cross(SDF_TURN_AXIS, point) * sine
        + SDF_TURN_AXIS * (dot(SDF_TURN_AXIS, point) * (1.0 - cosine));
}

fn sdf_resolved_octaves(size: f32, cell: f32) -> u32 {
    let finest = max(size / (SDF_RESOLVE_CELLS * max(cell, 1.0e-6)), 1.0);
    return u32(floor(log2(finest))) + 1u;
}

fn sdf_shape(edit: SdfEdit, world: vec3<f32>, care: f32, cell: f32) -> f32 {
    let kind = u32(edit.kind_operation_scale.x);
    if kind == SDF_KIND_TERRAIN {
        return sdf_terrain(
            world,
            edit.params.x,
            edit.params.y,
            min(u32(edit.params.z), sdf_resolved_octaves(edit.params.y, cell)),
            edit.params.w,
            edit.bounds.x,
            edit.bounds.y,
            edit.bounds.z,
            care + sdf.terrain.y
        );
    }
    let local = (edit.inverse_transform * vec4<f32>(world, 1.0)).xyz;
    var reach = SDF_FAR;
    switch kind {
        case 0u: {
            reach = sdf_sphere(local, edit.params.x);
        }
        case 1u: {
            reach = sdf_cube(local, edit.params.xyz, edit.params.w);
        }
        case 2u: {
            reach = sdf_capsule(local, edit.params.x, edit.params.y);
        }
        case 3u: {
            reach = sdf_torus(local, edit.params.x, edit.params.y);
        }
        case 5u: {
            reach = sdf_octahedron(local, edit.params.x) - edit.params.y;
        }
        default: {
            reach = sdf_cylinder(local, edit.params.x, edit.params.y);
        }
    }
    return reach * edit.kind_operation_scale.z;
}

fn sdf_union(world: f32, shape: f32, smoothing: f32) -> f32 {
    if smoothing <= 0.0 {
        return min(world, shape);
    }
    let share = clamp(0.5 + 0.5 * (shape - world) / smoothing, 0.0, 1.0);
    return mix(shape, world, share) - smoothing * share * (1.0 - share);
}

fn sdf_subtract(world: f32, shape: f32, smoothing: f32) -> f32 {
    if smoothing <= 0.0 {
        return max(world, -shape);
    }
    let share = clamp(0.5 - 0.5 * (world + shape) / smoothing, 0.0, 1.0);
    return mix(world, -shape, share) + smoothing * share * (1.0 - share);
}

fn sdf_intersect(world: f32, shape: f32, smoothing: f32) -> f32 {
    if smoothing <= 0.0 {
        return max(world, shape);
    }
    let share = clamp(0.5 - 0.5 * (shape - world) / smoothing, 0.0, 1.0);
    return mix(shape, world, share) + smoothing * share * (1.0 - share);
}

fn sdf_lattice(point: vec3<f32>) -> f32 {
    let cell = floor(point);
    let offset = fract(point);
    var nearest = SDF_FAR;
    for (var corner = 0u; corner < 8u; corner = corner + 1u) {
        let at = vec3<f32>(f32(corner & 1u), f32((corner >> 1u) & 1u), f32(corner >> 2u));
        let radius = SDF_LATTICE_RADIUS * sdf_hash(cell + at);
        nearest = min(nearest, length(offset - at) - radius);
    }
    return nearest;
}

fn sdf_terrain(
    point: vec3<f32>,
    base: f32,
    size: f32,
    octaves: u32,
    gain: f32,
    clip: f32,
    clip_smooth: f32,
    join_smooth: f32,
    care: f32,
) -> f32 {
    var found = point.y - base;
    var amplitude = size;
    var frequency = 1.0 / size;
    var budget = SDF_NO_EARLY_EXIT;
    if abs(gain) < 1.0 {
        budget = (abs(clip) + SDF_OCTAVE_SWAY * (abs(clip_smooth) + abs(join_smooth))) / (1.0 - abs(gain));
    }
    for (var octave = 0u; octave < octaves; octave = octave + 1u) {
        let spheres = sdf_lattice(sdf_turn(point, octave) * frequency) * amplitude;
        let clipped = sdf_intersect(spheres, found - clip * amplitude, clip_smooth * amplitude);
        found = sdf_union(found, clipped, join_smooth * amplitude);
        amplitude = amplitude * gain;
        frequency = frequency * SDF_LACUNARITY;
        let sway = abs(amplitude) * budget;
        if found > sway + care {
            return found - sway;
        }
        if found < -(sway + care) {
            return found;
        }
    }
    return found;
}

struct SdfProbe {
    value: f32,
    nearest: f32,
    depth: f32,
    material: u32,
}

fn sdf_probe_into(probe: SdfProbe, index: u32, world: vec3<f32>, care: f32, cell: f32) -> SdfProbe {
    let edit = sdf_edits[index];
    if edit.kind_operation_scale.w < 0.5 {
        return probe;
    }
    let operation = u32(edit.kind_operation_scale.y);
    let smoothing = edit.tint_blend.w;
    if edit.bounds.w >= 0.0 && operation != SDF_INTERSECT {
        let gap = distance(world, edit.bounds.xyz) - edit.bounds.w;
        if gap > abs(probe.value) + smoothing {
            return probe;
        }
    }
    let shape = sdf_shape(edit, world, care, cell);
    var next = probe;
    if operation == SDF_UNION {
        next.value = sdf_union(probe.value, shape, smoothing);
    } else if operation == SDF_SUBTRACT {
        next.value = sdf_subtract(probe.value, shape, smoothing);
    } else {
        next.value = sdf_intersect(probe.value, shape, smoothing);
    }
    if operation != SDF_SUBTRACT && abs(shape) < probe.nearest {
        next.nearest = abs(shape);
        next.depth = shape;
        next.material = index;
    }
    return next;
}

fn sdf_probe(world: vec3<f32>, care: f32, cell: f32) -> SdfProbe {
    var probe = SdfProbe(SDF_FAR, SDF_FAR, SDF_FAR, SDF_NO_EDIT);
    let count = sdf.counts.x;
    if count == 0u {
        return probe;
    }
    var stack: array<u32, 32>;
    var depth = 1u;
    stack[0] = 0u;
    loop {
        if depth == 0u {
            break;
        }
        depth = depth - 1u;
        let node = sdf_tree[stack[depth]];
        let nearest = clamp(world, node.low.xyz, node.high.xyz);
        if distance(world, nearest) > abs(probe.value) {
            continue;
        }
        let held = node.high.w;
        if held < 0.0 {
            let index = stack[depth];
            if depth + 2u <= 32u {
                stack[depth] = index * 2u + 2u;
                stack[depth + 1u] = index * 2u + 1u;
                depth = depth + 2u;
            }
            continue;
        }
        let first = u32(node.low.w);
        let run = u32(held);
        for (var which = 0u; which < run; which = which + 1u) {
            probe = sdf_probe_into(probe, first + which, world, care, cell);
        }
    }
    return probe;
}

fn sdf_field(world: vec3<f32>, care: f32, cell: f32) -> f32 {
    return sdf_probe(world, care, cell).value;
}

fn sdf_material_cut(probe: SdfProbe) -> u32 {
    if probe.material == SDF_NO_EDIT {
        return probe.material;
    }
    let edit = sdf_edits[probe.material];
    if u32(edit.kind_operation_scale.x) != SDF_KIND_TERRAIN {
        return probe.material;
    }
    if probe.depth < probe.value - edit.extra.w {
        return probe.material | SDF_DIRT_BIT;
    }
    return probe.material;
}

fn sdf_tint_of(material: u32) -> vec3<f32> {
    if material == SDF_NO_EDIT {
        return SDF_DEFAULT_TINT;
    }
    let edit = sdf_edits[material & SDF_EDIT_BITS];
    if (material & SDF_DIRT_BIT) != 0u {
        return edit.extra.xyz;
    }
    return edit.tint_blend.xyz;
}

fn sdf_identity_of(material: u32) -> u32 {
    if material == SDF_NO_EDIT {
        return 0u;
    }
    return sdf_edits[material & SDF_EDIT_BITS].identity.x;
}


fn sdf_level_cell(level: u32) -> f32 {
    return sdf.sizes.x * exp2(f32(level));
}

fn sdf_level_brick(level: u32) -> f32 {
    return sdf_level_cell(level) * SDF_BRICK_CELLS;
}

fn sdf_base_at(brick: f32) -> vec3<i32> {
    let half_span = brick * f32(sdf.counts.z) * 0.5;
    return vec3<i32>(floor((sdf.camera.xyz - vec3<f32>(half_span)) / brick));
}

fn sdf_level_base(level: u32) -> vec3<i32> {
    return sdf_base_at(sdf_level_brick(level));
}

fn sdf_level_origin(level: u32) -> vec3<f32> {
    return vec3<f32>(sdf_level_base(level)) * sdf_level_brick(level);
}

fn sdf_level_span(level: u32) -> f32 {
    return sdf_level_brick(level) * f32(sdf.counts.z);
}

fn sdf_brick_at(brick: f32, point: vec3<f32>) -> vec3<i32> {
    return vec3<i32>(floor(point / brick));
}

fn sdf_brick_of(level: u32, point: vec3<f32>) -> vec3<i32> {
    return sdf_brick_at(sdf_level_brick(level), point);
}

fn sdf_wrap(value: i32, side: i32) -> i32 {
    return value & (side - 1);
}

fn sdf_pointer_index(level: u32, brick: vec3<i32>) -> u32 {
    let side = i32(sdf.counts.z);
    let stride = u32(side);
    let wrapped = vec3<u32>(
        u32(sdf_wrap(brick.x, side)),
        u32(sdf_wrap(brick.y, side)),
        u32(sdf_wrap(brick.z, side))
    );
    return level * stride * stride * stride
        + (wrapped.z * stride + wrapped.y) * stride + wrapped.x;
}

fn sdf_brick_from_index(level: u32, index: u32) -> vec3<i32> {
    let side = i32(sdf.counts.z);
    let within = i32(index) - i32(level) * side * side * side;
    let wrapped = vec3<i32>(within % side, (within / side) % side, within / (side * side));
    let base = sdf_level_base(level);
    let turned = vec3<i32>(
        sdf_wrap(wrapped.x - sdf_wrap(base.x, side), side),
        sdf_wrap(wrapped.y - sdf_wrap(base.y, side), side),
        sdf_wrap(wrapped.z - sdf_wrap(base.z, side), side)
    );
    return base + turned;
}

fn sdf_atlas_bricks() -> u32 {
    return sdf.flags.y;
}

fn sdf_atlas_texels() -> f32 {
    return f32(sdf.flags.y * SDF_BRICK_SAMPLES);
}

fn sdf_pack_distance(normalized: f32) -> vec2<f32> {
    let whole = u32(round((clamp(normalized, -1.0, 1.0) * 0.5 + 0.5) * 65535.0));
    return vec2<f32>(f32(whole >> 8u), f32(whole & 0xffu)) / 255.0;
}

fn sdf_stored_distance(stored: vec4<f32>) -> f32 {
    return (stored.r * 65280.0 + stored.a * 255.0) / 65535.0 * 2.0 - 1.0;
}

fn sdf_atlas_origin(slot: u32) -> vec3<f32> {
    let bricks = sdf_atlas_bricks();
    let plane = bricks * bricks;
    let along = vec3<u32>(
        slot % bricks,
        (slot / bricks) % bricks,
        slot / plane
    );
    return vec3<f32>(along) * f32(SDF_BRICK_SAMPLES);
}

fn sdf_brick_touched(low: vec3<f32>, high: vec3<f32>, ball: vec4<f32>) -> bool {
    let nearest = clamp(ball.xyz, low, high);
    let apart = nearest - ball.xyz;
    return dot(apart, apart) <= ball.w * ball.w;
}

fn sdf_pack_brick(level: u32, brick: vec3<i32>) -> vec2<u32> {
    let low = u32(brick.x + SDF_BRICK_BIAS) | (u32(brick.y + SDF_BRICK_BIAS) << 16u);
    let high = u32(brick.z + SDF_BRICK_BIAS) | (level << 16u);
    return vec2<u32>(low, high);
}

fn sdf_unpack_level(packed: vec2<u32>) -> u32 {
    return packed.y >> 16u;
}

fn sdf_unpack_brick(packed: vec2<u32>) -> vec3<i32> {
    return vec3<i32>(
        i32(packed.x & 0xffffu) - SDF_BRICK_BIAS,
        i32((packed.x >> 16u) & 0xffffu) - SDF_BRICK_BIAS,
        i32(packed.y & 0xffffu) - SDF_BRICK_BIAS
    );
}

fn sdf_pointer_is_brick(pointer: u32) -> bool {
    return pointer >= SDF_FIRST_BRICK && (pointer & SDF_EMPTY_TAG) == 0u;
}

fn sdf_pointer_is_empty(pointer: u32) -> bool {
    return (pointer & SDF_EMPTY_TAG) != 0u;
}

fn sdf_pointer_is_inside(pointer: u32) -> bool {
    return (pointer & SDF_INSIDE_TAG) != 0u;
}

fn sdf_empty_pointer(middle: f32, cell: f32) -> u32 {
    let units = min(abs(middle) / cell * SDF_EMPTY_STEPS_PER_CELL, f32(SDF_EMPTY_UNITS));
    return SDF_EMPTY_TAG | select(0u, SDF_INSIDE_TAG, middle < 0.0) | u32(units);
}

fn sdf_empty_reach(pointer: u32, cell: f32) -> f32 {
    return f32(pointer & SDF_EMPTY_UNITS) * cell / SDF_EMPTY_STEPS_PER_CELL;
}

fn sdf_level_reach(level: u32) -> f32 {
    let brick = sdf_level_brick(level);
    return brick * (f32(sdf.counts.z) * 0.5 - 1.0);
}

fn sdf_covered_by_finer(level: u32, low: vec3<f32>, high: vec3<f32>) -> bool {
    if level == 0u {
        return false;
    }
    let camera = sdf.camera.xyz;
    let farthest = max(abs(low - camera), abs(high - camera));
    return length(farthest) <= sdf_level_reach(level - 1u) * SDF_BLEND_START;
}

fn sdf_never_read(level: u32, low: vec3<f32>, high: vec3<f32>) -> bool {
    let camera = sdf.camera.xyz;
    let nearest = clamp(camera, low, high);
    return distance(nearest, camera) > sdf_level_reach(level);
}

fn sdf_doubt() -> f32 {
    return max(sdf.terrain.x, 1.0);
}
