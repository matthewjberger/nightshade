use crate::commands::{hurt, send_bolt};
use crate::components::{Cooldown, Drift, EnemyKind, EnemyMood, Group, Timer};
use crate::contract::{BoomstickEvent, FireQueue};
use crate::data;
use crate::data::bestiary::beast;
use crate::resources::{BoomstickResources, CLIP_ATTACK, CLIP_DIE, CLIP_IDLE, CLIP_WALK};
use crate::systems::effects;
use nightshade::prelude::*;
use nightshade_animation::contract::AnimationPlayer;
use nightshade_physics::prelude::{PhysicsWorld, physics_world_cast_ray};
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub type Horde<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item EnemyKind,
        &'item mut EnemyMood,
        &'item mut Timer,
        &'item mut Cooldown,
        &'item Group,
        &'item Drift,
        &'item mut LocalTransform,
        &'item mut AnimationPlayer,
    ),
>;

struct Volley {
    kind: EnemyKind,
    origin: Vec3,
    facing: Vec3,
}

pub fn update(
    time: Res<Time>,
    physics: Res<PhysicsWorld>,
    mut resources: ResMut<BoomstickResources>,
    mut queue: ResMut<FireQueue>,
    mut events: EventWriter<BoomstickEvent>,
    mut commands: Commands,
    horde: Horde<'_, '_>,
) {
    let delta_time = time.delta_time;
    let Some(player_at) = resources.player.at else {
        return;
    };
    let eye = player_at + Vec3::new(0.0, data::EYE_HEIGHT * 0.4, 0.0);
    let dead = resources.player.dead;

    let mut spent: Vec<(Entity, Vec3)> = Vec::new();
    let mut volleys: Vec<Volley> = Vec::new();
    let mut struck = 0.0;

    horde.for_each(
        |entity, (kind, mood, timer, cooldown, group, drift, transform, animation)| {
            let profile = beast(*kind);

            if *mood == EnemyMood::Dying {
                if animation.current_clip != Some(CLIP_DIE) {
                    play(animation, CLIP_DIE, 1.0, false);
                }
                timer.0 -= delta_time;
                if timer.0 <= 0.0 {
                    spent.push((entity, transform.translation));
                }
                return;
            }

            cooldown.0 = (cooldown.0 - delta_time).max(0.0);
            let at = transform.translation;
            let flat = Vec3::new(player_at.x - at.x, 0.0, player_at.z - at.z);
            let distance = flat.magnitude();
            if distance < 0.001 {
                return;
            }
            let facing = flat / distance;
            if *mood != EnemyMood::Asleep {
                let wanted = nalgebra_glm::quat_angle_axis(facing.x.atan2(facing.z), &Vec3::y());
                let share = (data::TURN_RATE * delta_time).min(1.0);
                transform.rotation = nalgebra_glm::quat_normalize(
                    &(transform.rotation + (wanted - transform.rotation) * share),
                );
            }

            match *mood {
                EnemyMood::Asleep => {
                    let chest = at + Vec3::new(0.0, profile.height * 0.6, 0.0);
                    if group.0 == 0
                        && !dead
                        && distance < profile.sight
                        && sees(&physics, chest, eye)
                    {
                        *mood = EnemyMood::Waking;
                        timer.0 = profile.wind * 0.7;
                        play(animation, CLIP_IDLE, 2.0, true);
                    }
                }
                EnemyMood::Waking => {
                    timer.0 -= delta_time;
                    if timer.0 <= 0.0 {
                        *mood = EnemyMood::Hunting;
                        play(animation, CLIP_WALK, 1.4, true);
                    }
                }
                EnemyMood::Reeling => {
                    timer.0 -= delta_time;
                    if timer.0 <= 0.0 {
                        *mood = EnemyMood::Hunting;
                        play(animation, CLIP_WALK, 1.4, true);
                    }
                }
                EnemyMood::Hunting => {
                    let reach = profile.reach + profile.height * 0.25;
                    let level = (player_at.y - at.y).abs() < 2.4;
                    let chest = at + Vec3::new(0.0, profile.height * 0.6, 0.0);
                    let open = distance < profile.sight && sees(&physics, chest, eye);

                    if !dead && cooldown.0 <= 0.0 {
                        let melee_ready = profile.melee > 0.0 && distance <= reach && level;
                        let shot_ready = profile.volley > 0
                            && open
                            && distance > profile.keep * 0.5
                            && distance > reach;
                        if melee_ready || shot_ready {
                            *mood = EnemyMood::Winding;
                            timer.0 = profile.wind;
                            play(animation, CLIP_ATTACK, 1.0, false);
                            return;
                        }
                    }

                    let closing = if profile.keep > 0.0 {
                        if distance > profile.keep {
                            1.0
                        } else if distance < profile.keep * 0.7 {
                            -1.0
                        } else {
                            0.0
                        }
                    } else if distance > reach * 0.8 {
                        1.0
                    } else {
                        0.0
                    };
                    let side = Vec3::new(-facing.z, 0.0, facing.x) * drift.0;
                    let wish = (facing * closing + side * data::DRIFT_RATE * open_share(open))
                        .try_normalize(0.0001)
                        .unwrap_or_else(Vec3::zeros);
                    let step = profile.speed * delta_time;
                    if step > 0.0
                        && wish.magnitude() > 0.0
                        && let Some(way) = free_step(&physics, at, wish, step)
                    {
                        transform.translation.x += way.x * step;
                        transform.translation.z += way.z * step;
                    }
                }
                EnemyMood::Winding => {
                    timer.0 -= delta_time;
                    if timer.0 > 0.0 {
                        return;
                    }
                    *mood = EnemyMood::Hunting;
                    cooldown.0 = profile.cooldown;
                    play(animation, CLIP_WALK, 1.4, true);
                    let reach = profile.reach + profile.height * 0.25;
                    let level = (player_at.y - at.y).abs() < 2.4;
                    if profile.melee > 0.0 && distance <= reach && level {
                        struck += profile.melee;
                        return;
                    }
                    if profile.volley > 0 {
                        volleys.push(Volley {
                            kind: *kind,
                            origin: at + Vec3::new(0.0, profile.height * 0.6, 0.0) + facing * 1.8,
                            facing: (eye - (at + Vec3::new(0.0, profile.height * 0.6, 0.0)))
                                .try_normalize(0.0001)
                                .unwrap_or(facing),
                        });
                    }
                }
                EnemyMood::Dying => {}
            }

            if let Some(ground) = ground_below(&physics, transform.translation) {
                transform.translation.y = ground;
            }
        },
    );

    for volley in volleys {
        let profile = beast(volley.kind);
        let spread = 0.06;
        for index in 0..profile.volley {
            let step = index as f32 - (profile.volley as f32 - 1.0) * 0.5;
            let side = Vec3::new(-volley.facing.z, 0.0, volley.facing.x);
            let aim = (volley.facing + side * step * spread).normalize();
            let lift = if profile.bolt_fall > 0.0 {
                Vec3::new(0.0, 0.34, 0.0)
            } else {
                Vec3::zeros()
            };
            send_bolt(&mut queue, profile, volley.origin, (aim + lift).normalize());
        }
    }

    if struck > 0.0 && !resources.player.dead {
        hurt(&mut resources.player, struck);
        resources.player.flash = 1.0;
        resources.player.kick += data::HURT_KICK;
        events.send(BoomstickEvent::PlayerHurt { damage: struck });
    }

    for (entity, at) in spent {
        effects::blood(&mut commands, at + Vec3::new(0.0, 0.6, 0.0));
        commands.despawn_recursive(entity);
    }
}

