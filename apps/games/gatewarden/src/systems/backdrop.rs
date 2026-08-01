use crate::forest::{FOREST_PASS, ForestPass};
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub fn install_forest_pass(
    graph: &mut nightshade_wgpu::backend::RenderGraph<nightshade_wgpu::backend::RenderInputs>,
    build: nightshade_wgpu::backend::PassBuild,
    _color_format: nightshade_wgpu::wgpu::TextureFormat,
) {
    let color = build.targets.scene_color;
    let depth = build.targets.depth;
    let pass = ForestPass::new(build);
    if let Err(error) = nightshade_wgpu::backend::install_lit_pass(
        graph,
        Box::new(pass),
        &[("color", color), ("depth", depth)],
    ) {
        tracing::error!("forest backdrop not installed: {error}");
    }
}

pub fn advance_forest(mut renderer: nightshade_wgpu::pre_render::Renderer, time: Res<Time>) {
    let Some(pass) =
        nightshade_wgpu::backend::render_graph_get_pass_mut(&mut renderer.graph, FOREST_PASS)
    else {
        return;
    };
    let Some(pass) = (pass as &mut dyn std::any::Any).downcast_mut::<ForestPass>() else {
        return;
    };
    pass.time += time.delta_time;
}
