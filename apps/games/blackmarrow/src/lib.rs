#[cfg(feature = "contract")]
pub mod contract;

#[cfg(feature = "plugin")]
pub mod commands;
#[cfg(feature = "plugin")]
mod components;
#[cfg(feature = "plugin")]
mod data;
#[cfg(feature = "plugin")]
mod plugin;
#[cfg(feature = "plugin")]
pub mod queries;
#[cfg(feature = "plugin")]
mod resources;
#[cfg(feature = "plugin")]
mod systems;
#[cfg(feature = "plugin")]
mod tags;

#[cfg(feature = "plugin")]
pub use plugin::BlackmarrowPlugin;

#[cfg(feature = "plugin")]
pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<resources::Arguments>(
        nightshade_app::Opening::titled(data::theme::TITLE).settling_at(240),
    )
    .add_plugin(nightshade_physics::PhysicsPlugin)
    .add_plugin(nightshade_animation::AnimationPlugin)
    .add_plugin(nightshade_gltf::GltfPlugin)
    .add_plugin(nightshade_ui::UiPlugin::new())
    .add_plugin(nightshade_lofi::LofiPlugin {
        look: data::theme::look(),
    })
    .add_plugin(nightshade_flashlight::FlashlightPlugin {
        settings: data::rules::flashlight_settings(),
    })
    .add_plugin(nightshade_flicker::FlickerPlugin::default())
    .add_plugin(nightshade_senses::SensesPlugin {
        settings: data::rules::senses_settings(),
    })
    .add_plugin(BlackmarrowPlugin)
    .run()
}

#[cfg(feature = "plugin")]
nightshade_platform::android_main!(launch);
