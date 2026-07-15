use crate::ecs::{
    GRID_CELL, GridCell, Position, Projectile, RangeIndicator, TOWER, Tower, TowerDefenseResources,
    TowerType,
};
use crate::systems::{create_muzzle_flash, spawn_money_popup};
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::prelude::*;

pub fn spawn_tower(
    game_world: &mut TowerDefenseResources,
    world: &mut World,
    grid_x: i32,
    grid_z: i32,
    tower_type: TowerType,
) {
    let position = nalgebra_glm::vec3(grid_x as f32, 0.0, grid_z as f32);
    let color = tower_type.color();

    let tower_entity = spawn_mesh(
        world,
        "Cylinder",
        position,
        nalgebra_glm::vec3(0.4, 0.8, 0.4),
    );

    let material_name = format!("Tower_{}", tower_entity.id);
    material_registry_insert(
        &mut world.resources.assets.material_registry,
        material_name.clone(),
        Material {
            base_color: color.into(),
            ..Default::default()
        },
    );
    if let Some(&index) = world
        .resources
        .assets
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        registry_add_reference(
            &mut world.resources.assets.material_registry.registry,
            index,
        );
    }
    world.set(tower_entity, MaterialRef::new(material_name));

    world.set(tower_entity, Position(position));
    world.set(
        tower_entity,
        Tower {
            tower_type,
            cooldown: 0.0,
            target: None,
            fire_animation: 0.0,
            tracking_time: 0.0,
            target_line: None,
        },
    );

    game_world.money -= tower_type.cost();
    game_world
        .towers_by_position
        .insert((grid_x, grid_z), tower_entity);

    spawn_range_indicator(world, position, tower_type.range(), tower_entity);
}

fn spawn_range_indicator(world: &mut World, center: Vec3, range: f32, tower_entity: Entity) {
    let segments = 32;
    let mut line_entities = Vec::new();

    let tower_color = if let Some(tower) = world.get::<Tower>(tower_entity) {
        tower.tower_type.color()
    } else {
        nalgebra_glm::vec4(0.0, 1.0, 0.0, 1.0)
    };

    for segment_index in 0..segments {
        let angle1 = (segment_index as f32 / segments as f32) * std::f32::consts::TAU;
        let angle2 = ((segment_index + 1) as f32 / segments as f32) * std::f32::consts::TAU;

        let start = center + nalgebra_glm::vec3(angle1.cos() * range, 0.1, angle1.sin() * range);
        let end = center + nalgebra_glm::vec3(angle2.cos() * range, 0.1, angle2.sin() * range);

        let line_entity = spawn_entities(
            world,
            NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LINES | VISIBILITY,
            1,
        )[0];

        world.set(
            line_entity,
            Name(format!("Range Indicator Line {}", segment_index)),
        );
        world.set(
            line_entity,
            LocalTransform {
                translation: nalgebra_glm::vec3(0.0, 0.0, 0.0),
                ..Default::default()
            },
        );
        world.set(line_entity, GlobalTransform::default());

        world.set(
            line_entity,
            Lines::new(vec![Line {
                start,
                end,
                color: nalgebra_glm::vec4(tower_color.x, tower_color.y, tower_color.z, 0.5),
            }]),
        );

        world.set(line_entity, Visibility { visible: false });

        line_entities.push(line_entity);
    }

    world.set(
        tower_entity,
        RangeIndicator {
            line_entities,
            visible: false,
        },
    );
}

pub fn despawn_entity(world: &mut World, entity: Entity) {
    let target_line = world
        .get::<Tower>(entity)
        .and_then(|tower| tower.target_line);
    if let Some(line_entity) = target_line {
        queue_ecs_command(
            world,
            EcsCommand::DespawnRecursive {
                entity: line_entity,
            },
        );
    }

    let line_entities = world
        .get::<RangeIndicator>(entity)
        .map(|indicator| indicator.line_entities.clone());
    if let Some(line_entities) = line_entities {
        for line in line_entities {
            queue_ecs_command(world, EcsCommand::DespawnRecursive { entity: line });
        }
    }

    despawn_recursive_immediate(world, entity);
}

