mod plugin;
mod systems;
mod theme;

use plugin::ExamplePlugin;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<nightshade_app::NoArguments>(nightshade_app::Opening::titled(
        theme::TITLE,
    ))
    .add_plugin(nightshade_ui::UiPlugin::new())
    .add_plugin(ExamplePlugin)
    .run()
}
