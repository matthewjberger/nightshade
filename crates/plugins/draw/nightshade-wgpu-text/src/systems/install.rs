use crate::systems::text::TextPass;
use nightshade_wgpu::backend::{PassBuild, RenderGraph, RenderInputs};
use nightshade_wgpu::wgpu;

pub fn install_text_sub_pass(
    graph: &mut RenderGraph<RenderInputs>,
    build: PassBuild,
    _: wgpu::TextureFormat,
) {
    let pass = TextPass::new(build, build.hdr_format, build.depth_format);
    if !nightshade_wgpu_geometry::backend::add_overlay_sub_pass(graph, Box::new(pass)) {
        panic!("the text pass found no scene overlay to draw in");
    }
}