pub fn sell_tower(
    game_world: &mut TowerDefenseResources,
    world: &mut World,
    tower_entity: Entity,
    grid_x: i32,
    grid_z: i32,
) {
    if let Some(tower) = world.get::<Tower>(tower_entity).copied() {
        let refund = (tower.tower_type.cost() as f32 * 0.7) as u32;
        game_world.money += refund;

        let position = nalgebra_glm::vec3(grid_x as f32, 0.5, grid_z as f32);
        spawn_money_popup(world, position, refund as i32);

        let cell_entities: Vec<Entity> = world.ecs.worlds[GAME].query_entities(GRID_CELL).collect();
        for cell_entity in cell_entities {
            if let Some(cell) = world.get_mut::<GridCell>(cell_entity)
                && cell.x == grid_x
                && cell.z == grid_z
            {
                cell.occupied = false;
                break;
            }
        }

        game_world.towers_by_position.remove(&(grid_x, grid_z));

        despawn_entity(world, tower_entity);
    }
}

pub fn tower_targeting_system(game_world: &mut TowerDefenseResources, world: &mut World) {
    let delta_time = world
        .expect_resource_mut::<nightshade::ecs::window::resources::Window>()
        .timing
        .delta_time
        * game_world.game_speed;
    let tower_entities: Vec<Entity> = world.ecs.worlds[GAME].query_entities(TOWER).collect();

    for tower_entity in tower_entities {
        if let Some(mut tower) = world.get::<Tower>(tower_entity).copied() {
            let is_sniper = tower.tower_type == TowerType::Sniper;

            if is_sniper {
                let mut target_valid = false;
                if let Some(current_target) = tower.target
                    && game_world.enemies_list.contains(&current_target)
                {
                    target_valid = true;
                }

                if !target_valid {
                    tower.target = None;
                    tower.tracking_time = 0.0;

                    if let Some(line_entity) = tower.target_line.take() {
                        queue_ecs_command(
                            world,
                            EcsCommand::DespawnRecursive {
                                entity: line_entity,
                            },
                        );
                    }
                }
            } else if let Some(target) = tower.target
                && !game_world.enemies_list.contains(&target)
            {
                tower.target = None;
            }

            if tower.target.is_none()
                && let Some(tower_pos) = world.get::<Position>(tower_entity).copied()
            {
                let range = tower.tower_type.range();
                let mut closest_enemy: Option<(Entity, f32)> = None;

                for &enemy_entity in &game_world.enemies_list {
                    if let Some(enemy_pos) = world.get::<Position>(enemy_entity).copied() {
                        let distance = (enemy_pos.0 - tower_pos.0).magnitude();
                        if distance <= range {
                            if let Some((_, closest_dist)) = closest_enemy {
                                if distance < closest_dist {
                                    closest_enemy = Some((enemy_entity, distance));
                                }
                            } else {
                                closest_enemy = Some((enemy_entity, distance));
                            }
                        }
                    }
                }

                if let Some((enemy, _)) = closest_enemy {
                    tower.target = Some(enemy);
                }
            } else if is_sniper && tower.target.is_some() {
                tower.tracking_time += delta_time;

                if let (Some(target), Some(tower_pos)) =
                    (tower.target, world.get::<Position>(tower_entity).copied())
                    && game_world.enemies_list.contains(&target)
                    && let Some(enemy_pos) = world.get::<Position>(target).copied()
                {
                    if tower.target_line.is_none() {
                        let line_entity = spawn_entities(
                            world,
                            LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LINES | VISIBILITY,
                            1,
                        )[0];
                        tower.target_line = Some(line_entity);
                    }

                    if let Some(line_entity) = tower.target_line {
                        if let Some(lines_comp) =
                            world.get_mut::<nightshade::ecs::lines::components::Lines>(line_entity)
                        {
                            lines_comp.lines.clear();
                            lines_comp.lines.push(Line {
                                start: tower_pos.0 + nalgebra_glm::vec3(0.0, 0.5, 0.0),
                                end: enemy_pos.0 + nalgebra_glm::vec3(0.0, 0.3, 0.0),
                                color: nalgebra_glm::vec4(1.0, 0.0, 0.0, 0.8),
                            });
                        }

                        if let Some(visibility) =
                            world.get_mut::<nightshade::ecs::primitives::Visibility>(line_entity)
                        {
                            visibility.visible = true;
                        }
                    }
                }
            }

            world.set(tower_entity, tower);
        }
    }
}

