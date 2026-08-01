use nightshade::app::App;
use nightshade::plugin::Plugin;
use nightshade_wgpu::RenderAppExt;

#[derive(Default)]
pub struct ScreenSpaceRenderPlugin;

impl Plugin for ScreenSpaceRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_render_graph_config(crate::systems::install::install_screen_space_passes);
    }
}
