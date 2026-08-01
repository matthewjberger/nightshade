struct Uniform {
    time: f32,
    aspect: f32,
    progress: f32,
    seed: f32,
    width: f32,
    height: f32,
    steps: f32,
    pad: f32,
};

@group(0) @binding(0)
var<uniform> u: Uniform;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    let x = f32(i32(index & 1u)) * 4.0 - 1.0;
    let y = f32(i32(index >> 1u)) * 4.0 - 1.0;
    var out: VertexOutput;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>(x * 0.5 + 0.5, 1.0 - (y * 0.5 + 0.5));
    return out;
}

const PI: f32 = 3.14159265;
const ISLAND: f32 = 26.0;
const LAKE: f32 = 0.0;
const DROP: f32 = 48.0;
const FLYERS: i32 = 22;
const CORE: vec3<f32> = vec3<f32>(0.0, -13.0, 0.0);

fn mix_bits(value: u32) -> u32 {
    var h = value;
    h = h ^ (h >> 16u);
    h = h * 0x7feb352du;
    h = h ^ (h >> 15u);
    h = h * 0x846ca68bu;
    h = h ^ (h >> 16u);
    return h;
}

fn hash21(p: vec2<f32>) -> f32 {
    let cell = vec2<i32>(floor(p));
    let x = bitcast<u32>(cell.x) * 0x9e3779b1u;
    let y = mix_bits(bitcast<u32>(cell.y) + 0x85ebca77u);
    return f32(mix_bits(x ^ y)) / 4294967295.0;
}

fn hash31(p: vec3<f32>) -> f32 {
    let cell = vec3<i32>(floor(p));
    let x = bitcast<u32>(cell.x) * 0x9e3779b1u;
    let y = mix_bits(bitcast<u32>(cell.y) + 0x85ebca77u);
    let z = mix_bits(bitcast<u32>(cell.z) ^ 0x27d4eb2fu);
    return f32(mix_bits(x ^ y ^ (z * 0x165667b1u))) / 4294967295.0;
}

fn noise2(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let s = f * f * (3.0 - 2.0 * f);
    let a = hash21(i);
    let b = hash21(i + vec2<f32>(1.0, 0.0));
    let c = hash21(i + vec2<f32>(0.0, 1.0));
    let d = hash21(i + vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, s.x), mix(c, d, s.x), s.y);
}

fn fbm(p: vec2<f32>, octaves: i32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var q = p;
    for (var index = 0; index < octaves; index = index + 1) {
        value = value + amplitude * noise2(q);
        q = q * 2.03 + vec2<f32>(17.0, 9.0);
        amplitude = amplitude * 0.5;
    }
    return value;
}

struct Column {
    present: bool,
    top: f32,
    bottom: f32,
    settled: f32,
    tree: bool,
    fresh: f32,
    lift: f32,
};

fn column_at(cell: vec2<f32>) -> Column {
    var column: Column;
    column.present = false;
    column.top = 0.0;
    column.bottom = 0.0;
    column.settled = 0.0;
    column.tree = false;
    column.fresh = 0.0;
    column.lift = 0.0;

    let drift = vec2<f32>(u.seed * 37.0, u.seed * 11.0);
    let middle = cell + vec2<f32>(0.5, 0.5);
    let reach = length(middle);
    let wobble = (fbm(cell * 0.07 + drift, 3) - 0.5) * 14.0;
    let radius = ISLAND + wobble;
    if (reach >= radius) {
        return column;
    }
    let mask = smoothstep(radius, radius * 0.45, reach);
    let relief = fbm(cell * 0.05 + drift + vec2<f32>(5.0, 2.0), 4);
    let ridge = fbm(cell * 0.14 + drift + vec2<f32>(9.0, 4.0), 3);
    var top = floor((relief - 0.36) * 19.0 * mask + (ridge - 0.5) * 3.0 + 3.0 * mask - 1.0);
    let seed_tree = hash21(cell + vec2<f32>(41.0, 13.0));
    if (top >= 1.0 && top < 9.0 && seed_tree > 0.88) {
        column.tree = true;
        top = top + 3.0;
    }
    let depth = fbm(cell * 0.06 + drift + vec2<f32>(21.0, 8.0), 3);
    let bottom = -floor(mask * mask * (5.0 + 22.0 * depth)) - 1.0;

    let birth = clamp((reach / ISLAND) * 0.66 + hash21(cell + vec2<f32>(11.0, 29.0)) * 0.16, 0.0, 0.84);
    let age = u.progress - birth;
    if (age <= 0.0) {
        return column;
    }
    let rise = smoothstep(0.0, 0.14, age);
    let bounce = sin(rise * PI) * (1.0 - rise) * 2.2;
    let lift = (1.0 - rise) * DROP - bounce;

    column.present = true;
    column.settled = top;
    column.top = top + lift;
    column.bottom = bottom + lift;
    column.lift = lift;
    column.fresh = smoothstep(0.0, 0.05, age) * (1.0 - smoothstep(0.14, 0.40, age));
    return column;
}

