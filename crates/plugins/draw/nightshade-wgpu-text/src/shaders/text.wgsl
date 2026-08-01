struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) character_index: u32,
    @location(3) flags: u32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_pos: vec3<f32>,
    @location(2) @interpolate(flat) character_index: u32,
    @location(3) @interpolate(flat) flags: u32,
};

struct Uniforms {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    camera_position: vec4<f32>,
};

struct TextUniforms {
    model: mat4x4<f32>,
    color: vec4<f32>,
    outline_color: vec4<f32>,
    outline_width: f32,
    smoothing: f32,
    _padding: vec2<f32>,
};

const SPREAD: f32 = 8.0;

const GLYPH_COLORED: u32 = 1u;

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(0)
var<uniform> text_uniforms: TextUniforms;

@group(2) @binding(0)
var font_texture: texture_2d<f32>;

@group(2) @binding(1)
var font_sampler: sampler;

@group(2) @binding(2)
var<storage, read> character_colors: array<vec4<f32>>;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let world_position = text_uniforms.model * vec4<f32>(in.position, 1.0);
    out.world_pos = world_position.xyz;
    out.position = uniforms.projection * uniforms.view * world_position;
    out.tex_coords = in.tex_coords;
    out.character_index = in.character_index;
    out.flags = in.flags;

    return out;
}

fn shade_distance(
    distance: f32,
    pixel: f32,
    base_color: vec4<f32>,
    outline_color: vec4<f32>,
    outline_width: f32,
    smoothing: f32,
) -> vec4<f32> {
    let soft = 0.5 * pixel + smoothing;
    let fill = smoothstep(-soft, soft, distance);
    if outline_width > 0.0 && outline_color.a > 0.0 {
        let border = smoothstep(-soft, soft, distance + outline_width);
        return vec4<f32>(
            mix(outline_color.rgb, base_color.rgb, fill),
            mix(border * outline_color.a, base_color.a, fill),
        );
    }
    return vec4<f32>(base_color.rgb, fill * base_color.a);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let atlas_size = vec2<f32>(textureDimensions(font_texture));
    let sample = textureSample(font_texture, font_sampler, in.tex_coords / atlas_size);

    let char_color = character_colors[in.character_index];
    let use_override = char_color.a > 0.0;
    let base_color = select(text_uniforms.color, char_color, use_override);

    let pixel = length(fwidth(in.tex_coords)) * 0.7071;

    var shaded: vec4<f32>;
    if (in.flags & GLYPH_COLORED) != 0u {
        shaded = vec4<f32>(base_color.rgb * sample.rgb, sample.a * base_color.a);
    } else {
        let distance = (sample.a - 0.5) * 2.0 * SPREAD;
        shaded = shade_distance(
            distance,
            pixel,
            base_color,
            text_uniforms.outline_color,
            text_uniforms.outline_width,
            text_uniforms.smoothing,
        );
    }

    if shaded.a < 0.01 {
        discard;
    }

    return shaded;
}
