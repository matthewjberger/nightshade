//! Typed access to the built-in passes. Each accessor pairs a pass's
//! registered name with its concrete type exactly once, so the rest of the
//! renderer works with typed pass references instead of repeating name
//! strings and downcasts at every call site.

use crate::wgpu::passes;
use crate::wgpu::render_configs::RenderInputs;
use crate::wgpu::rendergraph::{RenderGraph, render_graph_get_pass_mut};

/// Registered name of the mesh pass.
pub const MESH_PASS: &str = "mesh_pass";
/// Registered name of the skinned mesh pass.
pub const SKINNED_MESH_PASS: &str = "skinned_mesh_pass";
/// Registered name of the meshlet pass.
#[cfg(feature = "meshlet")]
pub const MESHLET_PASS: &str = "meshlet_pass";
/// Registered name of the shadow depth pass.
pub const SHADOW_DEPTH_PASS: &str = "shadow_depth_pass";
/// Registered name of the sky pass.
pub const SKY_PASS: &str = "sky_pass";
/// Registered name of the scene overlay pass.
pub const SCENE_OVERLAY_PASS: &str = "scene_overlay_pass";
/// Registered name of the UI pass.
pub const UI_PASS: &str = "ui_pass";
/// Registered name of the post-process pass.
pub const POSTPROCESS_PASS: &str = "postprocess_pass";
/// Registered name of the decal pass.
pub const DECAL_PASS: &str = "decal_pass";
/// Registered name of the cloth pass.
pub const CLOTH_PASS: &str = "cloth_pass";
/// Registered name of the egui pass.
#[cfg(feature = "egui")]
pub const EGUI_PASS: &str = "egui_pass";

fn pass_mut<'graph, P: 'static>(
    graph: &'graph mut RenderGraph<RenderInputs>,
    name: &str,
) -> Option<&'graph mut P> {
    let pass = render_graph_get_pass_mut(graph, name)?;
    (pass as &mut dyn std::any::Any).downcast_mut::<P>()
}

/// Returns the typed mesh pass, or `None` if it is not in the graph.
pub fn mesh_pass_mut(graph: &mut RenderGraph<RenderInputs>) -> Option<&mut passes::MeshPass> {
    pass_mut(graph, MESH_PASS)
}

/// Returns the typed skinned mesh pass, or `None` if it is not in the graph.
pub fn skinned_mesh_pass_mut(
    graph: &mut RenderGraph<RenderInputs>,
) -> Option<&mut passes::geometry::SkinnedMeshPass> {
    pass_mut(graph, SKINNED_MESH_PASS)
}

/// Returns the typed meshlet pass, or `None` if it is not in the graph.
#[cfg(feature = "meshlet")]
pub fn meshlet_pass_mut(graph: &mut RenderGraph<RenderInputs>) -> Option<&mut passes::MeshletPass> {
    pass_mut(graph, MESHLET_PASS)
}

/// Returns the typed shadow depth pass, or `None` if it is not in the graph.
pub fn shadow_depth_pass_mut(
    graph: &mut RenderGraph<RenderInputs>,
) -> Option<&mut passes::ShadowDepthPass> {
    pass_mut(graph, SHADOW_DEPTH_PASS)
}

/// Returns the typed sky pass, or `None` if it is not in the graph.
pub fn sky_pass_mut(
    graph: &mut RenderGraph<RenderInputs>,
) -> Option<&mut passes::geometry::SkyPass> {
    pass_mut(graph, SKY_PASS)
}

/// Returns the typed scene overlay pass, or `None` if it is not in the graph.
pub fn scene_overlay_pass_mut(
    graph: &mut RenderGraph<RenderInputs>,
) -> Option<&mut passes::SceneOverlayPass> {
    pass_mut(graph, SCENE_OVERLAY_PASS)
}

/// Returns the typed UI pass, or `None` if it is not in the graph.
pub fn ui_pass_mut(graph: &mut RenderGraph<RenderInputs>) -> Option<&mut passes::PaintPass> {
    pass_mut(graph, UI_PASS)
}

/// Returns the typed post-process pass, or `None` if it is not in the graph.
pub fn postprocess_pass_mut(
    graph: &mut RenderGraph<RenderInputs>,
) -> Option<&mut passes::PostProcessPass> {
    pass_mut(graph, POSTPROCESS_PASS)
}

/// Returns the typed decal pass, or `None` if it is not in the graph.
pub fn decal_pass_mut(graph: &mut RenderGraph<RenderInputs>) -> Option<&mut passes::DecalPass> {
    pass_mut(graph, DECAL_PASS)
}

/// Returns the typed cloth pass, or `None` if it is not in the graph.
pub fn cloth_pass_mut(
    graph: &mut RenderGraph<RenderInputs>,
) -> Option<&mut passes::geometry::ClothPass> {
    pass_mut(graph, CLOTH_PASS)
}

/// Returns the typed egui pass, or `None` if it is not in the graph.
#[cfg(feature = "egui")]
pub fn egui_pass_mut(graph: &mut RenderGraph<RenderInputs>) -> Option<&mut passes::EguiPass> {
    pass_mut(graph, EGUI_PASS)
}
