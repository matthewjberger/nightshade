use crate::resources::{Compiled, Pictures, Preview};
use crate::shader::{GRAPH_PASS, GraphPass};
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub fn install_graph_pass(
    graph: &mut nightshade_wgpu::backend::RenderGraph<nightshade_wgpu::backend::RenderInputs>,
    build: nightshade_wgpu::backend::PassBuild,
    _color_format: nightshade_wgpu::wgpu::TextureFormat,
) {
    let color = build.targets.scene_color;
    let depth = build.targets.depth;
    let pass = GraphPass::new(build);
    if let Err(error) = nightshade_wgpu::backend::install_lit_pass(
        graph,
        Box::new(pass),
        &[("color", color), ("depth", depth)],
    ) {
        tracing::error!("shader graph pass not installed: {error}");
    }
}

pub fn feed_the_pass(
    mut renderer: nightshade_wgpu::pre_render::Renderer,
    time: Res<Time>,
    compiled: Res<Compiled>,
    preview: Res<Preview>,
    pictures: Res<Pictures>,
) {
    let Some(pass) =
        nightshade_wgpu::backend::render_graph_get_pass_mut(&mut renderer.graph, GRAPH_PASS)
    else {
        return;
    };
    let Some(pass) = (pass as &mut dyn std::any::Any).downcast_mut::<GraphPass>() else {
        return;
    };
    pass.time += time.delta_time;
    pass.take_the_body(compiled.source.clone(), compiled.revision);
    pass.take_the_inset(preview.left, preview.right, preview.top, preview.bottom);
    pass.take_the_pictures(&pictures.held, pictures.revision);
}
