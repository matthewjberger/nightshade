use crate::systems::geometry::{MeshPass, SceneOverlayPass, SkinnedMeshPass};
use nightshade_wgpu::backend::{RenderGraph, RenderInputs, render_graph_get_pass_mut};

pub const MESH_PASS: &str = "mesh_pass";
pub const SKINNED_MESH_PASS: &str = "skinned_mesh_pass";
pub const SCENE_OVERLAY_PASS: &str = "scene_overlay_pass";

fn pass_mut<'graph, P: 'static>(
    graph: &'graph mut RenderGraph<RenderInputs>,
    name: &str,
) -> Option<&'graph mut P> {
    let pass = render_graph_get_pass_mut(graph, name)?;
    (pass as &mut dyn std::any::Any).downcast_mut::<P>()
}

pub fn mesh_pass_mut(graph: &mut RenderGraph<RenderInputs>) -> Option<&mut MeshPass> {
    pass_mut(graph, MESH_PASS)
}

pub fn skinned_mesh_pass_mut(
    graph: &mut RenderGraph<RenderInputs>,
) -> Option<&mut SkinnedMeshPass> {
    pass_mut(graph, SKINNED_MESH_PASS)
}

pub fn scene_overlay_pass_mut(
    graph: &mut RenderGraph<RenderInputs>,
) -> Option<&mut SceneOverlayPass> {
    pass_mut(graph, SCENE_OVERLAY_PASS)
}

pub fn overlay_sub_pass_mut<P: crate::systems::geometry::OverlaySubPass>(
    graph: &mut RenderGraph<RenderInputs>,
) -> Option<&mut P> {
    scene_overlay_pass_mut(graph)?.sub_pass_mut::<P>()
}

pub fn add_overlay_sub_pass(
    graph: &mut RenderGraph<RenderInputs>,
    sub_pass: Box<dyn crate::systems::geometry::OverlaySubPass>,
) -> bool {
    let Some(overlay) = scene_overlay_pass_mut(graph) else {
        return false;
    };
    overlay.sub_passes.push(sub_pass);
    true
}
