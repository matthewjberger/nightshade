use crate::commands::Scene;
use crate::components::{
    ArcHeight, Cooldown, Damage, FireAnimation, FlightProgress, GridCoord, IndicatorLines,
    IndicatorVisible, Occupied, Position, Speed, StartPosition, Target, TargetLine, TrackingTime,
};
use crate::data::TowerType;
use crate::queries::read_or_default;
use crate::resources::TowerDefenseResources;
use crate::systems::{create_muzzle_flash, spawn_money_popup};
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_render::commands::material_registry::material_registry_mutate_id;

pub fn spawn_tower(
    game_world: &mut TowerDefenseResources,
    scene: &mut Scene<'_, '_>,
    grid_x: i32,
    grid_z: i32,
    tower_type: TowerType,
) {
    let position = nalgebra_glm::vec3(grid_x as f32, 0.0, grid_z as f32);
    let color = crate::data::tower_color(tower_type);

    let tower_entity = spawn_mesh(
        scene.commands,
        "Cylinder",
        position,
        nalgebra_glm::vec3(0.4, 0.8, 0.4),
    );

    set_material(
        scene.registry,
        scene.commands,
        tower_entity,
        format!("Tower_{}", tower_entity.id),
        Material {
            base_color: color.into(),
            ..Default::default()
        },
    );

    scene.commands.set(tower_entity, Position(position));
    scene.commands.set(tower_entity, tower_type);
    scene.commands.set(tower_entity, Cooldown(0.0));
    scene.commands.set(tower_entity, Target(None));
    scene.commands.set(tower_entity, FireAnimation(0.0));
    scene.commands.set(tower_entity, TrackingTime(0.0));
    scene.commands.set(tower_entity, TargetLine(None));

    game_world.money -= crate::data::tower(tower_type).cost;
    game_world
        .towers_by_position
        .insert((grid_x, grid_z), tower_entity);

    spawn_range_indicator(
        scene,
        position,
        crate::data::tower(tower_type).range,
        tower_entity,
    );
}

fn spawn_range_indicator(
    scene: &mut Scene<'_, '_>,
    center: Vec3,
    range: f32,
    tower_entity: Entity,
) {
    let segments = 32;
    let mut line_entities = Vec::new();

    let tower_color = match scene.commands.staged::<TowerType>(tower_entity) {
        Some(tower_type) => crate::data::tower_color(*tower_type),
        None => nalgebra_glm::vec4(0.0, 1.0, 0.0, 1.0),
    };

    for segment_index in 0..segments {
        let angle1 = (segment_index as f32 / segments as f32) * std::f32::consts::TAU;
        let angle2 = ((segment_index + 1) as f32 / segments as f32) * std::f32::consts::TAU;

        let start = center + nalgebra_glm::vec3(angle1.cos() * range, 0.1, angle1.sin() * range);
        let end = center + nalgebra_glm::vec3(angle2.cos() * range, 0.1, angle2.sin() * range);

        let line_entity = scene.commands.spawn(());

        scene.commands.set(
            line_entity,
            Name(format!("Range Indicator Line {}", segment_index)),
        );
        scene.commands.set(
            line_entity,
            LocalTransform {
                translation: nalgebra_glm::vec3(0.0, 0.0, 0.0),
                ..Default::default()
            },
        );
        scene.commands.set(line_entity, GlobalTransform::default());

        scene.commands.set(
            line_entity,
            Lines::new(vec![Line {
                start,
                end,
                color: nalgebra_glm::vec4(tower_color.x, tower_color.y, tower_color.z, 0.5),
            }]),
        );

        scene
            .commands
            .set(line_entity, Visibility { visible: false });

        line_entities.push(line_entity);
    }

    scene
        .commands
        .set(tower_entity, IndicatorLines(line_entities));
    scene.commands.set(tower_entity, IndicatorVisible(false));
}

pub fn despawn_entity(scene: &mut Scene<'_, '_>, entity: Entity) {
    let target_line = scene
        .storage
        .get::<TargetLine>(entity)
        .and_then(|line| line.0);
    if let Some(line_entity) = target_line {
        scene.commands.despawn_recursive(line_entity);
    }

    let line_entities = scene
        .storage
        .get::<IndicatorLines>(entity)
        .map(|indicator| indicator.0.clone());
    if let Some(line_entities) = line_entities {
        for line in line_entities {
            scene.commands.despawn_recursive(line);
        }
    }

    scene.commands.despawn_recursive(entity);
}

