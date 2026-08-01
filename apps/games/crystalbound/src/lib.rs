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
mod tags;
#[cfg(feature = "plugin")]
mod theme;

#[cfg(feature = "plugin")]
pub use plugin::CrystalboundPlugin;

#[cfg(feature = "plugin")]
pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<resources::Arguments>(
        nightshade_app::Opening::titled(theme::TITLE).settling_at(theme::SETTLED),
    )
    .add_plugin(nightshade_sprite::SpritePlugin)
    .add_plugin(nightshade_ui::UiPlugin::new())
    .add_plugin(CrystalboundPlugin)
    .run()
}

#[cfg(feature = "plugin")]
nightshade_platform::android_main!(launch);
