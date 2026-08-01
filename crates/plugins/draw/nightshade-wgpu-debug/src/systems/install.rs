use crate::systems::grid::GridPass;
use crate::systems::lines::LinesPass;
use nightshade_wgpu::backend::{PassBuild, RenderGraph, RenderInputs};
use nightshade_wgpu::wgpu;
use nightshade_wgpu_geometry::backend::add_overlay_sub_pass;

pub fn install_debug_sub_passes(
    graph: &mut RenderGraph<RenderInputs>,
    build: PassBuild,
    _: wgpu::TextureFormat,
) {
    let grid = GridPass::new(build, build.hdr_format, build.depth_format);
    if !add_overlay_sub_pass(graph, Box::new(grid)) {
        panic!("the grid pass found no scene overlay to draw in");
    }

    let lines = LinesPass::new(build, build.hdr_format);
    if !add_overlay_sub_pass(graph, Box::new(lines)) {
        panic!("the lines pass found no scene overlay to draw in");
    }
}
