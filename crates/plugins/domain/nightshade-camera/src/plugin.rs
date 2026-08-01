use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::window::Window => "PlatformPlugin",
            nightshade_render::data::config::RenderSettings => "GraphicsPlugin",
            nightshade_render::resources::viewport::Viewport => "GraphicsPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.insert_resource(crate::resources::ActiveCamera::default());
        app.world
            .resource_or_insert_default::<nightshade_render::contract::RenderViews>();
        app.add_system(Stage::Render, crate::systems::views::publish_render_views);
    }
}
