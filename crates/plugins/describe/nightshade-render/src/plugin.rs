use nightshade::app::App;
use nightshade::plugin::Plugin;

pub struct RenderDescriptionPlugin;

impl Plugin for RenderDescriptionPlugin {
    fn build(&self, app: &mut App) {
        crate::commands::install_render_resources(&mut app.world.resources);
    }
}
