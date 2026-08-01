use crate::renderer::wgpu::render_configs::RenderInputs;
use crate::renderer::wgpu::rendergraph::{
    PassNode, PassPosition, RenderGraph, ResourceId, Result, render_graph_add_pass_at,
};

pub fn install_scene_pass(
    graph: &mut RenderGraph<RenderInputs>,
    pass: Box<dyn PassNode<RenderInputs>>,
    slot_mappings: &[(&str, ResourceId)],
) -> Result<()> {
    render_graph_add_pass_at(graph, pass, slot_mappings, PassPosition::SceneEnd)?;
    Ok(())
}

pub fn install_lit_pass(
    graph: &mut RenderGraph<RenderInputs>,
    pass: Box<dyn PassNode<RenderInputs>>,
    slot_mappings: &[(&str, ResourceId)],
) -> Result<()> {
    render_graph_add_pass_at(graph, pass, slot_mappings, PassPosition::LitOverlay)?;
    Ok(())
}

pub fn install_swapchain_pass(
    graph: &mut RenderGraph<RenderInputs>,
    pass: Box<dyn PassNode<RenderInputs>>,
    slot_mappings: &[(&str, ResourceId)],
) -> Result<()> {
    render_graph_add_pass_at(graph, pass, slot_mappings, PassPosition::Swapchain)?;
    Ok(())
}
