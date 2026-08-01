#[cfg(feature = "plugin")]
pub mod commands;
#[cfg(feature = "plugin")]
pub mod data;
#[cfg(feature = "plugin")]
mod plugin;
#[cfg(feature = "plugin")]
pub mod queries;
#[cfg(feature = "plugin")]
pub mod resources;
#[cfg(feature = "plugin")]
mod systems;

#[cfg(feature = "plugin")]
pub use plugin::DescriptionPlugin;

#[cfg(feature = "plugin")]
pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<resources::Arguments>(nightshade_app::Opening::titled(data::TITLE))
        .add_plugin(nightshade_ui::UiPlugin::new())
        .add_plugin(nightshade_gltf::GltfPlugin)
        .add_plugin(nightshade_gamepad::GamepadPlugin)
        .add_plugin(DescriptionPlugin)
        .run()
}

#[cfg(feature = "plugin")]
nightshade_platform::android_main!(launch);
