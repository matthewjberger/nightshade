use crate::systems::edge::OutlinePass;
use crate::systems::mask::OutlineMaskPass;
use nightshade_wgpu::backend::{
    PassBuild, PassPosition, RenderGraph, RenderInputs, render_graph_add_pass_at,
};
use nightshade_wgpu::wgpu;

pub fn install_outline_passes(
    graph: &mut RenderGraph<RenderInputs>,
    build: PassBuild,
    _: wgpu::TextureFormat,
) {
    let targets = build.targets;

    if let Err(error) = render_graph_add_pass_at(
        graph,
        Box::new(OutlineMaskPass::new(build)),
        &[
            ("outline_mask", targets.outline_mask),
            ("entity_id", targets.entity_id),
        ],
        PassPosition::PostProcess,
    ) {
        panic!("the outline mask pass could not be installed: {error}");
    }

    if let Err(error) = render_graph_add_pass_at(
        graph,
        Box::new(OutlinePass::new(build, build.hdr_format)),
        &[
            ("outline_mask", targets.outline_mask),
            ("color", targets.scene_color),
        ],
        PassPosition::PostProcess,
    ) {
        panic!("the outline pass could not be installed: {error}");
    }
}
