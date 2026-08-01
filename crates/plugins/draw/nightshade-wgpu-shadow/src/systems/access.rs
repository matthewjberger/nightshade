use crate::systems::shadow_depth::ShadowDepthPass;
use nightshade_wgpu::backend::{RenderGraph, RenderInputs, render_graph_get_pass_mut};

pub const SHADOW_DEPTH_PASS: &str = "shadow_depth_pass";

pub fn shadow_depth_pass_mut(
    graph: &mut RenderGraph<RenderInputs>,
) -> Option<&mut ShadowDepthPass> {
    let pass = render_graph_get_pass_mut(graph, SHADOW_DEPTH_PASS)?;
    (pass as &mut dyn std::any::Any).downcast_mut::<ShadowDepthPass>()
}
