
struct Screen {
    time: f32,
    curvature: f32,
    vignette: f32,
    corner: f32,
    falloff: f32,
    scanlines: f32,
    grille: f32,
    scanline_pitch: f32,
    grille_pitch: f32,
    bleed: f32,
    gain: f32,
    warm: f32,
    bezel: f32,
    bezel_depth: f32,
    bezel_highlight: f32,
    bezel_sheen: f32,
    bezel_radius: f32,
    bezel_mirror: f32,
    room_light: f32,
    lamp: f32,
    convergence: f32,
    reflection: f32,
    grime: f32,
    triad: f32,
    lens: f32,
    lens_x: f32,
    lens_y: f32,
    lens_radius: f32,
    lens_zoom: f32,
    shutoff: f32,
    pad_one: f32,
    pad_two: f32,
}

@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(0) @binding(2) var<uniform> screen: Screen;

const TAU: f32 = 6.2831853;
const PHOSPHOR: vec3<f32> = vec3<f32>(0.66, 0.74, 1.0);
const PLASTIC: vec3<f32> = vec3<f32>(0.196, 0.202, 0.228);
const ROOM: vec3<f32> = vec3<f32>(0.020, 0.021, 0.028);
const KEY: vec3<f32> = vec3<f32>(-0.48, 0.74, -0.47);
const LAMP: vec3<f32> = vec3<f32>(0.34, 0.88, 0.46);
const EYE: f32 = 3.4;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn fullscreenVertex(@builtin(vertex_index) index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32((index << 1u) & 2u);
    let y = f32(index & 2u);
    out.uv = vec2<f32>(x, y);
    out.position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    return out;
}

fn curve(uv: vec2<f32>, amount: f32) -> vec2<f32> {
    var centered = uv * 2.0 - 1.0;
    let offset = abs(centered.yx) / (vec2<f32>(6.0, 4.0) / max(amount, 0.0001));
    centered = centered + centered * offset * offset;
    return centered * 0.5 + 0.5;
}

fn vignette_at(uv: vec2<f32>) -> f32 {
    let corners = uv.x * uv.y * (1.0 - uv.x) * (1.0 - uv.y);
    let shaped = clamp(pow(screen.corner * corners, screen.falloff), 0.0, 1.0);
    return mix(1.0, shaped, screen.vignette);
}

fn bleed_sample(uv: vec2<f32>, texel: vec2<f32>) -> vec3<f32> {
    if screen.bleed <= 0.0 && screen.convergence <= 0.0 {
        return textureSampleLevel(source, source_sampler, uv, 0.0).rgb;
    }
    let from_centre = uv - vec2<f32>(0.5);
    let drift = from_centre * dot(from_centre, from_centre) * screen.convergence;
    let offset = vec2<f32>(texel.x * screen.bleed, 0.0) + drift;
    return vec3<f32>(
        textureSampleLevel(source, source_sampler, uv + offset, 0.0).r,
        textureSampleLevel(source, source_sampler, uv, 0.0).g,
        textureSampleLevel(source, source_sampler, uv - offset, 0.0).b,
    );
}

fn hash12(seed: vec2<f32>) -> f32 {
    var p = fract(vec3<f32>(seed.x, seed.y, seed.x) * 0.1031);
    p = p + dot(p, p.yzx + 33.33);
    return fract((p.x + p.y) * p.z);
}

fn noise12(at: vec2<f32>) -> f32 {
    let whole = floor(at);
    let part = at - whole;
    let eased = part * part * (3.0 - 2.0 * part);
    return mix(
        mix(hash12(whole), hash12(whole + vec2<f32>(1.0, 0.0)), eased.x),
        mix(
            hash12(whole + vec2<f32>(0.0, 1.0)),
            hash12(whole + vec2<f32>(1.0, 1.0)),
            eased.x,
        ),
        eased.y,
    );
}

fn glass_normal(uv: vec2<f32>) -> vec3<f32> {
    let from_centre = (uv - vec2<f32>(0.5)) * 2.0;
    let tilt = from_centre * screen.curvature * 0.5;
    return normalize(vec3<f32>(tilt.x, tilt.y, -1.0));
}

fn window_glare(uv: vec2<f32>, normal: vec3<f32>) -> f32 {
    let at = (uv - vec2<f32>(0.235, 0.195)) + normal.xy * 0.30;
    let pane = vec2<f32>(0.150, 0.125);
    let inside = abs(at) - pane;
    let edge = length(max(inside, vec2<f32>(0.0))) + min(max(inside.x, inside.y), 0.0);
    let shape = smoothstep(0.115, -0.020, edge);
    let roll = smoothstep(0.022, 0.0, abs(at.y + pane.y * 0.58));
    let mullion = smoothstep(0.010, 0.0, abs(at.x));
    return shape * (1.0 - roll * 0.55) * (1.0 - mullion * 0.35);
}

