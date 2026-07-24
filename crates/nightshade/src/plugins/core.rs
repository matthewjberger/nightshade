//! The core engine plugin: registers the default per-frame systems (input,
//! animation, transforms, command processing) that every composed engine
//! program relies on, and exposes the same set for runners that drive their
//! own loop instead of composing an [`App`](crate::app::App).

use crate::app::{App, Plugin, Stage, push_frame_system};
use crate::ecs::world::World;

/// Registers the engine's default per-frame systems into an [`App`].
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        register_default_frame_systems(&mut app.stages);
    }
}

/// Registers the engine's default per-frame systems into the frame stages
/// of `stages`, which must already declare the [`Stage::FRAME_STAGES`]. Both
/// [`App::new`] and [`build_default_frame_stages`] populate through this, so
/// the App and runner-owned schedules stay in lock-step.
pub(crate) fn register_default_frame_systems(stages: &mut nightshade_ecs::Stages<World>) {
    push_frame_system(
        stages,
        Stage::FrameStart,
        crate::ecs::event::advance_event_frame_system,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        crate::ecs::graphics::systems::day_night_cycle_system,
    );
    #[cfg(target_arch = "wasm32")]
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        crate::ecs::camera::systems::sync_wasm_canvas_size,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        crate::ecs::bounding_volume::systems::ensure_bounding_volumes,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        crate::ecs::animation::systems::evaluate_animation_graphs,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        crate::ecs::animation::systems::evaluate_blend_trees,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        crate::ecs::animation::systems::update_animation_players,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        crate::ecs::animation::systems::apply_animations,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        crate::ecs::animation::systems::apply_root_motion,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        crate::ecs::animation::systems::foot_ik_system,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        crate::ecs::tween::update_tweens_system,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        crate::ecs::particles::systems::update_particle_emitters,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        crate::ecs::cloth::systems::sync_cloth_meshes_system,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        crate::ecs::vfx::systems::update_vfx_system,
    );
    push_frame_system(
        stages,
        Stage::FramePostUpdate,
        crate::ecs::transform::systems::run_systems,
    );
    push_frame_system(
        stages,
        Stage::FramePostUpdate,
        crate::ecs::mesh::systems::update_instanced_mesh_caches_system,
    );
    push_frame_system(
        stages,
        Stage::Last,
        crate::ecs::input::systems::apply_ime_allowed_system,
    );
    push_frame_system(
        stages,
        Stage::Last,
        crate::ecs::input::systems::reset_mouse_system,
    );
    push_frame_system(
        stages,
        Stage::Last,
        crate::ecs::input::systems::reset_keyboard_system,
    );
    push_frame_system(
        stages,
        Stage::Last,
        crate::ecs::input::systems::reset_touch_system,
    );
    push_frame_system(
        stages,
        Stage::Last,
        crate::ecs::builtins::process_commands_system,
    );
    push_frame_system(
        stages,
        Stage::Last,
        crate::ecs::builtins::cleanup_unused_resources_system,
    );
}

/// Builds a [`nightshade_ecs::Stages`] holding only the engine's frame stages,
/// populated with the default per-frame systems, for runners that drive
/// their own [`State`] loop instead of composing an [`App`]. The stage
/// order matches the tail of [`App`]'s stages, so a runner that runs these
/// after its own logic reproduces the App's per-frame order.
pub fn build_default_frame_stages() -> nightshade_ecs::Stages<World> {
    let mut stages = nightshade_ecs::Stages::new();
    for stage in Stage::FRAME_STAGES {
        stages.add_stage(stage.name());
    }
    register_default_frame_systems(&mut stages);
    stages
}
