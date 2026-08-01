struct CameraUniforms {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    view_projection: mat4x4<f32>,
    inverse_view_projection: mat4x4<f32>,
    camera_position: vec4<f32>,
    screen_size: vec2<f32>,
    near_plane: f32,
    far_plane: f32,
};

struct DecalData {
    model: mat4x4<f32>,
    inverse_model: mat4x4<f32>,
    color: vec4<f32>,
    emissive: vec4<f32>,
    size_depth: vec4<f32>,
    params: vec4<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniforms;
@group(0) @binding(1) var<storage, read> decals: array<DecalData>;
@group(0) @binding(2) var depth_texture: texture_depth_2d;
@group(0) @binding(3) var decal_texture: texture_2d<f32>;
@group(0) @binding(4) var decal_sampler: sampler;
@group(0) @binding(5) var emissive_texture: texture_2d<f32>;
@group(0) @binding(6) var emissive_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @builtin(instance_index) instance_index: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) @interpolate(flat) instance_index: u32,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    let decal = decals[input.instance_index];

    var scaled_position = input.position;
    scaled_position.x = scaled_position.x * decal.size_depth.x;
    scaled_position.y = scaled_position.y * decal.size_depth.y;
    scaled_position.z = scaled_position.z * decal.size_depth.z;

    let world_position = decal.model * vec4<f32>(scaled_position, 1.0);
    let clip_position = camera.view_projection * world_position;

    var output: VertexOutput;
    output.clip_position = clip_position;
    output.world_position = world_position.xyz;
    output.instance_index = input.instance_index;
    return output;
}

fn reconstruct_world_position(screen_uv: vec2<f32>, depth: f32) -> vec3<f32> {
    let ndc_x = screen_uv.x * 2.0 - 1.0;
    let ndc_y = (1.0 - screen_uv.y) * 2.0 - 1.0;
    let clip_position = vec4<f32>(ndc_x, ndc_y, depth, 1.0);
    let world_position = camera.inverse_view_projection * clip_position;
    return world_position.xyz / world_position.w;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let decal = decals[input.instance_index];

    let screen_uv = input.clip_position.xy / camera.screen_size;
    let screen_coord = vec2<i32>(input.clip_position.xy);
    let depth_dims = textureDimensions(depth_texture);
    let clamped_coord = clamp(screen_coord, vec2<i32>(0, 0), vec2<i32>(depth_dims) - vec2<i32>(1, 1));
    let scene_depth = textureLoad(depth_texture, clamped_coord, 0);

    if scene_depth <= 0.0 {
        discard;
    }

    let world_pos = reconstruct_world_position(screen_uv, scene_depth);

    let local_pos = decal.inverse_model * vec4<f32>(world_pos, 1.0);
    let scaled_local = vec3<f32>(
        local_pos.x / decal.size_depth.x,
        local_pos.y / decal.size_depth.y,
        local_pos.z / decal.size_depth.z
    );

    let half = 0.5;
    if abs(scaled_local.x) > half || abs(scaled_local.y) > half || abs(scaled_local.z) > half {
        discard;
    }

    let uv = vec2<f32>(scaled_local.x + 0.5, 0.5 - scaled_local.y);

    var base_color = textureSample(decal_texture, decal_sampler, uv);
    base_color = base_color * decal.color;

    if base_color.a < 0.01 {
        discard;
    }

    let emissive_strength = decal.emissive.w;
    var emissive_color = vec3<f32>(0.0);
    let emissive_tex_sample = textureSample(emissive_texture, emissive_sampler, uv).rgb;
    let has_emissive_texture = decal.params.x > 0.5;
    if emissive_strength > 0.0 {
        if has_emissive_texture {
            emissive_color = emissive_tex_sample;
        } else {
            emissive_color = base_color.rgb;
        }
        emissive_color = emissive_color * decal.emissive.rgb * emissive_strength;
    }

    let final_color = base_color.rgb + emissive_color;

    let distance_to_camera = length(camera.camera_position.xyz - world_pos);
    let fade_start = decal.params.y;
    let fade_end = decal.params.z;
    var fade = 1.0;
    if fade_end > fade_start {
        fade = 1.0 - clamp((distance_to_camera - fade_start) / (fade_end - fade_start), 0.0, 1.0);
    }

    let final_alpha = base_color.a * fade;

    return vec4<f32>(final_color, final_alpha);
}
