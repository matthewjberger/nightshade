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

struct LightGrid {
    offset: u32,
    count: u32,
};

struct Light {
    position: vec4<f32>,
    direction: vec4<f32>,
    color: vec4<f32>,
    light_type: u32,
    range: f32,
    inner_cone: f32,
    outer_cone: f32,
    shadow_index: i32,
    light_size: f32,
    cookie_layer: u32,
    _padding: f32,
};

const LIGHT_TYPE_DIRECTIONAL: u32 = 0u;
const LIGHT_TYPE_POINT: u32 = 1u;
const LIGHT_TYPE_SPOT: u32 = 2u;

const MAX_LIGHTS_PER_CLUSTER: u32 = 256u;

@group(0) @binding(0)
var<uniform> uniforms: ClusterUniforms;

@group(0) @binding(1)
var<storage, read> cluster_bounds: array<ClusterBounds>;

@group(0) @binding(2)
var<storage, read_write> light_grid: array<LightGrid>;

@group(0) @binding(3)
var<storage, read_write> light_indices: array<u32>;

@group(0) @binding(4)
var<storage, read> lights: array<Light>;

@group(0) @binding(5)
var<uniform> view_matrix: mat4x4<f32>;

fn sphere_aabb_intersection(sphere_center: vec3<f32>, sphere_radius: f32, aabb_min: vec3<f32>, aabb_max: vec3<f32>) -> bool {
    let closest = clamp(sphere_center, aabb_min, aabb_max);
    let distance_sq = dot(closest - sphere_center, closest - sphere_center);
    return distance_sq <= sphere_radius * sphere_radius;
}

fn cone_aabb_intersection(
    cone_tip: vec3<f32>,
    cone_direction: vec3<f32>,
    cone_range: f32,
    cone_angle_cos: f32,
    aabb_min: vec3<f32>,
    aabb_max: vec3<f32>
) -> bool {
    if sphere_aabb_intersection(cone_tip, cone_range, aabb_min, aabb_max) == false {
        return false;
    }

    let aabb_center = (aabb_min + aabb_max) * 0.5;
    let aabb_extents = (aabb_max - aabb_min) * 0.5;
    let aabb_radius = length(aabb_extents);

    let to_center = aabb_center - cone_tip;
    let dist_along_axis = dot(to_center, cone_direction);

    if dist_along_axis < 0.0 {
        let dist_to_tip = length(to_center);
        return dist_to_tip <= aabb_radius;
    }

    if dist_along_axis > cone_range + aabb_radius {
        return false;
    }

    let point_on_axis = cone_tip + cone_direction * dist_along_axis;
    let perpendicular = aabb_center - point_on_axis;
    let perp_dist = length(perpendicular);

    let cone_radius_at_dist = dist_along_axis * sqrt(1.0 - cone_angle_cos * cone_angle_cos) / cone_angle_cos;

    return perp_dist <= cone_radius_at_dist + aabb_radius;
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let tile_x = global_id.x;
    let tile_y = global_id.y;
    let slice = global_id.z;

    if tile_x >= uniforms.cluster_count.x || tile_y >= uniforms.cluster_count.y || slice >= uniforms.cluster_count.z {
        return;
    }

    let cluster_index = tile_x + tile_y * uniforms.cluster_count.x + slice * uniforms.cluster_count.x * uniforms.cluster_count.y;

    let bounds = cluster_bounds[cluster_index];
    let aabb_min = bounds.min_point.xyz;
    let aabb_max = bounds.max_point.xyz;

    let num_local_lights = uniforms.num_lights - uniforms.num_directional_lights;
    let base = cluster_index * MAX_LIGHTS_PER_CLUSTER;
    var light_count = 0u;

    for (var light_index = 0u; light_index < num_local_lights; light_index = light_index + 1u) {
        let light = lights[uniforms.num_directional_lights + light_index];

        let world_light_pos = light.position.xyz;
        let view_light_pos = (view_matrix * vec4<f32>(world_light_pos, 1.0)).xyz;

        let world_light_dir = normalize(light.direction.xyz);
        let view_light_dir = normalize((view_matrix * vec4<f32>(world_light_dir, 0.0)).xyz);

        let range = light.range;

        var intersects = false;

        if range <= 0.0 {
            intersects = true;
        } else if light.light_type == LIGHT_TYPE_POINT {
            intersects = sphere_aabb_intersection(view_light_pos, range, aabb_min, aabb_max);
        } else if light.light_type == LIGHT_TYPE_SPOT {
            intersects = cone_aabb_intersection(
                view_light_pos,
                view_light_dir,
                range,
                light.outer_cone,
                aabb_min,
                aabb_max
            );
        }

        if intersects && light_count < MAX_LIGHTS_PER_CLUSTER {
            light_indices[base + light_count] = light_index;
            light_count = light_count + 1u;
        }
    }

    light_grid[cluster_index].count = light_count;
}
