struct TextInstanceData {
    color: vec4<f32>,
    outline_color: vec4<f32>,
    clip_rect: vec4<f32>,
    params: vec4<f32>,
    position: vec4<f32>,
}

@group(0) @binding(0) var<storage, read> text_instances: array<TextInstanceData>;
@group(0) @binding(1) var<storage, read> character_colors: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> character_background_colors: array<vec4<f32>>;
@group(1) @binding(0) var font_texture: texture_2d<f32>;
@group(1) @binding(1) var font_sampler: sampler;

struct GlobalUniforms {
    projection: mat4x4<f32>,
}

@group(2) @binding(0) var<uniform> globals: GlobalUniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) character_index: u32,
    @location(3) text_instance_index: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) screen_pos: vec2<f32>,
    @location(2) @interpolate(flat) character_index: u32,
    @location(3) @interpolate(flat) text_instance_index: u32,
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    let inst = text_instances[vertex.text_instance_index];
    let depth = inst.params.y;
    let world_pos = vertex.position.xy + inst.position.xy;

    output.position = globals.projection * vec4<f32>(world_pos, 0.0, 1.0);
    output.position.z = depth;
    output.tex_coords = vertex.tex_coords;
    output.screen_pos = world_pos;
    output.character_index = vertex.character_index;
    output.text_instance_index = vertex.text_instance_index;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let inst = text_instances[input.text_instance_index];

    let clip = inst.clip_rect;
    if clip.z > clip.x {
        let frag_pos = input.screen_pos;
        if frag_pos.x < clip.x || frag_pos.x > clip.z ||
           frag_pos.y < clip.y || frag_pos.y > clip.w {
            discard;
        }
    }

    let alpha = textureSample(font_texture, font_sampler, input.tex_coords).a;

    let char_color = character_colors[input.character_index];
    let use_override = char_color.a > 0.0;
    let base_color = select(inst.color, char_color, use_override);

    let bg_color = character_background_colors[input.character_index];
    let has_bg = bg_color.a > 0.0;

    if has_bg {
        let fg = vec4<f32>(base_color.rgb, base_color.a * alpha);
        return vec4<f32>(
            mix(bg_color.rgb, fg.rgb, fg.a),
            max(bg_color.a, fg.a),
        );
    }

    return vec4<f32>(base_color.rgb, base_color.a * alpha);
}
