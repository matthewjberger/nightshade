use crate::ecs::{
    ENEMY, ENTITY_HANDLE, Enemy, EnemyType, EntityHandle, GameState, GameWorld, POSITION, Position,
};
use crate::systems::{
    create_death_effect, create_poison_bubble_effect, despawn_entity, spawn_money_popup,
};
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::prelude::*;

pub fn spawn_enemy(game_world: &mut GameWorld, world: &mut World, enemy_type: EnemyType) {
    if game_world.resources.path.is_empty() {
        return;
    }

    let position = game_world.resources.path[0];
    let wave = game_world.resources.wave;
    let base_y = enemy_type.y_offset();
    let color = enemy_type.color();

    let engine_entity = match enemy_type {
        EnemyType::Normal => spawn_normal_enemy(world, position, base_y, color),
        EnemyType::Fast => spawn_fast_enemy(world, position, base_y, color),
        EnemyType::Tank => spawn_tank_enemy(world, position, base_y, color),
        EnemyType::Flying => spawn_flying_enemy(world, position, base_y, color),
        EnemyType::Shielded => spawn_shielded_enemy(world, position, base_y, color),
        EnemyType::Healer => spawn_healer_enemy(world, position, base_y, color),
        EnemyType::Boss => spawn_boss_enemy(world, position, base_y, color),
    };

    let game_entity = game_world.spawn_entities(ENTITY_HANDLE | POSITION | ENEMY, 1)[0];

    game_world.set_entity_handle(game_entity, EntityHandle(engine_entity));
    game_world.set_position(game_entity, Position(position));

    game_world.set_enemy(
        game_entity,
        Enemy {
            health: enemy_type.health(wave),
            shield_health: enemy_type.shield(),
            speed: enemy_type.speed(),
            path_index: 0,
            path_progress: 0.0,
            value: enemy_type.value(wave),
            enemy_type,
            slow_duration: 0.0,
            poison_duration: 0.0,
            poison_damage: 0.0,
        },
    );

    game_world.resources.enemies_list.push(game_entity);
}

