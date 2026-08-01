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
mod theme;

#[cfg(feature = "plugin")]
pub use plugin::TetherPlugin;

#[cfg(feature = "plugin")]
#[cfg(feature = "plugin")]
pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<nightshade_app::NoArguments>(nightshade_app::Opening::titled(
        "Tether",
    ))
    .add_plugin(nightshade_physics::PhysicsPlugin)
    .add_plugin(nightshade_ui::UiPlugin::new())
    .add_plugin(nightshade_gamepad::GamepadPlugin)
    .add_plugin(TetherPlugin)
    .run()
}

#[cfg(feature = "plugin")]
nightshade_platform::android_main!(launch);
