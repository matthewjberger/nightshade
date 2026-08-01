use crate::commands::Scene;
use crate::components::{
    ArcHeight, Bounty, Damage, FlightProgress, Health, PoisonDamage, PoisonDuration, Position,
    ShieldHealth, SlowDuration, Speed, StartPosition, Target,
};
use crate::data::{EnemyType, TowerType};
use crate::queries::read_or_default;
use crate::resources::TowerDefenseResources;
use crate::systems::{
    create_death_effect, create_explosion_effect, despawn_entity, spawn_money_popup,
};
use nightshade::prelude::*;

pub fn projectile_movement_system(
    game_world: &mut TowerDefenseResources,
    scene: &mut Scene<'_, '_>,
    delta_time: f32,
) {
    let mut projectiles_to_remove = Vec::new();

    for projectile_entity in game_world.projectiles_list.clone() {
        let Some(projectile_position) = scene
            .storage
            .get::<Position>(projectile_entity)
            .copied()
            .map(|position| position.0)
        else {
            continue;
        };
        let Some(target) = read_or_default::<Target>(scene, projectile_entity).0 else {
            projectiles_to_remove.push(projectile_entity);
            continue;
        };
        let Some(tower_type) = scene.storage.get::<TowerType>(projectile_entity).copied() else {
            continue;
        };
        let damage = read_or_default::<Damage>(scene, projectile_entity).0;
        let speed = read_or_default::<Speed>(scene, projectile_entity).0;
        let arc_height = read_or_default::<ArcHeight>(scene, projectile_entity).0;
        let start_position = read_or_default::<StartPosition>(scene, projectile_entity).0;
        let flight_progress = read_or_default::<FlightProgress>(scene, projectile_entity).0;

        if !game_world.enemies_list.contains(&target) {
            projectiles_to_remove.push(projectile_entity);
            continue;
        }

        let Some(target_pos) = scene.storage.get::<Position>(target).copied() else {
            continue;
        };

        let target_visual_pos = match scene.storage.get::<EnemyType>(target).copied() {
            Some(enemy_type) => {
                target_pos.0 + nalgebra_glm::vec3(0.0, crate::data::enemy_y_offset(enemy_type), 0.0)
            }
            None => target_pos.0,
        };

        let new_pos = if arc_height > 0.0 {
            let total_distance = (target_visual_pos - start_position).magnitude();
            let mut new_flight_progress = flight_progress + (speed * delta_time) / total_distance;
            new_flight_progress = new_flight_progress.min(1.0);

            let base_pos =
                start_position + (target_visual_pos - start_position) * new_flight_progress;
            let arc_offset = 4.0 * arc_height * new_flight_progress * (1.0 - new_flight_progress);
            base_pos + nalgebra_glm::vec3(0.0, arc_offset, 0.0)
        } else {
            let direction = (target_visual_pos - projectile_position).normalize();
            projectile_position + direction * speed * delta_time
        };

        let distance_to_target = (target_visual_pos - new_pos).magnitude();
        if distance_to_target < 0.3 || (arc_height > 0.0 && flight_progress >= 1.0) {
            if matches!(tower_type, TowerType::Cannon | TowerType::Sniper) {
                create_explosion_effect(game_world, scene, target_visual_pos);
            }

            apply_hit(
                game_world,
                scene,
                target,
                tower_type,
                damage,
                target_visual_pos,
            );

            projectiles_to_remove.push(projectile_entity);
        } else {
            if arc_height > 0.0 {
                let total_distance = (target_visual_pos - start_position).magnitude();
                let advanced = (flight_progress + (speed * delta_time) / total_distance).min(1.0);
                scene
                    .storage
                    .set(projectile_entity, FlightProgress(advanced));
            }

            scene.storage.set(projectile_entity, Position(new_pos));

            if let Some(transform) = scene
                .storage
                .get_mut::<nightshade_scene::contract::LocalTransform>(projectile_entity)
            {
                transform.translation = new_pos;
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
        despawn_entity(scene, entity);
    }
}

fn apply_hit(
    game_world: &mut TowerDefenseResources,
    scene: &mut Scene<'_, '_>,
    target: Entity,
    tower_type: TowerType,
    damage: f32,
    impact_position: Vec3,
) {
    if !scene.storage.has::<Health>(target) {
        return;
    }

    let mut shield_health = read_or_default::<ShieldHealth>(scene, target).0;
    let mut health = read_or_default::<Health>(scene, target).0;
    let bounty = read_or_default::<Bounty>(scene, target).0;
    let mut damage_remaining = damage;

    if shield_health > 0.0 {
        let shield_damage = damage_remaining.min(shield_health);
        shield_health -= shield_damage;
        damage_remaining -= shield_damage;
    }

    if damage_remaining > 0.0 {
        health -= damage_remaining;
    }

    if health <= 0.0 {
        game_world.money += bounty;
        spawn_money_popup(scene, impact_position, bounty as i32);
        create_death_effect(scene, impact_position);

        if let Some(index) = game_world
            .enemies_list
            .iter()
            .position(|&candidate| candidate == target)
        {
            game_world.enemies_list.remove(index);
        }
        despawn_entity(scene, target);
        return;
    }

    scene.storage.set(target, ShieldHealth(shield_health));
    scene.storage.set(target, Health(health));

    if matches!(tower_type, TowerType::Frost) {
        scene.storage.set(target, SlowDuration(2.0));
    }

    if matches!(tower_type, TowerType::Poison) {
        scene.storage.set(target, PoisonDuration(3.0));
        scene.storage.set(target, PoisonDamage(2.0));
    }
}