fn grime_at(uv: vec2<f32>) -> f32 {
    let dust = noise12(uv * vec2<f32>(7.0, 5.0)) * 0.55
        + noise12(uv * vec2<f32>(17.0, 13.0)) * 0.25;

    let arc = 0.52 + sin(uv.x * 2.4) * 0.045;
    let wipe = smoothstep(0.16, 0.0, abs(uv.y - arc)) * (0.45 + dust * 0.55);

    let thumb = uv - vec2<f32>(0.86, 0.87);
    let ridges = sin(length(thumb) * 190.0) * 0.5 + 0.5;
    let print = smoothstep(0.055, 0.0, length(thumb)) * ridges;

    return clamp(dust * 0.42 + wipe * 0.38 + print * 0.55, 0.0, 1.0);
}

fn sd_round_box(point: vec3<f32>, half: vec3<f32>, radius: f32) -> f32 {
    let from_face = abs(point) - half + vec3<f32>(radius);
    return length(max(from_face, vec3<f32>(0.0)))
        + min(max(from_face.x, max(from_face.y, from_face.z)), 0.0)
        - radius;
}

fn glass_of(half: vec2<f32>) -> vec2<f32> {
    return max(half - vec2<f32>(screen.bezel), vec2<f32>(0.05));
}

fn bowed_glass(point: vec3<f32>, half: vec2<f32>) -> vec2<f32> {
    let glass = glass_of(half);
    let reach = min(abs(point.xy) / glass, vec2<f32>(1.4));
    let bow = vec2<f32>(reach.y, reach.x) * screen.curvature / vec2<f32>(6.0, 4.0);
    return glass / (vec2<f32>(1.0) + bow * bow);
}

fn case_sdf(point: vec3<f32>, half: vec2<f32>) -> f32 {
    let depth = max(screen.bezel_depth, 0.004);
    let outer = sd_round_box(
        point - vec3<f32>(0.0, 0.0, -depth * 0.5),
        vec3<f32>(half, depth * 0.5),
        screen.bezel_radius,
    );
    let glass = glass_of(half);
    let hole = sd_round_box(
        point - vec3<f32>(0.0, 0.0, -depth),
        vec3<f32>(bowed_glass(point, half), depth * 3.0),
        screen.bezel_radius * 0.30,
    );
    let recess = sd_round_box(
        point - vec3<f32>(0.0, 0.0, -depth),
        vec3<f32>(glass + vec2<f32>(screen.bezel * 0.44), depth * 0.34),
        screen.bezel_radius * 0.34,
    );
    return max(outer, -min(hole, recess));
}

fn case_normal(point: vec3<f32>, half: vec2<f32>) -> vec3<f32> {
    let step = vec2<f32>(0.0012, 0.0);
    return normalize(vec3<f32>(
        case_sdf(point + step.xyy, half) - case_sdf(point - step.xyy, half),
        case_sdf(point + step.yxy, half) - case_sdf(point - step.yxy, half),
        case_sdf(point + step.yyx, half) - case_sdf(point - step.yyx, half),
    ));
}

fn to_screen(point: vec3<f32>, half: vec2<f32>) -> vec2<f32> {
    return (point.xy / glass_of(half)) * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5);
}

fn glow_at(at: vec2<f32>) -> vec3<f32> {
    if at.x < 0.0 || at.x > 1.0 || at.y < 0.0 || at.y > 1.0 {
        return vec3<f32>(0.0);
    }
    return textureSampleLevel(source, source_sampler, at, 0.0).rgb;
}

fn shade_case(hit: vec3<f32>, ray: vec3<f32>, half: vec2<f32>) -> vec3<f32> {
    let normal = case_normal(hit, half);
    let toward_eye = -ray;
    let key = normalize(KEY);

    let lambert = max(dot(normal, key), 0.0);
    var color = PLASTIC * (screen.room_light * 0.30 + screen.bezel_highlight * lambert);

    let half_way = normalize(key + toward_eye);
    let tight = pow(max(dot(normal, half_way), 0.0), 64.0);
    let broad = pow(max(dot(normal, half_way), 0.0), 6.0);
    color += vec3<f32>(1.0) * screen.bezel_sheen * (tight * 0.34 + broad * 0.09);

    let bounce = reflect(ray, normal);
    if bounce.z > 0.001 {
        let travel = -hit.z / bounce.z;
        let landing = hit + bounce * travel;
        let fresnel = pow(1.0 - max(dot(toward_eye, normal), 0.0), 4.0);
        color += glow_at(to_screen(landing, half))
            * screen.bezel_mirror
            * (0.26 + fresnel * 1.20);
    }

    let lamp_at = vec3<f32>(
        half.x * 0.80,
        -half.y + screen.bezel * 0.45,
        -screen.bezel_depth,
    );
    let reach = distance(hit, lamp_at);
    color += LAMP * screen.lamp * smoothstep(screen.bezel * 0.22, 0.0, reach);
    color += LAMP * screen.lamp * 0.30 * exp(-reach / max(screen.bezel, 0.001));
    return color;
}

