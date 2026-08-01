mod plugin;
mod systems;
mod theme;

use plugin::CobblePlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<nightshade_app::NoArguments>(
        nightshade_app::Opening::titled(theme::TITLE).settling_at(blocker_screen::theme::SETTLED),
    )
    .add_plugin(nightshade_gamepad::GamepadPlugin)
    .add_plugin(nightshade_sprite::SpritePlugin)
    .add_plugin(nightshade_particles::ParticlePlugin)
    .add_plugin(CobblePlugin)
    .run()
}

nightshade_platform::android_main!(launch);
