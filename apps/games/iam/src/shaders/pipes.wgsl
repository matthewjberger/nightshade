
const SEGMENTS: u32 = 64u;
const EYE: f32 = 3.2;
const JOINT: f32 = 1.24;
const KEY: vec3<f32> = vec3<f32>(-0.48, 0.74, -0.47);
const VOID: vec3<f32> = vec3<f32>(0.008, 0.009, 0.014);

struct Pipes {
    frame: vec4<f32>,
    room: vec4<f32>,
    low: vec4<f32>,
    high: vec4<f32>,
    heads: array<vec4<f32>, SEGMENTS>,
    tails: array<vec4<f32>, SEGMENTS>,
}

@group(0) @binding(0) var<uniform> pipes: Pipes;

const ENAMEL: array<vec3<f32>, 6> = array<vec3<f32>, 6>(
    vec3<f32>(0.36, 0.62, 0.94),
    vec3<f32>(0.94, 0.44, 0.36),
    vec3<f32>(0.42, 0.86, 0.52),
    vec3<f32>(0.92, 0.78, 0.34),
    vec3<f32>(0.74, 0.46, 0.92),
    vec3<f32>(0.38, 0.86, 0.88),
);

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

fn sd_capsule(point: vec3<f32>, head: vec3<f32>, tail: vec3<f32>, radius: f32) -> f32 {
    let along = tail - head;
    let toward = point - head;
    let travel = clamp(dot(toward, along) / max(dot(along, along), 0.0001), 0.0, 1.0);
    return length(toward - along * travel) - radius;
}

struct Nearest {
    reach: f32,
    which: i32,
}

fn pipes_at(point: vec3<f32>) -> Nearest {
    var found = Nearest(1.0e9, 0);
    let radius = pipes.room.z;
    let live = i32(pipes.frame.z);
    let growing = pipes.frame.w;
    for (var index = 0; index < i32(SEGMENTS); index++) {
        if index >= live {
            break;
        }
        let head = pipes.heads[index].xyz;
        var tail = pipes.tails[index].xyz;
        if index == live - 1 {
            tail = head + (tail - head) * growing;
        }
        let joint = length(point - head) - radius * JOINT;
        let run = sd_capsule(point, head, tail, radius);
        let reach = min(joint, run);
        if reach < found.reach {
            found = Nearest(reach, index);
        }
    }
    return found;
}

fn pipes_normal(point: vec3<f32>) -> vec3<f32> {
    let step = 0.0015;
    let first = vec3<f32>(1.0, -1.0, -1.0);
    let second = vec3<f32>(-1.0, -1.0, 1.0);
    let third = vec3<f32>(-1.0, 1.0, -1.0);
    let fourth = vec3<f32>(1.0, 1.0, 1.0);
    return normalize(
        first * pipes_at(point + first * step).reach
            + second * pipes_at(point + second * step).reach
            + third * pipes_at(point + third * step).reach
            + fourth * pipes_at(point + fourth * step).reach,
    );
}

fn box_span(eye: vec3<f32>, ray: vec3<f32>) -> vec2<f32> {
    let guard = vec3<f32>(1.0e-6);
    let turn = select(vec3<f32>(1.0), vec3<f32>(-1.0), ray < vec3<f32>(0.0));
    let inverse = turn / max(abs(ray), guard);
    let first = (pipes.low.xyz - eye) * inverse;
    let second = (pipes.high.xyz - eye) * inverse;
    let near = min(first, second);
    let far = max(first, second);
    return vec2<f32>(
        max(max(near.x, near.y), near.z),
        min(min(far.x, far.y), far.z),
    );
}

fn shade_pipe(hit: vec3<f32>, ray: vec3<f32>, which: i32) -> vec3<f32> {
    let normal = pipes_normal(hit);
    let toward_eye = -ray;
    let key = normalize(KEY);

    let enamel = ENAMEL[u32(pipes.heads[which].w) % 6u];
    let lambert = max(dot(normal, key), 0.0);
    var color = enamel * (pipes.room.w * 0.22 + lambert * 0.78);

    let half_way = normalize(key + toward_eye);
    let tight = pow(max(dot(normal, half_way), 0.0), 48.0);
    let broad = pow(max(dot(normal, half_way), 0.0), 8.0);
    color += vec3<f32>(1.0) * (tight * 0.42 + broad * 0.10);

    let rim = pow(1.0 - max(dot(toward_eye, normal), 0.0), 3.0);
    color += enamel * rim * 0.30;
    return color;
}

@fragment
fn fragmentMain(in: VertexOutput) -> @location(0) vec4<f32> {
    let cover = pipes.frame.y;
    if cover <= 0.0 {
        discard;
    }

    let size = pipes.room.xy;
    let pixel = vec2<f32>(in.uv.x, 1.0 - in.uv.y) * size;
    let uv = (pixel + pixel - size) / size.y;

    let eye = vec3<f32>(0.0, 0.0, -EYE);
    let ray = normalize(vec3<f32>(uv.x, uv.y, 0.0) - eye);

    var color = VOID * pipes.room.w;

    let span = box_span(eye, ray);
    let entry = max(span.x, 0.0);
    if span.y < entry {
        return vec4<f32>(color, cover);
    }

    var travelled = entry;
    for (var step = 0; step < 72; step++) {
        let point = eye + ray * travelled;
        let found = pipes_at(point);
        if found.reach < 0.0009 {
            color = shade_pipe(point, ray, found.which);
            break;
        }
        travelled += max(found.reach, 0.0012);
        if travelled > span.y {
            break;
        }
    }
    return vec4<f32>(color, cover);
}
