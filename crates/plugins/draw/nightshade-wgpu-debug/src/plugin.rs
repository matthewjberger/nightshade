use nightshade::app::App;
use nightshade::plugin::Plugin;
use nightshade_wgpu::RenderAppExt;

#[derive(Default)]
pub struct DebugRenderPlugin;

impl Plugin for DebugRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_render_graph_sub_pass(crate::systems::install::install_debug_sub_passes);
        app.on_frame_inputs(crate::systems::frame::sync_debug_lines);
    }
}
