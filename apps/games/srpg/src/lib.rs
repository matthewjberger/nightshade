mod commands;
mod components;
mod data;
mod plugin;
mod queries;
mod resources;
mod systems;
mod tags;
mod theme;

use plugin::SrpgPlugin;
use resources::SrpgArguments;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = nightshade_app::compose::<SrpgArguments>(
        nightshade_app::Opening::titled(theme::TITLE).settling_at(theme::SETTLED),
    );
    app.add_plugin(nightshade_gamepad::GamepadPlugin)
        .add_plugin(nightshade_sprite::SpritePlugin)
        .add_plugin(nightshade_particles::ParticlePlugin)
        .add_plugin(nightshade_ui::UiPlugin::new())
        .add_plugin(SrpgPlugin)
        .run()
}

nightshade_platform::android_main!(launch);
