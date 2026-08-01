mod commands;
mod data;
mod plugin;
mod queries;
mod resources;
mod systems;

use plugin::LocomotionPlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = nightshade_app::compose::<nightshade_app::NoArguments>(
        nightshade_app::Opening::titled("Locomotion"),
    );
    app.add_plugin(nightshade_ui::UiPlugin::new())
        .add_plugin(LocomotionPlugin)
        .run()
}

nightshade_platform::android_main!(launch);
