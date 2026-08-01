mod commands;
mod components;
mod data;
mod plugin;
mod queries;
mod resources;
mod systems;
mod tags;
mod theme;

pub use systems::generation::digest_the_world;

use plugin::VoxelsPlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<nightshade_app::NoArguments>(
        nightshade_app::Opening::titled(theme::TITLE).settling_at(theme::SETTLED),
    )
    .add_plugin(nightshade_gamepad::GamepadPlugin)
    .add_plugin(nightshade_sprite::SpritePlugin)
    .add_plugin(nightshade_particles::ParticlePlugin)
    .add_plugin(nightshade_text::TextPlugin)
    .add_plugin(nightshade_ui::UiPlugin::new())
    .add_plugin(VoxelsPlugin)
    .run()
}

nightshade_platform::android_main!(launch);