struct Hit {
    found: bool,
    water: bool,
    distance: f32,
    normal: vec3<f32>,
    column: Column,
    cell: vec2<f32>,
};

fn march(origin: vec3<f32>, direction: vec3<f32>, steps: i32, far: f32) -> Hit {
    var hit: Hit;
    hit.found = false;
    hit.water = false;
    hit.distance = far;
    hit.normal = vec3<f32>(0.0, 1.0, 0.0);
    hit.cell = vec2<f32>(0.0, 0.0);

    let safe = vec2<f32>(
        select(direction.x, 1e-5, abs(direction.x) < 1e-5),
        select(direction.z, 1e-5, abs(direction.z) < 1e-5)
    );
    var cell = floor(origin.xz);
    let step = sign(safe);
    let delta = abs(1.0 / safe);
    var t_max = vec2<f32>(
        (cell.x + max(step.x, 0.0) - origin.x) / safe.x,
        (cell.y + max(step.y, 0.0) - origin.z) / safe.y
    );
    var t = 0.0;
    var last_axis = -1;
    for (var index = 0; index < steps; index = index + 1) {
        if (t > far) {
            return hit;
        }
        let t_next = min(t_max.x, t_max.y);
        let inside = length(cell + vec2<f32>(0.5, 0.5)) < ISLAND + 8.0;
        if (inside) {
            let column = column_at(cell);
            if (column.present) {
                let y_enter = origin.y + direction.y * t;
                let y_exit = origin.y + direction.y * t_next;
                if (y_enter <= column.top && y_enter >= column.bottom) {
                    hit.found = true;
                    hit.distance = t;
                    hit.column = column;
                    hit.cell = cell;
                    if (last_axis == 0) {
                        hit.normal = vec3<f32>(-step.x, 0.0, 0.0);
                    } else if (last_axis == 1) {
                        hit.normal = vec3<f32>(0.0, 0.0, -step.y);
                    } else {
                        hit.normal = vec3<f32>(0.0, 1.0, 0.0);
                    }
                    return hit;
                }
                let lake = LAKE + column.lift;
                if (column.top < lake && y_enter > lake && y_exit <= lake) {
                    hit.found = true;
                    hit.water = true;
                    hit.distance = (lake - origin.y) / direction.y;
                    hit.column = column;
                    hit.cell = cell;
                    hit.normal = vec3<f32>(0.0, 1.0, 0.0);
                    return hit;
                }
                if (y_enter > column.top && y_exit <= column.top) {
                    hit.found = true;
                    hit.distance = (column.top - origin.y) / direction.y;
                    hit.column = column;
                    hit.cell = cell;
                    hit.normal = vec3<f32>(0.0, 1.0, 0.0);
                    return hit;
                }
                if (y_enter < column.bottom && y_exit >= column.bottom) {
                    hit.found = true;
                    hit.distance = (column.bottom - origin.y) / direction.y;
                    hit.column = column;
                    hit.cell = cell;
                    hit.normal = vec3<f32>(0.0, -1.0, 0.0);
                    return hit;
                }
            }
        }
        if (t_max.x < t_max.y) {
            t = t_max.x;
            t_max.x = t_max.x + delta.x;
            cell.x = cell.x + step.x;
            last_axis = 0;
        } else {
            t = t_max.y;
            t_max.y = t_max.y + delta.y;
            cell.y = cell.y + step.y;
            last_axis = 1;
        }
    }
    return hit;
}

