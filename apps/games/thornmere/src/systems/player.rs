use crate::commands::audio::{cue, cue_one_of};
use crate::commands::effects::{burst, fire_trail, frost_trail};
use crate::commands::scenery::prop;
use crate::commands::spells::{bolt_light, halo, heal_aura, slash, sprint_trail, whirl};
use crate::commands::{place_on_ground, push_chat, record_hit};
use crate::components::{
    Airborne, Approach, Carried, Cooldowns, Giver, Heading, Health, Leap, Level, Mana, MaxHealth,
    MaxMana, Pickup, Projectile, Recovery, Speed, Sprint, Strength, Stride, Strike, Target, Worth,
};
use crate::contract::Channel;
use crate::data::{
    ABILITIES, ABILITY_ASKS, APPROACH_STOP, Ask, CALLINGS, CRIT_CHANCE, CRIT_MULTIPLIER, Effect,
    GIVER_REACH, GLOBAL_COOLDOWN, GRAVITY, HEALTH_PER_LEVEL, JUMP_SPEED, MANA_PER_LEVEL,
    PICKUP_REACH, SELECT_RANGE, SOUND_DENY, SOUND_FAINT, SOUND_FIREBALL, SOUND_FROST, SOUND_HEAL,
    SOUND_JUMP, SOUND_PLAIN, SOUND_QUIET, SOUND_SLOT, SOUND_SPRINT, SOUND_WHIRL, SPRINT_MULTIPLIER,
    STEP_CLIPS, STEP_STRIDE, STRENGTH_PER_LEVEL, STRIKE_SECONDS, SWIM_FACTOR, SWING_CLIPS,
    TARGET_CLIPS,
};
use crate::queries::{
    flat_distance, footing, gear_totals, is_alive, swimming, wall_blocks, zone_clamp,
};
use crate::resources::{
    BurstKind, Bursts, Cues, Feel, Hit, HitSource, Hits, ThornmereResources, Walls,
};
use nightshade::prelude::*;
use nightshade_camera_controller::prelude::ThirdPersonCamera;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;
use std::collections::HashMap;

const DUST_STRIDE: f32 = 1.3;

type Runners<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item Speed,
        &'item Health,
        &'item Sprint,
        &'item mut Heading,
        &'item mut Stride,
        &'item mut LocalTransform,
        &'item mut Approach,
    ),
    With<Cooldowns>,
>;

type Goals<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item LocalTransform,
        Option<&'item Giver>,
        Option<&'item Pickup>,
    ),
    Without<Cooldowns>,
>;

pub fn run(
    time: Res<Time>,
    actions: Res<Actions<Ask>>,
    mut resources: ResMut<ThornmereResources>,
    mut feel: ResMut<Feel>,
    mut bursts: ResMut<Bursts>,
    mut cues: ResMut<Cues>,
    walls: Res<Walls>,
    cameras: Query<(&ThirdPersonCamera,)>,
    mut goals: Goals<'_, '_>,
    mut players: Runners<'_, '_>,
) {
    let mut yaw = 0.0;
    cameras.for_each(|_entity, (camera,)| yaw = camera.yaw);

    let direction = actions.axis2(Ask::Walk);
    let steering = direction != Vec2::zeros();
    let forward = Vec3::new(-yaw.sin(), 0.0, -yaw.cos());
    let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
    let keyed = forward * direction.y + right * direction.x;
    let delta_time = time.delta_time;

    let mut wanted = Vec::new();
    players.for_each_mut(|_entity, (_, _, _, _, _, _, approach)| {
        if let Some(goal) = approach.0 {
            wanted.push(goal);
        }
    });
    let mut aims: HashMap<Entity, (Vec3, f32)> = HashMap::new();
    for goal in wanted {
        goals.for_entity(goal, |(spot, giver, pickup)| {
            let stop = if giver.is_some() {
                GIVER_REACH - 0.6
            } else if pickup.is_some() {
                PICKUP_REACH - 0.4
            } else {
                APPROACH_STOP
            };
            aims.insert(goal, (spot.translation, stop));
        });
    }

    let mut footfall = None;
    players.for_each_mut(
        |_entity, (speed, health, sprint, heading, stride, transform, approach)| {
            if !is_alive(health.0) {
                return;
            }
            let movement = if steering {
                approach.0 = None;
                keyed
            } else {
                let Some(goal) = approach.0 else {
                    return;
                };
                let Some(&(spot, stop)) = aims.get(&goal) else {
                    approach.0 = None;
                    return;
                };
                let toward = Vec3::new(
                    spot.x - transform.translation.x,
                    0.0,
                    spot.z - transform.translation.z,
                );
                if toward.magnitude() <= stop {
                    approach.0 = None;
                    return;
                }
                toward.normalize()
            };
            let sprinting = sprint.0 > 0.0;
            let mut pace = if sprinting {
                speed.0 * SPRINT_MULTIPLIER
            } else {
                speed.0
            };
            if swimming(transform.translation.x, transform.translation.z) {
                pace *= SWIM_FACTOR;
            }
            let step = movement * pace * delta_time;
            let here = transform.translation;
            let tries = [
                here + step,
                here + Vec3::new(step.x, 0.0, 0.0),
                here + Vec3::new(0.0, 0.0, step.z),
            ];
            let landed = tries
                .into_iter()
                .find(|next| !wall_blocks(&walls, Vec2::new(next.x, next.z)))
                .unwrap_or(here);
            let step = landed - here;
            transform.translation = zone_clamp(landed);
            stride.0 += step.magnitude();
            heading.0 = (-movement.x).atan2(-movement.z);
            place_on_ground(transform, heading, stride);
            footfall = Some((stride.0, transform.translation, sprinting));
        },
    );

    let Some((stride, position, sprinting)) = footfall else {
        return;
    };
    let dust_every = if sprinting {
        DUST_STRIDE * 0.5
    } else {
        DUST_STRIDE
    };
    if stride - feel.dust_marker >= dust_every {
        feel.dust_marker = stride;
        burst(
            &mut bursts,
            BurstKind::Dust,
            position + Vec3::new(0.0, 0.1, 0.0),
        );
    }
    if stride - feel.step_marker >= STEP_STRIDE {
        feel.step_marker = stride;
        cue_one_of(&mut cues, &mut resources.random, &STEP_CLIPS, SOUND_FAINT);
    }
}

