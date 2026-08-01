mod commands;
mod components;
mod contract;
mod data;
mod plugin;
mod queries;
mod resources;
mod systems;
mod tags;
mod theme;

pub use systems::generation::digest_the_world;

use plugin::HollowmerePlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<nightshade_app::NoArguments>(
        nightshade_app::Opening::titled(theme::TITLE).settling_at(theme::SETTLED),
    )
    .add_plugin(nightshade_sprite::SpritePlugin)
    .add_plugin(nightshade_particles::ParticlePlugin)
    .add_plugin(nightshade_text::TextPlugin)
    .add_plugin(nightshade_animation::AnimationPlugin)
    .add_plugin(nightshade_gltf::GltfPlugin)
    .add_plugin(nightshade_audio::AudioPlugin)
    .add_plugin(nightshade_ui::UiPlugin::new())
    .add_plugin(HollowmerePlugin)
    .run()
}

nightshade_platform::android_main!(launch);
