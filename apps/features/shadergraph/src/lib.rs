mod commands;
mod components;
mod data;
mod plugin;
mod queries;
mod resources;
mod shader;
mod systems;
mod theme;

use plugin::ShaderGraphPlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<nightshade_app::NoArguments>(nightshade_app::Opening::titled(
        theme::TITLE,
    ))
    .add_plugin(ShaderGraphPlugin)
    .run()
}

nightshade_platform::android_main!(launch);