struct BoxHit {
    found: bool,
    distance: f32,
    normal: vec3<f32>,
};

fn box_hit(origin: vec3<f32>, direction: vec3<f32>, center: vec3<f32>, half: f32) -> BoxHit {
    var hit: BoxHit;
    hit.found = false;
    hit.distance = 1e9;
    hit.normal = vec3<f32>(0.0, 1.0, 0.0);
    let inverse = 1.0 / vec3<f32>(
        select(direction.x, 1e-5, abs(direction.x) < 1e-5),
        select(direction.y, 1e-5, abs(direction.y) < 1e-5),
        select(direction.z, 1e-5, abs(direction.z) < 1e-5)
    );
    let low = (center - vec3<f32>(half, half, half) - origin) * inverse;
    let high = (center + vec3<f32>(half, half, half) - origin) * inverse;
    let near = min(low, high);
    let far = max(low, high);
    let enter = max(max(near.x, near.y), near.z);
    let exit = min(min(far.x, far.y), far.z);
    if (exit < max(enter, 0.0)) {
        return hit;
    }
    hit.found = true;
    hit.distance = enter;
    if (enter == near.x) {
        hit.normal = vec3<f32>(-sign(direction.x), 0.0, 0.0);
    } else if (enter == near.y) {
        hit.normal = vec3<f32>(0.0, -sign(direction.y), 0.0);
    } else {
        hit.normal = vec3<f32>(0.0, 0.0, -sign(direction.z));
    }
    return hit;
}

fn nebula(direction: vec3<f32>) -> vec3<f32> {
    let flat = direction.xz / (abs(direction.y) + 0.35);
    let drift = vec2<f32>(u.seed * 23.0, u.seed * 5.0) + vec2<f32>(u.time * 0.004, 0.0);
    let veil = fbm(flat * 0.9 + drift, 5);
    let wisp = fbm(flat * 2.1 - drift * 3.0 + vec2<f32>(7.0, 3.0), 4);
    let deep = vec3<f32>(0.015, 0.012, 0.045);
    let violet = vec3<f32>(0.34, 0.10, 0.52);
    let teal = vec3<f32>(0.04, 0.36, 0.46);
    let rose = vec3<f32>(0.85, 0.35, 0.55);
    var color = deep;
    color = mix(color, violet, smoothstep(0.42, 0.80, veil) * 0.8);
    color = mix(color, teal, smoothstep(0.50, 0.85, wisp) * 0.7);
    color = color + rose * pow(smoothstep(0.62, 0.95, veil * wisp * 2.2), 2.0) * 0.6;
    return color;
}

fn stars(direction: vec3<f32>) -> vec3<f32> {
    let flat = direction.xz / (abs(direction.y) + 0.35);
    let scale = 90.0;
    let cell = floor(flat * scale);
    let spark = hash21(cell + vec2<f32>(u.seed * 100.0, 0.0));
    if (spark < 0.985) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }
    let inside = length(fract(flat * scale) - 0.5);
    let twinkle = 0.65 + 0.35 * sin(u.time * (1.0 + spark * 4.0) + spark * 60.0);
    let bright = (spark - 0.985) / 0.015;
    let tint = mix(vec3<f32>(0.75, 0.85, 1.0), vec3<f32>(1.0, 0.9, 0.8), hash21(cell + vec2<f32>(3.0, 9.0)));
    return tint * (1.0 - smoothstep(0.0, 0.28, inside)) * bright * twinkle * 1.6;
}

fn aurora(direction: vec3<f32>) -> vec3<f32> {
    if (direction.y < 0.02) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }
    let angle = atan2(direction.x, direction.z);
    var glow = vec3<f32>(0.0, 0.0, 0.0);
    for (var band = 0; band < 3; band = band + 1) {
        let fb = f32(band);
        let ripple = noise2(vec2<f32>(angle * 2.5 + u.time * 0.08 + fb * 3.0, u.time * 0.05 + fb));
        let height = 0.16 + fb * 0.09 + ripple * 0.12;
        let width = 0.045 + fb * 0.015;
        let band_shape = exp(-pow((direction.y - height) / width, 2.0));
        let curtain = smoothstep(0.25, 0.9, fbm(vec2<f32>(angle * 6.0 + fb * 11.0 + u.time * 0.12, direction.y * 12.0), 3));
        let tint = mix(vec3<f32>(0.15, 0.95, 0.55), vec3<f32>(0.55, 0.25, 0.95), fb / 2.0);
        glow = glow + tint * band_shape * curtain * 0.55;
    }
    return glow * smoothstep(0.02, 0.15, direction.y);
}

