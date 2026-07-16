//! The demo-friendly exit key: composing it closes the app on Escape.

use crate::app::{App, Plugin, Stage};

/// Sets `window.should_exit` when Escape is pressed. Demos and tools
/// compose it; games that bind Escape to menus handle the key themselves.
pub struct ExitOnEscapePlugin;

impl Plugin for ExitOnEscapePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            Stage::FrameUpdate,
            crate::ecs::input::systems::escape_key_exit_system,
        );
    }
}
