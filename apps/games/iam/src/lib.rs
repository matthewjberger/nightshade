mod components;
mod data;
mod plugin;
mod resources;
mod systems;
#[cfg(test)]
mod tests;

use plugin::GamePlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<nightshade_app::NoArguments>(nightshade_app::Opening::titled(
        data::office::TITLE,
    ))
    .add_plugin(nightshade_ui::UiPlugin::new())
    .add_plugin(GamePlugin)
    .run()
}

nightshade_platform::android_main!(launch);
