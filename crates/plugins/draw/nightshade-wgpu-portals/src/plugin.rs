use nightshade::app::App;
use nightshade::plugin::Plugin;
use nightshade_wgpu::RenderAppExt;

#[derive(Default)]
pub struct PortalRenderPlugin;

impl Plugin for PortalRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_render_graph_config(crate::systems::portals::install_portal_pass);
        app.on_pre_render(crate::systems::portals::sync_portal_pass);
    }
}
