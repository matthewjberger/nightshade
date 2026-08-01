use crate::commands::interface::say;
use crate::commands::pickups::{burst, drop_from};
use crate::components::{
    Breaks, Combo, Facing, Flash, Fling, Health, Holding, Kind, Lift, Pose, Rise, Step, Strike,
    Stun,
};
use crate::contract::CrosstownEvent;
use crate::data;
use crate::queries::in_reach;
use crate::resources::{Blow, Ring, Sheets, Talk};
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

fn blow_shape(step: u32, armed: Option<usize>) -> (f32, f32, f32, bool) {
    if let Some(weapon) = armed {
        let plan = &data::WEAPON_KINDS[weapon];
        return (plan.reach, plan.damage, data::SWING_PUSH, true);
    }
    match step {
        3 => (data::REACH, data::AIR_DAMAGE, data::AIR_PUSH, true),
        4 => (data::DASH_REACH, data::DASH_DAMAGE, data::DASH_PUSH, true),
        _ => (
            data::REACH,
            data::HERO_DAMAGE[(step as usize).min(2)],
            data::HERO_PUSH[(step as usize).min(2)],
            step == 2,
        ),
    }
}

pub fn wind(time: Res<Time>, mut ring: ResMut<Ring>, heroes: Winders) {
    ring.hit_stop = (ring.hit_stop - time.delta_time).max(0.0);
    ring.shake = (ring.shake - time.delta_time * data::CAMERA_SHAKE_FALL).max(0.0);
    ring.hero_blow = None;
    let delta_time = if ring.hit_stop > 0.0 {
        0.0
    } else {
        time.delta_time
    };
    let mut blow = None;
    heroes.for_each(
        |_entity, (strike, stun, pose, transform, facing, step, lift, holding)| {
            stun.0 = (stun.0 - delta_time).max(0.0);
            if strike.0 <= 0.0 {
                if stun.0 <= 0.0 && pose.0 == data::POSE_HURT {
                    pose.0 = if holding.0 == 0 {
                        data::POSE_IDLE
                    } else {
                        data::POSE_HOLD
                    };
                }
                return;
            }
            let armed = holding.0.checked_sub(1).map(|weapon| weapon as usize);
            let moment = if armed.is_some() {
                data::SWING_MOMENT
            } else {
                data::BLOW_MOMENT
            };
            let before = strike.0;
            strike.0 = (strike.0 - delta_time).max(0.0);
            if before > moment && strike.0 <= moment {
                let (reach, damage, push, down) = blow_shape(step.0, armed);
                blow = Some(Blow {
                    at: transform.translation,
                    facing: facing.0,
                    step: step.0,
                    reach,
                    damage,
                    push,
                    down,
                });
            }
            if strike.0 <= 0.0 && lift.0 <= 0.0 {
                pose.0 = if holding.0 == 0 {
                    data::POSE_IDLE
                } else {
                    data::POSE_HOLD
                };
            }
        },
    );
    ring.hero_blow = blow;
}

pub fn land(
    time: Res<Time>,
    mut ring: ResMut<Ring>,
    mut talk: ResMut<Talk>,
    mut events: EventWriter<CrosstownEvent>,
    mut commands: Commands,
    thugs: Foes,
) {
    let delta_time = if ring.hit_stop > 0.0 {
        0.0
    } else {
        time.delta_time
    };
    let blow = ring.hero_blow;
    let mut alive = 0;
    let mut stop = 0.0f32;
    let mut shake = 0.0f32;
    let mut mark = None;
    let mut floored = 0;
    let mut score = 0;
    let mut gone = Vec::new();
    let mut flung = Vec::new();

    thugs.for_each(
        |entity, (stun, health, pose, fling, rise, flash, kind, transform)| {
            stun.0 = (stun.0 - delta_time).max(0.0);
            flash.0 = (flash.0 - delta_time).max(0.0);
            if health.0 <= 0.0 {
                if stun.0 <= 0.0 {
                    gone.push(entity);
                }
                return;
            }
            alive += 1;
            let plan = &data::THUG_KINDS[kind.0 as usize];
            if pose.0 == data::POSE_DOWN && stun.0 > 0.0 && stun.0 < data::RISE_TIME {
                pose.0 = data::POSE_RISE;
            }

            let Some(blow) = blow else {
                return;
            };
            if !in_reach(&blow, transform.translation, plan.radius) {
                return;
            }
            let thrown = plan.flinches && stun.0 > 0.0 && pose.0 == data::POSE_HURT;
            let hurt = if thrown {
                data::FLING_DAMAGE
            } else {
                blow.damage
            };
            health.0 = (health.0 - hurt).max(0.0);
            flash.0 = data::FLASH_TIME;
            if thrown {
                fling.0 = blow.facing * data::FLING_SPEED;
                rise.0 = data::FLING_RISE;
                flung.push(transform.translation);
            } else {
                fling.0 += blow.facing * blow.push * data::KNOCK_RATE;
            }
            let down = thrown || blow.down || health.0 <= 0.0;
            if plan.flinches || down {
                stun.0 = if down {
                    data::DOWN_TIME
                } else {
                    data::HURT_TIME
                };
                pose.0 = if down {
                    data::POSE_DOWN
                } else {
                    data::POSE_HURT
                };
            }
            stop = stop.max(if down {
                data::HIT_STOP_HEAVY
            } else {
                data::HIT_STOP
            });
            shake = shake.max(if down {
                data::SHAKE_DOWN
            } else {
                data::SHAKE_HIT
            });
            mark = Some((health.0 / plan.health, plan.name));
            if down {
                floored += 1;
            }
            if health.0 <= 0.0 {
                score += plan.score;
            }
            events.send(CrosstownEvent::Landed { down });
        },
    );

    for entity in gone {
        commands.despawn_recursive(entity);
    }
    ring.alive = alive;
    ring.hit_stop = ring.hit_stop.max(stop);
    ring.shake = ring.shake.max(shake);
    ring.score += score + floored * data::COMBO_SCORE;
    if let Some((share, name)) = mark {
        ring.mark = share;
        ring.mark_name = name;
    }
    for _ in 0..floored {
        let pick = talk.next_downed as usize % data::DOWNED.len();
        talk.next_downed += 1;
        say(&mut talk, data::DOWNED[pick]);
    }
    ring.flung = flung;
}

