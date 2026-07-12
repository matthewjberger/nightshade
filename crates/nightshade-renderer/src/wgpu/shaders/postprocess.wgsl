const TONEMAP_ACES: u32 = 0u;
const TONEMAP_REINHARD: u32 = 1u;
const TONEMAP_REINHARD_EXTENDED: u32 = 2u;
const TONEMAP_UNCHARTED2: u32 = 3u;
const TONEMAP_AGX: u32 = 4u;
const TONEMAP_NEUTRAL: u32 = 5u;
const TONEMAP_NONE: u32 = 6u;
const TONEMAP_ACES2: u32 = 7u;

struct PostProcessParams {
    bloom_enabled: f32,
    bloom_intensity: f32,
    letterbox_amount: f32,
    gamma: f32,
    saturation: f32,
    brightness: f32,
    contrast: f32,
    tonemap_algorithm: u32,
    ssao_enabled: f32,
    texture_debug_stripes: u32,
    time: f32,
    texture_debug_stripes_speed: f32,
    ssgi_enabled: f32,
    ssgi_intensity: f32,
    ssao_visualization: f32,
    ssr_enabled: f32,
    exposure: f32,
    auto_exposure_enabled: u32,
    auto_exposure_target: f32,
    exposure_compensation_scale: f32,
    auto_exposure_min_scale: f32,
    auto_exposure_max_scale: f32,
    vibrance: f32,
    vignette_intensity: f32,
    vignette_radius: f32,
    vignette_smoothness: f32,
    chromatic_aberration: f32,
    color_lut_weight: f32,
}

struct AutoExposureBuffer {
    target_log_luminance: f32,
    current_log_luminance: f32,
    adaptation_rate: f32,
    delta_time: f32,
    primed: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

@group(0) @binding(0) var hdr_texture: texture_2d<f32>;
@group(0) @binding(1) var hdr_sampler: sampler;
@group(0) @binding(2) var<uniform> params: PostProcessParams;
@group(0) @binding(3) var bloom_texture: texture_2d<f32>;
@group(0) @binding(4) var bloom_sampler: sampler;
@group(0) @binding(5) var ssao_texture: texture_2d<f32>;
@group(0) @binding(6) var ssao_sampler: sampler;
@group(0) @binding(7) var ssgi_texture: texture_2d<f32>;
@group(0) @binding(8) var ssgi_sampler: sampler;
@group(0) @binding(9) var ssr_texture: texture_2d<f32>;
@group(0) @binding(10) var ssr_sampler: sampler;
@group(0) @binding(11) var<storage, read> auto_exposure: AutoExposureBuffer;
@group(0) @binding(12) var color_lut: texture_3d<f32>;
@group(0) @binding(13) var color_lut_sampler: sampler;

fn apply_color_lut(color: vec3<f32>, weight: f32) -> vec3<f32> {
    let lut_size = 16.0;
    let scale = (lut_size - 1.0) / lut_size;
    let offset = 0.5 / lut_size;
    let coord = clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)) * scale + offset;
    let graded = textureSampleLevel(color_lut, color_lut_sampler, coord, 0.0).rgb;
    return mix(color, graded, weight);
}

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

