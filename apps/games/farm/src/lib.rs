mod commands;
mod components;
mod data;
mod plugin;
mod queries;
mod resources;
mod systems;
mod tags;
mod theme;

use plugin::Hd2dPlugin;
use resources::Hd2dArguments;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = nightshade_app::compose::<Hd2dArguments>(
        nightshade_app::Opening::titled(theme::TITLE).settling_at(theme::SETTLED),
    );
    app.add_plugin(nightshade_gamepad::GamepadPlugin)
        .add_plugin(nightshade_sprite::SpritePlugin)
        .add_plugin(nightshade_particles::ParticlePlugin)
        .add_plugin(nightshade_ui::UiPlugin::new())
        .add_plugin(Hd2dPlugin)
        .run()
}

nightshade_platform::android_main!(launch);
