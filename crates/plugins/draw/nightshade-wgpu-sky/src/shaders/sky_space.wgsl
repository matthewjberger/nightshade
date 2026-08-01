struct Uniform {
    proj: mat4x4<f32>,
    proj_inv: mat4x4<f32>,
    view: mat4x4<f32>,
    cam_pos: vec4<f32>,
    time: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
};

@group(0) @binding(0)
var<uniform> u: Uniform;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_dir: vec3<f32>,
};

@vertex
fn vs_sky(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let tmp1 = i32(vertex_index) / 2;
    let tmp2 = i32(vertex_index) & 1;
    let pos = vec4<f32>(
        f32(tmp1) * 4.0 - 1.0,
        f32(tmp2) * 4.0 - 1.0,
        0.0,
        1.0
    );
    let inv_model_view = transpose(mat3x3<f32>(u.view[0].xyz, u.view[1].xyz, u.view[2].xyz));
    let unprojected = u.proj_inv * pos;
    var result: VertexOutput;
    result.world_dir = inv_model_view * unprojected.xyz;
    result.position = pos;
    return result;
}

fn hash(p: vec2<f32>) -> f32 {
    let k = vec2<f32>(0.3183099, 0.3678794);
    let x = p * k + k.yx;
    return fract(16.0 * k.x * fract(x.x * x.y * (x.x + x.y)));
}

fn hash2(p: vec2<f32>) -> vec2<f32> {
    let k = vec2<f32>(0.3183099, 0.3678794);
    let q = p * k + k.yx;
    return fract(16.0 * k * fract(q.x * q.y * (q.x + q.y))) * 2.0 - 1.0;
}

fn star_layer(uv: vec2<f32>, scale: f32, seed: f32) -> f32 {
    let grid_uv = uv * scale;
    let grid_id = floor(grid_uv);
    let grid_fract = fract(grid_uv) - 0.5;

    let rand = hash(grid_id + seed);

    if rand > 0.97 {
        let offset = hash2(grid_id + seed + 100.0) * 0.4;
        let dist = length(grid_fract - offset);

        let star_size = 0.02 + hash(grid_id + seed + 200.0) * 0.03;
        let brightness = (1.0 - smoothstep(0.0, star_size, dist));

        let twinkle_speed = 1.0 + hash(grid_id + seed + 300.0) * 3.0;
        let twinkle_phase = hash(grid_id + seed + 400.0) * 6.283;
        let twinkle = 0.7 + 0.3 * sin(u.time * twinkle_speed + twinkle_phase);

        return brightness * twinkle;
    }

    return 0.0;
}

fn dir_to_uv(dir: vec3<f32>) -> vec2<f32> {
    let phi = atan2(dir.z, dir.x);
    let theta = asin(clamp(dir.y, -1.0, 1.0));
    return vec2<f32>(phi / 6.283185 + 0.5, theta / 3.141593 + 0.5);
}

@fragment
fn fs_sky(in: VertexOutput) -> @location(0) vec4<f32> {
    let dir = normalize(in.world_dir);
    let uv = dir_to_uv(dir);

    var stars = 0.0;
    stars += star_layer(uv, 50.0, 0.0);
    stars += star_layer(uv, 100.0, 10.0);
    stars += star_layer(uv, 200.0, 20.0) * 0.7;
    stars += star_layer(uv, 400.0, 30.0) * 0.4;

    let star_hue = hash(floor(uv * 100.0));
    var star_color = vec3<f32>(1.0);
    if star_hue < 0.3 {
        star_color = vec3<f32>(0.8, 0.85, 1.0);
    } else if star_hue > 0.7 {
        star_color = vec3<f32>(1.0, 0.95, 0.85);
    }

    let final_color = star_color * stars;

    return vec4<f32>(final_color, 1.0);
}
