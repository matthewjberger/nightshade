mod components;
mod plugin;
mod resources;
mod systems;
mod theme;

use plugin::TextStylePlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = nightshade_app::compose::<nightshade_app::NoArguments>(
        nightshade_app::Opening::titled(theme::TITLE),
    );
    app.add_plugin(nightshade_text::TextPlugin)
        .add_plugin(nightshade_ui::UiPlugin::new())
        .add_plugin(nightshade_camera_controller::CameraControllerPlugin)
        .add_plugin(TextStylePlugin)
        .run()
}

nightshade_platform::android_main!(launch);
