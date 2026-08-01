use crate::renderer::wgpu::passes;
use crate::renderer::wgpu::render_configs::RenderInputs;
use crate::renderer::wgpu::rendergraph::{RenderGraph, render_graph_get_pass_mut};

pub const POSTPROCESS_PASS: &str = "postprocess_pass";

fn pass_mut<'graph, P: 'static>(
    graph: &'graph mut RenderGraph<RenderInputs>,
    name: &str,
) -> Option<&'graph mut P> {
    let pass = render_graph_get_pass_mut(graph, name)?;
    (pass as &mut dyn std::any::Any).downcast_mut::<P>()
}

pub fn postprocess_pass_mut(
    graph: &mut RenderGraph<RenderInputs>,
) -> Option<&mut passes::PostProcessPass> {
    pass_mut(graph, POSTPROCESS_PASS)
}