pub fn sell_tower(
    game_world: &mut TowerDefenseResources,
    scene: &mut Scene<'_, '_>,
    tower_entity: Entity,
    grid_x: i32,
    grid_z: i32,
) {
    if let Some(tower_type) = scene.storage.get::<TowerType>(tower_entity).copied() {
        let refund = (crate::data::tower(tower_type).cost as f32 * 0.7) as u32;
        game_world.money += refund;

        let position = nalgebra_glm::vec3(grid_x as f32, 0.5, grid_z as f32);
        spawn_money_popup(scene, position, refund as i32);

        let cell_entities: Vec<Entity> = scene.storage.query_entities::<(GridCoord,)>().collect();
        for cell_entity in cell_entities {
            if scene.storage.get::<GridCoord>(cell_entity).copied()
                == Some(GridCoord(grid_x, grid_z))
            {
                scene.storage.set(cell_entity, Occupied(false));
                break;
            }
        }

        game_world.towers_by_position.remove(&(grid_x, grid_z));

        despawn_entity(scene, tower_entity);
    }
}

pub fn tower_targeting_system(
    game_world: &mut TowerDefenseResources,
    scene: &mut Scene<'_, '_>,
    delta_time: f32,
) {
    let tower_entities: Vec<Entity> = scene
        .storage
        .query_entities::<(TowerType, Cooldown)>()
        .collect();

    for tower_entity in tower_entities {
        let Some(tower_type) = scene.storage.get::<TowerType>(tower_entity).copied() else {
            continue;
        };
        let mut target = read_or_default::<Target>(scene, tower_entity).0;
        let mut tracking_time = read_or_default::<TrackingTime>(scene, tower_entity).0;
        let mut target_line = read_or_default::<TargetLine>(scene, tower_entity).0;

        let is_sniper = tower_type == TowerType::Sniper;

        if is_sniper {
            let target_valid = target
                .map(|current| game_world.enemies_list.contains(&current))
                .unwrap_or(false);

            if !target_valid {
                target = None;
                tracking_time = 0.0;

                if let Some(line_entity) = target_line.take() {
                    scene.commands.despawn_recursive(line_entity);
                }
            }
        } else if let Some(current) = target
            && !game_world.enemies_list.contains(&current)
        {
            target = None;
        }

        if target.is_none()
            && let Some(tower_pos) = scene.storage.get::<Position>(tower_entity).copied()
        {
            let range = crate::data::tower(tower_type).range;
            let mut closest_enemy: Option<(Entity, f32)> = None;

            for &enemy_entity in &game_world.enemies_list {
                if let Some(enemy_pos) = scene.storage.get::<Position>(enemy_entity).copied() {
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
                target = Some(enemy);
            }
        } else if is_sniper && target.is_some() {
            tracking_time += delta_time;

            if let (Some(current), Some(tower_pos)) =
                (target, scene.storage.get::<Position>(tower_entity).copied())
                && game_world.enemies_list.contains(&current)
                && let Some(enemy_pos) = scene.storage.get::<Position>(current).copied()
            {
                if target_line.is_none() {
                    let line_entity = scene.commands.spawn(());
                    target_line = Some(line_entity);
                }

                if let Some(line_entity) = target_line {
                    if let Some(lines_comp) =
                        scene
                            .storage
                            .get_mut::<nightshade_render::components::lines::Lines>(line_entity)
                    {
                        lines_comp.lines.clear();
                        lines_comp.lines.push(Line {
                            start: tower_pos.0 + nalgebra_glm::vec3(0.0, 0.5, 0.0),
                            end: enemy_pos.0 + nalgebra_glm::vec3(0.0, 0.3, 0.0),
                            color: nalgebra_glm::vec4(1.0, 0.0, 0.0, 0.8),
                        });
                    }

                    if let Some(visibility) = scene
                        .storage
                        .get_mut::<nightshade_scene::contract::primitives::Visibility>(
                        line_entity,
                    ) {
                        visibility.visible = true;
                    }
                }
            }
        }

        scene.commands.set(tower_entity, Target(target));
        scene
            .commands
            .set(tower_entity, TrackingTime(tracking_time));
        scene.commands.set(tower_entity, TargetLine(target_line));
    }
}

pub fn tower_shooting_system(
    game_world: &mut TowerDefenseResources,
    scene: &mut Scene<'_, '_>,
    delta_time: f32,
) {
    let tower_entities: Vec<Entity> = scene
        .storage
        .query_entities::<(TowerType, Cooldown)>()
        .collect();

    for tower_entity in tower_entities {
        let Some(tower_type) = scene.storage.get::<TowerType>(tower_entity).copied() else {
            continue;
        };
        let target = read_or_default::<Target>(scene, tower_entity).0;
        let tracking_time = read_or_default::<TrackingTime>(scene, tower_entity).0;
        let mut cooldown = read_or_default::<Cooldown>(scene, tower_entity).0;
        let mut fire_animation = read_or_default::<FireAnimation>(scene, tower_entity).0;

        cooldown = (cooldown - delta_time).max(0.0);

        let mut needs_fire_animation_update = false;
        let mut animation_scale = 1.0;

        if fire_animation > 0.0 {
            fire_animation -= delta_time * 3.0;
            needs_fire_animation_update = true;
            animation_scale = 1.0 + fire_animation * 0.2;
        }

        if cooldown <= 0.0
            && let Some(target) = target
        {
            let can_fire = if tower_type == TowerType::Sniper {
                tracking_time >= 2.0
            } else {
                true
            };

            if can_fire
                && game_world.enemies_list.contains(&target)
                && scene.storage.has::<Position>(target)
                && let Some(tower_pos) = scene.storage.get::<Position>(tower_entity).copied()
            {
                let muzzle = tower_pos.0 + nalgebra_glm::vec3(0.0, 0.5, 0.0);
                let projectile_entity = spawn_mesh(
                    scene.commands,
                    "Sphere",
                    muzzle,
                    nalgebra_glm::vec3(0.15, 0.15, 0.15),
                );

                set_material(
                    scene.registry,
                    scene.commands,
                    projectile_entity,
                    format!("Projectile_{}", projectile_entity.id),
                    Material {
                        base_color: crate::data::tower_color(tower_type).into(),
                        emissive_factor: [
                            crate::data::tower_color(tower_type).x * 0.5,
                            crate::data::tower_color(tower_type).y * 0.5,
                            crate::data::tower_color(tower_type).z * 0.5,
                        ],
                        ..Default::default()
                    },
                );

                scene.commands.set(projectile_entity, Position(muzzle));
                scene.commands.set(
                    projectile_entity,
                    Damage(crate::data::tower(tower_type).damage),
                );
                scene.commands.set(projectile_entity, Target(Some(target)));
                scene.commands.set(
                    projectile_entity,
                    Speed(crate::data::tower(tower_type).projectile_speed),
                );
                scene.commands.set(projectile_entity, tower_type);
                scene.commands.set(projectile_entity, StartPosition(muzzle));
                scene.commands.set(
                    projectile_entity,
                    ArcHeight(if tower_type == TowerType::Cannon {
                        2.0
                    } else {
                        0.0
                    }),
                );
                scene.commands.set(projectile_entity, FlightProgress(0.0));

                game_world.projectiles_list.push(projectile_entity);

                cooldown = crate::data::tower(tower_type).fire_rate;
                fire_animation = 1.0;
                needs_fire_animation_update = true;
                animation_scale = 1.0 + fire_animation * 0.2;

                if tower_type == TowerType::Cannon {
                    create_muzzle_flash(
                        game_world,
                        scene,
                        tower_pos.0 + nalgebra_glm::vec3(0.0, 0.8, 0.0),
                    );
                }
            }
        }

        if needs_fire_animation_update {
            if let Some(transform) = scene
                .storage
                .get_mut::<nightshade_scene::contract::LocalTransform>(tower_entity)
            {
                transform.scale =
                    nalgebra_glm::vec3(0.4 * animation_scale, 0.8, 0.4 * animation_scale);
            }

            if let Some(material_ref) = scene
                .storage
                .get::<nightshade_render::components::material::MaterialRef>(tower_entity)
                .cloned()
            {
                let flash_intensity = if animation_scale > 1.05 {
                    (animation_scale - 1.0) * 5.0
                } else {
                    0.0
                };
                material_registry_mutate_id(scene.registry, material_ref.id, |material| {
                    material.emissive_factor = [
                        crate::data::tower_color(tower_type).x * 0.5 + flash_intensity,
                        crate::data::tower_color(tower_type).y * 0.5 + flash_intensity,
                        crate::data::tower_color(tower_type).z * 0.5 + flash_intensity,
                    ];
                });
            }
        }

        scene.commands.set(tower_entity, Cooldown(cooldown));
        scene
            .commands
            .set(tower_entity, FireAnimation(fire_animation));
    }
}