fn spawn_normal_enemy(world: &mut World, position: Vec3, base_y: f32, color: Vec4) -> Entity {
    let main_entity = spawn_mesh(
        world,
        "Cube",
        position + nalgebra_glm::vec3(0.0, base_y, 0.0),
        nalgebra_glm::vec3(0.25, 0.3, 0.2),
    );
    let material_name = format!("GoblinBody_{}", main_entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        Material {
            base_color: color.into(),
            ..Default::default()
        },
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world.set_material_ref(main_entity, MaterialRef::new(material_name));

    spawn_child_sphere(
        world,
        main_entity,
        "Goblin Head",
        nalgebra_glm::vec3(0.0, 0.25, 0.0),
        nalgebra_glm::vec3(0.15, 0.15, 0.15),
        [color.x * 0.9, color.y * 0.9, color.z * 0.9, 1.0],
    );
    spawn_child_cube(
        world,
        main_entity,
        "Goblin Arm L",
        nalgebra_glm::vec3(-0.2, -0.05, 0.0),
        nalgebra_glm::vec3(0.08, 0.2, 0.08),
        [color.x * 0.8, color.y * 0.8, color.z * 0.8, 1.0],
    );
    spawn_child_cube(
        world,
        main_entity,
        "Goblin Arm R",
        nalgebra_glm::vec3(0.2, -0.05, 0.0),
        nalgebra_glm::vec3(0.08, 0.2, 0.08),
        [color.x * 0.8, color.y * 0.8, color.z * 0.8, 1.0],
    );

    main_entity
}

fn spawn_fast_enemy(world: &mut World, position: Vec3, base_y: f32, color: Vec4) -> Entity {
    let main_entity = spawn_mesh(
        world,
        "Cone",
        position + nalgebra_glm::vec3(0.0, base_y, 0.0),
        nalgebra_glm::vec3(0.22, 0.4, 0.22),
    );
    let material_name = format!("FastBody_{}", main_entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        Material {
            base_color: color.into(),
            ..Default::default()
        },
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world.set_material_ref(main_entity, MaterialRef::new(material_name));

    spawn_child_sphere(
        world,
        main_entity,
        "Fast Head",
        nalgebra_glm::vec3(0.0, 0.3, 0.0),
        nalgebra_glm::vec3(0.13, 0.13, 0.13),
        [color.x * 1.2, color.y * 1.2, color.z * 1.2, 1.0],
    );

    main_entity
}

fn spawn_tank_enemy(world: &mut World, position: Vec3, base_y: f32, color: Vec4) -> Entity {
    let main_entity = spawn_mesh(
        world,
        "Cube",
        position + nalgebra_glm::vec3(0.0, base_y, 0.0),
        nalgebra_glm::vec3(0.35, 0.4, 0.3),
    );
    let material_name = format!("TankBody_{}", main_entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        Material {
            base_color: color.into(),
            ..Default::default()
        },
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world.set_material_ref(main_entity, MaterialRef::new(material_name));

    spawn_child_cube(
        world,
        main_entity,
        "Tank Head",
        nalgebra_glm::vec3(0.0, 0.3, 0.0),
        nalgebra_glm::vec3(0.25, 0.25, 0.25),
        [color.x * 0.8, color.y * 0.8, color.z * 0.8, 1.0],
    );
    spawn_child_cube(
        world,
        main_entity,
        "Tank Shoulder L",
        nalgebra_glm::vec3(-0.25, 0.15, 0.0),
        nalgebra_glm::vec3(0.15, 0.15, 0.15),
        [color.x * 1.2, color.y * 1.2, color.z * 1.2, 1.0],
    );
    spawn_child_cube(
        world,
        main_entity,
        "Tank Shoulder R",
        nalgebra_glm::vec3(0.25, 0.15, 0.0),
        nalgebra_glm::vec3(0.15, 0.15, 0.15),
        [color.x * 1.2, color.y * 1.2, color.z * 1.2, 1.0],
    );

    main_entity
}

fn spawn_flying_enemy(world: &mut World, position: Vec3, base_y: f32, color: Vec4) -> Entity {
    let main_entity = spawn_mesh(
        world,
        "Sphere",
        position + nalgebra_glm::vec3(0.0, base_y, 0.0),
        nalgebra_glm::vec3(0.22, 0.28, 0.22),
    );
    let material_name = format!("FlyingBody_{}", main_entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        Material {
            base_color: color.into(),
            emissive_factor: [color.x * 0.35, color.y * 0.35, color.z * 0.35],
            ..Default::default()
        },
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world.set_material_ref(main_entity, MaterialRef::new(material_name));

    spawn_child_cone_rotated(
        world,
        main_entity,
        "Wing L",
        nalgebra_glm::vec3(-0.3, 0.0, 0.0),
        nalgebra_glm::vec3(0.18, 0.1, 0.18),
        [color.x * 0.5, color.y * 0.5, color.z * 0.5, 0.7],
        std::f32::consts::FRAC_PI_2,
    );
    spawn_child_cone_rotated(
        world,
        main_entity,
        "Wing R",
        nalgebra_glm::vec3(0.3, 0.0, 0.0),
        nalgebra_glm::vec3(0.18, 0.1, 0.18),
        [color.x * 0.5, color.y * 0.5, color.z * 0.5, 0.7],
        -std::f32::consts::FRAC_PI_2,
    );

    main_entity
}

fn spawn_shielded_enemy(world: &mut World, position: Vec3, base_y: f32, color: Vec4) -> Entity {
    let main_entity = spawn_mesh(
        world,
        "Cube",
        position + nalgebra_glm::vec3(0.0, base_y, 0.0),
        nalgebra_glm::vec3(0.25, 0.3, 0.2),
    );
    let material_name = format!("ShieldedBody_{}", main_entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        Material {
            base_color: color.into(),
            ..Default::default()
        },
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world.set_material_ref(main_entity, MaterialRef::new(material_name));

    spawn_child_sphere(
        world,
        main_entity,
        "Shielded Head",
        nalgebra_glm::vec3(0.0, 0.25, 0.0),
        nalgebra_glm::vec3(0.15, 0.15, 0.15),
        [color.x * 0.9, color.y * 0.9, color.z * 0.9, 1.0],
    );
    spawn_child_cylinder(
        world,
        main_entity,
        "Shield",
        nalgebra_glm::vec3(0.2, 0.0, 0.0),
        nalgebra_glm::vec3(0.3, 0.06, 0.3),
        [0.7, 0.7, 1.0, 0.65],
        true,
    );

    main_entity
}

fn spawn_healer_enemy(world: &mut World, position: Vec3, base_y: f32, color: Vec4) -> Entity {
    let main_entity = spawn_mesh(
        world,
        "Torus",
        position + nalgebra_glm::vec3(0.0, base_y, 0.0),
        nalgebra_glm::vec3(0.25, 0.25, 0.25),
    );
    let material_name = format!("HealerBody_{}", main_entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        Material {
            base_color: color.into(),
            emissive_factor: [color.x * 0.5, color.y * 0.5, color.z * 0.5],
            ..Default::default()
        },
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world.set_material_ref(main_entity, MaterialRef::new(material_name));

    spawn_child_sphere_emissive(
        world,
        main_entity,
        "Healer Orb",
        nalgebra_glm::vec3(0.0, 0.0, 0.0),
        nalgebra_glm::vec3(0.16, 0.16, 0.16),
        [0.3, 1.0, 0.3, 1.0],
        [0.0, 0.6, 0.0],
    );

    main_entity
}

fn spawn_boss_enemy(world: &mut World, position: Vec3, base_y: f32, color: Vec4) -> Entity {
    let main_entity = spawn_mesh(
        world,
        "Cube",
        position + nalgebra_glm::vec3(0.0, base_y, 0.0),
        nalgebra_glm::vec3(0.5, 0.6, 0.4),
    );
    let material_name = format!("BossBody_{}", main_entity.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        Material {
            base_color: color.into(),
            ..Default::default()
        },
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world.set_material_ref(main_entity, MaterialRef::new(material_name));

    spawn_child_cube(
        world,
        main_entity,
        "Boss Head",
        nalgebra_glm::vec3(0.0, 0.5, 0.0),
        nalgebra_glm::vec3(0.4, 0.4, 0.4),
        [color.x * 0.7, color.y * 0.7, color.z * 0.7, 1.0],
    );
    spawn_child_cone_emissive(
        world,
        main_entity,
        "Boss Crown",
        nalgebra_glm::vec3(0.0, 0.8, 0.0),
        nalgebra_glm::vec3(0.28, 0.28, 0.28),
        [1.0, 0.85, 0.0, 1.0],
        [0.6, 0.4, 0.0],
    );
    spawn_child_cube(
        world,
        main_entity,
        "Boss Shoulder L",
        nalgebra_glm::vec3(-0.4, 0.25, 0.0),
        nalgebra_glm::vec3(0.2, 0.25, 0.2),
        [color.x * 1.3, color.y * 1.3, color.z * 1.3, 1.0],
    );
    spawn_child_cube(
        world,
        main_entity,
        "Boss Shoulder R",
        nalgebra_glm::vec3(0.4, 0.25, 0.0),
        nalgebra_glm::vec3(0.2, 0.25, 0.2),
        [color.x * 1.3, color.y * 1.3, color.z * 1.3, 1.0],
    );

    main_entity
}

fn spawn_child_sphere(
    world: &mut World,
    parent: Entity,
    name: &str,
    translation: Vec3,
    scale: Vec3,
    color: [f32; 4],
) {
    let child = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | PARENT,
        1,
    )[0];
    world.set_name(child, Name(name.to_string()));
    world.set_local_transform(
        child,
        LocalTransform {
            translation,
            scale,
            ..Default::default()
        },
    );
    world.set_global_transform(child, GlobalTransform::default());
    world.set_local_transform_dirty(child, LocalTransformDirty);
    world.set_render_mesh(child, RenderMesh::new("Sphere"));
    let material_name = format!("{}_{}", name.replace(' ', ""), child.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        Material {
            base_color: color,
            ..Default::default()
        },
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world.set_material_ref(child, MaterialRef::new(material_name));
    world.set_bounding_volume(child, BoundingVolume::from_mesh_type("Sphere"));
    world.set_parent(child, Parent(Some(parent)));
}

fn spawn_child_sphere_emissive(
    world: &mut World,
    parent: Entity,
    name: &str,
    translation: Vec3,
    scale: Vec3,
    color: [f32; 4],
    emissive: [f32; 3],
) {
    let child = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | PARENT,
        1,
    )[0];
    world.set_name(child, Name(name.to_string()));
    world.set_local_transform(
        child,
        LocalTransform {
            translation,
            scale,
            ..Default::default()
        },
    );
    world.set_global_transform(child, GlobalTransform::default());
    world.set_local_transform_dirty(child, LocalTransformDirty);
    world.set_render_mesh(child, RenderMesh::new("Sphere"));
    let material_name = format!("{}_{}", name.replace(' ', ""), child.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        Material {
            base_color: color,
            emissive_factor: emissive,
            ..Default::default()
        },
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world.set_material_ref(child, MaterialRef::new(material_name));
    world.set_bounding_volume(child, BoundingVolume::from_mesh_type("Sphere"));
    world.set_parent(child, Parent(Some(parent)));
}

fn spawn_child_cube(
    world: &mut World,
    parent: Entity,
    name: &str,
    translation: Vec3,
    scale: Vec3,
    color: [f32; 4],
) {
    let child = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | PARENT,
        1,
    )[0];
    world.set_name(child, Name(name.to_string()));
    world.set_local_transform(
        child,
        LocalTransform {
            translation,
            scale,
            ..Default::default()
        },
    );
    world.set_global_transform(child, GlobalTransform::default());
    world.set_local_transform_dirty(child, LocalTransformDirty);
    world.set_render_mesh(child, RenderMesh::new("Cube"));
    let material_name = format!("{}_{}", name.replace(' ', ""), child.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        Material {
            base_color: color,
            ..Default::default()
        },
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world.set_material_ref(child, MaterialRef::new(material_name));
    world.set_bounding_volume(child, BoundingVolume::from_mesh_type("Cube"));
    world.set_parent(child, Parent(Some(parent)));
}

fn spawn_child_cylinder(
    world: &mut World,
    parent: Entity,
    name: &str,
    translation: Vec3,
    scale: Vec3,
    color: [f32; 4],
    blend: bool,
) {
    let child = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | PARENT,
        1,
    )[0];
    world.set_name(child, Name(name.to_string()));
    world.set_local_transform(
        child,
        LocalTransform {
            translation,
            scale,
            ..Default::default()
        },
    );
    world.set_global_transform(child, GlobalTransform::default());
    world.set_local_transform_dirty(child, LocalTransformDirty);
    world.set_render_mesh(child, RenderMesh::new("Cylinder"));
    let material_name = format!("{}_{}", name.replace(' ', ""), child.id);
    let alpha_mode = if blend {
        AlphaMode::Blend
    } else {
        AlphaMode::Opaque
    };
    let emissive = if blend {
        [0.3, 0.3, 0.6]
    } else {
        [0.0, 0.0, 0.0]
    };
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        Material {
            base_color: color,
            alpha_mode,
            emissive_factor: emissive,
            ..Default::default()
        },
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world.set_material_ref(child, MaterialRef::new(material_name));
    world.set_bounding_volume(child, BoundingVolume::from_mesh_type("Cylinder"));
    world.set_parent(child, Parent(Some(parent)));
}

fn spawn_child_cone_rotated(
    world: &mut World,
    parent: Entity,
    name: &str,
    translation: Vec3,
    scale: Vec3,
    color: [f32; 4],
    rotation_z: f32,
) {
    let child = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | PARENT,
        1,
    )[0];
    world.set_name(child, Name(name.to_string()));
    world.set_local_transform(
        child,
        LocalTransform {
            translation,
            scale,
            rotation: nalgebra_glm::quat_angle_axis(rotation_z, &nalgebra_glm::vec3(0.0, 0.0, 1.0)),
        },
    );
    world.set_global_transform(child, GlobalTransform::default());
    world.set_local_transform_dirty(child, LocalTransformDirty);
    world.set_render_mesh(child, RenderMesh::new("Cone"));
    let material_name = format!("{}_{}", name.replace(' ', ""), child.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        Material {
            base_color: color,
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        },
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world.set_material_ref(child, MaterialRef::new(material_name));
    world.set_bounding_volume(child, BoundingVolume::from_mesh_type("Cone"));
    world.set_parent(child, Parent(Some(parent)));
}

fn spawn_child_cone_emissive(
    world: &mut World,
    parent: Entity,
    name: &str,
    translation: Vec3,
    scale: Vec3,
    color: [f32; 4],
    emissive: [f32; 3],
) {
    let child = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | PARENT,
        1,
    )[0];
    world.set_name(child, Name(name.to_string()));
    world.set_local_transform(
        child,
        LocalTransform {
            translation,
            scale,
            ..Default::default()
        },
    );
    world.set_global_transform(child, GlobalTransform::default());
    world.set_local_transform_dirty(child, LocalTransformDirty);
    world.set_render_mesh(child, RenderMesh::new("Cone"));
    let material_name = format!("{}_{}", name.replace(' ', ""), child.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        Material {
            base_color: color,
            emissive_factor: emissive,
            ..Default::default()
        },
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world.set_material_ref(child, MaterialRef::new(material_name));
    world.set_bounding_volume(child, BoundingVolume::from_mesh_type("Cone"));
    world.set_parent(child, Parent(Some(parent)));
}

pub fn enemy_movement_system(game_world: &mut GameWorld, world: &mut World) {
    let delta_time = world.resources.window.timing.delta_time * game_world.resources.game_speed;
    let path = game_world.resources.path.clone();
    let mut enemies_to_remove = Vec::new();

    for entity in game_world.resources.enemies_list.clone() {
        if let Some(mut enemy) = game_world.get_enemy(entity).copied() {
            if enemy.path_index >= path.len() - 1 {
                enemies_to_remove.push(entity);

                if game_world.resources.current_hp > 0 {
                    game_world.resources.current_hp =
                        game_world.resources.current_hp.saturating_sub(1);
                }

                if game_world.resources.current_hp == 0 {
                    game_world.resources.current_hp = game_world.resources.max_hp;
                    game_world.resources.lives = game_world.resources.lives.saturating_sub(1);

                    if game_world.resources.lives == 0 {
                        game_world.resources.game_state = GameState::GameOver;
                    }
                }
                continue;
            }

            let current_speed = if enemy.slow_duration > 0.0 {
                enemy.speed * 0.5
            } else {
                enemy.speed
            };

            enemy.slow_duration = (enemy.slow_duration - delta_time).max(0.0);

            if enemy.poison_duration > 0.0 {
                enemy.health -= enemy.poison_damage * delta_time;
                enemy.poison_duration = (enemy.poison_duration - delta_time).max(0.0);

                if let Some(pos) = game_world.get_position(entity) {
                    create_poison_bubble_effect(
                        game_world,
                        world,
                        pos.0 + nalgebra_glm::vec3(0.0, enemy.enemy_type.y_offset(), 0.0),
                    );
                }

                if enemy.health <= 0.0 {
                    game_world.resources.money += enemy.value;
                    let death_pos = if let Some(pos) = game_world.get_position(entity) {
                        pos.0 + nalgebra_glm::vec3(0.0, enemy.enemy_type.y_offset(), 0.0)
                    } else {
                        nalgebra_glm::vec3(0.0, 0.0, 0.0)
                    };

                    spawn_money_popup(game_world, world, death_pos, enemy.value as i32);
                    create_death_effect(game_world, world, death_pos);

                    if let Some(idx) = game_world
                        .resources
                        .enemies_list
                        .iter()
                        .position(|&e| e == entity)
                    {
                        game_world.resources.enemies_list.remove(idx);
                    }
                    enemies_to_remove.push(entity);
                    continue;
                }
            }

            let start = path[enemy.path_index];
            let end = path[enemy.path_index + 1];
            let segment_length = (end - start).magnitude();

            enemy.path_progress += (current_speed * delta_time) / segment_length;

            while enemy.path_progress >= 1.0 && enemy.path_index < path.len() - 1 {
                enemy.path_progress -= 1.0;
                enemy.path_index += 1;

                if enemy.path_index >= path.len() - 1 {
                    enemies_to_remove.push(entity);

                    if game_world.resources.current_hp > 0 {
                        game_world.resources.current_hp =
                            game_world.resources.current_hp.saturating_sub(1);
                    }

                    if game_world.resources.current_hp == 0 {
                        game_world.resources.current_hp = game_world.resources.max_hp;
                        game_world.resources.lives = game_world.resources.lives.saturating_sub(1);

                        if game_world.resources.lives == 0 {
                            game_world.resources.game_state = GameState::GameOver;
                        }
                    }
                    break;
                }
            }

            if enemy.path_index < path.len() - 1 && !enemies_to_remove.contains(&entity) {
                let current_start = path[enemy.path_index];
                let current_end = path[enemy.path_index + 1];
                let new_position =
                    current_start + (current_end - current_start) * enemy.path_progress;

                game_world.set_enemy(entity, enemy);
                game_world.set_position(entity, Position(new_position));

                if let Some(handle) = game_world.get_entity_handle(entity)
                    && let Some(transform) = world.get_local_transform_mut(handle.0)
                {
                    let visual_position =
                        new_position + nalgebra_glm::vec3(0.0, enemy.enemy_type.y_offset(), 0.0);
                    transform.translation = visual_position;
                    world.set_local_transform_dirty(handle.0, LocalTransformDirty);
                }
            }
        }
    }

    for entity in enemies_to_remove {
        if let Some(idx) = game_world
            .resources
            .enemies_list
            .iter()
            .position(|&e| e == entity)
        {
            game_world.resources.enemies_list.remove(idx);
        }
        despawn_entity(game_world, world, entity);
    }
}
