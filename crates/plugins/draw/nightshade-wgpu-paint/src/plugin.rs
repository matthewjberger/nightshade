use nightshade::app::App;
use nightshade::plugin::Plugin;
use nightshade_wgpu::RenderAppExt;

#[derive(Default)]
pub struct PaintRenderPlugin;

impl Plugin for PaintRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(nightshade_paint::PaintPlugin);
        app.add_render_graph_config(crate::systems::frame::install_paint_passes);
        app.on_pre_render(crate::systems::frame::sync_paint_passes);
    }
}
