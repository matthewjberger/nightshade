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
pub use plugin::HearthsidePlugin;

#[cfg(feature = "plugin")]
#[cfg(feature = "plugin")]
pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = nightshade_app::compose::<resources::Arguments>(nightshade_app::Opening::titled(
        "Hearthside",
    ));
    app.add_plugin(nightshade_camera_controller::CameraControllerPlugin)
        .add_plugin(nightshade_gltf::GltfPlugin)
        .add_plugin(nightshade_ui::UiPlugin::new())
        .add_plugin(HearthsidePlugin)
        .run()
}

#[cfg(feature = "plugin")]
nightshade_platform::android_main!(launch);
