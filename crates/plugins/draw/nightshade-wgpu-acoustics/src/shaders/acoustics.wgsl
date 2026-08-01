struct Settings {
    listener: vec4<f32>,
    origin: vec4<f32>,
    counts: vec4<u32>,
    air: vec4<f32>,
    zones: vec4<u32>,
    spread: vec4<f32>,
    bounces: vec4<f32>,
    caps: vec4<f32>,
    slicing: vec4<u32>,
};

@group(0) @binding(0) var<uniform> settings: Settings;
@group(0) @binding(1) var<storage, read> emitters: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> cells: array<u32>;
@group(0) @binding(3) var<storage, read_write> tally: array<atomic<u32>>;
@group(0) @binding(4) var<storage, read_write> echoes: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read_write> paths: array<vec4<f32>>;
@group(0) @binding(6) var<storage, read> materials: array<vec4<f32>>;

const RETURNS: u32 = 0u;
const RETURN_SPAN: u32 = 1u;
const OUTDOORS: u32 = 2u;
const OPENING_X: u32 = 3u;
const SURFACE_SUM: u32 = 6u;
const SURFACE_COUNT: u32 = 7u;
const MAX_ECHOES: u32 = 1024u;

const MAX_PATHS: u32 = 640u;
const PATH_STRIDE: u32 = 3u;
const PATH_RAYS: u32 = 24u;
const STEP_CLEAR: f32 = 0.0;
const STEP_BLOCKED: f32 = 1.0;
const STEP_BOUNCE: f32 = 2.0;
const STEP_HOME: f32 = 3.0;
const STEP_FOUND: f32 = 5.0;
const CHAIN_MAX: u32 = 9u;
const SCATTER_SUM: u32 = 8u;
const AMBIENT_LOW: u32 = 9u;
const AMBIENT_HIGH: u32 = 10u;
const AMBIENT_THROUGH_LOW: u32 = 81u;
const AMBIENT_THROUGH_HIGH: u32 = 82u;
const BUCKET_LOW: u32 = 11u;
const BUCKET_HIGH: u32 = 43u;
const EARLY_DIRECTION: u32 = 75u;
const LATE_DIRECTION: u32 = 78u;
const ECHO_BUCKETS: u32 = 32u;
const EARLY_SECONDS: f32 = 0.08;
const ENERGY_SCALE: f32 = 100000.0;
const ROOM_BLOCK: u32 = 83u;
const MAX_ZONES: u32 = 4u;
const ECHO_COUNT: u32 = 415u;
const PATH_COUNT: u32 = 416u;
const EMITTER_BASE: u32 = 417u;
const EMITTER_STRIDE: u32 = 8u;
const CENTIMETRES: f32 = 100.0;
const LONGEST: f32 = 600.0;
const GOLDEN_TURN: f32 = 2.399963229728653;

struct Walk {
    hit: bool,
    left: bool,
    met: bool,
    travel: f32,
    soak_low: f32,
    soak_high: f32,
    surface: u32,
    normal: vec3<f32>,
    place: vec3<f32>,
};

struct Surface {
    absorb_low: f32,
    absorb_high: f32,
    crossing_low: f32,
    crossing_high: f32,
    scatter: f32,
    flat_low: f32,
    flat_high: f32,
    solid: f32,
};

fn surface_of(code: u32) -> Surface {
    let base = min(code, 7u) * 2u;
    let first = materials[base];
    let second = materials[base + 1u];
    var result: Surface;
    result.absorb_low = first.x;
    result.absorb_high = first.y;
    result.crossing_low = max(first.z, 1.0e-3);
    result.crossing_high = max(first.w, 1.0e-3);
    result.scatter = second.x;
    result.flat_low = second.y;
    result.flat_high = second.z;
    result.solid = second.w;
    return result;
}

fn fixed_energy(energy: f32) -> u32 {
    return u32(clamp(energy, 0.0, 1.0) * ENERGY_SCALE);
}

