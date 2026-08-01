mod curtain;
mod data;
mod plugin;
mod resources;
mod systems;
mod theme;

use plugin::BlockerPlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<nightshade_app::NoArguments>(
        nightshade_app::Opening::titled(blocker_hud::theme::TITLE)
            .settling_at(blocker_screen::theme::SETTLED),
    )
    .add_plugin(nightshade_gamepad::GamepadPlugin)
    .add_plugin(nightshade_gltf::GltfPlugin)
    .add_plugin(nightshade_sprite::SpritePlugin)
    .add_plugin(nightshade_particles::ParticlePlugin)
    .add_plugin(nightshade_text::TextPlugin)
    .add_plugin(nightshade_ui::UiPlugin::new())
    .add_plugin(BlockerPlugin)
    .run()
}

nightshade_platform::android_main!(launch);