pub fn leap(
    time: Res<Time>,
    actions: Res<Actions<Ask>>,
    mut cues: ResMut<Cues>,
    mut jumpers: Query<(&mut Leap, &mut Airborne, &Health, &mut LocalTransform), With<Cooldowns>>,
) {
    let delta_time = time.delta_time;
    let pressed = actions.just_pressed(Ask::Jump);
    let mut jumped = false;
    jumpers.for_each_mut(|_entity, (leap, airborne, health, transform)| {
        if !is_alive(health.0) {
            return;
        }
        if pressed && airborne.0 <= 0.0 {
            leap.0 = JUMP_SPEED;
            jumped = true;
        }
        if leap.0 == 0.0 && airborne.0 <= 0.0 {
            return;
        }
        leap.0 -= GRAVITY * delta_time;
        airborne.0 = (airborne.0 + leap.0 * delta_time).max(0.0);
        if airborne.0 <= 0.0 {
            leap.0 = 0.0;
        }
        transform.translation.y =
            footing(transform.translation.x, transform.translation.z) + airborne.0;
    });
    if jumped {
        cue(&mut cues, SOUND_JUMP, SOUND_QUIET);
    }
}

pub fn select_target(
    actions: Res<Actions<Ask>>,
    mut resources: ResMut<ThornmereResources>,
    mut cues: ResMut<Cues>,
    monsters: Query<(&Health, &LocalTransform), With<Worth>>,
    mut players: Query<(&mut Target, &LocalTransform), With<Cooldowns>>,
) {
    if !actions.just_pressed(Ask::SelectTarget) {
        return;
    }

    let mut candidates = Vec::new();
    monsters.for_each(|entity, (health, transform)| {
        if is_alive(health.0) {
            candidates.push((entity, transform.translation));
        }
    });

    let mut chosen = false;
    players.for_each_mut(|_entity, (target, transform)| {
        let position = transform.translation;
        let mut best = None;
        let mut best_distance = SELECT_RANGE;
        for (entity, spot) in &candidates {
            if Some(*entity) == target.0 {
                continue;
            }
            let distance = flat_distance(position, *spot);
            if distance < best_distance {
                best_distance = distance;
                best = Some(*entity);
            }
        }
        if let Some(best) = best {
            target.0 = Some(best);
            chosen = true;
        }
    });
    if chosen {
        cue_one_of(&mut cues, &mut resources.random, &TARGET_CLIPS, SOUND_QUIET);
    }
}

fn pressed_slot(actions: &Actions<Ask>) -> Option<usize> {
    ABILITY_ASKS
        .iter()
        .position(|ask| actions.just_pressed(*ask))
}

type Casters<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item mut Cooldowns,
        &'item mut Recovery,
        &'item mut Mana,
        &'item mut Sprint,
        &'item Target,
        &'item Health,
        &'item LocalTransform,
        &'item Level,
    ),
>;

