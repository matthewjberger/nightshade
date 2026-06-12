use crate::ecs::SandboxWorld;
use crate::systems::screens::pause;
use crate::systems::shell::state::ShellAction;
use crate::systems::world::{character, player};
use nightshade::prelude::*;

pub fn apply_shell_actions(
    sandbox_world: &mut SandboxWorld,
    world: &mut World,
    action: ShellAction,
) {
    match action {
        ShellAction::TeleportPlayer(position) => {
            if let Some(player_entity) = sandbox_world.resources.player.player_entity {
                if let Some(transform) = world.core.get_local_transform_mut(player_entity) {
                    transform.translation = position;
                }
                if let Some(controller) = world.core.get_character_controller_mut(player_entity) {
                    controller.velocity = Vec3::zeros();
                }
                reset_physics_interpolation(world, player_entity);
                mark_local_transform_dirty(world, player_entity);
            }
            character::teleport(sandbox_world, world, position);
        }
        ShellAction::PrintPlayerPosition => {
            let entity = if sandbox_world.resources.character.third_person {
                sandbox_world.resources.character.body
            } else {
                sandbox_world.resources.player.player_entity
            };
            if let Some(entity) = entity
                && let Some(transform) = world.core.get_global_transform(entity)
            {
                let position = transform.translation();
                tracing::info!(
                    "player @ ({:.2}, {:.2}, {:.2})",
                    position.x,
                    position.y,
                    position.z
                );
            }
        }
        ShellAction::ResetPlayer => {
            player::reset(sandbox_world, world);
        }
        ShellAction::SetFly(value) => {
            let enabled = value.unwrap_or(!sandbox_world.resources.player.fly);
            player::set_fly(sandbox_world, world, enabled);
            let checkbox = sandbox_world.resources.ui_handles.pause.fly_checkbox;
            pause::set_checkbox_value(world, checkbox, enabled);
        }
        ShellAction::SetThirdPerson(value) => {
            let enabled = value.unwrap_or(!sandbox_world.resources.character.third_person);
            character::set_third_person(sandbox_world, world, enabled);
            let checkbox = sandbox_world
                .resources
                .ui_handles
                .pause
                .third_person_checkbox;
            pause::set_checkbox_value(world, checkbox, enabled);
        }
        ShellAction::SetStrideWheel(value) => {
            let state = &mut sandbox_world.resources.character;
            let enabled = value.unwrap_or(!state.stride_wheel_visible);
            state.stride_wheel_visible = enabled;
            let checkbox = sandbox_world
                .resources
                .ui_handles
                .pause
                .stride_wheel_checkbox;
            pause::set_checkbox_value(world, checkbox, enabled);
        }
        ShellAction::SetSpheres(value) => {
            let state = &mut sandbox_world.resources.character;
            let enabled = value.unwrap_or(!state.spheres_visible);
            state.spheres_visible = enabled;
            let checkbox = sandbox_world.resources.ui_handles.pause.spheres_checkbox;
            pause::set_checkbox_value(world, checkbox, enabled);
        }
        ShellAction::SetTimeOfDay(hour) => {
            world.resources.renderer_state.day_night.hour = hour.clamp(0.0, 24.0);
        }
        ShellAction::PauseTimeOfDay(paused) => {
            world.resources.renderer_state.day_night.auto_cycle = !paused;
        }
        ShellAction::SetTimeSpeed(value) => {
            world.resources.renderer_state.day_night.speed = value;
        }
        ShellAction::SaveLevel => {
            #[cfg(not(target_arch = "wasm32"))]
            crate::systems::world::level::save_current_level(world);
            #[cfg(target_arch = "wasm32")]
            tracing::warn!("save_level is not supported on wasm");
        }
    }
}
