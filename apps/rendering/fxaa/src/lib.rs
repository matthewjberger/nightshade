mod plugin;
mod resources;
mod systems;
mod theme;

use plugin::FxaaPlugin;
use resources::FxaaArguments;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = nightshade_app::compose::<FxaaArguments>(
        nightshade_app::Opening::titled(theme::TITLE).settling_at(theme::SETTLED),
    );
    app.add_plugin(nightshade_camera_controller::CameraControllerPlugin)
        .add_plugin(nightshade_ui::UiPlugin::new())
        .add_plugin(FxaaPlugin)
        .run()
}

nightshade_platform::android_main!(launch);
