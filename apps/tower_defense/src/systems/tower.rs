use crate::ecs::{
    ENTITY_HANDLE, EntityHandle, GRID_CELL, GameWorld, POSITION, PROJECTILE, Position, Projectile,
    RANGE_INDICATOR, RangeIndicator, TOWER, Tower, TowerType,
};
use crate::systems::{create_muzzle_flash, spawn_money_popup};
use nightshade::ecs::generational_registry::registry_entry_by_name_mut;
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::prelude::*;

pub fn spawn_tower(
    game_world: &mut GameWorld,
    world: &mut World,
    grid_x: i32,
    grid_z: i32,
    tower_type: TowerType,
) -> freecs::Entity {
    let position = nalgebra_glm::vec3(grid_x as f32, 0.0, grid_z as f32);
    let color = tower_type.color();

    let engine_entity = spawn_mesh(
        world,
        "Cylinder",
        position,
        nalgebra_glm::vec3(0.4, 0.8, 0.4),
    );

    let material_name = format!("Tower_{}", engine_entity.id);
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
    world.set_material_ref(engine_entity, MaterialRef::new(material_name));

    let game_entity = game_world.spawn_entities(ENTITY_HANDLE | POSITION | TOWER, 1)[0];
    game_world.set_entity_handle(game_entity, EntityHandle(engine_entity));
    game_world.set_position(game_entity, Position(position));
    game_world.set_tower(
        game_entity,
        Tower {
            tower_type,
            cooldown: 0.0,
            target: None,
            fire_animation: 0.0,
            tracking_time: 0.0,
            target_line: None,
        },
    );

    game_world.resources.money -= tower_type.cost();
    game_world
        .resources
        .towers_by_position
        .insert((grid_x, grid_z), game_entity);

    spawn_range_indicator(game_world, world, position, tower_type.range(), game_entity);

    game_entity
}

pub fn spawn_range_indicator(
    game_world: &mut GameWorld,
    world: &mut World,
    center: Vec3,
    range: f32,
    tower_entity: freecs::Entity,
) {
    let segments = 32;
    let mut line_entities = Vec::new();

    let tower_color = if let Some(tower) = game_world.get_tower(tower_entity) {
        tower.tower_type.color()
    } else {
        nalgebra_glm::vec4(0.0, 1.0, 0.0, 1.0)
    };

    for segment_index in 0..segments {
        let angle1 = (segment_index as f32 / segments as f32) * std::f32::consts::TAU;
        let angle2 = ((segment_index + 1) as f32 / segments as f32) * std::f32::consts::TAU;

        let start = center + nalgebra_glm::vec3(angle1.cos() * range, 0.1, angle1.sin() * range);
        let end = center + nalgebra_glm::vec3(angle2.cos() * range, 0.1, angle2.sin() * range);

        let line_entity = world.spawn_entities(
            NAME | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LOCAL_TRANSFORM_DIRTY | LINES | VISIBILITY,
            1,
        )[0];

        world.set_name(
            line_entity,
            Name(format!("Range Indicator Line {}", segment_index)),
        );
        world.set_local_transform(
            line_entity,
            LocalTransform {
                translation: nalgebra_glm::vec3(0.0, 0.0, 0.0),
                ..Default::default()
            },
        );
        world.set_global_transform(line_entity, GlobalTransform::default());
        world.set_local_transform_dirty(line_entity, LocalTransformDirty);

        world.set_lines(
            line_entity,
            Lines::new(vec![Line {
                start,
                end,
                color: nalgebra_glm::vec4(tower_color.x, tower_color.y, tower_color.z, 0.5),
            }]),
        );

        world.set_visibility(line_entity, Visibility { visible: false });

        line_entities.push(line_entity);
    }

    let game_entity = game_world.spawn_entities(RANGE_INDICATOR, 1)[0];
    game_world.set_range_indicator(
        game_entity,
        RangeIndicator {
            tower_entity,
            line_entities,
            visible: false,
        },
    );
}

