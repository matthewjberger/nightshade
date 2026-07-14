//! The retained UI composition. The UI subsystem itself lives in
//! [`crate::ecs::ui`] because it is core, not feature-gated; this plugin is
//! what activates it.

use crate::app::{App, Plugin};

/// Installs the retained UI: builds the UI sub-schedule and inserts the
/// frame-schedule entry that runs it, shown from the start. Hide and show
/// the whole tree at runtime through `world.resources.retained_ui.visible`.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.world.resources.retained_ui.visible = true;
        app.add_startup_system("install_retained_ui", |world| {
            world.resources.schedules.retained_ui =
                crate::schedule::build_default_retained_ui_schedule();
            crate::schedule::schedule_insert_after(
                &mut world.resources.schedules.frame,
                crate::schedule::system_names::RENDER_SYNC_SKINNING,
                crate::schedule::system_names::RETAINED_UI,
                crate::schedule::run_retained_ui_schedule,
            );
        });
    }
}
