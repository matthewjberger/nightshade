//! The retained UI composition: the ordered UI pipeline, the sub-schedule
//! runner, and [`UiPlugin`]. The UI subsystem itself lives in
//! [`crate::ui`] because it is core, not feature-gated; this plugin
//! is what activates it.

use crate::app::{App, Plugin, Stage};
use crate::ecs::world::World;
use crate::schedule::SystemFn;

/// The UI pipeline in run order. Gamepad navigation has to run after
/// picking: picking resets every entity's `interaction.clicked` flag at
/// the top of the frame, so running navigation after it lets the
/// gamepad's South-press flag survive into widget interaction and click
/// event emission.
pub fn build_default_retained_ui_schedule() -> Vec<SystemFn> {
    nightshade_ui::default_ui_systems()
}

pub fn run_retained_ui_schedule(world: &mut World) {
    let systems = std::mem::take(
        &mut world
            .res_mut::<crate::schedule::RetainedUiSchedule>()
            .systems,
    );
    for system in &systems {
        system(&mut world.ecs);
    }
    world
        .res_mut::<crate::schedule::RetainedUiSchedule>()
        .systems = systems;
}

/// Populates the retained-UI sub-schedule and marks the tree visible. The
/// per-frame entry that runs it is registered separately by
/// [`register_frame_systems`].
pub fn install(world: &mut World) {
    world
        .res_mut::<crate::ui::resources::RetainedUiState>()
        .visible = true;
    world
        .res_mut::<crate::schedule::RetainedUiSchedule>()
        .systems = build_default_retained_ui_schedule();
}

/// Registers the retained-UI runner into the post-update stage, explicitly
/// after the transform propagation the layout depends on so the ordering
/// holds regardless of plugin composition order.
pub fn register_frame_systems(stages: &mut nightshade_ecs::Stages<World>) {
    stages
        .stage_mut(Stage::FramePostUpdate.name())
        .insert_after(
            std::any::type_name_of_val(&crate::ecs::transform::systems::run_systems),
            std::any::type_name_of_val(&run_retained_ui_schedule),
            run_retained_ui_schedule,
        );
}

/// Installs the retained UI: [`install`] seeds the world state and
/// [`register_frame_systems`] adds the runner. Hide and show the whole tree
/// at runtime through `world.res::<crate::ui::resources::RetainedUiState>().visible`.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        install(&mut app.world);
        register_frame_systems(&mut app.stages);
    }
}
