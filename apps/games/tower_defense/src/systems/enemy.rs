use crate::commands::Scene;
use crate::components::{
    Bounty, Health, PathIndex, PathProgress, PoisonDamage, PoisonDuration, Position, ShieldHealth,
    SlowDuration, Speed,
};
use crate::data::{EnemyType, GameState};
use crate::queries::read_or_default;
use crate::resources::TowerDefenseResources;
use crate::systems::{
    create_death_effect, create_poison_bubble_effect, despawn_entity, spawn_money_popup,
};
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;

pub fn spawn_enemy(
    game_world: &mut TowerDefenseResources,
    scene: &mut Scene<'_, '_>,
    enemy_type: EnemyType,
) {
    if game_world.path.is_empty() {
        return;
    }

    let position = game_world.path[0];
    let wave = game_world.wave;
    let base_y = crate::data::enemy_y_offset(enemy_type);
    let color = crate::data::enemy_color(enemy_type);

    let enemy_entity = match enemy_type {
        EnemyType::Normal => spawn_normal_enemy(scene, position, base_y, color),
        EnemyType::Fast => spawn_fast_enemy(scene, position, base_y, color),
        EnemyType::Tank => spawn_tank_enemy(scene, position, base_y, color),
        EnemyType::Flying => spawn_flying_enemy(scene, position, base_y, color),
        EnemyType::Shielded => spawn_shielded_enemy(scene, position, base_y, color),
        EnemyType::Healer => spawn_healer_enemy(scene, position, base_y, color),
        EnemyType::Boss => spawn_boss_enemy(scene, position, base_y, color),
    };

    scene.commands.set(enemy_entity, Position(position));
    scene.commands.set(
        enemy_entity,
        Health(crate::data::enemy_health(enemy_type, wave)),
    );
    scene.commands.set(
        enemy_entity,
        ShieldHealth(crate::data::enemy(enemy_type).shield),
    );
    scene
        .commands
        .set(enemy_entity, Speed(crate::data::enemy(enemy_type).speed));
    scene.commands.set(enemy_entity, PathIndex(0));
    scene.commands.set(enemy_entity, PathProgress(0.0));
    scene.commands.set(
        enemy_entity,
        Bounty(crate::data::enemy_value(enemy_type, wave)),
    );
    scene.commands.set(enemy_entity, enemy_type);
    scene.commands.set(enemy_entity, SlowDuration(0.0));
    scene.commands.set(enemy_entity, PoisonDuration(0.0));
    scene.commands.set(enemy_entity, PoisonDamage(0.0));

    game_world.enemies_list.push(enemy_entity);
}

fn spawn_normal_enemy(
    scene: &mut Scene<'_, '_>,
    position: Vec3,
    base_y: f32,
    color: Vec4,
) -> Entity {
    let main_entity = {
        spawn_mesh(
            scene.commands,
            "Cube",
            position + nalgebra_glm::vec3(0.0, base_y, 0.0),
            nalgebra_glm::vec3(0.25, 0.3, 0.2),
        )
    };
    {
        set_material(
            scene.registry,
            scene.commands,
            main_entity,
            format!("GoblinBody_{}", main_entity.id),
            Material {
                base_color: color.into(),
                ..Default::default()
            },
        );
    };

    spawn_child_sphere(
        scene,
        main_entity,
        "Goblin Head",
        nalgebra_glm::vec3(0.0, 0.25, 0.0),
        nalgebra_glm::vec3(0.15, 0.15, 0.15),
        [color.x * 0.9, color.y * 0.9, color.z * 0.9, 1.0],
    );
    spawn_child_cube(
        scene,
        main_entity,
        "Goblin Arm L",
        nalgebra_glm::vec3(-0.2, -0.05, 0.0),
        nalgebra_glm::vec3(0.08, 0.2, 0.08),
        [color.x * 0.8, color.y * 0.8, color.z * 0.8, 1.0],
    );
    spawn_child_cube(
        scene,
        main_entity,
        "Goblin Arm R",
        nalgebra_glm::vec3(0.2, -0.05, 0.0),
        nalgebra_glm::vec3(0.08, 0.2, 0.08),
        [color.x * 0.8, color.y * 0.8, color.z * 0.8, 1.0],
    );

    main_entity
}

