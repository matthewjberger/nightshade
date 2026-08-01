use crate::components::{Health, Hit, Label, Life, Respawn, Target, Velocity};
use crate::queries::{
    aim_points_down, direction_to_world, hand_pose, origin_pose, palm_direction, to_world,
};
use crate::resources::Jarvis;
use crate::systems::effects::burst;
use crate::theme::{
    BOLT_COLOR, BOLT_COOLDOWN, BOLT_HIT_RADIUS, BOLT_LIFE, BOLT_SPEED, DRONE_RESPAWN,
    LOCK_INTERVAL, MISSILE_COLOR, MISSILE_HIT_RADIUS, MISSILE_LIFE, MISSILE_SPEED, MISSILE_TURN,
};
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::{EntityCaches, despawn_recursive_now};
use nightshade_graphics::prelude::*;
use nightshade_openxr::contract::{XrFrame, XrHand, XrInput, XrPose, XrRig, XrRigSlot};
use nightshade_platform::prelude::*;
use nightshade_render::commands::material_registry::queue_set_material;
use nightshade_render::data::material::Material;

const BOLT_SCALE: f32 = 0.05;
const MISSILE_SCALE: f32 = 0.07;
const PALM_OFFSET: f32 = 0.09;
const LAUNCH_LIFT: f32 = 0.6;
const FRONT_COSINE: f32 = 0.15;

pub fn fire_weapons(
    rig: Res<XrRigSlot>,
    input: Res<XrInput>,
    xr_frame: Res<XrFrame>,
    time: Res<Time>,
    mut jarvis: ResMut<Jarvis>,
    mut caches: EntityCaches,
    mut commands: Commands,
    storage: &mut nightshade::ecs::Storage,
) {
    let Some(rig) = rig.0 else {
        return;
    };
    let xr_input = *input;
    let frame = xr_frame.clone();
    let delta_time = time.delta_time.min(0.05);
    let origin = origin_pose(storage, &rig);
    let suit_velocity = jarvis.velocity;

    for hand in XrHand::ALL {
        let state = XrRig::hand_state(&xr_input, hand);
        let Some(pose) = hand_pose(storage, &rig, hand) else {
            continue;
        };
        let palm_stage = palm_direction(hand, &pose.orientation);
        let muzzle_world = to_world(&origin, pose.position + palm_stage * PALM_OFFSET);
        let palm_world = direction_to_world(&origin, palm_stage);

        let fire_bolt = {
            let gauntlet = jarvis.gauntlet_mut(hand);
            gauntlet.bolt_cooldown = (gauntlet.bolt_cooldown - delta_time).max(0.0);
            let pressed = state.tracked && state.grip_pressed();
            let fire = pressed && !gauntlet.grip_armed && gauntlet.bolt_cooldown <= 0.0;
            gauntlet.grip_armed = pressed;
            if fire {
                gauntlet.bolt_cooldown = BOLT_COOLDOWN;
                jarvis.shots += 1;
            }
            fire
        };
        if fire_bolt {
            let bolt =
                spawn_projectile(&mut commands, muzzle_world, BOLT_SCALE, BOLT_COLOR, "bolt");
            commands.set(bolt, Velocity(palm_world * BOLT_SPEED + suit_velocity));
            commands.set(bolt, Life(BOLT_LIFE));
        }

        let arming = state.tracked && aim_points_down(&state) && state.trigger_pressed();
        let holds_this_hand = jarvis.targeting.arming_hand == Some(hand);
        if arming {
            lock_next(storage, &mut jarvis, &frame, &origin, hand, delta_time);
        } else if holds_this_hand && !state.trigger_pressed() {
            let locks = {
                let targeting = &mut jarvis.targeting;
                targeting.arming_hand = None;
                targeting.launched = 1.0;
                std::mem::take(&mut targeting.locks)
            };
            for target in locks {
                let alive = crate::queries::drone_alive(storage, target);
                if !alive {
                    continue;
                }
                let launch = (palm_world * 0.4 + Vec3::y() * LAUNCH_LIFT).normalize();
                let missile = spawn_projectile(
                    &mut commands,
                    muzzle_world,
                    MISSILE_SCALE,
                    MISSILE_COLOR,
                    "missile",
                );
                commands.set(missile, Target(target));
                commands.set(
                    missile,
                    Velocity(launch * MISSILE_SPEED * 0.5 + suit_velocity),
                );
                commands.set(missile, Life(MISSILE_LIFE));
                jarvis.shots += 1;
            }
        }
    }

    fly_bolts(storage, &mut jarvis, &mut caches, delta_time);
    fly_missiles(storage, &mut jarvis, &mut caches, delta_time);

    let jarvis = &mut *jarvis;
    jarvis.targeting.launched = (jarvis.targeting.launched - delta_time * 1.5).max(0.0);
    jarvis.kill_flash = (jarvis.kill_flash - delta_time * 1.2).max(0.0);
}

