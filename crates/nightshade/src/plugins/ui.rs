//! The retained UI composition: the ordered UI pipeline, the sub-schedule
//! runner, and [`UiPlugin`]. The UI subsystem itself lives in
//! [`crate::ui`] because it is core, not feature-gated; this plugin
//! is what activates it.

use crate::app::{App, Plugin, Stage};
use crate::ecs::world::World;
use nightshade_ui::{UiSystemFn, UiSystemPhase};

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
pub fn install(world: &mut World, systems: &[(UiSystemPhase, UiSystemFn)]) {
    world
        .res_mut::<crate::ui::resources::RetainedUiRuntime>()
        .visible = true;
    world
        .res_mut::<crate::schedule::RetainedUiSchedule>()
        .systems = nightshade_ui::ui_systems_with(systems);
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
/// at runtime through `world.res::<crate::ui::resources::RetainedUiRuntime>().visible`.
///
/// [`UiPlugin::with_system`] splices a system of the app's own into the UI
/// schedule, which is how a canvas-drawn widget the app defines runs at the same
/// point in the frame the built-in widgets do:
///
/// ```ignore
/// app.add_plugin(UiPlugin::new().with_system(UiSystemPhase::PreDraw, draw_my_widget));
/// ```
#[derive(Default)]
pub struct UiPlugin {
    systems: Vec<(UiSystemPhase, UiSystemFn)>,
}

impl UiPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    /// Runs `system` inside the UI schedule at `phase`, after any system already
    /// registered for the same phase.
    pub fn with_system(mut self, phase: UiSystemPhase, system: UiSystemFn) -> Self {
        self.systems.push((phase, system));
        self
    }
}

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        install(&mut app.world, &self.systems);
        register_frame_systems(&mut app.stages);
    }
}