enum Launch {
    Bolt {
        target: Entity,
        origin: Vec3,
        hit: Hit,
    },
    Slash {
        at: Vec3,
        yaw: f32,
    },
    Whirl {
        at: Vec3,
    },
    Halo {
        at: Vec3,
        caster: Entity,
    },
    Sprint {
        caster: Entity,
        seconds: f32,
        at: Vec3,
    },
}

pub fn cast(
    actions: Res<Actions<Ask>>,
    mut resources: ResMut<ThornmereResources>,
    mut hits: ResMut<Hits>,
    mut bursts: ResMut<Bursts>,
    mut cues: ResMut<Cues>,
    mut feel: ResMut<Feel>,
    mut commands: Commands,
    monsters: Query<(&Health, &LocalTransform), With<Worth>>,
    mut players: Casters<'_, '_>,
    mut strikers: Query<(&mut Strike,)>,
) {
    let Some(slot) = pressed_slot(&actions) else {
        return;
    };
    let ability = &ABILITIES[CALLINGS[resources.class].abilities[slot]];
    let resources = &mut *resources;

    let mut monster_spots = Vec::new();
    monsters.for_each(|entity, (health, transform)| {
        if is_alive(health.0) {
            monster_spots.push((entity, transform.translation));
        }
    });

    let mut notice = None;
    let mut launches = Vec::new();
    let random = &mut resources.random;
    players.for_each_mut(
        |player, (cooldowns, recovery, mana, sprint, target, health, transform, level)| {
            if !is_alive(health.0) {
                return;
            }
            if recovery.0 > 0.0 || cooldowns.0[slot] > 0.0 {
                notice = Some("That ability is not ready yet.");
                return;
            }
            if mana.0 < ability.mana {
                notice = Some("Not enough mana.");
                return;
            }
            let power = ability.power + level.0 as f32 * 2.0;
            let position = transform.translation;
            let forward =
                nalgebra_glm::quat_rotate_vec3(&transform.rotation, &Vec3::new(0.0, 0.0, -1.0));
            let yaw = (-forward.x).atan2(-forward.z);

            let landed = match ability.effect {
                Effect::Melee | Effect::Ranged | Effect::Chill => {
                    let Some(target_entity) = target.0 else {
                        notice = Some("You have no target.");
                        return;
                    };
                    let Some(&(_, spot)) = monster_spots
                        .iter()
                        .find(|(entity, _)| *entity == target_entity)
                    else {
                        notice = Some("Your target is dead.");
                        return;
                    };
                    if flat_distance(position, spot) > ability.range {
                        notice = Some("Out of range.");
                        return;
                    }
                    let crit = random.f32() < CRIT_CHANCE;
                    let damage = if crit { power * CRIT_MULTIPLIER } else { power };
                    let chill = ability.effect == Effect::Chill;
                    if ability.effect == Effect::Melee {
                        record_hit(
                            &mut hits,
                            player,
                            target_entity,
                            damage,
                            crit,
                            false,
                            HitSource::Player,
                        );
                        launches.push(Launch::Slash {
                            at: spot + Vec3::new(0.0, 1.1, 0.0),
                            yaw,
                        });
                    } else {
                        launches.push(Launch::Bolt {
                            target: target_entity,
                            origin: position + forward * 0.8 + Vec3::new(0.0, 1.5, 0.0),
                            hit: Hit {
                                attacker: player,
                                target: target_entity,
                                damage,
                                crit,
                                chill,
                                source: HitSource::Player,
                            },
                        });
                    }
                    true
                }
                Effect::Heal => {
                    record_hit(
                        &mut hits,
                        player,
                        player,
                        -power,
                        false,
                        false,
                        HitSource::Heal,
                    );
                    launches.push(Launch::Halo {
                        at: position,
                        caster: player,
                    });
                    true
                }
                Effect::Cleave => {
                    let mut struck = false;
                    for (entity, spot) in &monster_spots {
                        if flat_distance(position, *spot) <= ability.range {
                            let crit = random.f32() < CRIT_CHANCE;
                            let damage = if crit { power * CRIT_MULTIPLIER } else { power };
                            record_hit(
                                &mut hits,
                                player,
                                *entity,
                                damage,
                                crit,
                                false,
                                HitSource::Player,
                            );
                            struck = true;
                        }
                    }
                    if !struck {
                        notice = Some("There is nothing in reach.");
                        return;
                    }
                    launches.push(Launch::Whirl { at: position });
                    true
                }
                Effect::Sprint => {
                    sprint.0 = ability.power;
                    launches.push(Launch::Sprint {
                        caster: player,
                        seconds: ability.power,
                        at: position,
                    });
                    true
                }
            };

            if landed {
                cooldowns.0[slot] = ability.cooldown;
                recovery.0 = GLOBAL_COOLDOWN;
                mana.0 -= ability.mana;
            }
        },
    );

    if let Some(text) = notice {
        push_chat(&mut resources.chat, Channel::Combat, text.to_string());
        cue(&mut cues, SOUND_DENY, SOUND_QUIET);
        return;
    }
    if launches.is_empty() {
        return;
    }
    feel.slot_flash[slot] = 1.0;
    cue(&mut cues, SOUND_SLOT, SOUND_FAINT);

    for launch in launches {
        match launch {
            Launch::Bolt {
                target,
                origin,
                hit,
            } => {
                let frosty = hit.chill;
                let material = if frosty {
                    resources.palette.frost
                } else {
                    resources.palette.fire
                };
                let bolt = prop(
                    &mut commands,
                    "Sphere",
                    origin,
                    Vec3::new(0.34, 0.34, 0.34),
                    material,
                );
                commands.set(bolt, Projectile(Some(target)));
                commands.set(bolt, Carried(hit));
                if frosty {
                    commands.set(bolt, frost_trail(origin));
                    bolt_light(&mut commands, bolt, Vec3::new(0.5, 0.75, 1.0), 18.0);
                    burst(&mut bursts, BurstKind::FrostCast, origin);
                    cue(&mut cues, SOUND_FROST, SOUND_PLAIN);
                } else {
                    commands.set(bolt, fire_trail(origin));
                    bolt_light(&mut commands, bolt, Vec3::new(1.0, 0.55, 0.2), 22.0);
                    burst(&mut bursts, BurstKind::Cast, origin);
                    cue(&mut cues, SOUND_FIREBALL, SOUND_PLAIN);
                }
                if let Some(player) = resources.player {
                    strikers.for_entity(player, |(strike,)| strike.0 = STRIKE_SECONDS);
                }
            }
            Launch::Slash { at, yaw } => {
                slash(&mut commands, &resources.palette, at, yaw);
                cue_one_of(&mut cues, &mut resources.random, &SWING_CLIPS, SOUND_PLAIN);
                if let Some(player) = resources.player {
                    strikers.for_entity(player, |(strike,)| strike.0 = STRIKE_SECONDS);
                }
            }
            Launch::Whirl { at } => {
                whirl(&mut commands, &resources.palette, at);
                cue(&mut cues, SOUND_WHIRL, SOUND_PLAIN);
                feel.trauma = (feel.trauma + 0.3).min(1.0);
                if let Some(player) = resources.player {
                    strikers.for_entity(player, |(strike,)| strike.0 = STRIKE_SECONDS);
                }
            }
            Launch::Halo { at, caster } => {
                halo(&mut commands, &resources.palette, at);
                heal_aura(&mut commands, caster);
                cue(&mut cues, SOUND_HEAL, SOUND_PLAIN);
            }
            Launch::Sprint {
                caster,
                seconds,
                at,
            } => {
                sprint_trail(&mut commands, caster, seconds);
                burst(&mut bursts, BurstKind::Dust, at + Vec3::new(0.0, 0.2, 0.0));
                cue(&mut cues, SOUND_SPRINT, SOUND_PLAIN);
            }
        }
    }
}

