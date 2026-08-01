mod plugin;
mod queries;
mod resources;
mod systems;
mod theme;

use plugin::TracedBouncePlugin;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = nightshade_app::compose::<nightshade_app::NoArguments>(
        nightshade_app::Opening::titled(theme::TITLE),
    );
    app.add_plugin(nightshade_camera_controller::CameraControllerPlugin)
        .add_plugin(TracedBouncePlugin)
        .run()
}
