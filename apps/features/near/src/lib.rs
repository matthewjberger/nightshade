mod commands;
mod plugin;
mod resources;
mod systems;
mod theme;

use plugin::NearExamplePlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<nightshade_app::NoArguments>(nightshade_app::Opening::titled(
        theme::TITLE,
    ))
    .add_plugin(NearExamplePlugin)
    .run()
}

nightshade_platform::android_main!(launch);
