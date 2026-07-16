//! The retained UI composition: the ordered UI pipeline, the sub-schedule
//! runner, and [`UiPlugin`]. The UI subsystem itself lives in
//! [`crate::ecs::ui`] because it is core, not feature-gated; this plugin
//! is what activates it.

use crate::app::{App, Plugin};
use crate::ecs::world::World;
use crate::schedule::{FramePhase, SystemFn, schedule_push_after};

/// The UI pipeline in run order. Gamepad navigation has to run after
/// picking: picking resets every entity's `interaction.clicked` flag at
/// the top of the frame, so running navigation after it lets the
/// gamepad's South-press flag survive into widget interaction and click
/// event emission.
pub fn build_default_retained_ui_schedule() -> Vec<SystemFn> {
    vec![
        crate::ecs::ui::widget_systems::ui_retained_input_sync_system,
        crate::ecs::ui::picking::ui_layout_picking_system,
        #[cfg(feature = "gamepad")]
        crate::ecs::ui::gamepad_navigation::ui_gamepad_navigation_system,
        crate::ecs::ui::widget_systems::ui_widget_interaction_system,
        crate::ecs::ui::widget_systems::ui_event_bubble_system,
        crate::ecs::ui::widget_systems::ui_emit_click_events_system,
        crate::ecs::ui::systems::ui_layout_state_update_system,
        crate::ecs::ui::systems::ui_docked_panel_layout_system,
        crate::ecs::ui::systems::ui_responsive_apply_system,
        crate::ecs::ui::systems::ui_layout_compute_system,
        crate::ecs::ui::systems::ui_theme_transition_tick_system,
        crate::ecs::ui::systems::ui_theme_apply_system,
        crate::ecs::ui::systems::ui_layout_color_blend_system,
        crate::ecs::ui::systems::ui_layout_transform_blend_system,
        crate::ecs::ui::render_sync::ui_layout_render_sync_system,
    ]
}

pub fn run_retained_ui_schedule(world: &mut World) {
    let systems = std::mem::take(
        &mut world
            .expect_resource_mut::<crate::schedule::Schedules>()
            .retained_ui,
    );
    for system in &systems {
        system(world);
    }
    world
        .expect_resource_mut::<crate::schedule::Schedules>()
        .retained_ui = systems;
}

/// Builds the UI sub-schedule and registers the frame-schedule entry that
/// runs it, explicitly after the transform propagation the layout depends
/// on so the ordering holds regardless of plugin composition order, with
/// the tree shown.
pub fn install(world: &mut World) {
    world.resources.retained_ui.visible = true;
    world
        .expect_resource_mut::<crate::schedule::Schedules>()
        .retained_ui = build_default_retained_ui_schedule();
    schedule_push_after(
        &mut world
            .expect_resource_mut::<crate::schedule::Schedules>()
            .frame,
        FramePhase::PostUpdate,
        run_retained_ui_schedule,
        crate::ecs::transform::systems::run_systems,
    );
}

/// Installs the retained UI through [`install`]. Hide and show the whole
/// tree at runtime through `world.resources.retained_ui.visible`.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        install(&mut app.world);
    }
}