fn spawn_fast_enemy(scene: &mut Scene<'_, '_>, position: Vec3, base_y: f32, color: Vec4) -> Entity {
    let main_entity = {
        spawn_mesh(
            scene.commands,
            "Cone",
            position + nalgebra_glm::vec3(0.0, base_y, 0.0),
            nalgebra_glm::vec3(0.22, 0.4, 0.22),
        )
    };
    {
        set_material(
            scene.registry,
            scene.commands,
            main_entity,
            format!("FastBody_{}", main_entity.id),
            Material {
                base_color: color.into(),
                ..Default::default()
            },
        );
    };

    spawn_child_sphere(
        scene,
        main_entity,
        "Fast Head",
        nalgebra_glm::vec3(0.0, 0.3, 0.0),
        nalgebra_glm::vec3(0.13, 0.13, 0.13),
        [color.x * 1.2, color.y * 1.2, color.z * 1.2, 1.0],
    );

    main_entity
}

fn spawn_tank_enemy(scene: &mut Scene<'_, '_>, position: Vec3, base_y: f32, color: Vec4) -> Entity {
    let main_entity = {
        spawn_mesh(
            scene.commands,
            "Cube",
            position + nalgebra_glm::vec3(0.0, base_y, 0.0),
            nalgebra_glm::vec3(0.35, 0.4, 0.3),
        )
    };
    {
        set_material(
            scene.registry,
            scene.commands,
            main_entity,
            format!("TankBody_{}", main_entity.id),
            Material {
                base_color: color.into(),
                ..Default::default()
            },
        );
    };

    spawn_child_cube(
        scene,
        main_entity,
        "Tank Head",
        nalgebra_glm::vec3(0.0, 0.3, 0.0),
        nalgebra_glm::vec3(0.25, 0.25, 0.25),
        [color.x * 0.8, color.y * 0.8, color.z * 0.8, 1.0],
    );
    spawn_child_cube(
        scene,
        main_entity,
        "Tank Shoulder L",
        nalgebra_glm::vec3(-0.25, 0.15, 0.0),
        nalgebra_glm::vec3(0.15, 0.15, 0.15),
        [color.x * 1.2, color.y * 1.2, color.z * 1.2, 1.0],
    );
    spawn_child_cube(
        scene,
        main_entity,
        "Tank Shoulder R",
        nalgebra_glm::vec3(0.25, 0.15, 0.0),
        nalgebra_glm::vec3(0.15, 0.15, 0.15),
        [color.x * 1.2, color.y * 1.2, color.z * 1.2, 1.0],
    );

    main_entity
}

fn spawn_flying_enemy(
    scene: &mut Scene<'_, '_>,
    position: Vec3,
    base_y: f32,
    color: Vec4,
) -> Entity {
    let main_entity = {
        spawn_mesh(
            scene.commands,
            "Sphere",
            position + nalgebra_glm::vec3(0.0, base_y, 0.0),
            nalgebra_glm::vec3(0.22, 0.28, 0.22),
        )
    };
    {
        set_material(
            scene.registry,
            scene.commands,
            main_entity,
            format!("FlyingBody_{}", main_entity.id),
            Material {
                base_color: color.into(),
                emissive_factor: [color.x * 0.35, color.y * 0.35, color.z * 0.35],
                ..Default::default()
            },
        );
    };

    spawn_child_cone_rotated(
        scene,
        main_entity,
        "Wing L",
        nalgebra_glm::vec3(-0.3, 0.0, 0.0),
        nalgebra_glm::vec3(0.18, 0.1, 0.18),
        [color.x * 0.5, color.y * 0.5, color.z * 0.5, 0.7],
        std::f32::consts::FRAC_PI_2,
    );
    spawn_child_cone_rotated(
        scene,
        main_entity,
        "Wing R",
        nalgebra_glm::vec3(0.3, 0.0, 0.0),
        nalgebra_glm::vec3(0.18, 0.1, 0.18),
        [color.x * 0.5, color.y * 0.5, color.z * 0.5, 0.7],
        -std::f32::consts::FRAC_PI_2,
    );

    main_entity
}

