//! The standard plugin bundle composed above the [`App`](crate::app::App)
//! core and the capability plugins it names.

use crate::app::{PluginGroup, PluginGroupBuilder, WindowPlugin};

/// The standard composition: a visible window, logging, the renderer, and
/// the UI systems. The capability plugins in [`crate::plugins`] stay
/// opt-in: physics because most apps drive it per screen, and audio,
/// gamepad, and egui so a program only acquires the devices and overlays
/// it composes. Customize members at the call site with
/// [`set`](PluginGroup::set) and [`disable`](PluginGroup::disable).
pub struct DefaultPlugins;

impl PluginGroup for DefaultPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::new()
            .add_plugin(WindowPlugin::default())
            .add_plugin(crate::plugins::log::LogPlugin::default())
            .add_plugin(crate::plugins::render::RenderPlugin::default())
            .add_plugin(crate::plugins::ui::UiPlugin)
    }
}
