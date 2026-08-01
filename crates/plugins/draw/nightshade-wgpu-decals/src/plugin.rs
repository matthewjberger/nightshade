use nightshade::app::App;
use nightshade::plugin::Plugin;
use nightshade_wgpu::RenderAppExt;

#[derive(Default)]
pub struct DecalRenderPlugin;

impl Plugin for DecalRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_render_graph_config(crate::systems::decals::install_decal_pass);
        app.on_pre_render(crate::systems::decals::sync_decal_pass);
    }
}
