//! Typed access to the built-in passes. Each accessor pairs a pass's
//! registered name with its concrete type exactly once, so the rest of the
//! renderer works with typed pass references instead of repeating name
//! strings and downcasts at every call site.

use crate::wgpu::passes;
use crate::wgpu::render_configs::RenderInputs;
use crate::wgpu::rendergraph::{RenderGraph, render_graph_get_pass_mut};

pub const MESH_PASS: &str = "mesh_pass";
pub const SKINNED_MESH_PASS: &str = "skinned_mesh_pass";
pub const SHADOW_DEPTH_PASS: &str = "shadow_depth_pass";
pub const SKY_PASS: &str = "sky_pass";
pub const SCENE_OVERLAY_PASS: &str = "scene_overlay_pass";
pub const UI_PASS: &str = "ui_pass";
pub const POSTPROCESS_PASS: &str = "postprocess_pass";
pub const DECAL_PASS: &str = "decal_pass";
pub const CLOTH_PASS: &str = "cloth_pass";
#[cfg(feature = "egui")]
pub const EGUI_PASS: &str = "egui_pass";

fn pass_mut<'graph, P: 'static>(
    graph: &'graph mut RenderGraph<RenderInputs>,
    name: &str,
) -> Option<&'graph mut P> {
    let pass = render_graph_get_pass_mut(graph, name)?;
    (pass as &mut dyn std::any::Any).downcast_mut::<P>()
}

pub fn mesh_pass_mut(graph: &mut RenderGraph<RenderInputs>) -> Option<&mut passes::MeshPass> {
    pass_mut(graph, MESH_PASS)
}

pub fn skinned_mesh_pass_mut(
    graph: &mut RenderGraph<RenderInputs>,
) -> Option<&mut passes::geometry::SkinnedMeshPass> {
    pass_mut(graph, SKINNED_MESH_PASS)
}

pub fn shadow_depth_pass_mut(
    graph: &mut RenderGraph<RenderInputs>,
) -> Option<&mut passes::ShadowDepthPass> {
    pass_mut(graph, SHADOW_DEPTH_PASS)
}

pub fn sky_pass_mut(
    graph: &mut RenderGraph<RenderInputs>,
) -> Option<&mut passes::geometry::SkyPass> {
    pass_mut(graph, SKY_PASS)
}

pub fn scene_overlay_pass_mut(
    graph: &mut RenderGraph<RenderInputs>,
) -> Option<&mut passes::SceneOverlayPass> {
    pass_mut(graph, SCENE_OVERLAY_PASS)
}

pub fn ui_pass_mut(graph: &mut RenderGraph<RenderInputs>) -> Option<&mut passes::UiPass> {
    pass_mut(graph, UI_PASS)
}

pub fn postprocess_pass_mut(
    graph: &mut RenderGraph<RenderInputs>,
) -> Option<&mut passes::PostProcessPass> {
    pass_mut(graph, POSTPROCESS_PASS)
}

pub fn decal_pass_mut(graph: &mut RenderGraph<RenderInputs>) -> Option<&mut passes::DecalPass> {
    pass_mut(graph, DECAL_PASS)
}

pub fn cloth_pass_mut(
    graph: &mut RenderGraph<RenderInputs>,
) -> Option<&mut passes::geometry::ClothPass> {
    pass_mut(graph, CLOTH_PASS)
}

#[cfg(feature = "egui")]
pub fn egui_pass_mut(graph: &mut RenderGraph<RenderInputs>) -> Option<&mut passes::EguiPass> {
    pass_mut(graph, EGUI_PASS)
}
