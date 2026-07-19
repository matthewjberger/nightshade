#define_import_path nightshade_renderer::cull_common

struct DrawIndexedIndirect {
    index_count: u32,
    instance_count: atomic<u32>,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
};

fn sphere_in_frustum(planes: array<vec4<f32>, 6>, center: vec3<f32>, radius: f32) -> bool {
    for (var i = 0u; i < 6u; i = i + 1u) {
        let plane = planes[i];
        if dot(plane.xyz, center) + plane.w < -radius {
            return false;
        }
    }
    return true;
}
