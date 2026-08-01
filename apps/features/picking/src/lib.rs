mod components;
mod plugin;
mod resources;
mod systems;
mod theme;

use plugin::PickingPlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = nightshade_app::compose::<nightshade_app::NoArguments>(
        nightshade_app::Opening::titled("Picking Demo"),
    );
    app.add_plugin(nightshade_camera_controller::CameraControllerPlugin)
        .add_plugin(nightshade_ui::UiPlugin::new())
        .add_plugin(PickingPlugin)
        .run()
}

nightshade_platform::android_main!(launch);
