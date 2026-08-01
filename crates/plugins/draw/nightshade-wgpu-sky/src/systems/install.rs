use crate::systems::sky::SkyPass;
use nightshade_wgpu::backend::{
    PassBuild, PassPosition, RenderGraph, RenderInputs, render_graph_add_pass_at,
};
use nightshade_wgpu::wgpu;

pub fn install_sky_pass(
    graph: &mut RenderGraph<RenderInputs>,
    build: PassBuild,
    _: wgpu::TextureFormat,
) {
    if let Err(error) = render_graph_add_pass_at(
        graph,
        Box::new(SkyPass::new(
            build,
            build.hdr_format,
            build.depth_format,
            build.low_power,
        )),
        &[("color", build.targets.scene_color)],
        PassPosition::Background,
    ) {
        panic!("the sky pass could not be installed: {error}");
    }
}
