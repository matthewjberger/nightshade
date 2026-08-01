use nightshade_wgpu::backend::ShaderComposer;

const PUBLISHED: [(&str, &str); 9] = [
    (include_str!("../shaders/pbr_brdf.wgsl"), "pbr_brdf.wgsl"),
    (
        include_str!("../shaders/material_sampling.wgsl"),
        "material_sampling.wgsl",
    ),
    (
        include_str!("../shaders/cull_common.wgsl"),
        "cull_common.wgsl",
    ),
    (
        include_str!("../shaders/material_data.wgsl"),
        "material_data.wgsl",
    ),
    (
        include_str!("../shaders/area_shape.wgsl"),
        "area_shape.wgsl",
    ),
    (
        include_str!("../shaders/area_lighting.wgsl"),
        "area_lighting.wgsl",
    ),
    (
        include_str!("../shaders/fog_common.wgsl"),
        "fog_common.wgsl",
    ),
    (
        include_str!("../shaders/mesh_common.wgsl"),
        "mesh_common.wgsl",
    ),
    (
        include_str!("../shaders/shadow_cutout.wgsl"),
        "shadow_cutout.wgsl",
    ),
];

pub fn publish(composer: &ShaderComposer) {
    for (source, file_name) in PUBLISHED {
        composer.publish_module(source, file_name);
    }
}

pub const DEPTH_CLEAR: &str = include_str!("../shaders/depth_clear.wgsl");
pub const SKINNING_COMPUTE: &str = include_str!("../shaders/skinning_compute.wgsl");
