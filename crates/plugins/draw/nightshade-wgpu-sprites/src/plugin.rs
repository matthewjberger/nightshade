use nightshade::app::App;
use nightshade::plugin::Plugin;
use nightshade_wgpu::RenderAppExt;

#[derive(Default)]
pub struct SpriteRenderPlugin;

impl Plugin for SpriteRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_render_graph_config(crate::systems::sprites::install_sprite_pass);
        app.on_pre_render(crate::systems::sprites::sync_sprite_pass);
    }
}
