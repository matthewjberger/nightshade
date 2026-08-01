mod data;
mod plugin;
mod queries;
mod resources;
mod systems;

use plugin::ActionsPlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = nightshade_app::compose::<nightshade_app::NoArguments>(
        nightshade_app::Opening::titled("Actions"),
    );
    app.add_plugin(nightshade_camera_controller::CameraControllerPlugin)
        .add_plugin(nightshade_ui::UiPlugin::new())
        .add_plugin(nightshade_gamepad::GamepadPlugin)
        .add_plugin(ActionsPlugin)
        .run()
}

nightshade_platform::android_main!(launch);
