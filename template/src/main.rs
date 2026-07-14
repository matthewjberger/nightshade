use nightshade::prelude::{App, CameraControllerPlugin, DefaultPlugins};
use template_core::TemplatePlugin;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(CameraControllerPlugin)
        .add_plugin(TemplatePlugin)
        .run()
}