fn sky(direction: vec3<f32>, sun: vec3<f32>) -> vec3<f32> {
    var color = nebula(direction);
    color = color + stars(direction);
    color = color + aurora(direction);
    let toward = max(dot(direction, sun), 0.0);
    color = color + vec3<f32>(1.0, 0.82, 0.55) * pow(toward, 14.0) * 0.35;
    color = color + vec3<f32>(1.0, 0.95, 0.85) * smoothstep(0.9990, 0.9996, toward) * 2.5;
    return color;
}

fn top_color(column: Column, cell: vec2<f32>) -> vec3<f32> {
    let tint = 0.90 + 0.20 * hash21(cell + vec2<f32>(5.0, 3.0));
    let height = column.settled;
    if (column.tree) {
        return vec3<f32>(0.14, 0.42, 0.20) * tint;
    }
    var color = vec3<f32>(0.80, 0.72, 0.50);
    if (height >= 1.0) {
        color = mix(vec3<f32>(0.30, 0.58, 0.24), vec3<f32>(0.42, 0.62, 0.26), hash21(cell + vec2<f32>(2.0, 8.0)));
    }
    if (height >= 9.0) {
        color = vec3<f32>(0.48, 0.47, 0.50);
    }
    if (height >= 13.0) {
        color = vec3<f32>(0.92, 0.94, 0.97);
    }
    return color * tint;
}

fn side_color(column: Column, cell: vec2<f32>, below_top: f32, above_bottom: f32) -> vec3<f32> {
    let tint = 0.90 + 0.20 * hash21(cell + vec2<f32>(9.0, 1.0));
    if (column.tree) {
        if (below_top < 2.0) {
            return vec3<f32>(0.12, 0.36, 0.17) * tint;
        }
        if (below_top < 3.0) {
            return vec3<f32>(0.36, 0.24, 0.13) * tint;
        }
    }
    if (above_bottom < 3.0) {
        return vec3<f32>(0.22, 0.20, 0.26) * tint;
    }
    if (column.settled >= 1.0 && column.settled < 9.0 && below_top < 1.0 && !column.tree) {
        return top_color(column, cell) * 0.86;
    }
    if (below_top < 4.0 && column.settled < 9.0) {
        return vec3<f32>(0.46, 0.34, 0.21) * tint;
    }
    return vec3<f32>(0.42, 0.41, 0.45) * tint;
}

fn ore_glow(cell: vec2<f32>, y: f32) -> vec3<f32> {
    let roll = hash31(vec3<f32>(cell.x, floor(y), cell.y));
    if (roll < 0.93) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }
    let pulse = 0.7 + 0.3 * sin(u.time * 2.0 + roll * 40.0);
    if (roll > 0.975) {
        return vec3<f32>(1.0, 0.55, 0.15) * pulse;
    }
    return vec3<f32>(0.2, 0.9, 0.85) * pulse;
}

fn occlusion(cell: vec2<f32>, height: f32) -> f32 {
    var taller = 0.0;
    taller = taller + step(height + 0.5, column_at(cell + vec2<f32>(1.0, 0.0)).top);
    taller = taller + step(height + 0.5, column_at(cell - vec2<f32>(1.0, 0.0)).top);
    taller = taller + step(height + 0.5, column_at(cell + vec2<f32>(0.0, 1.0)).top);
    taller = taller + step(height + 0.5, column_at(cell - vec2<f32>(0.0, 1.0)).top);
    return 1.0 - 0.17 * taller;
}

fn core_glow(origin: vec3<f32>, direction: vec3<f32>, limit: f32) -> vec3<f32> {
    let to_core = CORE - origin;
    let along = clamp(dot(to_core, direction), 0.0, limit);
    let nearest = origin + direction * along;
    let gap = length(nearest - CORE);
    let pulse = 0.75 + 0.25 * sin(u.time * 1.7);
    let strength = (0.35 + 1.4 * u.progress) * pulse;
    let tint = mix(vec3<f32>(0.2, 0.9, 0.95), vec3<f32>(1.0, 0.62, 0.25), smoothstep(0.3, 0.9, u.progress));
    return tint * strength * 9.0 / (gap * gap + 3.0);
}