fn march_case(uv: vec2<f32>, half: vec2<f32>) -> vec3<f32> {
    let on_glass = vec3<f32>((uv.x - 0.5) * 2.0 * half.x, (0.5 - uv.y) * 2.0 * half.y, 0.0);
    let eye = vec3<f32>(0.0, 0.0, -EYE);
    let ray = normalize(on_glass - eye);

    var travelled = EYE - screen.bezel_depth * 2.0 - 0.5;
    for (var step = 0; step < 56; step++) {
        let point = eye + ray * travelled;
        let reach = case_sdf(point, half);
        if reach < 0.0006 {
            return shade_case(point, ray, half);
        }
        travelled += max(reach, 0.0008);
        if travelled > EYE + 0.7 {
            break;
        }
    }
    return ROOM * screen.room_light;
}

@fragment
fn fragmentMain(in: VertexOutput) -> @location(0) vec4<f32> {
    let size = vec2<f32>(textureDimensions(source));
    let aspect = size.x / max(size.y, 1.0);
    let half = vec2<f32>(aspect, 1.0);

    let inset = vec2<f32>(screen.bezel / aspect, screen.bezel) * 0.5;
    let placed = (in.uv - inset) / max(vec2<f32>(1.0) - inset * 2.0, vec2<f32>(0.0001));
    let curved = curve(placed, screen.curvature);
    if curved.x < 0.0 || curved.x > 1.0 || curved.y < 0.0 || curved.y > 1.0 {
        return vec4<f32>(march_case(in.uv, half), 1.0);
    }

    var looked = curved;
    var rim = 0.0;
    if screen.lens > 0.0 {
        let middle = vec2<f32>(screen.lens_x, screen.lens_y);
        let scaled = vec2<f32>(aspect, 1.0);
        let offset = (curved - middle) * scaled;
        let reach = length(offset);
        if reach < screen.lens_radius {
            let pull = mix(1.0, 1.0 / max(screen.lens_zoom, 1.0), screen.lens);
            looked = middle + offset * pull / scaled;
            rim = smoothstep(screen.lens_radius * 0.84, screen.lens_radius, reach) * screen.lens;
        }
    }

    var dying = 0.0;
    if screen.shutoff > 0.0 {
        let squeeze = clamp(screen.shutoff * 1.45, 0.0, 1.0);
        let pinch = clamp((screen.shutoff - 0.62) / 0.38, 0.0, 1.0);
        let band = mix(1.0, 0.006, squeeze);
        let span = mix(1.0, 0.0, pinch);
        let taken = vec2<f32>(
            0.5 + (looked.x - 0.5) / max(span, 0.0001),
            0.5 + (looked.y - 0.5) / max(band, 0.0001),
        );
        if taken.x < 0.0 || taken.x > 1.0 || taken.y < 0.0 || taken.y > 1.0 {
            return vec4<f32>(march_case(in.uv, half) * (1.0 - screen.shutoff), 1.0);
        }
        looked = taken;
        dying = screen.shutoff;
    }

    var color = bleed_sample(looked, 1.0 / size);
    color = mix(color, color * PHOSPHOR, screen.warm);
    color *= vignette_at(curved);

    let drift = 18.0 * screen.time;
    let scan_wave = 0.5 + 0.5 * cos(TAU * (in.position.y + drift) / screen.scanline_pitch);
    let grille_wave = 0.5 + 0.5 * cos(TAU * in.position.x / screen.grille_pitch);
    let scanline = 1.0 - screen.scanlines * scan_wave;
    let grille = 1.0 - screen.grille * grille_wave;
    color *= scanline * grille;

    let lane = i32(floor(in.position.x)) % 3;
    var triad = vec3<f32>(1.0 - screen.triad);
    if lane == 0 {
        triad.r = 1.0;
    } else if lane == 1 {
        triad.g = 1.0;
    } else {
        triad.b = 1.0;
    }
    color *= triad;

    let masked = (1.0 - screen.scanlines * 0.5)
        * (1.0 - screen.grille * 0.5)
        * (1.0 - screen.triad * 2.0 / 3.0);
    color = color / max(masked, 0.2) * screen.gain;

    let normal = glass_normal(placed);
    let dirt = grime_at(placed) * screen.grime;
    let glare = window_glare(placed, normal) * screen.reflection;

    let brightness = max(color.r, max(color.g, color.b));
    color += vec3<f32>(0.60, 0.63, 0.72) * dirt * (0.35 + brightness * 0.65);
    color += vec3<f32>(0.72, 0.78, 0.92) * glare;
    color += vec3<f32>(0.62, 0.68, 0.84) * rim * 0.55;
    color = mix(color, vec3<f32>(1.0, 0.97, 0.90), dying * 0.62);
    color *= 1.0 - smoothstep(0.88, 1.0, screen.shutoff);
    return vec4<f32>(color, 1.0);
}
