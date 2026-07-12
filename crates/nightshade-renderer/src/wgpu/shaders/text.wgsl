struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) character_index: u32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_pos: vec3<f32>,
    @location(2) @interpolate(flat) character_index: u32,
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

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sample = textureSample(font_texture, font_sampler, in.tex_coords);

    let char_color = character_colors[in.character_index];
    let use_override = char_color.a > 0.0;
    let base_color = select(text_uniforms.color, char_color, use_override);

    let tinted_rgb = base_color.rgb * sample.rgb;
    let final_alpha = sample.a * base_color.a;

    if (final_alpha < 0.01) {
        discard;
    }

    return vec4<f32>(tinted_rgb, final_alpha);
}
