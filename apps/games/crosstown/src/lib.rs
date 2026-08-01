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
pub use plugin::CrosstownPlugin;

#[cfg(feature = "plugin")]
pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<resources::Arguments>(
        nightshade_app::Opening::titled(data::TITLE).settling_at(data::SETTLED as u64),
    )
    .add_plugin(nightshade_gamepad::GamepadPlugin)
    .add_plugin(nightshade_sprite::SpritePlugin)
    .add_plugin(nightshade_gltf::GltfPlugin)
    .add_plugin(nightshade_ui::UiPlugin::new())
    .add_plugin(CrosstownPlugin)
    .run()
}

#[cfg(feature = "plugin")]
nightshade_platform::android_main!(launch);