fn epsilon() -> f32 {
    return bitcast<f32>(settings.zones.w);
}

fn inside_grid(cell: vec3<i32>) -> bool {
    let side = i32(settings.counts.x);
    return cell.x >= 0 && cell.y >= 0 && cell.z >= 0 && cell.x < side && cell.y < side && cell.z < side;
}

fn material_at(cell: vec3<i32>) -> u32 {
    if (!inside_grid(cell)) {
        return 0u;
    }
    let side = i32(settings.counts.x);
    let index = u32((cell.z * side + cell.y) * side + cell.x);
    let word = cells[index >> 2u];
    return (word >> ((index & 3u) * 8u)) & 255u;
}

fn plane_distance(position: f32, direction: f32, cell: i32) -> f32 {
    if (direction > 1.0e-6) {
        return (f32(cell) + 1.0 - position) / direction;
    }
    if (direction < -1.0e-6) {
        return (position - f32(cell)) / -direction;
    }
    return 1.0e30;
}

fn axis_delta(direction: f32) -> f32 {
    if (abs(direction) < 1.0e-6) {
        return 1.0e30;
    }
    return 1.0 / abs(direction);
}

fn walk(origin: vec3<f32>, direction: vec3<f32>, limit: f32, stop_at_solid: bool) -> Walk {
    var result: Walk;
    result.hit = false;
    result.left = false;
    result.met = false;
    result.travel = limit;
    result.soak_low = 1.0;
    result.soak_high = 1.0;
    result.surface = 0u;
    result.normal = vec3<f32>(0.0, 1.0, 0.0);
    result.place = origin + direction * limit;

    let cell_size = settings.origin.w;
    let local = (origin - settings.origin.xyz) / cell_size;
    var cell = vec3<i32>(floor(local));
    let step = vec3<i32>(sign(direction));

    var next = vec3<f32>(
        plane_distance(local.x, direction.x, cell.x),
        plane_distance(local.y, direction.y, cell.y),
        plane_distance(local.z, direction.z, cell.z),
    );
    let delta = vec3<f32>(
        axis_delta(direction.x),
        axis_delta(direction.y),
        axis_delta(direction.z),
    );

    let cell_limit = limit / cell_size;
    var travelled = 0.0;
    var normal = vec3<f32>(0.0, 1.0, 0.0);
    var soak_low = 1.0;
    var soak_high = 1.0;

    let cap = settings.counts.x * 3u + 4u;
    for (var guard = 0u; guard < cap; guard = guard + 1u) {
        if (travelled >= cell_limit) {
            break;
        }
        if (!inside_grid(cell)) {
            result.left = true;
            result.travel = travelled * cell_size;
            result.place = origin + direction * result.travel;
            result.normal = normal;
            result.soak_low = soak_low;
            result.soak_high = soak_high;
            return result;
        }

        let material = material_at(cell);
        let inside = surface_of(material);
        if (material != 0u && stop_at_solid && inside.solid > 0.5) {
            result.hit = true;
            result.met = true;
            result.surface = material;
            result.travel = travelled * cell_size;
            result.place = origin + direction * result.travel;
            result.normal = normal;
            result.soak_low = soak_low;
            result.soak_high = soak_high;
            return result;
        }

        var advance = next.z;
        var face = vec3<f32>(0.0, 0.0, -f32(step.z));
        if (next.x < next.y && next.x < next.z) {
            advance = next.x;
            face = vec3<f32>(-f32(step.x), 0.0, 0.0);
            next.x = next.x + delta.x;
            cell.x = cell.x + step.x;
        } else if (next.y < next.z) {
            advance = next.y;
            face = vec3<f32>(0.0, -f32(step.y), 0.0);
            next.y = next.y + delta.y;
            cell.y = cell.y + step.y;
        } else {
            next.z = next.z + delta.z;
            cell.z = cell.z + step.z;
        }

        let crossed = min(advance, cell_limit) - travelled;
        if (material != 0u) {
            result.met = true;
            if (inside.solid > 0.5) {
                let metres = crossed * cell_size * settings.air.w;
                soak_low = soak_low * max(0.0, 1.0 - metres / inside.crossing_low);
                soak_high = soak_high * max(0.0, 1.0 - metres / inside.crossing_high);
            } else {
                soak_low = soak_low * max(0.0, 1.0 - inside.flat_low);
                soak_high = soak_high * max(0.0, 1.0 - inside.flat_high);
            }
        }
        travelled = advance;
        normal = face;
    }

    result.soak_low = soak_low;
    result.soak_high = soak_high;
    return result;
}

