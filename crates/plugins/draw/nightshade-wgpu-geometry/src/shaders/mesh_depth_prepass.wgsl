#import nightshade_wgpu::mesh_common::{Uniforms, snap_to_the_grid}

#import nightshade_wgpu::material_data::{Material, TextureTransform, material_reveal_distance}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) tex_coords_1: vec2<f32>,
    @location(4) tangent: vec4<f32>,
    @location(5) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) flip_winding: u32,
    @location(1) world_pos: vec3<f32>,
    @location(2) @interpolate(flat) material_id: u32,
};

struct ModelMatrix {
    model: mat4x4<f32>,
    normal_matrix: mat3x3<f32>,
};

struct MorphDisplacement {
    position: vec3<f32>,
    _pad0: f32,
    normal: vec3<f32>,
    _pad1: f32,
    tangent: vec3<f32>,
    _pad2: f32,
};

struct ObjectData {
    transform_index: u32,
    mesh_id: u32,
    material_id: u32,
    batch_id: u32,
    morph_weights: array<f32, 8>,
    morph_target_count: u32,
    morph_displacement_offset: u32,
    mesh_vertex_offset: u32,
    mesh_vertex_count: u32,
    entity_id: u32,
    is_overlay: u32,
    skip_occlusion: u32,
    flip_winding: u32,
    culling_mask: u32,
    _pad_culling_0: u32,
    _pad_culling_1: u32,
    debug_view_exempt: u32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@group(1) @binding(0) var<storage, read> transforms: array<ModelMatrix>;
@group(1) @binding(1) var<storage, read> materials: array<Material>;
@group(1) @binding(2) var<storage, read> objects: array<ObjectData>;
@group(1) @binding(4) var<storage, read> visible_indices: array<u32>;
@group(1) @binding(5) var<storage, read> instance_custom_data: array<vec4<f32>>;
@group(1) @binding(6) var<storage, read> morph_displacements: array<MorphDisplacement>;

@vertex
fn vs_main(
    in: VertexInput,
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_id: u32
) -> VertexOutput {
    var out: VertexOutput;

    let object_index = visible_indices[instance_id];
    let object = objects[object_index];

    var position = in.position;

    if object.morph_target_count > 0u {
        let local_vertex_index = vertex_index - object.mesh_vertex_offset;

        for (var target_index = 0u; target_index < object.morph_target_count; target_index = target_index + 1u) {
            let weight = object.morph_weights[target_index];
            if abs(weight) > 0.0001 {
                let displacement_index = object.morph_displacement_offset
                    + target_index * object.mesh_vertex_count
                    + local_vertex_index;
                let displacement = morph_displacements[displacement_index];
                position = position + displacement.position * weight;
            }
        }
    }

    let transform = transforms[object.transform_index];
    let model_matrix = transform.model;
    let world_position = model_matrix * vec4<f32>(position, 1.0);
    let view_position = uniforms.view * world_position;
    var clip_position = uniforms.projection * view_position;

    if uniforms.snap_enabled == 1u {
        let snapped = snap_to_the_grid(
            clip_position,
            world_position.xyz,
            uniforms.view,
            uniforms.projection,
            uniforms.snap_resolution,
        );
        clip_position = snapped.clip;
    }

    out.position = clip_position;
    out.flip_winding = object.flip_winding;
    out.world_pos = world_position.xyz;
    out.material_id = object.material_id;

    return out;
}

@fragment
fn fs_main(in: VertexOutput, @builtin(front_facing) is_front_face: bool) {
    if in.flip_winding != 0u && is_front_face {
        discard;
    }
    if material_reveal_distance(materials[in.material_id], in.world_pos) > 0.0 {
        discard;
    }
}
