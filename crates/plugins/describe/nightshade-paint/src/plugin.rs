use nightshade::app::App;
use nightshade::plugin::Plugin;

pub struct PaintPlugin;

impl Plugin for PaintPlugin {
    fn build(&self, app: &mut App) {
        crate::commands::install_paint_resources(&mut app.world.resources);
    }
}