fn hash_unit(seed: u32) -> f32 {
    var value = seed;
    value = value ^ (value >> 17u);
    value = value * 2246822519u;
    value = value ^ (value >> 11u);
    value = value * 3266489917u;
    value = value ^ (value >> 15u);
    return f32(value & 16777215u) / 16777215.0;
}

fn scatter(ray: u32, bounce: u32, sown: u32) -> vec3<f32> {
    let seed = ray * 747796405u + bounce * 2891336453u + sown * 1013904223u;
    let x = hash_unit(seed);
    let y = hash_unit(seed ^ 1013904223u);
    let z = hash_unit(seed ^ 2654435761u);
    return vec3<f32>(x, y, z) * 2.0 - vec3<f32>(1.0);
}

fn sphere_direction(index: u32, count: u32) -> vec3<f32> {
    let total = max(f32(count), 1.0);
    let height = (f32(index) + 0.5) * (2.0 / total) - 1.0;
    let radius = sqrt(max(0.0, 1.0 - height * height));
    let angle = f32(index) * GOLDEN_TURN;
    return vec3<f32>(cos(angle) * radius, height, sin(angle) * radius);
}

fn fixed_point(metres: f32) -> u32 {
    return u32(clamp(metres, 0.0, LONGEST) * CENTIMETRES);
}

fn record(kind: f32, start: vec3<f32>, end: vec3<f32>, from_travel: f32, to_travel: f32) {
    let slot = atomicAdd(&tally[PATH_COUNT], 1u);
    if (slot >= MAX_PATHS) {
        return;
    }
    let base = slot * PATH_STRIDE;
    paths[base] = vec4<f32>(start, from_travel);
    paths[base + 1u] = vec4<f32>(end, to_travel);
    paths[base + 2u] = vec4<f32>(kind, 0.0, 0.0, 0.0);
}

fn echo_spokes() -> u32 {
    return clamp(MAX_ECHOES / max(settings.counts.w, 1u), 1u, 256u);
}

fn emitter_at(index: u32) -> vec4<f32> {
    return emitters[index * 4u];
}

fn emitter_budget(index: u32) -> vec4<f32> {
    return emitters[index * 4u + 1u];
}

fn emitter_counts(index: u32) -> vec4<f32> {
    return emitters[index * 4u + 2u];
}

fn emitter_bounces(index: u32) -> vec4<f32> {
    return emitters[index * 4u + 3u];
}

fn echo_probe(index: u32) {
    let count = max(settings.counts.w, 1u);
    let emitter = emitter_at(index % count);
    let direction = sphere_direction(index / count, echo_spokes());
    let step = walk(emitter.xyz, direction, emitter.w, true);
    if (!step.hit) {
        return;
    }
    let slot = atomicAdd(&tally[ECHO_COUNT], 1u);
    if (slot >= MAX_ECHOES) {
        return;
    }
    let strength = clamp(1.0 - step.travel / max(emitter.w, 1.0e-3), 0.0, 1.0);
    let base = slot * 2u;
    echoes[base] = vec4<f32>(step.place + step.normal * epsilon() * 3.0, strength);
    echoes[base + 1u] = vec4<f32>(f32(index % count), 0.0, 0.0, 0.0);
}

