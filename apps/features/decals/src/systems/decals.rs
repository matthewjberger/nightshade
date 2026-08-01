use crate::resources::{DecalType, DecalsResources};
use crate::systems::geometry::rotation_from_normal;
use crate::theme::{
    AWESOMEFACE_TEXTURE, INITIAL_DECALS, RANDOM_DECAL_SIZE_RANGE, RANDOM_WALLS, RandomWall,
};
use nightshade::prelude::*;
use nightshade_decals::contract::Decal;
use nightshade_scene::prelude::*;
use rand::Rng;

pub fn spawn_decal_at(
    commands: &mut Commands<'_>,
    position: Vec3,
    normal: Vec3,
    decal_type: DecalType,
    size: f32,
    emissive_strength: f32,
    use_awesomeface: bool,
) -> Entity {
    let texture = if use_awesomeface {
        AWESOMEFACE_TEXTURE
    } else {
        decal_type.texture()
    };
    let color = if use_awesomeface {
        [1.0, 1.0, 1.0, 1.0]
    } else {
        decal_type.color()
    };
    let emissive = if decal_type.is_emissive() {
        emissive_strength
    } else {
        0.0
    };

    let entity = commands.reserve_entity();
    commands.spawn_reserved(
        entity,
        (
            Name(format!("Decal_{}", entity.id)),
            LocalTransform {
                translation: position,
                rotation: rotation_from_normal(normal),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
            GlobalTransform::default(),
            Decal::new(texture)
                .with_size(size, size)
                .with_color(color)
                .with_emissive_strength(emissive)
                .with_depth(size),
        ),
    );
    entity
}

pub fn spawn_initial_decals(commands: &mut Commands<'_>, emissive_strength: f32) {
    for decal in &INITIAL_DECALS {
        spawn_decal_at(
            commands,
            decal.position,
            decal.normal,
            decal.decal_type,
            decal.size,
            emissive_strength,
            matches!(decal.decal_type, DecalType::AwesomeFace),
        );
    }
}

pub fn spawn_random_decals(commands: &mut Commands<'_>, count: usize, emissive_strength: f32) {
    let mut generator = rand::rng();
    for _ in 0..count {
        let wall = &RANDOM_WALLS[generator.random_range(0..RANDOM_WALLS.len())];
        let position = random_position_on(wall, &mut generator);
        let decal_type = DecalType::from_index(generator.random_range(0..6));
        let size = generator.random_range(RANDOM_DECAL_SIZE_RANGE.0..RANDOM_DECAL_SIZE_RANGE.1);
        spawn_decal_at(
            commands,
            position,
            wall.normal,
            decal_type,
            size,
            emissive_strength,
            matches!(decal_type, DecalType::AwesomeFace),
        );
    }
}

fn random_position_on(wall: &RandomWall, generator: &mut impl Rng) -> Vec3 {
    wall.origin
        + wall.across * generator.random_range(wall.across_range.0..wall.across_range.1)
        + wall.up * generator.random_range(wall.up_range.0..wall.up_range.1)
}

pub fn apply_clear(
    mut decals: ResMut<DecalsResources>,
    mut commands: Commands,
    existing: Query<(&Decal,)>,
) {
    if !decals.clear_requested {
        return;
    }
    decals.clear_requested = false;
    existing.for_each(|entity, _| commands.despawn_recursive(entity));
}