fn spawn_shielded_enemy(
    scene: &mut Scene<'_, '_>,
    position: Vec3,
    base_y: f32,
    color: Vec4,
) -> Entity {
    let main_entity = {
        spawn_mesh(
            scene.commands,
            "Cube",
            position + nalgebra_glm::vec3(0.0, base_y, 0.0),
            nalgebra_glm::vec3(0.25, 0.3, 0.2),
        )
    };
    {
        set_material(
            scene.registry,
            scene.commands,
            main_entity,
            format!("ShieldedBody_{}", main_entity.id),
            Material {
                base_color: color.into(),
                ..Default::default()
            },
        );
    };

    spawn_child_sphere(
        scene,
        main_entity,
        "Shielded Head",
        nalgebra_glm::vec3(0.0, 0.25, 0.0),
        nalgebra_glm::vec3(0.15, 0.15, 0.15),
        [color.x * 0.9, color.y * 0.9, color.z * 0.9, 1.0],
    );
    spawn_child_cylinder(
        scene,
        main_entity,
        "Shield",
        nalgebra_glm::vec3(0.2, 0.0, 0.0),
        nalgebra_glm::vec3(0.3, 0.06, 0.3),
        [0.7, 0.7, 1.0, 0.65],
        true,
    );

    main_entity
}

fn spawn_healer_enemy(
    scene: &mut Scene<'_, '_>,
    position: Vec3,
    base_y: f32,
    color: Vec4,
) -> Entity {
    let main_entity = {
        spawn_mesh(
            scene.commands,
            "Torus",
            position + nalgebra_glm::vec3(0.0, base_y, 0.0),
            nalgebra_glm::vec3(0.25, 0.25, 0.25),
        )
    };
    {
        set_material(
            scene.registry,
            scene.commands,
            main_entity,
            format!("HealerBody_{}", main_entity.id),
            Material {
                base_color: color.into(),
                emissive_factor: [color.x * 0.5, color.y * 0.5, color.z * 0.5],
                ..Default::default()
            },
        );
    };

    spawn_child_sphere_emissive(
        scene,
        main_entity,
        "Healer Orb",
        nalgebra_glm::vec3(0.0, 0.0, 0.0),
        nalgebra_glm::vec3(0.16, 0.16, 0.16),
        [0.3, 1.0, 0.3, 1.0],
        [0.0, 0.6, 0.0],
    );

    main_entity
}

fn spawn_boss_enemy(scene: &mut Scene<'_, '_>, position: Vec3, base_y: f32, color: Vec4) -> Entity {
    let main_entity = {
        spawn_mesh(
            scene.commands,
            "Cube",
            position + nalgebra_glm::vec3(0.0, base_y, 0.0),
            nalgebra_glm::vec3(0.5, 0.6, 0.4),
        )
    };
    {
        set_material(
            scene.registry,
            scene.commands,
            main_entity,
            format!("BossBody_{}", main_entity.id),
            Material {
                base_color: color.into(),
                ..Default::default()
            },
        );
    };

    spawn_child_cube(
        scene,
        main_entity,
        "Boss Head",
        nalgebra_glm::vec3(0.0, 0.5, 0.0),
        nalgebra_glm::vec3(0.4, 0.4, 0.4),
        [color.x * 0.7, color.y * 0.7, color.z * 0.7, 1.0],
    );
    spawn_child_cone_emissive(
        scene,
        main_entity,
        "Boss Crown",
        nalgebra_glm::vec3(0.0, 0.8, 0.0),
        nalgebra_glm::vec3(0.28, 0.28, 0.28),
        [1.0, 0.85, 0.0, 1.0],
        [0.6, 0.4, 0.0],
    );
    spawn_child_cube(
        scene,
        main_entity,
        "Boss Shoulder L",
        nalgebra_glm::vec3(-0.4, 0.25, 0.0),
        nalgebra_glm::vec3(0.2, 0.25, 0.2),
        [color.x * 1.3, color.y * 1.3, color.z * 1.3, 1.0],
    );
    spawn_child_cube(
        scene,
        main_entity,
        "Boss Shoulder R",
        nalgebra_glm::vec3(0.4, 0.25, 0.0),
        nalgebra_glm::vec3(0.2, 0.25, 0.2),
        [color.x * 1.3, color.y * 1.3, color.z * 1.3, 1.0],
    );

    main_entity
}

fn spawn_child_sphere(
    scene: &mut Scene<'_, '_>,
    parent: Entity,
    name: &str,
    translation: Vec3,
    scale: Vec3,
    color: [f32; 4],
) {
    let child = scene.commands.spawn(());
    scene.commands.set(child, Name(name.to_string()));
    scene.commands.set(
        child,
        LocalTransform {
            translation,
            scale,
            ..Default::default()
        },
    );
    scene.commands.set(child, GlobalTransform::default());
    scene.commands.set(child, RenderMesh::new("Sphere"));
    {
        set_material(
            scene.registry,
            scene.commands,
            child,
            format!("{}_{}", name.replace(' ', ""), child.id),
            Material {
                base_color: color,
                ..Default::default()
            },
        );
    };
    scene
        .commands
        .set(child, BoundingVolume::from_mesh_type("Sphere"));
    scene.commands.set(child, ChildOf(parent));
}

