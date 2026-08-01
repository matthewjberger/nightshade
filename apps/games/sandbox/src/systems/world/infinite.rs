use crate::resources::SandboxResources;
use crate::systems::world::textures::{FLOOR_TEXTURE, prototype_material};
use nightshade_physics::prelude::*;

const GROUND_SIZE: f32 = 4000.0;
const TEXTURE_PERIOD: f32 = 12.5;
const GROUND_CENTER_HEIGHT: f32 = -0.51;
const PARKED_HEIGHT: f32 = -10_000.0;

pub fn set_enabled(
    sandbox_world: &mut SandboxResources,
    scene: &mut crate::commands::Scene<'_, '_>,
    enabled: bool,
) {
    if sandbox_world.infinite_world.enabled == enabled {
        return;
    }
    sandbox_world.infinite_world.enabled = enabled;
    apply_ground_height(sandbox_world, scene);
}

fn apply_ground_height(
    sandbox_world: &mut SandboxResources,
    scene: &mut crate::commands::Scene<'_, '_>,
) {
    let active = sandbox_world.infinite_world.enabled;
    let ground = ensure_ground(sandbox_world, scene);
    let height = if active {
        GROUND_CENTER_HEIGHT
    } else {
        PARKED_HEIGHT
    };
    scene.commands.queue(move |storage, _| {
        if let Some(transform) =
            storage.get_mut::<nightshade_scene::contract::LocalTransform>(ground)
        {
            transform.translation.y = height;
        }
    });
    if active {
        recenter(sandbox_world, scene);
    }
}

pub fn update(
    sandbox_world: Res<crate::resources::SandboxResources>,
    stage: crate::commands::Stage,
    mut commands: Commands,
    storage: &mut nightshade::ecs::Storage,
) {
    if !sandbox_world.infinite_world.enabled {
        return;
    }
    let mut owned = crate::commands::Scene {
        storage,
        commands: &mut commands,
        stage,
    };
    recenter(&sandbox_world, &mut owned);
}

fn recenter(sandbox_world: &SandboxResources, scene: &mut crate::commands::Scene<'_, '_>) {
    let Some(ground) = sandbox_world.infinite_world.ground else {
        return;
    };
    let Some(player_entity) = sandbox_world.player.player_entity else {
        return;
    };
    let Some(player_translation) = scene
        .storage
        .get::<nightshade_scene::contract::LocalTransform>(player_entity)
        .map(|transform| transform.translation)
    else {
        return;
    };
    let snapped_x = (player_translation.x / TEXTURE_PERIOD).round() * TEXTURE_PERIOD;
    let snapped_z = (player_translation.z / TEXTURE_PERIOD).round() * TEXTURE_PERIOD;
    if let Some(transform) = scene
        .storage
        .get_mut::<nightshade_scene::contract::LocalTransform>(ground)
        && (transform.translation.x != snapped_x || transform.translation.z != snapped_z)
    {
        transform.translation.x = snapped_x;
        transform.translation.z = snapped_z;
    }
}

fn ensure_ground(
    sandbox_world: &mut SandboxResources,
    scene: &mut crate::commands::Scene<'_, '_>,
) -> Entity {
    if let Some(ground) = sandbox_world.infinite_world.ground {
        return ground;
    }
    let mut material = prototype_material(FLOOR_TEXTURE, Vec3::new(0.55, 0.6, 0.7), 0.85, 0.05);
    let repeats = GROUND_SIZE / TEXTURE_PERIOD;
    material.base_texture_transform.scale = [repeats, repeats];
    let entity = {
        let commands = &mut *scene.commands;
        spawn_static_physics_cube_with_material(
            commands,
            Vec3::new(0.0, PARKED_HEIGHT, 0.0),
            Vec3::new(GROUND_SIZE, 1.0, GROUND_SIZE),
            material,
        )
    };
    scene.commands.queue(move |storage, _| {
        if let Some(name) = storage.get_mut::<nightshade_scene::contract::primitives::Name>(entity)
        {
            name.0 = "InfiniteGround".to_string();
        }
    });
    sandbox_world.infinite_world.ground = Some(entity);
    entity
}
