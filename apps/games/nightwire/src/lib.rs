#[cfg(feature = "contract")]
pub mod contract;

#[cfg(feature = "plugin")]
pub mod commands;
#[cfg(feature = "plugin")]
pub mod components;
#[cfg(feature = "plugin")]
pub mod data;
#[cfg(feature = "plugin")]
pub mod glass;
#[cfg(feature = "plugin")]
mod plugin;
#[cfg(feature = "plugin")]
pub mod queries;
#[cfg(feature = "plugin")]
pub mod resources;
#[cfg(feature = "plugin")]
mod systems;
#[cfg(feature = "plugin")]
pub mod theme;

#[cfg(feature = "plugin")]
pub use plugin::NightwirePlugin;

#[cfg(feature = "plugin")]
pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<crate::resources::Arguments>(nightshade_app::Opening::titled(
        "Nightwire",
    ))
    .add_plugin(nightshade_ui::UiPlugin::new())
    .add_plugin(NightwirePlugin)
    .run()
}

#[cfg(feature = "plugin")]
nightshade_platform::android_main!(launch);
