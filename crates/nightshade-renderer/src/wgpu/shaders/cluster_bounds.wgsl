struct ClusterUniforms {
    inverse_projection: mat4x4<f32>,
    screen_size: vec2<f32>,
    z_near: f32,
    z_far: f32,
    cluster_count: vec4<u32>,
    tile_size: vec2<f32>,
    num_lights: u32,
    num_directional_lights: u32,
};

struct ClusterBounds {
    min_point: vec4<f32>,
    max_point: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: ClusterUniforms;

@group(0) @binding(1)
var<storage, read_write> cluster_bounds: array<ClusterBounds>;

fn screen_to_view(screen_coord: vec2<f32>, depth: f32) -> vec3<f32> {
    let ndc = vec4<f32>(
        (screen_coord.x / uniforms.screen_size.x) * 2.0 - 1.0,
        (1.0 - screen_coord.y / uniforms.screen_size.y) * 2.0 - 1.0,
        depth,
        1.0
    );
    let view_pos = uniforms.inverse_projection * ndc;
    return view_pos.xyz / view_pos.w;
}

fn line_intersection_with_z_plane(start: vec3<f32>, end: vec3<f32>, z: f32) -> vec3<f32> {
    let direction = end - start;
    let t = (z - start.z) / direction.z;
    return start + t * direction;
}

fn cluster_depth_to_view_z(slice: u32) -> f32 {
    let z_near = uniforms.z_near;
    let z_far = uniforms.z_far;
    let num_slices = f32(uniforms.cluster_count.z);
    let t = f32(slice) / num_slices;
    return -z_near * pow(z_far / z_near, t);
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let cluster_x = global_id.x;
    let cluster_y = global_id.y;
    let cluster_z = global_id.z;

    if cluster_x >= uniforms.cluster_count.x ||
       cluster_y >= uniforms.cluster_count.y ||
       cluster_z >= uniforms.cluster_count.z {
        return;
    }

    let tile_size = uniforms.tile_size;
    let min_screen = vec2<f32>(f32(cluster_x) * tile_size.x, f32(cluster_y) * tile_size.y);
    let max_screen = vec2<f32>(f32(cluster_x + 1u) * tile_size.x, f32(cluster_y + 1u) * tile_size.y);

    let eye_pos = vec3<f32>(0.0, 0.0, 0.0);

    let min_screen_view_near = screen_to_view(min_screen, 1.0);
    let max_screen_view_near = screen_to_view(max_screen, 1.0);

    let cluster_near_z = cluster_depth_to_view_z(cluster_z);
    let cluster_far_z = cluster_depth_to_view_z(cluster_z + 1u);

    let min_point_near = line_intersection_with_z_plane(eye_pos, min_screen_view_near, cluster_near_z);
    let min_point_far = line_intersection_with_z_plane(eye_pos, min_screen_view_near, cluster_far_z);
    let max_point_near = line_intersection_with_z_plane(eye_pos, max_screen_view_near, cluster_near_z);
    let max_point_far = line_intersection_with_z_plane(eye_pos, max_screen_view_near, cluster_far_z);

    let aabb_min = min(min(min_point_near, min_point_far), min(max_point_near, max_point_far));
    let aabb_max = max(max(min_point_near, min_point_far), max(max_point_near, max_point_far));

    let cluster_index = cluster_x +
                        cluster_y * uniforms.cluster_count.x +
                        cluster_z * uniforms.cluster_count.x * uniforms.cluster_count.y;

    cluster_bounds[cluster_index].min_point = vec4<f32>(aabb_min, 0.0);
    cluster_bounds[cluster_index].max_point = vec4<f32>(aabb_max, 0.0);
}
