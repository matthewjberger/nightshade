struct BoundingVolumeData {
    center: vec3<f32>,
    _pad0: f32,
    half_extents: vec3<f32>,
    _pad1: f32,
    orientation: vec4<f32>,
    transform: mat4x4<f32>,
    color: vec4<f32>,
}

struct LineData {
    start: vec4<f32>,
    end: vec4<f32>,
    color: vec4<f32>,
    entity_id: u32,
    visible: u32,
    depth_mode: u32,
    _padding: u32,
}

struct Params {
    bounding_volume_count: u32,
    line_offset: u32,
    _padding: vec2<u32>,
}

@group(0) @binding(0)
var<storage, read> bounding_volumes: array<BoundingVolumeData>;

@group(0) @binding(1)
var<storage, read_write> lines: array<LineData>;

@group(0) @binding(2)
var<uniform> params: Params;

fn quat_rotate(q: vec4<f32>, v: vec3<f32>) -> vec3<f32> {
    let u = q.xyz;
    let s = q.w;
    return 2.0 * dot(u, v) * u + (s * s - dot(u, u)) * v + 2.0 * s * cross(u, v);
}

fn get_obb_corner(center: vec3<f32>, half_extents: vec3<f32>, orientation: vec4<f32>, signs: vec3<f32>) -> vec3<f32> {
    let local_corner = half_extents * signs;
    return center + quat_rotate(orientation, local_corner);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let bv_index = global_id.x;
    if (bv_index >= params.bounding_volume_count) {
        return;
    }

    let bv = bounding_volumes[bv_index];
    let transform = bv.transform;

    let local_corners = array<vec3<f32>, 8>(
        get_obb_corner(bv.center, bv.half_extents, bv.orientation, vec3<f32>(-1.0, -1.0, -1.0)),
        get_obb_corner(bv.center, bv.half_extents, bv.orientation, vec3<f32>( 1.0, -1.0, -1.0)),
        get_obb_corner(bv.center, bv.half_extents, bv.orientation, vec3<f32>(-1.0,  1.0, -1.0)),
        get_obb_corner(bv.center, bv.half_extents, bv.orientation, vec3<f32>( 1.0,  1.0, -1.0)),
        get_obb_corner(bv.center, bv.half_extents, bv.orientation, vec3<f32>(-1.0, -1.0,  1.0)),
        get_obb_corner(bv.center, bv.half_extents, bv.orientation, vec3<f32>( 1.0, -1.0,  1.0)),
        get_obb_corner(bv.center, bv.half_extents, bv.orientation, vec3<f32>(-1.0,  1.0,  1.0)),
        get_obb_corner(bv.center, bv.half_extents, bv.orientation, vec3<f32>( 1.0,  1.0,  1.0)),
    );

    var world_corners: array<vec3<f32>, 8>;
    for (var i = 0u; i < 8u; i++) {
        let world_pos = transform * vec4<f32>(local_corners[i], 1.0);
        world_corners[i] = world_pos.xyz;
    }

    let edges = array<vec2<u32>, 12>(
        vec2<u32>(0u, 1u),
        vec2<u32>(1u, 3u),
        vec2<u32>(3u, 2u),
        vec2<u32>(2u, 0u),
        vec2<u32>(4u, 5u),
        vec2<u32>(5u, 7u),
        vec2<u32>(7u, 6u),
        vec2<u32>(6u, 4u),
        vec2<u32>(0u, 4u),
        vec2<u32>(1u, 5u),
        vec2<u32>(2u, 6u),
        vec2<u32>(3u, 7u),
    );

    let base_line_index = params.line_offset + bv_index * 12u;
    for (var i = 0u; i < 12u; i++) {
        let edge = edges[i];
        let line_index = base_line_index + i;

        lines[line_index].start = vec4<f32>(world_corners[edge.x], 1.0);
        lines[line_index].end = vec4<f32>(world_corners[edge.y], 1.0);
        lines[line_index].color = bv.color;
        lines[line_index].entity_id = bv_index;
        lines[line_index].visible = 1u;
        lines[line_index].depth_mode = 0u;
        lines[line_index]._padding = 0u;
    }
}
