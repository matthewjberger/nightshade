mod commands;
mod components;
mod data;
mod plugin;
mod queries;
mod resources;
mod systems;
mod tags;

use plugin::AshfallPlugin;
use resources::AshfallArguments;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<AshfallArguments>(nightshade_app::Opening::titled(data::theme::TITLE))
        .add_plugin(nightshade_gamepad::GamepadPlugin)
        .add_plugin(nightshade_physics::PhysicsPlugin)
        .add_plugin(nightshade_particles::ParticlePlugin)
        .add_plugin(nightshade_decals::DecalsPlugin)
        .add_plugin(nightshade_ui::UiPlugin::new())
        .add_plugin(nightshade_lofi::LofiPlugin {
            look: nightshade_lofi::playstation(),
        })
        .add_plugin(nightshade_ui_flash::FlashPlugin)
        .add_plugin(nightshade_flashlight::FlashlightPlugin {
            settings: data::rules::flashlight_settings(),
        })
        .add_plugin(nightshade_flicker::FlickerPlugin {
            settings: data::rules::flicker_settings(),
        })
        .add_plugin(nightshade_senses::SensesPlugin {
            settings: data::rules::senses_settings(),
        })
        .add_plugin(nightshade_phase::PhasePlugin {
            looks: data::theme::phase_looks(),
            start: 0,
        })
        .add_plugin(AshfallPlugin)
        .run()
}

nightshade_platform::android_main!(launch);
