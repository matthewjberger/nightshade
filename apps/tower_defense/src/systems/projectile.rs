use crate::ecs::{Enemy, Position, Projectile, TowerDefenseResources, TowerType};
use crate::systems::{
    create_death_effect, create_explosion_effect, despawn_entity, spawn_money_popup,
};
use nightshade::prelude::*;

pub fn projectile_movement_system(game_world: &mut TowerDefenseResources, world: &mut World) {
    let delta_time = world.res::<Time>().delta_time * game_world.game_speed;
    let mut projectiles_to_remove = Vec::new();

    for projectile_entity in game_world.projectiles_list.clone() {
        if let (Some(projectile), Some(projectile_position)) = (
            world.get::<Projectile>(projectile_entity).copied(),
            world.get::<Position>(projectile_entity).copied(),
        ) {
            if !game_world.enemies_list.contains(&projectile.target) {
                projectiles_to_remove.push(projectile_entity);
                continue;
            }

            if let Some(target_pos) = world.get::<Position>(projectile.target).copied() {
                let target_visual_pos = if let Some(enemy) = world.get::<Enemy>(projectile.target) {
                    target_pos.0 + nalgebra_glm::vec3(0.0, enemy.enemy_type.y_offset(), 0.0)
                } else {
                    target_pos.0
                };

                let new_pos = if projectile.arc_height > 0.0 {
                    let total_distance =
                        (target_visual_pos - projectile.start_position).magnitude();
                    let mut new_flight_progress = projectile.flight_progress
                        + (projectile.speed * delta_time) / total_distance;
                    new_flight_progress = new_flight_progress.min(1.0);

                    let base_pos = projectile.start_position
                        + (target_visual_pos - projectile.start_position) * new_flight_progress;
                    let arc_offset = 4.0
                        * projectile.arc_height
                        * new_flight_progress
                        * (1.0 - new_flight_progress);
                    base_pos + nalgebra_glm::vec3(0.0, arc_offset, 0.0)
                } else {
                    let direction = (target_visual_pos - projectile_position.0).normalize();
                    projectile_position.0 + direction * projectile.speed * delta_time
                };

                let distance = (target_visual_pos - new_pos).magnitude();
                let distance_to_target = distance;
                if distance_to_target < 0.3
                    || (projectile.arc_height > 0.0 && projectile.flight_progress >= 1.0)
                {
                    if matches!(projectile.tower_type, TowerType::Cannon | TowerType::Sniper) {
                        create_explosion_effect(game_world, world, target_visual_pos);
                    }

                    if let Some(mut enemy) = world.get::<Enemy>(projectile.target).copied() {
                        let mut damage_remaining = projectile.damage;

                        if enemy.shield_health > 0.0 {
                            let shield_damage = damage_remaining.min(enemy.shield_health);
                            enemy.shield_health -= shield_damage;
                            damage_remaining -= shield_damage;
                        }

                        if damage_remaining > 0.0 {
                            enemy.health -= damage_remaining;
                        }

                        if matches!(projectile.tower_type, TowerType::Frost) {
                            enemy.slow_duration = 2.0;
                        }

                        if matches!(projectile.tower_type, TowerType::Poison) {
                            enemy.poison_duration = 3.0;
                            enemy.poison_damage = 2.0;
                        }

                        if enemy.health <= 0.0 {
                            game_world.money += enemy.value;
                            spawn_money_popup(world, target_visual_pos, enemy.value as i32);
                            create_death_effect(world, target_visual_pos);

                            if let Some(index) = game_world
                                .enemies_list
                                .iter()
                                .position(|&candidate| candidate == projectile.target)
                            {
                                game_world.enemies_list.remove(index);
                            }
                            despawn_entity(world, projectile.target);
                        } else {
                            world.set(projectile.target, enemy);
                        }
                    }

                    projectiles_to_remove.push(projectile_entity);
                } else {
                    if projectile.arc_height > 0.0 {
                        let total_distance =
                            (target_visual_pos - projectile.start_position).magnitude();
                        let mut updated_projectile = projectile;
                        updated_projectile.flight_progress +=
                            (projectile.speed * delta_time) / total_distance;
                        updated_projectile.flight_progress =
                            updated_projectile.flight_progress.min(1.0);
                        world.set(projectile_entity, updated_projectile);
                    }

                    world.set(projectile_entity, Position(new_pos));

                    if let Some(transform) = world
                        .get_mut::<nightshade::ecs::transform::components::LocalTransform>(
                        projectile_entity,
                    ) {
                        transform.translation = new_pos;
                    }
                }
            }
        }
    }

    for entity in projectiles_to_remove {
        if let Some(index) = game_world
            .projectiles_list
            .iter()
            .position(|&candidate| candidate == entity)
        {
            game_world.projectiles_list.remove(index);
        }
        despawn_entity(world, entity);
    }
}