fn lock_next(
    storage: &mut nightshade::ecs::Storage,
    jarvis: &mut Jarvis,
    frame: &XrFrame,
    origin: &XrPose,
    hand: XrHand,
    delta_time: f32,
) {
    let head_forward = nalgebra_glm::quat_rotate_vec3(&frame.head_orientation, &-Vec3::z());
    let head_world = to_world(origin, frame.head_position);
    let forward_world = direction_to_world(origin, head_forward);

    let targeting = &mut jarvis.targeting;
    if targeting.arming_hand != Some(hand) {
        targeting.arming_hand = Some(hand);
        targeting.lock_timer = 0.0;
    }
    targeting.lock_timer -= delta_time;
    if targeting.lock_timer > 0.0 {
        return;
    }
    targeting.lock_timer = LOCK_INTERVAL;

    let mut nearest: Option<(f32, Entity)> = None;
    for entity in storage.query_entities::<(Label, LocalTransform)>() {
        if targeting.locks.contains(&entity) {
            continue;
        }
        let alive = crate::queries::drone_alive(storage, entity);
        if !alive {
            continue;
        }
        let Some(position) = storage
            .get::<LocalTransform>(entity)
            .map(|transform| transform.translation)
        else {
            continue;
        };
        let to_drone = position - head_world;
        let distance = to_drone.norm();
        if distance < 1.0e-3 || to_drone.dot(&forward_world) / distance < FRONT_COSINE {
            continue;
        }
        if nearest.is_none_or(|(best, _)| distance < best) {
            nearest = Some((distance, entity));
        }
    }
    if let Some((_, entity)) = nearest {
        targeting.locks.push(entity);
    }
}

fn spawn_projectile(
    commands: &mut Commands,
    position: Vec3,
    scale: f32,
    color: [f32; 3],
    name: &str,
) -> Entity {
    let entity = spawn_sphere_at(commands, position);
    commands.set(
        entity,
        LocalTransform {
            translation: position,
            rotation: Quat::identity(),
            scale: Vec3::new(scale, scale, scale),
        },
    );
    queue_set_material(
        commands,
        entity,
        format!("{name}_{}", entity.id),
        Material {
            base_color: [color[0], color[1], color[2], 1.0],
            emissive_factor: color,
            roughness: 0.1,
            ..Default::default()
        },
    );
    entity
}

fn fly_bolts(
    storage: &mut nightshade::ecs::Storage,
    jarvis: &mut Jarvis,
    caches: &mut EntityCaches<'_>,
    delta_time: f32,
) {
    let bolts: Vec<Entity> = storage
        .query_entities::<(Velocity, Life, LocalTransform)>()
        .filter(|entity| !storage.entity_has_components::<(Target,)>(*entity))
        .collect();
    for entity in bolts {
        let Some(velocity) = storage.get::<Velocity>(entity).map(|velocity| velocity.0) else {
            continue;
        };
        let Some(position) = storage.get_mut::<LocalTransform>(entity).map(|transform| {
            transform.translation += velocity * delta_time;
            transform.translation
        }) else {
            continue;
        };
        let life = storage
            .get::<Life>(entity)
            .map(|life| life.0)
            .unwrap_or_default()
            - delta_time;
        storage.set(entity, Life(life));
        let struck = nearest_drone(storage, position, BOLT_HIT_RADIUS);
        if let Some(drone) = struck {
            damage_drone(storage, jarvis, drone, 1, position);
        }
        if struck.is_some() || life <= 0.0 || position.y < -1.0 {
            despawn_recursive_now(storage, caches, entity);
        }
    }
}