fn tonemap_aces(color: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((color * (a * color + b)) / (color * (c * color + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

fn tonemap_reinhard(color: vec3<f32>) -> vec3<f32> {
    return color / (color + vec3<f32>(1.0));
}

fn tonemap_reinhard_extended(color: vec3<f32>) -> vec3<f32> {
    let max_white = 4.0;
    let numerator = color * (1.0 + color / (max_white * max_white));
    return numerator / (1.0 + color);
}

fn uncharted2_partial(x: vec3<f32>) -> vec3<f32> {
    let a = 0.15;
    let b = 0.50;
    let c = 0.10;
    let d = 0.20;
    let e = 0.02;
    let f = 0.30;
    return ((x * (a * x + c * b) + d * e) / (x * (a * x + b) + d * f)) - e / f;
}

fn tonemap_uncharted2(color: vec3<f32>) -> vec3<f32> {
    let exposure_bias = 2.0;
    let curr = uncharted2_partial(color * exposure_bias);
    let white_scale = vec3<f32>(1.0) / uncharted2_partial(vec3<f32>(11.2));
    return curr * white_scale;
}

fn agx_default_contrast_approx(x: vec3<f32>) -> vec3<f32> {
    let x2 = x * x;
    let x4 = x2 * x2;
    return 15.5 * x4 * x2 - 40.14 * x4 * x + 31.96 * x4 - 6.868 * x2 * x + 0.4298 * x2 + 0.1191 * x - 0.00232;
}

fn tonemap_agx(color: vec3<f32>) -> vec3<f32> {
    let agx_mat = mat3x3<f32>(
        vec3<f32>(0.842479062253094, 0.0423282422610123, 0.0423756549057051),
        vec3<f32>(0.0784335999999992, 0.878468636469772, 0.0784336),
        vec3<f32>(0.0792237451477643, 0.0791661274605434, 0.879142973793104)
    );
    let min_ev = -12.47393;
    let max_ev = 4.026069;

    var val = agx_mat * max(color, vec3<f32>(0.0));
    val = max(val, vec3<f32>(1e-10));
    val = clamp(log2(val), vec3<f32>(min_ev), vec3<f32>(max_ev));
    val = (val - min_ev) / (max_ev - min_ev);
    val = agx_default_contrast_approx(val);

    let agx_mat_inv = mat3x3<f32>(
        vec3<f32>(1.19687900512017, -0.0528968517574562, -0.0529716355144438),
        vec3<f32>(-0.0980208811401368, 1.15190312990417, -0.0980434501171241),
        vec3<f32>(-0.0990297440797205, -0.0989611768448433, 1.15107367264116)
    );

    return agx_mat_inv * val;
}

fn tonemap_neutral(color: vec3<f32>) -> vec3<f32> {
    let start_compression = 0.8 - 0.04;
    let desaturation = 0.15;

    let x = min(color.r, min(color.g, color.b));
    let offset = select(0.04, x - 6.25 * x * x, x < 0.08);
    var result = color - vec3<f32>(offset);

    let peak = max(result.r, max(result.g, result.b));
    if peak < start_compression {
        return result;
    }

    let d = 1.0 - start_compression;
    let new_peak = 1.0 - d * d / (peak + d - start_compression);
    result = result * (new_peak / peak);

    let g = 1.0 - 1.0 / (desaturation * (peak - new_peak) + 1.0);
    return mix(result, vec3<f32>(new_peak), g);
}

fn tonemap_aces2(color: vec3<f32>) -> vec3<f32> {
    let aces_input = mat3x3<f32>(
        vec3<f32>(0.59719, 0.07600, 0.02840),
        vec3<f32>(0.35458, 0.90834, 0.13383),
        vec3<f32>(0.04823, 0.01566, 0.83777),
    );
    let aces_output = mat3x3<f32>(
        vec3<f32>(1.60475, -0.10208, -0.00327),
        vec3<f32>(-0.53108, 1.10813, -0.07276),
        vec3<f32>(-0.07367, -0.00605, 1.07602),
    );
    var v = aces_input * (color / 0.6);
    let a = v * (v + 0.0245786) - 0.000090537;
    let b = v * (0.983729 * v + 0.4329510) + 0.238081;
    v = a / b;
    return clamp(aces_output * v, vec3<f32>(0.0), vec3<f32>(1.0));
}

fn apply_tonemap(color: vec3<f32>, algorithm: u32) -> vec3<f32> {
    switch algorithm {
        case TONEMAP_ACES: {
            return tonemap_aces(color);
        }
        case TONEMAP_ACES2: {
            return tonemap_aces2(color);
        }
        case TONEMAP_REINHARD: {
            return tonemap_reinhard(color);
        }
        case TONEMAP_REINHARD_EXTENDED: {
            return tonemap_reinhard_extended(color);
        }
        case TONEMAP_UNCHARTED2: {
            return tonemap_uncharted2(color);
        }
        case TONEMAP_AGX: {
            return tonemap_agx(color);
        }
        case TONEMAP_NEUTRAL: {
            return tonemap_neutral(color);
        }
        default: {
            return clamp(color, vec3<f32>(0.0), vec3<f32>(1.0));
        }
    }
}

fn apply_saturation(color: vec3<f32>, saturation: f32) -> vec3<f32> {
    let luminance = dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
    return mix(vec3<f32>(luminance), color, saturation);
}

fn apply_vibrance(color: vec3<f32>, vibrance: f32) -> vec3<f32> {
    let luminance = dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
    let mx = max(color.r, max(color.g, color.b));
    let mn = min(color.r, min(color.g, color.b));
    let sat = clamp(mx - mn, 0.0, 1.0);
    let amount = vibrance * (1.0 - sat);
    return mix(vec3<f32>(luminance), color, 1.0 + amount);
}

fn apply_brightness(color: vec3<f32>, brightness: f32) -> vec3<f32> {
    return color + vec3<f32>(brightness);
}

fn apply_contrast(color: vec3<f32>, contrast: f32) -> vec3<f32> {
    return (color - 0.5) * contrast + 0.5;
}

fn apply_gamma(color: vec3<f32>, gamma: f32) -> vec3<f32> {
    let inv_gamma = 1.0 / gamma;
    return pow(max(color, vec3<f32>(0.0)), vec3<f32>(inv_gamma));
}

fn compute_letterbox(uv: vec2<f32>, amount: f32) -> f32 {
    let bar_height = amount * 0.12;
    let edge_softness = 0.008;

    let top_bar = smoothstep(bar_height - edge_softness, bar_height + edge_softness, uv.y);
    let bottom_bar = smoothstep(bar_height - edge_softness, bar_height + edge_softness, 1.0 - uv.y);

    return top_bar * bottom_bar;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if params.ssao_visualization > 0.5 {
        let ao = textureSample(ssao_texture, ssao_sampler, in.uv).r;
        return vec4<f32>(ao, ao, ao, 1.0);
    }

    var color = textureSample(hdr_texture, hdr_sampler, in.uv).rgb;

    if params.chromatic_aberration > 0.0 {
        let center_offset = in.uv - vec2<f32>(0.5);
        let aberration = center_offset * params.chromatic_aberration * dot(center_offset, center_offset);
        let r = textureSample(hdr_texture, hdr_sampler, in.uv - aberration).r;
        let b = textureSample(hdr_texture, hdr_sampler, in.uv + aberration).b;
        color = vec3<f32>(r, color.g, b);
    }

    if params.ssao_enabled > 0.5 {
        let ao = textureSample(ssao_texture, ssao_sampler, in.uv).r;
        color = color * ao;
    }

    if params.ssgi_enabled > 0.5 {
        let indirect = textureSample(ssgi_texture, ssgi_sampler, in.uv).rgb;
        color = color + indirect * params.ssgi_intensity;
    }

    if params.ssr_enabled > 0.5 {
        let ssr_sample = textureSample(ssr_texture, ssr_sampler, in.uv);
        let ssr_color = ssr_sample.rgb;
        let ssr_confidence = saturate(ssr_sample.a);
        color = mix(color, ssr_color / max(ssr_confidence, 0.001), ssr_confidence);
    }

    if params.bloom_enabled > 0.5 {
        let bloom = textureSample(bloom_texture, bloom_sampler, in.uv).rgb;
        color = color + bloom * params.bloom_intensity;
    }

    var effective_exposure = params.exposure;
    if params.auto_exposure_enabled != 0u {
        let raw_log_lum = auto_exposure.current_log_luminance;
        let safe_log_lum = select(0.0, clamp(raw_log_lum, -10.0, 4.0), raw_log_lum == raw_log_lum);
        let avg_lum = exp2(safe_log_lum);
        let target_lum = max(params.auto_exposure_target, 1e-4);
        let auto_scale = clamp(
            target_lum / max(avg_lum, 1e-4),
            params.auto_exposure_min_scale,
            params.auto_exposure_max_scale,
        );
        effective_exposure = effective_exposure * auto_scale;
    }
    effective_exposure = effective_exposure * params.exposure_compensation_scale;
    var exposed = color * effective_exposure;
    exposed = apply_saturation(exposed, params.saturation);
    exposed = apply_vibrance(exposed, params.vibrance);
    exposed = apply_brightness(exposed, params.brightness);
    exposed = apply_contrast(exposed, params.contrast);

    let tonemapped = apply_tonemap(exposed, params.tonemap_algorithm);
    var final_color = clamp(apply_gamma(tonemapped, params.gamma), vec3<f32>(0.0), vec3<f32>(1.0));

    if params.color_lut_weight > 0.0 {
        final_color = apply_color_lut(final_color, params.color_lut_weight);
    }

    if params.vignette_intensity > 0.0 {
        let vignette_offset = in.uv - vec2<f32>(0.5);
        let vignette_dist = length(vignette_offset) * 1.41421356;
        let vignette = 1.0 - params.vignette_intensity * smoothstep(params.vignette_radius, params.vignette_radius + params.vignette_smoothness, vignette_dist);
        final_color = final_color * vignette;
    }

    if params.letterbox_amount > 0.001 {
        let letterbox_mask = compute_letterbox(in.uv, params.letterbox_amount);
        final_color = final_color * letterbox_mask;
    }

    if params.texture_debug_stripes != 0u {
        let screen_pos = in.clip_position.xy;
        let num_stripes = 7u;
        let line_width = 3.0;
        let max_diagonal = 2560.0 + 1440.0;

        let diagonal = screen_pos.x + screen_pos.y - params.time * params.texture_debug_stripes_speed;
        let band_size = max_diagonal / f32(num_stripes);
        let wrapped_pos = ((diagonal % max_diagonal) + max_diagonal) % max_diagonal;
        let stripe_index = u32(floor(wrapped_pos / band_size)) % num_stripes;
        let pos_in_band = fract(wrapped_pos / band_size);

        let is_line = pos_in_band < (line_width / band_size) || pos_in_band > (1.0 - line_width / band_size);

        if is_line {
            final_color = vec3<f32>(1.0);
        }
    }

    return vec4<f32>(final_color, 1.0);
}