fn probe_direct(index: u32, listener: vec3<f32>) {
    let emitter = emitter_at(index);
    let towards = emitter.xyz - listener;
    let span = length(towards);
    let slot = EMITTER_BASE + index * EMITTER_STRIDE;
    if (span < 1.0e-4) {
        atomicAdd(&tally[slot], 1u);
        return;
    }
    let through = walk(listener, towards / span, span, false);
    if (!through.met) {
        atomicAdd(&tally[slot], 1u);
    }
    record(
        select(STEP_BLOCKED, STEP_CLEAR, !through.met),
        emitter.xyz,
        listener,
        0.0,
        span,
    );
}

fn reaches(start: vec3<f32>, towards: vec3<f32>, span: f32) -> bool {
    let clear = walk(start, towards / span, span, true);
    return !clear.hit && !clear.left;
}

struct Bounce {
    gone: bool,
    left: bool,
    energy_low: f32,
    energy_high: f32,
    travelled: f32,
    at: vec3<f32>,
    direction: vec3<f32>,
    skew: f32,
};

fn take_bounce(
    room: u32,
    at: vec3<f32>,
    direction: vec3<f32>,
    remaining: f32,
    ray: u32,
    bounce: u32,
    sown: u32,
    skew: f32,
    counted: bool,
) -> Bounce {
    var result: Bounce;
    result.gone = false;
    result.left = false;
    result.energy_low = 1.0;
    result.energy_high = 1.0;
    result.skew = skew;
    result.direction = direction;

    let step = walk(at, direction, remaining, true);
    result.travelled = step.travel;
    result.at = step.place;

    if (step.left && settings.zones.z != 0u) {
        result.direction = normalize(reflect(direction, -step.normal) + scatter(ray, bounce, sown) * skew);
        result.at = step.place - result.direction * epsilon();
        return result;
    }
    if (step.left) {
        result.left = true;
        return result;
    }
    if (!step.hit) {
        result.gone = true;
        return result;
    }

    let face = surface_of(step.surface);
    result.energy_low = 1.0 - face.absorb_low;
    result.energy_high = 1.0 - face.absorb_high;
    result.skew = face.scatter;
    result.at = step.place + step.normal * epsilon();
    result.direction = normalize(reflect(direction, step.normal) + scatter(ray, bounce, sown) * face.scatter);
    if (counted) {
        atomicAdd(&tally[room + SURFACE_SUM], fixed_energy(face.absorb_high));
        atomicAdd(&tally[room + SCATTER_SUM], fixed_energy(face.scatter));
        atomicAdd(&tally[room + SURFACE_COUNT], 1u);
    }
    return result;
}

fn trace_reverb(ray: u32, rays: u32, origin: vec3<f32>, reach: f32, block: u32, bounces: u32, seconds: f32, sown: u32) {
    let room = block * ROOM_BLOCK;
    var direction = sphere_direction(ray, rays);
    var at = origin;
    var travelled = 0.0;
    var energy_low = 1.0;
    var energy_high = 1.0;
    var skew = 0.35;

    for (var bounce = 0u; bounce < bounces; bounce = bounce + 1u) {
        let remaining = reach - travelled;
        if (remaining <= 0.0) {
            return;
        }
        let step = take_bounce(room, at, direction, remaining, ray, bounce, sown, skew, true);
        travelled = travelled + step.travelled;
        if (step.gone || step.left) {
            return;
        }
        at = step.at;
        direction = step.direction;
        skew = step.skew;
        energy_low = energy_low * step.energy_low;
        energy_high = energy_high * step.energy_high;

        let back = origin - at;
        let back_span = length(back);
        if (back_span > 1.0e-3 && reaches(at, back, back_span)) {
            let home = travelled + back_span;
            atomicAdd(&tally[room + RETURNS], 1u);
            atomicAdd(&tally[room + RETURN_SPAN], fixed_point(home));
            let arrived = home * settings.air.w / settings.air.y;
            let bucket = u32(arrived / seconds);
            if (bucket < ECHO_BUCKETS) {
                atomicAdd(&tally[room + BUCKET_LOW + bucket], fixed_energy(energy_low));
                atomicAdd(&tally[room + BUCKET_HIGH + bucket], fixed_energy(energy_high));
            }
            let arrive = normalize(-back) * energy_low * 1000.0;
            var side = LATE_DIRECTION;
            if (arrived < EARLY_SECONDS) {
                side = EARLY_DIRECTION;
            }
            atomicAdd(&tally[room + side], bitcast<u32>(i32(arrive.x)));
            atomicAdd(&tally[room + side + 1u], bitcast<u32>(i32(arrive.y)));
            atomicAdd(&tally[room + side + 2u], bitcast<u32>(i32(arrive.z)));
            return;
        }
    }
}