fn grade(color: vec3<f32>) -> vec3<f32> {
    let luma = dot(color, vec3<f32>(0.3, 0.59, 0.11));
    let warm = color * vec3<f32>(1.05, 1.0, 0.94);
    let cool = color * vec3<f32>(0.94, 1.0, 1.10);
    let graded = mix(cool, warm, smoothstep(0.2, 0.9, luma));
    return graded / (graded + 0.72) * 1.55;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let time = u.time;
    let sun = normalize(vec3<f32>(0.55, 0.62, -0.42));

    let orbit = time * 0.11 + u.seed * 6.0;
    let origin = vec3<f32>(cos(orbit) * 52.0, 20.0 + 3.0 * sin(time * 0.23), sin(orbit) * 52.0);
    let aim = vec3<f32>(0.0, 1.5 + 2.0 * sin(time * 0.17), 0.0);
    let ahead = normalize(aim - origin);
    let right = normalize(cross(ahead, vec3<f32>(0.0, 1.0, 0.0)));
    let up = cross(right, ahead);
    let screen = (uv - vec2<f32>(0.5, 0.5)) * vec2<f32>(u.aspect, -1.0);
    let ray = normalize(ahead * 1.5 + right * screen.x + up * screen.y);

    let far = 140.0;
    let hit = march(origin, ray, i32(u.steps), far);
    var color = sky(ray, sun);
    var limit = far;
    if (hit.found) {
        limit = hit.distance;
    }

    var flyer_color = vec3<f32>(0.0, 0.0, 0.0);
    var flyer_hit = false;
    for (var index = 0; index < FLYERS; index = index + 1) {
        let fi = f32(index);
        let h1 = hash21(vec2<f32>(fi, 7.0 + u.seed * 50.0));
        let h2 = hash21(vec2<f32>(fi, 3.0 + u.seed * 50.0));
        let birth = 0.05 + 0.82 * (fi + h1 * 0.6) / f32(FLYERS);
        let window = 0.16;
        let share = (u.progress - (birth - window)) / window;
        if (share <= 0.0 || share >= 1.0) {
            continue;
        }
        let angle = h1 * 2.0 * PI;
        let cell = floor(vec2<f32>(cos(angle), sin(angle)) * (2.0 + h2 * 18.0));
        let target_column = column_at(cell);
        let landing = vec3<f32>(cell.x + 0.5, target_column.settled + 0.5 + target_column.lift, cell.y + 0.5);
        let start = vec3<f32>(cos(angle + 2.4) * 70.0, 30.0 + h2 * 16.0, sin(angle + 2.4) * 70.0);
        let eased = share * share * (3.0 - 2.0 * share);
        let arc = sin(share * PI) * 10.0;
        let center = mix(start, landing, eased) + vec3<f32>(0.0, arc, 0.0);
        let box = box_hit(origin, ray, center, 0.5);
        if (box.found && box.distance < limit) {
            limit = box.distance;
            flyer_hit = true;
            let lambert = max(dot(box.normal, sun), 0.0);
            let base = mix(vec3<f32>(0.46, 0.34, 0.21), vec3<f32>(0.32, 0.6, 0.25), step(0.5, box.normal.y));
            flyer_color = base * (0.35 + 0.9 * lambert) + vec3<f32>(1.0, 0.7, 0.3) * (1.0 - share) * 0.8;
        }
    }

    if (flyer_hit) {
        color = flyer_color;
    } else if (hit.found) {
        let point = origin + ray * hit.distance;
        let column = hit.column;
        if (hit.water) {
            let ripple = noise2(point.xz * 1.8 + vec2<f32>(time * 0.5, time * 0.3))
                + noise2(point.xz * 4.1 - vec2<f32>(time * 0.35, time * 0.6)) * 0.5;
            let wobble = normalize(vec3<f32>(ripple * 0.09 - 0.068, 1.0, ripple * 0.07 - 0.052));
            let reflect_ray = reflect(ray, wobble);
            let fresnel = pow(1.0 - max(dot(-ray, wobble), 0.0), 3.0);
            let depth = clamp(LAKE - column.settled, 0.0, 8.0);
            let body = mix(vec3<f32>(0.10, 0.45, 0.50), vec3<f32>(0.02, 0.10, 0.28), smoothstep(0.0, 5.0, depth));
            let glint = pow(max(dot(reflect_ray, sun), 0.0), 120.0);
            color = mix(body, sky(reflect_ray, sun), 0.3 + 0.6 * fresnel) + vec3<f32>(1.0, 0.95, 0.85) * glint * 1.4;
        } else {
            let local = fract(point.xz);
            let below_top = column.top - point.y;
            let above_bottom = point.y - column.bottom;
            var albedo = top_color(column, hit.cell);
            var emission = vec3<f32>(0.0, 0.0, 0.0);
            if (hit.normal.y < 0.5 && hit.normal.y > -0.5) {
                albedo = side_color(column, hit.cell, below_top, above_bottom);
                emission = ore_glow(hit.cell, point.y - column.lift);
            } else if (hit.normal.y < -0.5) {
                albedo = vec3<f32>(0.24, 0.21, 0.28) * (0.9 + 0.2 * hash21(hit.cell + vec2<f32>(4.0, 4.0)));
            }
            var edge = 1.0;
            if (abs(hit.normal.y) > 0.5) {
                let toward_edge = min(min(local.x, 1.0 - local.x), min(local.y, 1.0 - local.y));
                edge = 0.74 + 0.26 * smoothstep(0.0, 0.06, toward_edge);
            } else {
                let along = select(local.y, local.x, abs(hit.normal.x) > 0.5);
                let seam = fract(point.y);
                let toward_edge = min(min(along, 1.0 - along), min(seam, 1.0 - seam));
                edge = 0.74 + 0.26 * smoothstep(0.0, 0.06, toward_edge);
            }
            let lambert = max(dot(hit.normal, sun), 0.0);
            let shadow = march(point + hit.normal * 0.02 + sun * 0.02, sun, 26, 40.0);
            let lit = select(1.0, 0.0, shadow.found);
            var ao = 1.0;
            if (hit.normal.y > 0.5) {
                ao = occlusion(hit.cell, column.top);
            } else if (hit.normal.y < -0.5) {
                ao = 0.75;
            } else {
                ao = 1.0 - 0.3 * smoothstep(0.0, 5.0, min(below_top, above_bottom));
            }
            let direct = vec3<f32>(1.0, 0.86, 0.68) * lambert * lit * 1.3;
            let rim_from_below = max(dot(hit.normal, vec3<f32>(0.0, -1.0, 0.0)), 0.0);
            let bounce = nebula(hit.normal) * 0.9 + vec3<f32>(0.25, 0.55, 0.65) * rim_from_below * 0.6;
            let core_light = mix(vec3<f32>(0.2, 0.9, 0.95), vec3<f32>(1.0, 0.62, 0.25), smoothstep(0.3, 0.9, u.progress));
            let to_core = normalize(CORE - point);
            let core_lambert = max(dot(hit.normal, to_core), 0.0) * (0.3 + 1.2 * u.progress) * 60.0 / (dot(CORE - point, CORE - point) + 30.0);
            color = albedo * (direct + bounce + core_light * core_lambert) * edge * ao;
            color = color + emission * 1.6;
            let rim = vec3<f32>(1.0, 0.70, 0.30) * column.fresh * (0.6 + (1.0 - edge) * 3.0);
            color = color + rim;
        }
        let fog = 1.0 - exp(-hit.distance * 0.0085);
        color = mix(color, nebula(ray) * 1.1, fog);
    }

    color = color + core_glow(origin, ray, limit);

    color = grade(color);
    let vignette = 1.0 - smoothstep(0.55, 1.35, length((uv - 0.5) * vec2<f32>(1.15, 1.0)));
    color = color * (0.7 + 0.3 * vignette);
    let grain = (hash21(in.position.xy + vec2<f32>(fract(time) * 977.0, 0.0)) - 0.5) * 0.02;
    color = color + vec3<f32>(grain, grain, grain);
    return vec4<f32>(clamp(color, vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(1.0, 1.0, 1.0)), 1.0);
}