fn fly_missiles(
    storage: &mut nightshade::ecs::Storage,
    jarvis: &mut Jarvis,
    caches: &mut EntityCaches<'_>,
    delta_time: f32,
) {
    let missiles: Vec<Entity> = storage
        .query_entities::<(Target, Velocity, Life, LocalTransform)>()
        .collect();
    for entity in missiles {
        let Some(target) = storage.get::<Target>(entity).map(|target| target.0) else {
            continue;
        };
        let Some(velocity) = storage.get::<Velocity>(entity).map(|velocity| velocity.0) else {
            continue;
        };
        let Some(position) = storage
            .get::<LocalTransform>(entity)
            .map(|transform| transform.translation)
        else {
            continue;
        };
        let target_position = crate::queries::drone_alive(storage, target)
            .then(|| storage.get::<LocalTransform>(target))
            .flatten()
            .map(|transform| transform.translation);

        let mut direction = velocity.normalize();
        if let Some(target_position) = target_position {
            let wanted = (target_position - position).normalize();
            direction = direction.lerp(&wanted, (MISSILE_TURN * delta_time).min(1.0));
            direction = direction.normalize();
        }
        let velocity = direction * MISSILE_SPEED;
        let life = storage
            .get::<Life>(entity)
            .map(|life| life.0)
            .unwrap_or_default()
            - delta_time;
        let next_position = position + velocity * delta_time;
        if let Some(transform) = storage.get_mut::<LocalTransform>(entity) {
            transform.translation = next_position;
            transform.rotation = nalgebra_glm::quat_look_at(&direction, &Vec3::y());
        }
        storage.set(entity, Velocity(velocity));
        storage.set(entity, Life(life));

        let struck = target_position
            .filter(|reached| (reached - next_position).norm() < MISSILE_HIT_RADIUS)
            .map(|_| target)
            .or_else(|| nearest_drone(storage, next_position, MISSILE_HIT_RADIUS));
        if let Some(drone) = struck {
            damage_drone(storage, jarvis, drone, i32::MAX, next_position);
        }
        if struck.is_some() || life <= 0.0 || next_position.y < -1.0 {
            despawn_recursive_now(storage, caches, entity);
        }
    }
}

fn nearest_drone(
    storage: &nightshade::ecs::Storage,
    position: Vec3,
    radius: f32,
) -> Option<Entity> {
    let mut nearest: Option<(f32, Entity)> = None;
    for entity in storage.query_entities::<(Label, LocalTransform)>() {
        let alive = crate::queries::drone_alive(storage, entity);
        if !alive {
            continue;
        }
        let Some(drone_position) = storage
            .get::<LocalTransform>(entity)
            .map(|transform| transform.translation)
        else {
            continue;
        };
        let distance = (drone_position - position).norm();
        if distance < radius && nearest.is_none_or(|(best, _)| distance < best) {
            nearest = Some((distance, entity));
        }
    }
    nearest.map(|(_, entity)| entity)
}

fn damage_drone(
    storage: &mut nightshade::ecs::Storage,
    jarvis: &mut Jarvis,
    entity: Entity,
    damage: i32,
    at: Vec3,
) {
    let Some(health) = storage.get::<Health>(entity).map(|health| health.0) else {
        return;
    };
    let health = health.saturating_sub(damage);
    storage.set(entity, Health(health));
    storage.set(entity, Hit(1.0));
    let killed = health <= 0;
    if killed {
        storage.set(entity, Respawn(DRONE_RESPAWN));
    }
    let position = storage
        .get::<LocalTransform>(entity)
        .map(|transform| transform.translation)
        .unwrap_or(at);
    if killed {
        storage.set(entity, Visibility { visible: false });
    }
    if killed {
        jarvis.kills += 1;
        jarvis.kill_flash = 1.0;
        jarvis.targeting.locks.retain(|lock| *lock != entity);
    }
    burst(storage, position, killed);
}
