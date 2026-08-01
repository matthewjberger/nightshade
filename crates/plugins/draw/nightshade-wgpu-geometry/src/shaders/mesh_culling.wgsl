#import nightshade_wgpu::cull_common::{DrawIndexedIndirect, sphere_in_frustum}

struct ModelMatrix {
    model: mat4x4<f32>,
    normal_matrix: mat3x3<f32>,
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
    visible: u32,
    _pad_culling_1: u32,
    debug_view_exempt: u32,
};

struct MeshBounds {
    center: vec3<f32>,
    radius: f32,
};

struct CullingUniforms {
    frustum_planes: array<vec4<f32>, 6>,
    view_projection: mat4x4<f32>,
    occluder_view_projection: mat4x4<f32>,
    screen_size: vec2<f32>,
    object_count: u32,
    min_screen_pixel_size: f32,
    projection_scale_y: f32,
    camera_culling_mask: u32,
    hiz_mip_count: f32,
    occlusion_enabled: u32,
    frustum_enabled: u32,
};

@group(0) @binding(0)
var<storage, read_write> transforms: array<ModelMatrix>;

@group(0) @binding(1)
var<storage, read> objects: array<ObjectData>;

@group(0) @binding(2)
var<uniform> culling: CullingUniforms;

@group(0) @binding(3)
var<storage, read> mesh_bounds: array<MeshBounds>;

@group(0) @binding(4)
var<storage, read_write> indirect_commands: array<DrawIndexedIndirect>;

@group(0) @binding(5)
var<storage, read_write> visible_indices: array<u32>;

@group(0) @binding(6)
var hiz_texture: texture_2d<f32>;

struct ProjectedAabb {
    screen_min: vec2<f32>,
    screen_max: vec2<f32>,
    nearest_z: f32,
    valid: bool,
};

fn project_occluder_corners(corners: array<vec3<f32>, 8>) -> ProjectedAabb {
    var result: ProjectedAabb;
    result.screen_min = vec2<f32>(1.0);
    result.screen_max = vec2<f32>(0.0);
    result.nearest_z = 0.0;
    result.valid = true;
    for (var i = 0u; i < 8u; i = i + 1u) {
        let clip = culling.occluder_view_projection * vec4<f32>(corners[i], 1.0);
        if clip.w <= 0.0 {
            result.valid = false;
            return result;
        }
        let ndc = clip.xyz / clip.w;
        let uv = vec2<f32>(ndc.x * 0.5 + 0.5, 1.0 - (ndc.y * 0.5 + 0.5));
        result.screen_min = min(result.screen_min, uv);
        result.screen_max = max(result.screen_max, uv);
        result.nearest_z = max(result.nearest_z, ndc.z);
    }
    return result;
}

fn is_occluded(corners: array<vec3<f32>, 8>) -> bool {
    let projected = project_occluder_corners(corners);
    if !projected.valid {
        return false;
    }
    let pixel_pad = vec2<f32>(8.0) / culling.screen_size;
    let padded_min = clamp(projected.screen_min - pixel_pad, vec2<f32>(0.0), vec2<f32>(1.0));
    let padded_max = clamp(projected.screen_max + pixel_pad, vec2<f32>(0.0), vec2<f32>(1.0));
    if padded_min.x >= padded_max.x || padded_min.y >= padded_max.y {
        return false;
    }
    let rect_size = max(
        (padded_max.x - padded_min.x) * culling.screen_size.x,
        (padded_max.y - padded_min.y) * culling.screen_size.y,
    );
    let mip = i32(clamp(ceil(log2(max(rect_size, 1.0))), 0.0, culling.hiz_mip_count - 1.0));
    let mip_size = vec2<f32>(textureDimensions(hiz_texture, mip));
    let min_texel = clamp(vec2<i32>(floor(padded_min * mip_size)), vec2<i32>(0), vec2<i32>(mip_size) - vec2<i32>(1));
    let max_texel = clamp(vec2<i32>(floor(padded_max * mip_size)), vec2<i32>(0), vec2<i32>(mip_size) - vec2<i32>(1));
    var hiz_depth = 1.0;
    for (var y = min_texel.y; y <= max_texel.y; y = y + 1) {
        for (var x = min_texel.x; x <= max_texel.x; x = x + 1) {
            let d = textureLoad(hiz_texture, vec2<i32>(x, y), mip).r;
            hiz_depth = min(hiz_depth, d);
        }
    }
    let occlusion_bias = max(hiz_depth * 0.02, 0.0025);
    return projected.nearest_z < hiz_depth - occlusion_bias;
}

