use crate::systems::geometry::{MeshPass, SceneOverlayPass, SkinnedMeshPass};
use nightshade_wgpu::backend::{
    PassBuild, PassPosition, RenderGraph, RenderInputs, render_graph_add_pass_at,
};
use nightshade_wgpu::wgpu;

pub fn install_geometry_passes(
    graph: &mut RenderGraph<RenderInputs>,
    build: PassBuild,
    _: wgpu::TextureFormat,
) {
    let targets = build.targets;

    let geometry_slots = [
        ("color", targets.scene_color),
        ("depth", targets.depth),
        ("shadow_depth", targets.shadow_depth),
        ("spotlight_shadow_atlas", targets.spotlight_shadow_atlas),
        ("entity_id", targets.entity_id),
        ("view_normals", targets.view_normals),
        ("velocity", targets.velocity),
        ("point_shadow_cubemap", targets.point_shadow_cubemap),
        ("transmission_grab", targets.transmission_grab),
        ("brdf_lut", targets.brdf_lut),
        ("ibl_irradiance", targets.ibl_irradiance),
        ("ibl_prefiltered", targets.ibl_prefiltered),
        ("ibl_irradiance_b", targets.ibl_irradiance_b),
        ("ibl_prefiltered_b", targets.ibl_prefiltered_b),
    ];

    let mesh_pass = MeshPass::new(
        build,
        build.hdr_format,
        build.depth_format,
        (1, 1),
        build.bindless,
    );
    let air_reads = mesh_pass.air.reads();
    if let Err(error) = render_graph_add_pass_at(
        graph,
        Box::new(mesh_pass),
        &geometry_slots,
        PassPosition::Geometry,
    ) {
        panic!("the mesh pass could not be installed: {error}");
    }

    if let Err(error) = render_graph_add_pass_at(
        graph,
        Box::new(SkinnedMeshPass::new(
            build,
            build.hdr_format,
            build.depth_format,
            build.bindless,
            air_reads,
        )),
        &geometry_slots,
        PassPosition::Geometry,
    ) {
        panic!("the skinned mesh pass could not be installed: {error}");
    }

    let overlay = SceneOverlayPass {
        sub_passes: Vec::new(),
    };
    if let Err(error) = render_graph_add_pass_at(
        graph,
        Box::new(overlay),
        &[
            ("color", targets.scene_color),
            ("depth", targets.depth),
            ("glyph_atlas", targets.glyph_atlas),
        ],
        PassPosition::SceneOverlay,
    ) {
        panic!("the scene overlay pass could not be installed: {error}");
    }
}