pub fn tower_shooting_system(game_world: &mut TowerDefenseResources, world: &mut World) {
    let delta_time = world
        .expect_resource_mut::<nightshade::ecs::window::resources::Window>()
        .timing
        .delta_time
        * game_world.game_speed;
    let tower_entities: Vec<Entity> = world.ecs.worlds[GAME].query_entities(TOWER).collect();

    for tower_entity in tower_entities {
        if let Some(mut tower) = world.get::<Tower>(tower_entity).copied() {
            tower.cooldown = (tower.cooldown - delta_time).max(0.0);

            let mut needs_fire_animation_update = false;
            let mut animation_scale = 1.0;

            if tower.fire_animation > 0.0 {
                tower.fire_animation -= delta_time * 3.0;
                needs_fire_animation_update = true;
                animation_scale = 1.0 + tower.fire_animation * 0.2;
            }

            if tower.cooldown <= 0.0
                && let Some(target) = tower.target
            {
                let can_fire = if tower.tower_type == TowerType::Sniper {
                    tower.tracking_time >= 2.0
                } else {
                    true
                };

                if can_fire
                    && game_world.enemies_list.contains(&target)
                    && world.has::<Position>(target)
                    && let Some(tower_pos) = world.get::<Position>(tower_entity).copied()
                {
                    let projectile_entity = spawn_mesh(
                        world,
                        "Sphere",
                        tower_pos.0 + nalgebra_glm::vec3(0.0, 0.5, 0.0),
                        nalgebra_glm::vec3(0.15, 0.15, 0.15),
                    );

                    let material_name = format!("Projectile_{}", projectile_entity.id);
                    material_registry_insert(
                        &mut world.resources.assets.material_registry,
                        material_name.clone(),
                        Material {
                            base_color: tower.tower_type.color().into(),
                            emissive_factor: [
                                tower.tower_type.color().x * 0.5,
                                tower.tower_type.color().y * 0.5,
                                tower.tower_type.color().z * 0.5,
                            ],
                            ..Default::default()
                        },
                    );
                    if let Some(&index) = world
                        .resources
                        .assets
                        .material_registry
                        .registry
                        .name_to_index
                        .get(&material_name)
                    {
                        registry_add_reference(
                            &mut world.resources.assets.material_registry.registry,
                            index,
                        );
                    }
                    world.set(projectile_entity, MaterialRef::new(material_name));

                    world.set(
                        projectile_entity,
                        Position(tower_pos.0 + nalgebra_glm::vec3(0.0, 0.5, 0.0)),
                    );
                    world.set(
                        projectile_entity,
                        Projectile {
                            damage: tower.tower_type.damage(),
                            target,
                            speed: tower.tower_type.projectile_speed(),
                            tower_type: tower.tower_type,
                            start_position: tower_pos.0 + nalgebra_glm::vec3(0.0, 0.5, 0.0),
                            arc_height: if tower.tower_type == TowerType::Cannon {
                                2.0
                            } else {
                                0.0
                            },
                            flight_progress: 0.0,
                        },
                    );

                    game_world.projectiles_list.push(projectile_entity);

                    tower.cooldown = tower.tower_type.fire_rate();
                    tower.fire_animation = 1.0;
                    needs_fire_animation_update = true;
                    animation_scale = 1.0 + tower.fire_animation * 0.2;

                    if tower.tower_type == TowerType::Cannon {
                        create_muzzle_flash(
                            game_world,
                            world,
                            tower_pos.0 + nalgebra_glm::vec3(0.0, 0.8, 0.0),
                        );
                    }
                }
            }

            if needs_fire_animation_update {
                if let Some(transform) = world
                    .get_mut::<nightshade::ecs::transform::components::LocalTransform>(tower_entity)
                {
                    transform.scale =
                        nalgebra_glm::vec3(0.4 * animation_scale, 0.8, 0.4 * animation_scale);
                }

                if let Some(material_ref) = world
                    .get::<nightshade::ecs::material::components::MaterialRef>(tower_entity)
                    .cloned()
                {
                    let flash_intensity = if animation_scale > 1.05 {
                        (animation_scale - 1.0) * 5.0
                    } else {
                        0.0
                    };
                    material_registry_mutate(
                        &mut world.resources.assets.material_registry,
                        &material_ref.name,
                        |material| {
                            material.emissive_factor = [
                                tower.tower_type.color().x * 0.5 + flash_intensity,
                                tower.tower_type.color().y * 0.5 + flash_intensity,
                                tower.tower_type.color().z * 0.5 + flash_intensity,
                            ];
                        },
                    );
                }
            }

            world.set(tower_entity, tower);
        }
    }
}
