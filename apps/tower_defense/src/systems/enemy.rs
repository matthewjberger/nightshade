use crate::ecs::{Enemy, EnemyType, GameState, Position, TowerDefenseResources};
use crate::systems::{
    create_death_effect, create_poison_bubble_effect, despawn_entity, spawn_money_popup,
};
use nightshade::prelude::*;

pub fn spawn_enemy(
    game_world: &mut TowerDefenseResources,
    world: &mut World,
    enemy_type: EnemyType,
) {
    if game_world.path.is_empty() {
        return;
    }

    let position = game_world.path[0];
    let wave = game_world.wave;
    let base_y = enemy_type.y_offset();
    let color = enemy_type.color();

    let enemy_entity = match enemy_type {
        EnemyType::Normal => spawn_normal_enemy(world, position, base_y, color),
        EnemyType::Fast => spawn_fast_enemy(world, position, base_y, color),
        EnemyType::Tank => spawn_tank_enemy(world, position, base_y, color),
        EnemyType::Flying => spawn_flying_enemy(world, position, base_y, color),
        EnemyType::Shielded => spawn_shielded_enemy(world, position, base_y, color),
        EnemyType::Healer => spawn_healer_enemy(world, position, base_y, color),
        EnemyType::Boss => spawn_boss_enemy(world, position, base_y, color),
    };

    world.set(enemy_entity, Position(position));
    world.set(
        enemy_entity,
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

    game_world.enemies_list.push(enemy_entity);
}

fn spawn_normal_enemy(world: &mut World, position: Vec3, base_y: f32, color: Vec4) -> Entity {
    let main_entity = spawn_mesh(
        world,
        "Cube",
        position + nalgebra_glm::vec3(0.0, base_y, 0.0),
        nalgebra_glm::vec3(0.25, 0.3, 0.2),
    );
    set_material(
        world,
        main_entity,
        format!("GoblinBody_{}", main_entity.id),
        Material {
            base_color: color.into(),
            ..Default::default()
        },
    );

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
    set_material(
        world,
        main_entity,
        format!("FastBody_{}", main_entity.id),
        Material {
            base_color: color.into(),
            ..Default::default()
        },
    );

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
    set_material(
        world,
        main_entity,
        format!("TankBody_{}", main_entity.id),
        Material {
            base_color: color.into(),
            ..Default::default()
        },
    );

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
    set_material(
        world,
        main_entity,
        format!("FlyingBody_{}", main_entity.id),
        Material {
            base_color: color.into(),
            emissive_factor: [color.x * 0.35, color.y * 0.35, color.z * 0.35],
            ..Default::default()
        },
    );

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
    set_material(
        world,
        main_entity,
        format!("ShieldedBody_{}", main_entity.id),
        Material {
            base_color: color.into(),
            ..Default::default()
        },
    );

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
    set_material(
        world,
        main_entity,
        format!("HealerBody_{}", main_entity.id),
        Material {
            base_color: color.into(),
            emissive_factor: [color.x * 0.5, color.y * 0.5, color.z * 0.5],
            ..Default::default()
        },
    );

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
    set_material(
        world,
        main_entity,
        format!("BossBody_{}", main_entity.id),
        Material {
            base_color: color.into(),
            ..Default::default()
        },
    );

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
    let child = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | PARENT,
        1,
    )[0];
    world.set(child, Name(name.to_string()));
    world.set(
        child,
        LocalTransform {
            translation,
            scale,
            ..Default::default()
        },
    );
    world.set(child, GlobalTransform::default());
    world.set(child, RenderMesh::new("Sphere"));
    set_material(
        world,
        child,
        format!("{}_{}", name.replace(' ', ""), child.id),
        Material {
            base_color: color,
            ..Default::default()
        },
    );
    world.set(child, BoundingVolume::from_mesh_type("Sphere"));
    world.set(child, ChildOf(parent));
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
    let child = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | PARENT,
        1,
    )[0];
    world.set(child, Name(name.to_string()));
    world.set(
        child,
        LocalTransform {
            translation,
            scale,
            ..Default::default()
        },
    );
    world.set(child, GlobalTransform::default());
    world.set(child, RenderMesh::new("Sphere"));
    set_material(
        world,
        child,
        format!("{}_{}", name.replace(' ', ""), child.id),
        Material {
            base_color: color,
            emissive_factor: emissive,
            ..Default::default()
        },
    );
    world.set(child, BoundingVolume::from_mesh_type("Sphere"));
    world.set(child, ChildOf(parent));
}

