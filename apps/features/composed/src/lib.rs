mod commands;
mod data;
mod plugin;
mod queries;
mod resources;
mod systems;
mod theme;

use plugin::ComposedPlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<nightshade_app::NoArguments>(nightshade_app::Opening::titled(
        theme::TITLE,
    ))
    .add_plugin(nightshade_ui::UiPlugin::new())
    .add_plugin(ComposedPlugin)
    .run()
}

nightshade_platform::android_main!(launch);
