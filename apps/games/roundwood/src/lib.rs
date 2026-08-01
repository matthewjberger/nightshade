mod commands;
mod components;
mod data;
mod plugin;
mod queries;
mod resources;
mod systems;
mod tags;

use plugin::RoundwoodPlugin;
use resources::Arguments;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<Arguments>(nightshade_app::Opening::titled(data::TITLE))
        .add_plugin(nightshade_sprite::SpritePlugin)
        .add_plugin(nightshade_particles::ParticlePlugin)
        .add_plugin(nightshade_ui::UiPlugin::new())
        .add_plugin(RoundwoodPlugin)
        .run()
}

nightshade_platform::android_main!(launch);