fn trace_occlusion(ray: u32, rays: u32, origin: vec3<f32>, reach: f32, block: u32, bounces: u32, sown: u32) {
    let room = block * ROOM_BLOCK;
    var direction = sphere_direction(ray, rays);
    var at = origin;
    var travelled = 0.0;
    var energy_low = 1.0;
    var energy_high = 1.0;
    var skew = 0.35;
    let count = settings.counts.w;
    let drawn = min(u32(settings.caps.z), PATH_RAYS);
    let stride = max(rays / max(drawn, 1u), 1u);
    var tracked = block == 0u && drawn > 0u && (ray % stride) == 0u && (ray / stride) < drawn;
    var found = 0u;

    var chain_place: array<vec3<f32>, 9>;
    var chain_travel: array<f32, 9>;
    var links = 1u;
    chain_place[0] = origin;
    chain_travel[0] = 0.0;

    for (var bounce = 0u; bounce < bounces; bounce = bounce + 1u) {
        let remaining = reach - travelled;
        if (remaining <= 0.0) {
            return;
        }
        let step = take_bounce(room, at, direction, remaining, ray, bounce, sown, skew, false);
        travelled = travelled + step.travelled;
        if (step.gone || step.left) {
            return;
        }
        at = step.at;
        direction = step.direction;
        skew = step.skew;
        energy_low = energy_low * step.energy_low;
        energy_high = energy_high * step.energy_high;

        if (tracked) {
            if (links < CHAIN_MAX) {
                chain_place[links] = at;
                chain_travel[links] = travelled;
                links = links + 1u;
            } else {
                tracked = false;
            }
        }

        for (var index = 0u; index < count; index = index + 1u) {
            if (block != 0u) {
                break;
            }
            if ((found & (1u << index)) != 0u) {
                continue;
            }
            let emitter = emitter_at(index);
            let towards = emitter.xyz - at;
            let span = length(towards);
            if (span > emitter.w || span < 1.0e-3) {
                continue;
            }
            if (!reaches(at, towards, span)) {
                continue;
            }
            let slot = EMITTER_BASE + index * EMITTER_STRIDE;
            let reached = travelled + span;
            let air_high = pow(settings.air.x, reached * settings.air.w);
            atomicAdd(&tally[slot + 3u], 1u);
            atomicAdd(&tally[slot + 4u], fixed_energy(energy_low));
            atomicAdd(&tally[slot + 5u], fixed_energy(energy_high * air_high));
            atomicAdd(&tally[slot + 6u], fixed_point(reached));
            found = found | (1u << index);
            if (tracked) {
                record(STEP_FOUND, emitter.xyz, at, 0.0, span);
                for (var link = links - 1u; link >= 1u; link = link - 1u) {
                    record(
                        select(STEP_BOUNCE, STEP_HOME, link == 1u),
                        chain_place[link],
                        chain_place[link - 1u],
                        span + travelled - chain_travel[link],
                        span + travelled - chain_travel[link - 1u],
                    );
                }
                tracked = false;
            }
        }
        if (count > 0u && found == (1u << count) - 1u) {
            return;
        }
    }
}

