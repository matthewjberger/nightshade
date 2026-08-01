use crate::resources::SandboxResources;
use crate::systems::screens::pause;
use crate::systems::shell::state::ShellAction;
use crate::systems::world::{character, player};
use nightshade_physics::prelude::*;
use nightshade_ui::prelude::*;

pub fn apply_shell_actions(
    sandbox_world: &mut SandboxResources,
    scene: &mut crate::commands::Scene<'_, '_>,
    action: ShellAction,
) {
    match action {
        ShellAction::TeleportPlayer(position) => {
            if let Some(player_entity) = sandbox_world.player.player_entity {
                if let Some(transform) = scene
                    .storage
                    .get_mut::<nightshade_scene::contract::LocalTransform>(player_entity)
                {
                    transform.translation = position;
                }
                if let Some(controller) =
                    scene.storage.get_mut::<nightshade_physics::contract::components::CharacterControllerComponent>(
                        player_entity,
                    )
                {
                    controller.velocity = Vec3::zeros();
                }
                {
                    let commands = &mut *scene.commands;
                    reset_physics_interpolation(commands, player_entity);
                }
            }
            character::teleport(sandbox_world, scene, position);
        }
        ShellAction::PrintPlayerPosition => {
            let entity = if sandbox_world.character.third_person {
                sandbox_world.character.body
            } else {
                sandbox_world.player.player_entity
            };
            if let Some(entity) = entity
                && let Some(transform) = scene
                    .storage
                    .get::<nightshade_scene::contract::GlobalTransform>(entity)
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
            player::reset(sandbox_world, scene);
        }
        ShellAction::SetFly(value) => {
            let enabled = value.unwrap_or(!sandbox_world.player.fly);
            player::set_fly(sandbox_world, scene, enabled);
            let checkbox = sandbox_world.ui_handles.pause.fly_checkbox;
            pause::set_checkbox_value(scene, checkbox, enabled);
        }
        ShellAction::SetThirdPerson(value) => {
            let enabled = value.unwrap_or(!sandbox_world.character.third_person);
            character::set_third_person(sandbox_world, scene, enabled);
            let checkbox = sandbox_world.ui_handles.pause.third_person_checkbox;
            pause::set_checkbox_value(scene, checkbox, enabled);
        }
        ShellAction::SetStrideWheel(value) => {
            let state = &mut sandbox_world.character;
            let enabled = value.unwrap_or(!state.stride_wheel_visible);
            state.stride_wheel_visible = enabled;
            let checkbox = sandbox_world.ui_handles.pause.stride_wheel_checkbox;
            pause::set_checkbox_value(scene, checkbox, enabled);
        }
        ShellAction::SetSpheres(value) => {
            let state = &mut sandbox_world.character;
            let enabled = value.unwrap_or(!state.spheres_visible);
            state.spheres_visible = enabled;
            let checkbox = sandbox_world.ui_handles.pause.spheres_checkbox;
            pause::set_checkbox_value(scene, checkbox, enabled);
        }
        ShellAction::SetSkeleton(value) => {
            let state = &mut sandbox_world.character;
            let enabled = value.unwrap_or(!state.skeleton_visible);
            state.skeleton_visible = enabled;
            let checkbox = sandbox_world.ui_handles.pause.skeleton_checkbox;
            pause::set_checkbox_value(scene, checkbox, enabled);
        }
        ShellAction::SetRagdoll(mode) => {
            let mode = match mode.as_str() {
                "limp" => crate::resources::RagdollMode::Limp,
                "match" => crate::resources::RagdollMode::Match,
                "active" => crate::resources::RagdollMode::Active,
                _ => crate::resources::RagdollMode::Off,
            };
            crate::systems::world::ragdoll::set_mode(sandbox_world, scene, mode);
        }
        ShellAction::SetPose(name) => {
            if !crate::systems::world::pose::set_pose(sandbox_world, scene, &name) {
                let mut context = scene.ui_context();
                ui_show_toast(&mut context, "unknown pose", ToastSeverity::Warning, 1.2);
            }
        }
        ShellAction::SetTimeOfDay(hour) => {
            scene.stage.renderer_state.day_night.hour = hour.clamp(0.0, 24.0);
        }
        ShellAction::PauseTimeOfDay(paused) => {
            scene.stage.renderer_state.day_night.auto_cycle = !paused;
        }
        ShellAction::ReadCameraPosition => {
            if let Some(camera) = scene.stage.active_camera.0
                && let Some(transform) = scene
                    .storage
                    .get::<nightshade_scene::contract::LocalTransform>(camera)
            {
                let at = transform.translation;
                tracing::info!("camera @ ({:.2}, {:.2}, {:.2})", at.x, at.y, at.z);
            }
        }
        ShellAction::TeleportCamera(position) => {
            if let Some(camera) = scene.stage.active_camera.0
                && let Some(transform) = scene
                    .storage
                    .get_mut::<nightshade_scene::contract::LocalTransform>(camera)
            {
                transform.translation = position;
            }
        }
        ShellAction::SetFieldOfView(requested) => {
            let Some(camera) = scene.stage.active_camera.0 else {
                return;
            };
            let Some(camera) = scene
                .storage
                .get_mut::<nightshade_camera::components::Camera>(camera)
            else {
                return;
            };
            let nightshade_camera::components::Projection::Perspective(perspective) =
                &mut camera.projection
            else {
                return;
            };
            match requested {
                Some(value) => perspective.y_fov_rad = value.to_radians(),
                None => tracing::info!(
                    "field of view {:.1} degrees",
                    perspective.y_fov_rad.to_degrees()
                ),
            }
        }
        ShellAction::MoveEntity(entity, label, position) => {
            if let Some(transform) = scene
                .storage
                .get_mut::<nightshade_scene::contract::LocalTransform>(entity)
            {
                transform.translation = position;
                tracing::info!("moved {label}");
            }
        }
        ShellAction::RotateEntity(entity, label, degrees) => {
            if let Some(transform) = scene
                .storage
                .get_mut::<nightshade_scene::contract::LocalTransform>(entity)
            {
                let about_x = nalgebra_glm::quat_angle_axis(
                    degrees.x.to_radians(),
                    &Vec3::new(1.0, 0.0, 0.0),
                );
                let about_y = nalgebra_glm::quat_angle_axis(
                    degrees.y.to_radians(),
                    &Vec3::new(0.0, 1.0, 0.0),
                );
                let about_z = nalgebra_glm::quat_angle_axis(
                    degrees.z.to_radians(),
                    &Vec3::new(0.0, 0.0, 1.0),
                );
                transform.rotation = about_z * about_y * about_x * transform.rotation;
                tracing::info!("rotated {label}");
            }
        }
        ShellAction::ScaleEntity(entity, label, factor) => {
            if let Some(transform) = scene
                .storage
                .get_mut::<nightshade_scene::contract::LocalTransform>(entity)
            {
                transform.scale = transform.scale.component_mul(&factor);
                tracing::info!("scaled {label}");
            }
        }
        ShellAction::DeleteEntity(entity, label) => {
            scene.despawn(&[entity]);
            tracing::info!("deleted {label}");
        }
        ShellAction::InspectEntity(entity, label) => {
            if let Some(transform) = scene
                .storage
                .get::<nightshade_scene::contract::LocalTransform>(entity)
                .copied()
            {
                tracing::info!(
                    "{label} at ({:.2}, {:.2}, {:.2}) scale ({:.2}, {:.2}, {:.2})",
                    transform.translation.x,
                    transform.translation.y,
                    transform.translation.z,
                    transform.scale.x,
                    transform.scale.y,
                    transform.scale.z
                );
            }
        }
        ShellAction::SetTimeSpeed(value) => {
            scene.stage.renderer_state.day_night.speed = value;
        }
    }
}