fn spawn_child_cube(
    world: &mut World,
    parent: Entity,
    name: &str,
    translation: Vec3,
    scale: Vec3,
    color: [f32; 4],
) {
    let child = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | PARENT,
        1,
    )[0];
    world.set(child, Name(name.to_string()));
    world.set(
        child,
        LocalTransform {
            translation,
            scale,
            ..Default::default()
        },
    );
    world.set(child, GlobalTransform::default());
    world.set(child, RenderMesh::new("Cube"));
    set_material(
        world,
        child,
        format!("{}_{}", name.replace(' ', ""), child.id),
        Material {
            base_color: color,
            ..Default::default()
        },
    );
    world.set(child, BoundingVolume::from_mesh_type("Cube"));
    world.set(child, ChildOf(parent));
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
    let child = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | PARENT,
        1,
    )[0];
    world.set(child, Name(name.to_string()));
    world.set(
        child,
        LocalTransform {
            translation,
            scale,
            ..Default::default()
        },
    );
    world.set(child, GlobalTransform::default());
    world.set(child, RenderMesh::new("Cylinder"));
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
    set_material(
        world,
        child,
        material_name,
        Material {
            base_color: color,
            alpha_mode,
            emissive_factor: emissive,
            ..Default::default()
        },
    );
    world.set(child, BoundingVolume::from_mesh_type("Cylinder"));
    world.set(child, ChildOf(parent));
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
    let child = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | PARENT,
        1,
    )[0];
    world.set(child, Name(name.to_string()));
    world.set(
        child,
        LocalTransform {
            translation,
            scale,
            rotation: nalgebra_glm::quat_angle_axis(rotation_z, &nalgebra_glm::vec3(0.0, 0.0, 1.0)),
        },
    );
    world.set(child, GlobalTransform::default());
    world.set(child, RenderMesh::new("Cone"));
    set_material(
        world,
        child,
        format!("{}_{}", name.replace(' ', ""), child.id),
        Material {
            base_color: color,
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        },
    );
    world.set(child, BoundingVolume::from_mesh_type("Cone"));
    world.set(child, ChildOf(parent));
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
    let child = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | PARENT,
        1,
    )[0];
    world.set(child, Name(name.to_string()));
    world.set(
        child,
        LocalTransform {
            translation,
            scale,
            ..Default::default()
        },
    );
    world.set(child, GlobalTransform::default());
    world.set(child, RenderMesh::new("Cone"));
    set_material(
        world,
        child,
        format!("{}_{}", name.replace(' ', ""), child.id),
        Material {
            base_color: color,
            emissive_factor: emissive,
            ..Default::default()
        },
    );
    world.set(child, BoundingVolume::from_mesh_type("Cone"));
    world.set(child, ChildOf(parent));
}

pub fn enemy_movement_system(game_world: &mut TowerDefenseResources, world: &mut World) {
    let delta_time = world.res::<Time>().delta_time * game_world.game_speed;
    let path = game_world.path.clone();
    let mut enemies_to_remove = Vec::new();

    for entity in game_world.enemies_list.clone() {
        if let Some(mut enemy) = world.get::<Enemy>(entity).copied() {
            if enemy.path_index >= path.len() - 1 {
                enemies_to_remove.push(entity);

                if game_world.current_hp > 0 {
                    game_world.current_hp = game_world.current_hp.saturating_sub(1);
                }

                if game_world.current_hp == 0 {
                    game_world.current_hp = game_world.max_hp;
                    game_world.lives = game_world.lives.saturating_sub(1);

                    if game_world.lives == 0 {
                        game_world.game_state = GameState::GameOver;
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

                if let Some(pos) = world.get::<Position>(entity).copied() {
                    create_poison_bubble_effect(
                        game_world,
                        world,
                        pos.0 + nalgebra_glm::vec3(0.0, enemy.enemy_type.y_offset(), 0.0),
                    );
                }

                if enemy.health <= 0.0 {
                    game_world.money += enemy.value;
                    let death_pos = if let Some(pos) = world.get::<Position>(entity) {
                        pos.0 + nalgebra_glm::vec3(0.0, enemy.enemy_type.y_offset(), 0.0)
                    } else {
                        nalgebra_glm::vec3(0.0, 0.0, 0.0)
                    };

                    spawn_money_popup(world, death_pos, enemy.value as i32);
                    create_death_effect(world, death_pos);

                    if let Some(index) = game_world
                        .enemies_list
                        .iter()
                        .position(|&candidate| candidate == entity)
                    {
                        game_world.enemies_list.remove(index);
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

                    if game_world.current_hp > 0 {
                        game_world.current_hp = game_world.current_hp.saturating_sub(1);
                    }

                    if game_world.current_hp == 0 {
                        game_world.current_hp = game_world.max_hp;
                        game_world.lives = game_world.lives.saturating_sub(1);

                        if game_world.lives == 0 {
                            game_world.game_state = GameState::GameOver;
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

                world.set(entity, enemy);
                world.set(entity, Position(new_position));

                if let Some(transform) =
                    world.get_mut::<nightshade::ecs::transform::components::LocalTransform>(entity)
                {
                    let visual_position =
                        new_position + nalgebra_glm::vec3(0.0, enemy.enemy_type.y_offset(), 0.0);
                    transform.translation = visual_position;
                }
            }
        }
    }

    for entity in enemies_to_remove {
        if let Some(index) = game_world
            .enemies_list
            .iter()
            .position(|&candidate| candidate == entity)
        {
            game_world.enemies_list.remove(index);
        }
        despawn_entity(world, entity);
    }
}