pub fn despawn_entity(game_world: &mut GameWorld, world: &mut World, entity: freecs::Entity) {
    if let Some(handle) = game_world.get_entity_handle(entity) {
        world
            .resources
            .command_queue
            .push(WorldCommand::DespawnRecursive { entity: handle.0 });
    }

    if let Some(tower) = game_world.get_tower(entity)
        && let Some(line_entity) = tower.target_line
    {
        world
            .resources
            .command_queue
            .push(WorldCommand::DespawnRecursive {
                entity: line_entity,
            });
    }

    let range_indicators_to_remove: Vec<_> = game_world
        .query_entities(RANGE_INDICATOR)
        .filter_map(|range_entity| {
            game_world
                .get_range_indicator(range_entity)
                .filter(|indicator| indicator.tower_entity == entity)
                .map(|indicator| (range_entity, indicator.clone()))
        })
        .collect();

    for (range_entity, indicator) in range_indicators_to_remove {
        for &line in &indicator.line_entities {
            world
                .resources
                .command_queue
                .push(WorldCommand::DespawnRecursive { entity: line });
        }
        game_world.despawn_entities(&[range_entity]);
    }

    game_world.despawn_entities(&[entity]);
}

pub fn sell_tower(
    game_world: &mut GameWorld,
    world: &mut World,
    tower_entity: freecs::Entity,
    grid_x: i32,
    grid_z: i32,
) {
    if let Some(tower) = game_world.get_tower(tower_entity) {
        let refund = (tower.tower_type.cost() as f32 * 0.7) as u32;
        game_world.resources.money += refund;

        let position = nalgebra_glm::vec3(grid_x as f32, 0.5, grid_z as f32);
        spawn_money_popup(game_world, world, position, refund as i32);

        let entities: Vec<_> = game_world.query_entities(GRID_CELL).collect();
        for entity in entities {
            if let Some(cell) = game_world.get_grid_cell_mut(entity)
                && cell.x == grid_x
                && cell.z == grid_z
            {
                cell.occupied = false;
                break;
            }
        }

        game_world
            .resources
            .towers_by_position
            .remove(&(grid_x, grid_z));

        let range_indicators_to_remove: Vec<_> = game_world
            .query_entities(RANGE_INDICATOR)
            .filter_map(|range_entity| {
                game_world
                    .get_range_indicator(range_entity)
                    .filter(|indicator| indicator.tower_entity == tower_entity)
                    .map(|indicator| (range_entity, indicator.line_entities.clone()))
            })
            .collect();

        for (range_entity, line_entities) in range_indicators_to_remove {
            for line in line_entities {
                world
                    .resources
                    .command_queue
                    .push(WorldCommand::DespawnRecursive { entity: line });
            }
            game_world.despawn_entities(&[range_entity]);
        }

        despawn_entity(game_world, world, tower_entity);
    }
}

