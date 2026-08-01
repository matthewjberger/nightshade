use nightshade_wgpu::backend::PassBuild;
use nightshade_wgpu::wgpu;
mod images;
mod rect_text;

pub use images::PaintImagePass;
pub use rect_text::PaintPass;

pub(super) const PAINT_PASS: &str = "paint_pass";
pub(super) const PAINT_IMAGE_PASS: &str = "paint_image_pass";

pub fn install_paint_passes_into(
    graph: &mut nightshade_wgpu::backend::RenderGraph<nightshade_wgpu::backend::RenderInputs>,
    build: PassBuild,
    format: wgpu::TextureFormat,
    color: nightshade_wgpu::backend::ResourceId,
    depth: nightshade_wgpu::backend::ResourceId,
) {
    let slots = [("color", color), ("depth", depth)];
    let text_slots = vec![
        ("color", color),
        ("depth", depth),
        ("glyph_atlas", build.targets.glyph_atlas),
    ];
    if let Err(error) = nightshade_wgpu::backend::install_swapchain_pass(
        graph,
        Box::new(PaintPass::new(build, format)),
        &text_slots,
    ) {
        panic!("the paint pass could not be installed: {error}");
    }
    if let Err(error) = nightshade_wgpu::backend::install_swapchain_pass(
        graph,
        Box::new(PaintImagePass::new(build, format)),
        &slots,
    ) {
        panic!("the paint image pass could not be installed: {error}");
    }
}

pub fn paint_pass_mut(
    renderer: &mut nightshade_wgpu::backend::WgpuRenderer,
) -> Option<&mut PaintPass> {
    pass_mut(renderer, PAINT_PASS)
}

pub fn paint_image_pass_mut(
    renderer: &mut nightshade_wgpu::backend::WgpuRenderer,
) -> Option<&mut PaintImagePass> {
    pass_mut(renderer, PAINT_IMAGE_PASS)
}

fn pass_mut<'graph, P: 'static>(
    renderer: &'graph mut nightshade_wgpu::backend::WgpuRenderer,
    name: &str,
) -> Option<&'graph mut P> {
    let pass = nightshade_wgpu::backend::render_graph_get_pass_mut(&mut renderer.graph, name)?;
    (pass as &mut dyn std::any::Any).downcast_mut::<P>()
}