pub fn settle(physics: Res<PhysicsWorld>, resting: Query<(&EnemyMood, &mut LocalTransform)>) {
    resting.for_each(|_entity, (mood, transform)| {
        if *mood != EnemyMood::Asleep {
            return;
        }
        if let Some(ground) = ground_below(&physics, transform.translation) {
            transform.translation.y = ground;
        }
    });
}

fn open_share(open: bool) -> f32 {
    if open { 1.0 } else { 0.0 }
}

fn play(animation: &mut AnimationPlayer, clip: usize, speed: f32, looping: bool) {
    if animation.current_clip == Some(clip) && looping {
        return;
    }
    animation.current_clip = Some(clip);
    animation.time = 0.0;
    animation.speed = speed;
    animation.looping = looping;
    animation.playing = true;
}

fn sees(physics: &PhysicsWorld, from: Vec3, to: Vec3) -> bool {
    let gap = to - from;
    let range = gap.magnitude();
    if range < 0.001 {
        return true;
    }
    match physics_world_cast_ray(physics, from, gap / range, range, None) {
        Some(hit) => hit.distance > range - 0.6,
        None => true,
    }
}

fn free_step(physics: &PhysicsWorld, at: Vec3, wish: Vec3, step: f32) -> Option<Vec3> {
    let side = Vec3::new(-wish.z, 0.0, wish.x);
    let slide = data::SLIDE_SHARE;
    [
        wish,
        (wish + side * slide).normalize(),
        (wish - side * slide).normalize(),
        side,
        -side,
    ]
    .into_iter()
    .find(|way| walkable(physics, at, *way, step))
}

fn walkable(physics: &PhysicsWorld, at: Vec3, way: Vec3, step: f32) -> bool {
    let chest = at + Vec3::new(0.0, data::CHEST_HEIGHT, 0.0);
    let reach = step + data::STEP_AHEAD;
    if physics_world_cast_ray(physics, chest, way, reach, None).is_some() {
        return false;
    }
    let ahead = at + way * reach;
    match ground_below(physics, ahead) {
        Some(height) => at.y - height < data::DROP_LIMIT,
        None => false,
    }
}

fn ground_below(physics: &PhysicsWorld, at: Vec3) -> Option<f32> {
    let from = at + Vec3::new(0.0, data::GROUND_PROBE_ABOVE, 0.0);
    physics_world_cast_ray(
        physics,
        from,
        Vec3::new(0.0, -1.0, 0.0),
        data::GROUND_PROBE_DEPTH,
        None,
    )
    .map(|hit| hit.point.y)
}
