use nightshade::app::App;
use nightshade::plugin::Plugin;
use nightshade_wgpu::RenderAppExt;

#[derive(Default)]
pub struct SdfRenderPlugin;

impl Plugin for SdfRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_render_graph_config(crate::systems::sdf::install_sdf_pass);
        app.on_pre_render(crate::systems::sdf::sync_sdf_pass);
    }
}
