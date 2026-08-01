use crate::systems::sky::SkyPass;
use nightshade_wgpu::backend::{RenderGraph, RenderInputs, render_graph_get_pass_mut};

pub const SKY_PASS: &str = "sky_pass";

pub fn sky_pass_mut(graph: &mut RenderGraph<RenderInputs>) -> Option<&mut SkyPass> {
    let pass = render_graph_get_pass_mut(graph, SKY_PASS)?;
    (pass as &mut dyn std::any::Any).downcast_mut::<SkyPass>()
}
