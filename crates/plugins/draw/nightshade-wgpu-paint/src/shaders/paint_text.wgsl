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

const SPREAD: f32 = 8.0;

const GLYPH_COLORED: u32 = 1u;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) character_index: u32,
    @location(3) text_instance_index: u32,
    @location(4) flags: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) screen_pos: vec2<f32>,
    @location(2) @interpolate(flat) character_index: u32,
    @location(3) @interpolate(flat) text_instance_index: u32,
    @location(4) @interpolate(flat) flags: u32,
}

fn spin(point: vec2<f32>, pivot: vec2<f32>, angle: f32) -> vec2<f32> {
    if (angle == 0.0) {
        return point;
    }
    let turn = cos(angle);
    let lean = sin(angle);
    let offset = point - pivot;
    return pivot + vec2<f32>(
        offset.x * turn - offset.y * lean,
        offset.x * lean + offset.y * turn
    );
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    let inst = text_instances[vertex.text_instance_index];
    let depth = inst.params.y;
    let world_pos = spin(
        vertex.position.xy + inst.position.xy,
        inst.position.zw,
        inst.params.z
    );

    output.position = globals.projection * vec4<f32>(world_pos, 0.0, 1.0);
    output.position.z = depth;
    output.tex_coords = vertex.tex_coords;
    output.screen_pos = world_pos;
    output.character_index = vertex.character_index;
    output.text_instance_index = vertex.text_instance_index;
    output.flags = vertex.flags;

    return output;
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

    let atlas_size = vec2<f32>(textureDimensions(font_texture));
    let sample = textureSample(font_texture, font_sampler, input.tex_coords / atlas_size);

    let char_color = character_colors[input.character_index];
    let use_override = char_color.a > 0.0;
    let base_color = select(inst.color, char_color, use_override);

    let pixel = length(fwidth(input.tex_coords)) * 0.7071;

    var foreground: vec4<f32>;
    if (input.flags & GLYPH_COLORED) != 0u {
        foreground = vec4<f32>(base_color.rgb * sample.rgb, sample.a * base_color.a);
    } else {
        let distance = (sample.a - 0.5) * 2.0 * SPREAD;
        foreground = shade_distance(
            distance,
            pixel,
            base_color,
            inst.outline_color,
            inst.params.x,
            inst.params.z,
        );
    }

    let bg_color = character_background_colors[input.character_index];
    if bg_color.a > 0.0 {
        return vec4<f32>(
            mix(bg_color.rgb, foreground.rgb, foreground.a),
            max(bg_color.a, foreground.a),
        );
    }

    return foreground;
}