fn spawn_child_sphere_emissive(
    scene: &mut Scene<'_, '_>,
    parent: Entity,
    name: &str,
    translation: Vec3,
    scale: Vec3,
    color: [f32; 4],
    emissive: [f32; 3],
) {
    let child = scene.commands.spawn(());
    scene.commands.set(child, Name(name.to_string()));
    scene.commands.set(
        child,
        LocalTransform {
            translation,
            scale,
            ..Default::default()
        },
    );
    scene.commands.set(child, GlobalTransform::default());
    scene.commands.set(child, RenderMesh::new("Sphere"));
    {
        set_material(
            scene.registry,
            scene.commands,
            child,
            format!("{}_{}", name.replace(' ', ""), child.id),
            Material {
                base_color: color,
                emissive_factor: emissive,
                ..Default::default()
            },
        );
    };
    scene
        .commands
        .set(child, BoundingVolume::from_mesh_type("Sphere"));
    scene.commands.set(child, ChildOf(parent));
}

fn spawn_child_cube(
    scene: &mut Scene<'_, '_>,
    parent: Entity,
    name: &str,
    translation: Vec3,
    scale: Vec3,
    color: [f32; 4],
) {
    let child = scene.commands.spawn(());
    scene.commands.set(child, Name(name.to_string()));
    scene.commands.set(
        child,
        LocalTransform {
            translation,
            scale,
            ..Default::default()
        },
    );
    scene.commands.set(child, GlobalTransform::default());
    scene.commands.set(child, RenderMesh::new("Cube"));
    {
        set_material(
            scene.registry,
            scene.commands,
            child,
            format!("{}_{}", name.replace(' ', ""), child.id),
            Material {
                base_color: color,
                ..Default::default()
            },
        );
    };
    scene
        .commands
        .set(child, BoundingVolume::from_mesh_type("Cube"));
    scene.commands.set(child, ChildOf(parent));
}

fn spawn_child_cylinder(
    scene: &mut Scene<'_, '_>,
    parent: Entity,
    name: &str,
    translation: Vec3,
    scale: Vec3,
    color: [f32; 4],
    blend: bool,
) {
    let child = scene.commands.spawn(());
    scene.commands.set(child, Name(name.to_string()));
    scene.commands.set(
        child,
        LocalTransform {
            translation,
            scale,
            ..Default::default()
        },
    );
    scene.commands.set(child, GlobalTransform::default());
    scene.commands.set(child, RenderMesh::new("Cylinder"));
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
    {
        set_material(
            scene.registry,
            scene.commands,
            child,
            material_name,
            Material {
                base_color: color,
                alpha_mode,
                emissive_factor: emissive,
                ..Default::default()
            },
        );
    };
    scene
        .commands
        .set(child, BoundingVolume::from_mesh_type("Cylinder"));
    scene.commands.set(child, ChildOf(parent));
}

fn spawn_child_cone_rotated(
    scene: &mut Scene<'_, '_>,
    parent: Entity,
    name: &str,
    translation: Vec3,
    scale: Vec3,
    color: [f32; 4],
    rotation_z: f32,
) {
    let child = scene.commands.spawn(());
    scene.commands.set(child, Name(name.to_string()));
    scene.commands.set(
        child,
        LocalTransform {
            translation,
            scale,
            rotation: nalgebra_glm::quat_angle_axis(rotation_z, &nalgebra_glm::vec3(0.0, 0.0, 1.0)),
        },
    );
    scene.commands.set(child, GlobalTransform::default());
    scene.commands.set(child, RenderMesh::new("Cone"));
    {
        set_material(
            scene.registry,
            scene.commands,
            child,
            format!("{}_{}", name.replace(' ', ""), child.id),
            Material {
                base_color: color,
                alpha_mode: AlphaMode::Blend,
                ..Default::default()
            },
        );
    };
    scene
        .commands
        .set(child, BoundingVolume::from_mesh_type("Cone"));
    scene.commands.set(child, ChildOf(parent));
}

