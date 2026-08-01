mod commands;
mod data;
mod plugin;
mod queries;
mod resources;
mod systems;

use plugin::ExamplePlugin;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<nightshade_app::NoArguments>(nightshade_app::Opening::titled("Kit"))
        .add_plugin(nightshade_ui::UiPlugin::new())
        .add_plugin(ExamplePlugin)
        .run()
}
