mod data;
mod plugin;
mod resources;
mod systems;
mod theme;

use plugin::SponzaPlugin;
use resources::SponzaArguments;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let mut app =
        nightshade_app::compose::<SponzaArguments>(nightshade_app::Opening::titled(theme::TITLE));
    app.add_plugin(nightshade_gamepad::GamepadPlugin)
        .add_plugin(nightshade_physics::PhysicsPlugin)
        .add_plugin(nightshade_gltf::GltfPlugin)
        .add_plugin(nightshade_ui::UiPlugin::new())
        .add_plugin(SponzaPlugin)
        .run()
}

nightshade_platform::android_main!(launch);
