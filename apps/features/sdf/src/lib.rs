mod commands;
mod components;
mod data;
mod plugin;
mod queries;
mod resources;
mod systems;
mod theme;

use plugin::SdfDemoPlugin;
use resources::SdfArguments;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = nightshade_app::compose::<SdfArguments>(
        nightshade_app::Opening::titled(theme::TITLE).settling_at(theme::SETTLED),
    );
    app.add_plugin(nightshade_camera_controller::CameraControllerPlugin)
        .add_plugin(nightshade_ui::UiPlugin::new())
        .add_plugin(nightshade_sdf::SdfPlugin)
        .add_plugin(nightshade_physics::PhysicsPlugin)
        .add_plugin(SdfDemoPlugin)
        .run()
}

nightshade_platform::android_main!(launch);