fn trace_permeation(ray: u32, rays: u32, origin: vec3<f32>, reach: f32, block: u32, bounces: u32, sown: u32) {
    var direction = sphere_direction(ray, rays);
    var at = origin;
    var travelled = 0.0;
    var energy_low = 1.0;
    var energy_high = 1.0;
    let count = settings.counts.w;

    for (var bounce = 0u; bounce < bounces; bounce = bounce + 1u) {
        let remaining = reach - travelled;
        if (remaining <= 0.0) {
            return;
        }
        let through = walk(at, direction, remaining, false);
        travelled = travelled + through.travel;
        energy_low = energy_low * through.soak_low;
        energy_high = energy_high * through.soak_high;
        at = through.place;
        if (energy_low < 1.0e-4) {
            return;
        }

        for (var index = 0u; index < count; index = index + 1u) {
            if (block != 0u) {
                break;
            }
            let emitter = emitter_at(index);
            let towards = emitter.xyz - at;
            let span = length(towards);
            if (span > emitter.w || span < 1.0e-3) {
                continue;
            }
            let last = walk(at, towards / span, span, false);
            let slot = EMITTER_BASE + index * EMITTER_STRIDE;
            let reached = travelled + span;
            let air_high = pow(settings.air.x, reached * settings.air.w);
            atomicAdd(&tally[slot + 1u], fixed_energy(energy_low * last.soak_low));
            atomicAdd(&tally[slot + 2u], fixed_energy(energy_high * last.soak_high * air_high));
        }

        if (through.left) {
            return;
        }
        direction = normalize(direction + scatter(ray, bounce, sown) * 0.2);
    }
}

fn trace_ambient_occlusion(ray: u32, rays: u32, origin: vec3<f32>, reach: f32, block: u32, bounces: u32, sown: u32) {
    let room = block * ROOM_BLOCK;
    var direction = sphere_direction(ray, rays);
    var at = origin;
    var travelled = 0.0;
    var energy_low = 1.0;
    var energy_high = 1.0;
    var skew = 0.35;

    for (var bounce = 0u; bounce < bounces; bounce = bounce + 1u) {
        let remaining = reach - travelled;
        if (remaining <= 0.0) {
            return;
        }
        let step = take_bounce(room, at, direction, remaining, ray, bounce, sown, skew, false);
        travelled = travelled + step.travelled;
        if (step.left) {
            atomicAdd(&tally[room + OUTDOORS], 1u);
            atomicAdd(&tally[room + AMBIENT_LOW], fixed_energy(energy_low));
            atomicAdd(&tally[room + AMBIENT_HIGH], fixed_energy(energy_high));
            atomicAdd(&tally[room + OPENING_X], bitcast<u32>(i32(direction.x * 1000.0)));
            atomicAdd(&tally[room + OPENING_X + 1u], bitcast<u32>(i32(direction.y * 1000.0)));
            atomicAdd(&tally[room + OPENING_X + 2u], bitcast<u32>(i32(direction.z * 1000.0)));
            return;
        }
        if (step.gone) {
            return;
        }
        at = step.at;
        direction = step.direction;
        skew = step.skew;
        energy_low = energy_low * step.energy_low;
        energy_high = energy_high * step.energy_high;
    }
}

