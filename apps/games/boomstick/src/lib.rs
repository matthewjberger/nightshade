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
pub mod resources;
#[cfg(feature = "plugin")]
mod systems;
mod tags;

#[cfg(feature = "plugin")]
pub use plugin::BoomstickPlugin;

#[cfg(feature = "plugin")]
pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<resources::Arguments>(nightshade_app::Opening::titled("Boomstick"))
        .add_plugin(nightshade_physics::PhysicsPlugin)
        .add_plugin(nightshade_animation::AnimationPlugin)
        .add_plugin(nightshade_particles::ParticlePlugin)
        .add_plugin(nightshade_gltf::GltfPlugin)
        .add_plugin(nightshade_gamepad::GamepadPlugin)
        .add_plugin(nightshade_ui::UiPlugin::new())
        .add_plugin(BoomstickPlugin)
        .run()
}

#[cfg(feature = "plugin")]
nightshade_platform::android_main!(launch);
