use nightshade::app::App;
use nightshade::plugin::Plugin;
use nightshade_wgpu::RenderAppExt;

#[derive(Default)]
pub struct SkyRenderPlugin;

impl Plugin for SkyRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_render_graph_config(crate::systems::install::install_sky_pass);
        app.on_frame_inputs(crate::systems::ibl::sync_image_lighting);
    }
}