fn trace_ambient_permeation(ray: u32, rays: u32, origin: vec3<f32>, reach: f32, block: u32, bounces: u32, sown: u32) {
    let room = block * ROOM_BLOCK;
    var direction = sphere_direction(ray, rays);
    var at = origin;
    var travelled = 0.0;
    var energy_low = 1.0;
    var energy_high = 1.0;

    for (var bounce = 0u; bounce < bounces; bounce = bounce + 1u) {
        let remaining = reach - travelled;
        if (remaining <= 0.0) {
            return;
        }
        let through = walk(at, direction, remaining, false);
        travelled = travelled + through.travel;
        energy_low = energy_low * through.soak_low;
        energy_high = energy_high * through.soak_high;
        at = through.place;
        if (through.left) {
            atomicAdd(&tally[room + AMBIENT_THROUGH_LOW], fixed_energy(energy_low));
            atomicAdd(&tally[room + AMBIENT_THROUGH_HIGH], fixed_energy(energy_high));
            return;
        }
        if (energy_low < 1.0e-4) {
            return;
        }
        direction = normalize(direction + scatter(ray, bounce, sown) * 0.2);
    }
}

fn walk_block(
    local: u32,
    block: u32,
    origin: vec3<f32>,
    reach: f32,
    counts: vec4<f32>,
    caps: vec4<f32>,
    ambient: vec2<f32>,
    seconds: f32,
    sown: u32,
) {
    let reverb = u32(counts.x);
    let occlusion = u32(counts.y);
    let permeation = u32(counts.z);
    let ambient_occlusion = u32(counts.w);
    let ambient_permeation = u32(ambient.x);

    var start = 0u;
    if (local < reverb) {
        trace_reverb(local, reverb, origin, reach, block, max(u32(caps.x), 1u), seconds, sown);
        return;
    }
    start = start + reverb;
    if (local < start + occlusion) {
        trace_occlusion(local - start, occlusion, origin, reach, block, max(u32(caps.y), 1u), sown);
        return;
    }
    start = start + occlusion;
    if (local < start + permeation) {
        trace_permeation(local - start, permeation, origin, reach, block, max(u32(caps.z), 1u), sown);
        return;
    }
    start = start + permeation;
    if (local < start + ambient_occlusion) {
        trace_ambient_occlusion(local - start, ambient_occlusion, origin, reach, block, max(u32(caps.w), 1u), sown);
        return;
    }
    start = start + ambient_occlusion;
    if (local < start + ambient_permeation) {
        trace_ambient_permeation(local - start, ambient_permeation, origin, reach, block, max(u32(ambient.y), 1u), sown);
    }
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let local = id.x;
    let ray = settings.slicing.x + local;
    let probes = settings.counts.w * echo_spokes();
    let listener = settings.listener.xyz;
    let reach = settings.listener.w;

    if (settings.slicing.x == 0u) {
        if (local < settings.counts.w) {
            probe_direct(local, listener);
        }
        if (local < probes) {
            echo_probe(local);
        }
    }
    if (local >= settings.slicing.y) {
        return;
    }

    let spread = settings.spread;
    let extra = settings.caps;
    let listener_total =
        u32(spread.x) + u32(spread.y) + u32(spread.z) + u32(spread.w) + u32(extra.x);
    if (ray < listener_total) {
        walk_block(
            ray,
            0u,
            listener,
            reach,
            spread,
            settings.bounces,
            vec2<f32>(extra.x, extra.y),
            0.05,
            0u,
        );
        return;
    }

    let zones = min(settings.zones.x, MAX_ZONES);
    var base = listener_total;
    for (var zone = 0u; zone < zones; zone = zone + 1u) {
        let counts = emitter_counts(zone);
        let caps = emitter_bounces(zone);
        let budget = emitter_budget(zone);
        let total =
            u32(counts.x) + u32(counts.y) + u32(counts.z) + u32(counts.w) + u32(budget.x);
        if (ray < base + total) {
            walk_block(
                ray - base,
                zone + 1u,
                emitter_at(zone).xyz,
                reach,
                counts,
                caps,
                vec2<f32>(budget.x, budget.y),
                max(budget.z, 0.001),
                u32(budget.w),
            );
            return;
        }
        base = base + total;
    }
}