type Growers<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item Level,
        &'item mut MaxHealth,
        &'item mut Health,
        &'item mut MaxMana,
        &'item mut Mana,
        &'item mut Strength,
    ),
    With<Cooldowns>,
>;

pub fn grow(mut resources: ResMut<ThornmereResources>, mut players: Growers<'_, '_>) {
    let resources = &mut *resources;
    let class = &CALLINGS[resources.class];
    let (power, guard, spirit) = gear_totals(&resources.gear);
    let shown_level = resources.shown_level;
    let mut reached = shown_level;
    players.for_each_mut(
        |_entity, (level, max_health, health, max_mana, mana, strength)| {
            let steps = (level.0 - 1) as f32;
            let wanted_health = class.health + HEALTH_PER_LEVEL * steps + guard;
            let wanted_mana = class.mana + MANA_PER_LEVEL * steps + spirit;
            let wanted_strength = class.strength + STRENGTH_PER_LEVEL * steps + power;
            let settled = (max_health.0 - wanted_health).abs() < 0.01
                && (max_mana.0 - wanted_mana).abs() < 0.01
                && (strength.0 - wanted_strength).abs() < 0.01;
            if settled && level.0 == shown_level {
                return;
            }
            let leveled = level.0 != shown_level;
            max_health.0 = wanted_health;
            max_mana.0 = wanted_mana;
            strength.0 = wanted_strength;
            if leveled {
                health.0 = max_health.0;
                mana.0 = max_mana.0;
            } else {
                health.0 = health.0.min(max_health.0);
                mana.0 = mana.0.min(max_mana.0);
            }
            reached = level.0;
        },
    );
    resources.shown_level = reached;
}