pub fn sweep(mut ring: ResMut<Ring>, thugs: Struck) {
    let flung = std::mem::take(&mut ring.flung);
    if flung.is_empty() {
        return;
    }
    let mut score = 0;
    thugs.for_each(|_entity, (health, stun, pose, flash, kind, transform)| {
        if health.0 <= 0.0 || stun.0 > data::DOWN_TIME - 0.05 {
            return;
        }
        let hit = flung.iter().any(|at| {
            (at.x - transform.translation.x).abs() < data::FLING_REACH * 2.0
                && (at.z - transform.translation.z).abs() < data::FLING_REACH
        });
        if !hit {
            return;
        }
        health.0 = (health.0 - data::FLING_DAMAGE).max(0.0);
        stun.0 = data::DOWN_TIME;
        pose.0 = data::POSE_DOWN;
        flash.0 = data::FLASH_TIME;
        if health.0 <= 0.0 {
            score += data::THUG_KINDS[kind.0 as usize].score;
        }
    });
    ring.score += score;
}

pub fn splinter(
    time: Res<Time>,
    mut ring: ResMut<Ring>,
    sheets: Res<Sheets>,
    mut commands: Commands,
    crates: Query<(&mut Health, &mut Flash, &Breaks, &LocalTransform)>,
) {
    let delta_time = time.delta_time;
    let blow = ring.hero_blow;
    let stage = ring.piece;
    let mut broken = Vec::new();
    let mut hit = false;
    crates.for_each(|entity, (health, flash, breaks, transform)| {
        flash.0 = (flash.0 - delta_time).max(0.0);
        if health.0 <= 0.0 {
            return;
        }
        let Some(blow) = blow else {
            return;
        };
        let plan = &data::BREAK_KINDS[breaks.0 as usize];
        if !in_reach(&blow, transform.translation, plan.radius) {
            return;
        }
        health.0 = (health.0 - blow.damage).max(0.0);
        flash.0 = data::FLASH_TIME;
        hit = true;
        if health.0 <= 0.0 {
            broken.push((entity, transform.translation, breaks.0));
        }
    });
    if hit {
        ring.hit_stop = ring.hit_stop.max(data::HIT_STOP);
        ring.shake = ring.shake.max(data::SHAKE_HIT);
    }
    for (entity, at, kind) in broken {
        commands.despawn_recursive(entity);
        ring.score += data::BREAK_KINDS[kind as usize].score;
        let ground = nalgebra_glm::Vec3::new(at.x, 0.0, at.z);
        burst(&mut commands, kind, ground, stage);
        drop_from(&mut commands, &sheets, ring.drops, ground, stage);
        ring.drops += 1;
    }
}

pub fn hurt(
    time: Res<Time>,
    mut ring: ResMut<Ring>,
    mut talk: ResMut<Talk>,
    heroes: Query<(
        &mut Health,
        &mut Stun,
        &mut Pose,
        &mut Flash,
        &mut Fling,
        &Combo,
    )>,
) {
    let delta_time = time.delta_time;
    let damage = ring.thug_damage;
    let push = ring.thug_push;
    ring.thug_damage = 0.0;
    ring.thug_push = 0.0;
    let mut low = false;
    let mut health_left = ring.hero_health;
    heroes.for_each(|_entity, (health, stun, pose, flash, fling, _)| {
        flash.0 = (flash.0 - delta_time).max(0.0);
        if damage > 0.0 {
            let before = health.0;
            health.0 = (health.0 - damage).max(0.0);
            stun.0 = data::HURT_TIME;
            pose.0 = data::POSE_HURT;
            flash.0 = data::FLASH_TIME;
            fling.0 += push * data::KNOCK_RATE;
            low = before > data::HERO_HEALTH * data::LOW_HEALTH
                && health.0 <= data::HERO_HEALTH * data::LOW_HEALTH;
        }
        health_left = health.0;
    });
    ring.hero_health = health_left;
    if damage > 0.0 {
        ring.hit_stop = ring.hit_stop.max(data::HIT_STOP);
        ring.shake = ring.shake.max(data::SHAKE_HIT);
    }
    if low {
        say(&mut talk, data::HURT_LINE);
    }
}

pub type Winders<'world, 'item> = Query<
    'world,
    (
        &'item mut Strike,
        &'item mut Stun,
        &'item mut Pose,
        &'item LocalTransform,
        &'item Facing,
        &'item Step,
        &'item Lift,
        &'item Holding,
    ),
>;

pub type Struck<'world, 'item> = Query<
    'world,
    (
        &'item mut Health,
        &'item mut Stun,
        &'item mut Pose,
        &'item mut Flash,
        &'item Kind,
        &'item LocalTransform,
    ),
    Without<Combo>,
>;

pub type Foes<'world, 'item> = Query<
    'world,
    (
        &'item mut Stun,
        &'item mut Health,
        &'item mut Pose,
        &'item mut Fling,
        &'item mut Rise,
        &'item mut Flash,
        &'item Kind,
        &'item LocalTransform,
    ),
    Without<Combo>,
>;
