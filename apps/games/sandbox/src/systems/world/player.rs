use crate::resources::SandboxResources;
use crate::systems::world::level::{default_spawn_position, find_player_spawn};
use nightshade::prelude::*;
use nightshade_camera::commands::spawn_first_person_camera;
use nightshade_physics::commands::spawn_character_body;

pub const PLAYER_TAG: &str = "player";

const FLY_GROUND_CLEARANCE: f32 = 1.0;
const FLOOR_SURFACE_HEIGHT: f32 = 0.0;

pub fn clamp_fly_camera(
    sandbox_world: &SandboxResources,
    scene: &mut crate::commands::Scene<'_, '_>,
) {
    if !sandbox_world.player.fly {
        return;
    }
    let Some(fly_camera) = sandbox_world.player.fly_camera else {
        return;
    };
    let Some(translation) = scene
        .storage
        .get::<nightshade_scene::contract::LocalTransform>(fly_camera)
        .map(|transform| transform.translation)
    else {
        return;
    };
    let ground = FLOOR_SURFACE_HEIGHT;
    let minimum_y = ground + FLY_GROUND_CLEARANCE;
    let mut resolved = translation;
    resolved.y = resolved.y.max(minimum_y);
    if (resolved - translation).norm_squared() > 0.0
        && let Some(transform) = scene
            .storage
            .get_mut::<nightshade_scene::contract::LocalTransform>(fly_camera)
    {
        transform.translation = resolved;
    }
}

pub fn set_fly(
    sandbox_world: &mut SandboxResources,
    scene: &mut crate::commands::Scene<'_, '_>,
    enabled: bool,
) {
    sandbox_world.player.fly = enabled;
    let gameplay_camera = if sandbox_world.character.third_person {
        sandbox_world.character.camera
    } else {
        sandbox_world.player.camera_entity
    };
    if enabled {
        let pose = gameplay_camera
            .and_then(|camera| {
                scene
                    .storage
                    .get::<nightshade_scene::contract::GlobalTransform>(camera)
            })
            .map(|transform| {
                (
                    transform.translation(),
                    nalgebra_glm::to_quat(&nalgebra_glm::mat3_to_mat4(
                        &nalgebra_glm::mat4_to_mat3(&transform.0),
                    )),
                )
            });
        let Some((position, rotation)) = pose else {
            return;
        };
        let fly_camera = {
            let commands = &mut *scene.commands;
            nightshade_camera::commands::scene::spawn_camera(
                commands,
                position,
                "Fly Camera".into(),
            )
        };
        if let Some(transform) = scene
            .storage
            .get_mut::<nightshade_scene::contract::LocalTransform>(fly_camera)
        {
            transform.rotation = rotation;
        }
        sandbox_world.player.fly_camera = Some(fly_camera);
        scene.stage.active_camera.0 = Some(fly_camera);
    } else {
        scene.stage.active_camera.0 = gameplay_camera;
        if let Some(fly_camera) = sandbox_world.player.fly_camera.take() {
            scene.despawn(&[fly_camera]);
        }
    }
}

pub fn spawn(sandbox_world: &mut SandboxResources, scene: &mut crate::commands::Scene<'_, '_>) {
    let position = find_player_spawn(scene).unwrap_or_else(default_spawn_position);
    let (player_entity, camera_entity) = {
        let commands = &mut *scene.commands;
        let player_entity = spawn_character_body(commands, position, "Player");
        let camera_entity =
            spawn_first_person_camera(commands, player_entity, Vec3::new(0.0, 1.3, 0.0), 85.0);
        (player_entity, camera_entity)
    };
    scene
        .stage
        .entities
        .tags
        .entry(player_entity)
        .or_default()
        .push(PLAYER_TAG.to_string());
    sandbox_world.player.player_entity = Some(player_entity);
    sandbox_world.player.camera_entity = Some(camera_entity);
    scene.stage.active_camera.0 = Some(camera_entity);
    super::camera::apply(sandbox_world, scene);
}

pub fn reset(sandbox_world: &mut SandboxResources, scene: &mut crate::commands::Scene<'_, '_>) {
    let Some(player_entity) = sandbox_world.player.player_entity else {
        return;
    };
    let spawn = find_player_spawn(scene).unwrap_or_else(default_spawn_position);

    if let Some(transform) = scene
        .storage
        .get_mut::<nightshade_scene::contract::LocalTransform>(player_entity)
    {
        transform.translation = spawn;
    }
    if let Some(controller) = scene
        .storage
        .get_mut::<nightshade_physics::contract::components::CharacterControllerComponent>(
        player_entity,
    ) {
        controller.velocity = Vec3::zeros();
    }
    if let Some(interpolation) = scene
        .storage
        .get_mut::<nightshade_physics::contract::components::PhysicsInterpolation>(
        player_entity,
    ) {
        interpolation.previous_translation = spawn;
        interpolation.current_translation = spawn;
    }

    crate::systems::world::character::reset(sandbox_world, scene);
}
