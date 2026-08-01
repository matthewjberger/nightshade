use crate::contract::PortalFrame;
use crate::resources::PortalSettings;
use crate::systems;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

pub struct PortalPlugin;

impl Plugin for PortalPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
            nightshade_render::resources::viewport::Viewport => "GraphicsPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.world.resource_or_insert_default::<PortalFrame>();
        app.world.resource_or_insert_default::<PortalSettings>();
        app.add_system(Stage::Update, systems::view::open);
        app.add_system(Stage::Update, systems::view::aim);
    }
}
