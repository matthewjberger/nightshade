struct DownsampleParams {
    mip_level: u32,
    threshold: f32,
    knee: f32,
    exposure: f32,
}

@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(0) @binding(2) var<uniform> params: DownsampleParams;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vertex_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let uv = vec2<f32>(
        f32((vertex_index << 1u) & 2u),
        f32(vertex_index & 2u)
    );
    let clip_position = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);

    var out: VertexOutput;
    out.clip_position = clip_position;
    out.uv = vec2<f32>(uv.x, 1.0 - uv.y);
    return out;
}

fn rgb_to_luminance(col: vec3<f32>) -> f32 {
    return dot(col, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn karis_average(col: vec3<f32>) -> f32 {
    let luma = rgb_to_luminance(col) * 0.25;
    return 1.0 / (1.0 + luma);
}

fn soft_threshold(color: vec3<f32>, threshold: f32, knee: f32, exposure: f32) -> vec3<f32> {
    let exposed = color * exposure;
    let brightness = max(exposed.r, max(exposed.g, exposed.b));
    let soft = brightness - threshold + knee;
    let soft_clamped = clamp(soft, 0.0, 2.0 * knee);
    let contribution = select(0.0, soft_clamped * soft_clamped / (4.0 * knee + 0.00001), knee > 0.0);
    let total = max(brightness - threshold, contribution);
    let scale = max(total, 0.0) / max(brightness, 0.00001);
    return color * scale;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture_size = vec2<f32>(textureDimensions(source_texture));
    let texel_size = 1.0 / texture_size;
    let x = texel_size.x;
    let y = texel_size.y;

    let a = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(-2.0*x, 2.0*y)).rgb;
    let b = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(0.0, 2.0*y)).rgb;
    let c = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(2.0*x, 2.0*y)).rgb;

    let d = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(-2.0*x, 0.0)).rgb;
    let e = textureSample(source_texture, source_sampler, in.uv).rgb;
    let f = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(2.0*x, 0.0)).rgb;

    let g = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(-2.0*x, -2.0*y)).rgb;
    let h = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(0.0, -2.0*y)).rgb;
    let i = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(2.0*x, -2.0*y)).rgb;

    let j = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(-x, y)).rgb;
    let k = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(x, y)).rgb;
    let l = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(-x, -y)).rgb;
    let m = textureSample(source_texture, source_sampler, in.uv + vec2<f32>(x, -y)).rgb;

    var downsample: vec3<f32>;

    if params.mip_level == 0u {
        let at = soft_threshold(a, params.threshold, params.knee, params.exposure);
        let bt = soft_threshold(b, params.threshold, params.knee, params.exposure);
        let ct = soft_threshold(c, params.threshold, params.knee, params.exposure);
        let dt = soft_threshold(d, params.threshold, params.knee, params.exposure);
        let et = soft_threshold(e, params.threshold, params.knee, params.exposure);
        let ft = soft_threshold(f, params.threshold, params.knee, params.exposure);
        let gt = soft_threshold(g, params.threshold, params.knee, params.exposure);
        let ht = soft_threshold(h, params.threshold, params.knee, params.exposure);
        let it = soft_threshold(i, params.threshold, params.knee, params.exposure);
        let jt = soft_threshold(j, params.threshold, params.knee, params.exposure);
        let kt = soft_threshold(k, params.threshold, params.knee, params.exposure);
        let lt = soft_threshold(l, params.threshold, params.knee, params.exposure);
        let mt = soft_threshold(m, params.threshold, params.knee, params.exposure);

        var groups: array<vec3<f32>, 5>;
        groups[0] = (at + bt + dt + et) * (0.125 / 4.0);
        groups[1] = (bt + ct + et + ft) * (0.125 / 4.0);
        groups[2] = (dt + et + gt + ht) * (0.125 / 4.0);
        groups[3] = (et + ft + ht + it) * (0.125 / 4.0);
        groups[4] = (jt + kt + lt + mt) * (0.5 / 4.0);
        groups[0] *= karis_average(groups[0]);
        groups[1] *= karis_average(groups[1]);
        groups[2] *= karis_average(groups[2]);
        groups[3] *= karis_average(groups[3]);
        groups[4] *= karis_average(groups[4]);
        downsample = groups[0] + groups[1] + groups[2] + groups[3] + groups[4];
    } else {
        downsample = e * 0.125;
        downsample += (a + c + g + i) * 0.03125;
        downsample += (b + d + f + h) * 0.0625;
        downsample += (j + k + l + m) * 0.125;
    }

    downsample = max(downsample, vec3<f32>(0.0));

    return vec4<f32>(downsample, 1.0);
}
