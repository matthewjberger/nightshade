use crate::systems::shadow_depth::ShadowDepthPass;
use nightshade_wgpu::backend::{
    PassBuild, PassPosition, RenderGraph, RenderInputs, render_graph_add_pass_at,
};
use nightshade_wgpu::wgpu;

pub fn install_shadow_depth_pass(
    graph: &mut RenderGraph<RenderInputs>,
    build: PassBuild,
    _: wgpu::TextureFormat,
) {
    let targets = build.targets;
    if let Err(error) = render_graph_add_pass_at(
        graph,
        Box::new(ShadowDepthPass::new(
            build,
            build.queue,
            build.bindless,
            build.point_shadow_format,
        )),
        &[
            ("shadow_depth", targets.shadow_depth),
            ("spotlight_shadow_atlas", targets.spotlight_shadow_atlas),
            ("bone_matrices", targets.bone_matrices),
        ],
        PassPosition::Background,
    ) {
        panic!("the shadow depth pass could not be installed: {error}");
    }
}
