//! The demo-friendly exit key: composing it closes the app on Escape.

use crate::app::{App, Plugin};
use crate::ecs::world::World;
use crate::schedule::{FramePhase, schedule_push};

/// Registers the Escape check into the frame schedule's update phase.
pub fn install(world: &mut World) {
    schedule_push(
        &mut world.resources.schedules.frame,
        FramePhase::Update,
        crate::ecs::input::systems::escape_key_exit_system,
    );
}

/// Sets `window.should_exit` when Escape is pressed. Demos and tools
/// compose it; games that bind Escape to menus handle the key themselves.
pub struct ExitOnEscapePlugin;

impl Plugin for ExitOnEscapePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(install);
    }
}