fn spawn_child_cone_emissive(
    scene: &mut Scene<'_, '_>,
    parent: Entity,
    name: &str,
    translation: Vec3,
    scale: Vec3,
    color: [f32; 4],
    emissive: [f32; 3],
) {
    let child = scene.commands.spawn(());
    scene.commands.set(child, Name(name.to_string()));
    scene.commands.set(
        child,
        LocalTransform {
            translation,
            scale,
            ..Default::default()
        },
    );
    scene.commands.set(child, GlobalTransform::default());
    scene.commands.set(child, RenderMesh::new("Cone"));
    {
        set_material(
            scene.registry,
            scene.commands,
            child,
            format!("{}_{}", name.replace(' ', ""), child.id),
            Material {
                base_color: color,
                emissive_factor: emissive,
                ..Default::default()
            },
        );
    };
    scene
        .commands
        .set(child, BoundingVolume::from_mesh_type("Cone"));
    scene.commands.set(child, ChildOf(parent));
}

pub fn enemy_movement_system(
    game_world: &mut TowerDefenseResources,
    scene: &mut Scene<'_, '_>,
    delta_time: f32,
) {
    let path = game_world.path.clone();
    let mut enemies_to_remove = Vec::new();

    for entity in game_world.enemies_list.clone() {
        let Some(enemy_type) = scene.storage.get::<EnemyType>(entity).copied() else {
            continue;
        };
        let mut path_index = read_or_default::<PathIndex>(scene, entity).0;
        let mut path_progress = read_or_default::<PathProgress>(scene, entity).0;
        let mut health = read_or_default::<Health>(scene, entity).0;
        let mut slow_duration = read_or_default::<SlowDuration>(scene, entity).0;
        let mut poison_duration = read_or_default::<PoisonDuration>(scene, entity).0;
        let poison_damage = read_or_default::<PoisonDamage>(scene, entity).0;
        let speed = read_or_default::<Speed>(scene, entity).0;
        let bounty = read_or_default::<Bounty>(scene, entity).0;

        if path_index >= path.len() - 1 {
            enemies_to_remove.push(entity);
            take_life(game_world);
            continue;
        }

        let current_speed = if slow_duration > 0.0 {
            speed * 0.5
        } else {
            speed
        };

        slow_duration = (slow_duration - delta_time).max(0.0);

        if poison_duration > 0.0 {
            health -= poison_damage * delta_time;
            poison_duration = (poison_duration - delta_time).max(0.0);

            if let Some(pos) = scene.storage.get::<Position>(entity).copied() {
                create_poison_bubble_effect(
                    game_world,
                    scene,
                    pos.0 + nalgebra_glm::vec3(0.0, crate::data::enemy_y_offset(enemy_type), 0.0),
                );
            }

            if health <= 0.0 {
                game_world.money += bounty;
                let death_pos = if let Some(pos) = scene.storage.get::<Position>(entity) {
                    pos.0 + nalgebra_glm::vec3(0.0, crate::data::enemy_y_offset(enemy_type), 0.0)
                } else {
                    nalgebra_glm::vec3(0.0, 0.0, 0.0)
                };

                spawn_money_popup(scene, death_pos, bounty as i32);
                create_death_effect(scene, death_pos);

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

        let start = path[path_index];
        let end = path[path_index + 1];
        let segment_length = (end - start).magnitude();

        path_progress += (current_speed * delta_time) / segment_length;

        while path_progress >= 1.0 && path_index < path.len() - 1 {
            path_progress -= 1.0;
            path_index += 1;

            if path_index >= path.len() - 1 {
                enemies_to_remove.push(entity);
                take_life(game_world);
                break;
            }
        }

        if path_index < path.len() - 1 && !enemies_to_remove.contains(&entity) {
            let current_start = path[path_index];
            let current_end = path[path_index + 1];
            let new_position = current_start + (current_end - current_start) * path_progress;

            scene.storage.set(entity, Health(health));
            scene.storage.set(entity, SlowDuration(slow_duration));
            scene.storage.set(entity, PoisonDuration(poison_duration));
            scene.storage.set(entity, PathIndex(path_index));
            scene.storage.set(entity, PathProgress(path_progress));
            scene.storage.set(entity, Position(new_position));

            if let Some(transform) = scene
                .storage
                .get_mut::<nightshade_scene::contract::LocalTransform>(entity)
            {
                let visual_position = new_position
                    + nalgebra_glm::vec3(0.0, crate::data::enemy_y_offset(enemy_type), 0.0);
                transform.translation = visual_position;
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
        despawn_entity(scene, entity);
    }
}

fn take_life(game_world: &mut TowerDefenseResources) {
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
}
