use nightshade::app::App;
use nightshade::plugin::Plugin;
use nightshade_wgpu::RenderAppExt;

#[derive(Default)]
pub struct ShadowRenderPlugin;

impl Plugin for ShadowRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_render_graph_config(crate::systems::install::install_shadow_depth_pass);
    }
}