fn compute_normal_matrix(model: mat4x4<f32>) -> mat3x3<f32> {
    let a = model[0].xyz;
    let b = model[1].xyz;
    let c = model[2].xyz;
    let cofactor_0 = cross(b, c);
    let cofactor_1 = cross(c, a);
    let cofactor_2 = cross(a, b);
    let determinant = dot(a, cofactor_0);
    if abs(determinant) < 1e-8 {
        return mat3x3<f32>(
            vec3<f32>(1.0, 0.0, 0.0),
            vec3<f32>(0.0, 1.0, 0.0),
            vec3<f32>(0.0, 0.0, 1.0),
        );
    }
    let inverse_determinant = 1.0 / determinant;
    return mat3x3<f32>(
        cofactor_0 * inverse_determinant,
        cofactor_1 * inverse_determinant,
        cofactor_2 * inverse_determinant,
    );
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let object_index = global_id.x;

    if object_index >= culling.object_count {
        return;
    }

    let object = objects[object_index];

    if object.batch_id == 0xFFFFFFFFu {
        return;
    }

    if (object.culling_mask & culling.camera_culling_mask) == 0u {
        return;
    }

    if object.visible == 0u {
        return;
    }

    let bounds = mesh_bounds[object.mesh_id];
    let transform = transforms[object.transform_index];
    let local_center = vec4<f32>(bounds.center, 1.0);
    let world_center = transform.model * local_center;
    let world_pos = world_center.xyz;
    let scale_x = length(vec3<f32>(transform.model[0][0], transform.model[0][1], transform.model[0][2]));
    let scale_y = length(vec3<f32>(transform.model[1][0], transform.model[1][1], transform.model[1][2]));
    let scale_z = length(vec3<f32>(transform.model[2][0], transform.model[2][1], transform.model[2][2]));
    let max_scale = max(max(scale_x, scale_y), scale_z);
    let bounding_radius = max_scale * bounds.radius;

    var visible = culling.frustum_enabled == 0u
        || sphere_in_frustum(culling.frustum_planes, world_pos, bounding_radius);

    var screen_diameter = 0.0;
    if visible {
        let clip = culling.view_projection * vec4(world_pos, 1.0);
        if clip.w > 0.0 {
            screen_diameter = bounding_radius * culling.projection_scale_y * culling.screen_size.y / clip.w;
            if culling.frustum_enabled != 0u && culling.min_screen_pixel_size > 0.0 && screen_diameter < culling.min_screen_pixel_size {
                visible = false;
            }
        }
    }

    if visible && culling.occlusion_enabled != 0u && object.is_overlay == 0u && object.skip_occlusion == 0u {
        let lo = bounds.center - vec3<f32>(bounds.radius);
        let hi = bounds.center + vec3<f32>(bounds.radius);
        let model = transform.model;
        let edge0 = model[0].xyz * (hi.x - lo.x);
        let edge1 = model[1].xyz * (hi.y - lo.y);
        let edge2 = model[2].xyz * (hi.z - lo.z);
        let base = (model * vec4<f32>(lo, 1.0)).xyz;
        let corners = array<vec3<f32>, 8>(
            base,
            base + edge0,
            base + edge1,
            base + edge0 + edge1,
            base + edge2,
            base + edge0 + edge2,
            base + edge1 + edge2,
            base + edge0 + edge1 + edge2,
        );
        if is_occluded(corners) {
            visible = false;
        }
    }

    if visible {
        transforms[object.transform_index].normal_matrix = compute_normal_matrix(transform.model);

        let target_batch = object.batch_id;
        let write_index = atomicAdd(&indirect_commands[target_batch].instance_count, 1u);
        let first_instance = indirect_commands[target_batch].first_instance;
        visible_indices[first_instance + write_index] = object_index;
    }
}
