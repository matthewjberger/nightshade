use nightshade::prelude::*;
use nightshade_graphics::prelude::*;

use crate::data;
use crate::resources::LocomotionResources;

pub fn build(
    mut locomotion: ResMut<LocomotionResources>,
    mut registry: ResMut<nightshade_render::resources::material_registry::MaterialRegistry>,
    mut commands: Commands,
) {
    for (index, (center, half)) in data::BLOCKS.iter().enumerate() {
        let position = Vec3::new(center[0], half[1], center[2]);
        let block = spawn_mesh(
            &mut commands,
            "Cube",
            position,
            Vec3::new(half[0] * 2.0, half[1] * 2.0, half[2] * 2.0),
        );
        set_material(
            &mut registry,
            &mut commands,
            block,
            format!("block_{index}"),
            Material {
                base_color: data::BLOCK_COLOR,
                roughness: 0.8,
                ..Default::default()
            },
        );
        locomotion.arena.blocks.push((
            Vec3::new(center[0], half[1], center[2]),
            Vec3::new(half[0], half[1], half[2]),
        ));
    }

    for (index, spot) in data::PICKUPS.iter().enumerate() {
        let position = Vec3::new(spot[0], data::PICKUP_HEIGHT, spot[1]);
        let pickup = spawn_mesh(
            &mut commands,
            "Torus",
            position,
            Vec3::new(data::PICKUP_SIZE, data::PICKUP_SIZE, data::PICKUP_SIZE),
        );
        set_material(
            &mut registry,
            &mut commands,
            pickup,
            format!("pickup_{index}"),
            Material {
                base_color: data::PICKUP_COLOR,
                emissive_factor: data::PICKUP_GLOW,
                roughness: 0.35,
                metallic: 0.6,
                ..Default::default()
            },
        );
        locomotion.arena.pickups.push(pickup);
    }
    locomotion.arena.total = locomotion.arena.pickups.len();
}

pub fn spin(
    locomotion: Res<LocomotionResources>,
    time: Res<nightshade_platform::contract::time::Time>,
    mut transforms: Query<(&mut LocalTransform,)>,
) {
    let turn = time.uptime_seconds as f32 * data::PICKUP_SPIN;
    for (index, pickup) in locomotion.arena.pickups.iter().enumerate() {
        let phase = turn + index as f32 * 0.7;
        transforms.for_entity(*pickup, |(transform,)| {
            transform.rotation = nalgebra_glm::quat_angle_axis(phase, &Vec3::new(0.0, 1.0, 0.0));
            transform.translation.y = data::PICKUP_HEIGHT + (phase * 1.6).sin() * data::PICKUP_BOB;
        });
    }
}

pub fn collect(
    mut locomotion_resources: ResMut<LocomotionResources>,
    mut hierarchy_index: ResMut<nightshade::ecs::HierarchyIndex>,
    storage: &mut nightshade::ecs::Storage,
) {
    let Some(body) = locomotion_resources.character.body else {
        return;
    };
    let Some(here) = storage
        .get::<LocalTransform>(body)
        .map(|transform| transform.translation)
    else {
        return;
    };
    let taken: Vec<Entity> = locomotion_resources
        .arena
        .pickups
        .iter()
        .copied()
        .filter(|pickup| {
            storage
                .get::<LocalTransform>(*pickup)
                .map(|transform| {
                    let gap = transform.translation - here;
                    (gap.x * gap.x + gap.z * gap.z).sqrt() < data::PICKUP_REACH
                })
                .unwrap_or(false)
        })
        .collect();
    if taken.is_empty() {
        return;
    }
    for pickup in &taken {
        hierarchy_index.despawn_recursive(storage, *pickup);
    }
    let arena = &mut locomotion_resources.arena;
    arena.pickups.retain(|held| !taken.contains(held));
    arena.collected += taken.len();
}