pub fn tower_targeting_system(game_world: &mut GameWorld, world: &mut World) {
    let delta_time = world.resources.window.timing.delta_time * game_world.resources.game_speed;
    let tower_entities: Vec<_> = game_world.query_entities(TOWER).collect();

    for tower_entity in tower_entities {
        if let Some(mut tower) = game_world.get_tower(tower_entity).copied() {
            let is_sniper = tower.tower_type == TowerType::Sniper;

            if is_sniper {
                let mut target_valid = false;
                if let Some(current_target) = tower.target
                    && game_world.resources.enemies_list.contains(&current_target)
                {
                    target_valid = true;
                }

                if !target_valid {
                    tower.target = None;
                    tower.tracking_time = 0.0;

                    if let Some(line_entity) = tower.target_line.take() {
                        world
                            .resources
                            .command_queue
                            .push(WorldCommand::DespawnRecursive {
                                entity: line_entity,
                            });
                    }
                }
            } else if let Some(target) = tower.target
                && !game_world.resources.enemies_list.contains(&target)
            {
                tower.target = None;
            }

            if tower.target.is_none()
                && let Some(tower_pos) = game_world.get_position(tower_entity).copied()
            {
                let range = tower.tower_type.range();
                let mut closest_enemy: Option<(freecs::Entity, f32)> = None;
                let enemies_list = game_world.resources.enemies_list.clone();

                for &enemy_entity in &enemies_list {
                    if let Some(enemy_pos) = game_world.get_position(enemy_entity).copied() {
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
                    (tower.target, game_world.get_position(tower_entity).copied())
                {
                    for &enemy_entity in &game_world.resources.enemies_list.clone() {
                        if enemy_entity == target {
                            if let Some(enemy_pos) = game_world.get_position(enemy_entity).copied()
                            {
                                if tower.target_line.is_none() {
                                    let line_entity = world.spawn_entities(
                                        LOCAL_TRANSFORM | GLOBAL_TRANSFORM | LINES | VISIBILITY,
                                        1,
                                    )[0];
                                    tower.target_line = Some(line_entity);
                                }

                                if let Some(line_entity) = tower.target_line {
                                    if let Some(lines_comp) = world.get_lines_mut(line_entity) {
                                        lines_comp.lines.clear();
                                        lines_comp.lines.push(Line {
                                            start: tower_pos.0 + nalgebra_glm::vec3(0.0, 0.5, 0.0),
                                            end: enemy_pos.0 + nalgebra_glm::vec3(0.0, 0.3, 0.0),
                                            color: nalgebra_glm::vec4(1.0, 0.0, 0.0, 0.8),
                                        });
                                    }

                                    if let Some(visibility) = world.get_visibility_mut(line_entity)
                                    {
                                        visibility.visible = true;
                                    }
                                }
                            }
                            break;
                        }
                    }
                }
            }

            game_world.set_tower(tower_entity, tower);
        }
    }
}

pub fn tower_shooting_system(game_world: &mut GameWorld, world: &mut World) {
    let delta_time = world.resources.window.timing.delta_time * game_world.resources.game_speed;
    let tower_entities: Vec<_> = game_world.query_entities(TOWER).collect();

    for tower_entity in tower_entities {
        if let Some(mut tower) = game_world.get_tower(tower_entity).copied() {
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

                if can_fire {
                    let enemies_list = game_world.resources.enemies_list.clone();
                    if enemies_list.contains(&target)
                        && let (Some(tower_pos), Some(_enemy_pos)) = (
                            game_world.get_position(tower_entity).copied(),
                            game_world.get_position(target).copied(),
                        )
                    {
                        let engine_entity = spawn_mesh(
                            world,
                            "Sphere",
                            tower_pos.0 + nalgebra_glm::vec3(0.0, 0.5, 0.0),
                            nalgebra_glm::vec3(0.15, 0.15, 0.15),
                        );

                        let material_name = format!("Projectile_{}", engine_entity.id);
                        material_registry_insert(
                            &mut world.resources.material_registry,
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
                        world.set_material_ref(engine_entity, MaterialRef::new(material_name));

                        let projectile_entity =
                            game_world.spawn_entities(ENTITY_HANDLE | POSITION | PROJECTILE, 1)[0];

                        game_world
                            .set_entity_handle(projectile_entity, EntityHandle(engine_entity));
                        game_world.set_position(
                            projectile_entity,
                            Position(tower_pos.0 + nalgebra_glm::vec3(0.0, 0.5, 0.0)),
                        );
                        game_world.set_projectile(
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

                        game_world
                            .resources
                            .projectiles_list
                            .push(projectile_entity);

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
            }

            if needs_fire_animation_update
                && let Some(handle) = game_world.get_entity_handle(tower_entity)
            {
                if let Some(transform) = world.get_local_transform_mut(handle.0) {
                    transform.scale =
                        nalgebra_glm::vec3(0.4 * animation_scale, 0.8, 0.4 * animation_scale);
                    world.set_local_transform_dirty(handle.0, LocalTransformDirty);
                }

                if animation_scale > 1.05 {
                    if let Some(material_ref) = world.get_material_ref(handle.0).cloned()
                        && let Some(material) = registry_entry_by_name_mut(
                            &mut world.resources.material_registry.registry,
                            &material_ref.name,
                        )
                    {
                        let flash_intensity = (animation_scale - 1.0) * 5.0;
                        material.emissive_factor = [
                            tower.tower_type.color().x * 0.5 + flash_intensity,
                            tower.tower_type.color().y * 0.5 + flash_intensity,
                            tower.tower_type.color().z * 0.5 + flash_intensity,
                        ];
                    }
                } else if let Some(material_ref) = world.get_material_ref(handle.0).cloned()
                    && let Some(material) = registry_entry_by_name_mut(
                        &mut world.resources.material_registry.registry,
                        &material_ref.name,
                    )
                {
                    material.emissive_factor = [
                        tower.tower_type.color().x * 0.5,
                        tower.tower_type.color().y * 0.5,
                        tower.tower_type.color().z * 0.5,
                    ];
                }
            }

            game_world.set_tower(tower_entity, tower);
        }
    }
}
