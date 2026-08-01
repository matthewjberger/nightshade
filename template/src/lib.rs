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
pub use plugin::TemplatePlugin;

#[cfg(feature = "plugin")]
use nightshade::prelude::PluginGroup;

#[cfg(feature = "plugin")]
pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = nightshade::prelude::App::new();
    app.add_plugins(
        nightshade_wgpu::DefaultPlugins::<nightshade_cli::NoArguments>::new().set(
            nightshade_log::LogPlugin {
                title: Some("Template".to_string()),
                ..Default::default()
            },
        ),
    );
    #[cfg(feature = "vr")]
    app.add_plugin(nightshade_openxr::OpenXrPlugin::default());
    app.add_plugin(nightshade_camera_controller::CameraControllerPlugin)
        .add_plugin(nightshade_ui::UiPlugin::new())
        .add_plugin(TemplatePlugin)
        .run()
}

#[cfg(feature = "plugin")]
nightshade_platform::android_main!(launch);
