use crate::commands::audio::{cue, cue_one_of};
use crate::commands::effects::{burst, fire_trail, frost_trail};
use crate::commands::spells::{
    FIRE, FROST, bolt_light, halo, heal_aura, prop, slash, sprint_trail, whirl,
};
use crate::commands::{push_chat, record_hit};
use crate::components::{
    Approach, Cooldowns, Giver, Heading, Health, Level, Mana, MaxHealth, MaxMana, Pickup,
    PlateLevel, Projectile, Recovery, Speed, Sprint, Strength, Stride, Strike, Target, Worth,
};
use crate::contract::Channel;
use crate::data::{
    ABILITIES, ABILITY_ASKS, Ask, CALLINGS, Effect, SOUND_DENY, SOUND_FIREBALL, SOUND_FROST,
    SOUND_HEAL, SOUND_JUMP, SOUND_SLOT, SOUND_SPRINT, SOUND_WHIRL, STEP_CLIPS, SWING_CLIPS,
    TARGET_CLIPS,
};
use crate::queries::{box_in_liquid, flat_distance, hero_box, is_alive, slide_box};
use crate::resources::{
    BurstKind, Bursts, Cues, Feel, Hero, Hit, HitSource, Hits, Journal, Player, VoxelWorld,
};
use crate::theme;
use nightshade::prelude::*;
use nightshade_camera_controller::prelude::ThirdPersonCamera;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;
use std::collections::HashMap;

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
    world: Res<VoxelWorld>,
    mut player: ResMut<Player>,
    mut hero: ResMut<Hero>,
    mut feel: ResMut<Feel>,
    mut bursts: ResMut<Bursts>,
    mut cues: ResMut<Cues>,
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
    let jump = actions.just_pressed(Ask::Jump);
    let delta_time = time.delta_time.min(0.05);

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
                theme::GIVER_REACH - 0.6
            } else if pickup.is_some() {
                theme::PICKUP_REACH - 0.4
            } else {
                theme::APPROACH_STOP
            };
            aims.insert(goal, (spot.translation, stop));
        });
    }

    let mut footfall = None;
    let mut jumped = false;
    players.for_each_mut(
        |_entity, (speed, health, sprint, heading, stride, transform, approach)| {
            let alive = is_alive(health.0);
            let movement = if !alive {
                approach.0 = None;
                Vec3::zeros()
            } else if steering {
                approach.0 = None;
                keyed
            } else if let Some(goal) = approach.0 {
                match aims.get(&goal) {
                    Some(&(spot, stop)) => {
                        let toward = Vec3::new(
                            spot.x - transform.translation.x,
                            0.0,
                            spot.z - transform.translation.z,
                        );
                        if toward.magnitude() <= stop {
                            approach.0 = None;
                            Vec3::zeros()
                        } else {
                            toward.normalize()
                        }
                    }
                    None => {
                        approach.0 = None;
                        Vec3::zeros()
                    }
                }
            } else {
                Vec3::zeros()
            };

            let sprinting = sprint.0 > 0.0;
            let mut pace = if sprinting {
                speed.0 * theme::SPRINT_MULTIPLIER
            } else {
                speed.0
            };
            let (low_corner, high_corner) = hero_box(transform.translation);
            let swimming =
                box_in_liquid(&world, low_corner, high_corner + Vec3::new(0.0, -0.9, 0.0))
                    == Some(theme::WATER);
            if swimming {
                pace = theme::SWIM_SPEED.max(pace * 0.4);
            }

            let mut velocity = player.velocity;
            velocity.x = movement.x * pace;
            velocity.z = movement.z * pace;
            if swimming {
                velocity.y -= theme::WATER_GRAVITY * delta_time;
                velocity.y = velocity.y.max(-3.0);
                if jump && alive {
                    velocity.y = 3.2;
                }
            } else {
                velocity.y -= theme::GRAVITY * delta_time;
                if jump && alive && player.on_ground {
                    velocity.y = theme::JUMP_SPEED;
                    jumped = true;
                }
            }

            let low = Vec3::new(-theme::PLAYER_HALF_WIDTH, 0.0, -theme::PLAYER_HALF_WIDTH);
            let high = Vec3::new(
                theme::PLAYER_HALF_WIDTH,
                theme::PLAYER_HEIGHT,
                theme::PLAYER_HALF_WIDTH,
            );
            let before = transform.translation;
            let slide = slide_box(&world, before, low, high, velocity * delta_time);
            for axis in 0..3 {
                if slide.blocked[axis] {
                    velocity[axis] = 0.0;
                }
            }
            let mut position = slide.position;
            if position.y < 1.0 {
                position = player.home;
                velocity = Vec3::zeros();
            }
            transform.translation = position;
            let moved = Vec3::new(position.x - before.x, 0.0, position.z - before.z).magnitude();
            stride.0 += moved;
            if movement != Vec3::zeros() {
                heading.0 = (-movement.x).atan2(-movement.z);
            }
            transform.rotation = nalgebra_glm::quat_angle_axis(heading.0, &Vec3::y());

            player.position = position;
            player.velocity = velocity;
            player.yaw = heading.0;
            player.on_ground = slide.on_ground;
            player.in_water = swimming;
            if slide.on_ground && moved > 0.0 {
                footfall = Some((stride.0, position, sprinting));
            }
        },
    );

    if jumped {
        cue(&mut cues, SOUND_JUMP, theme::SOUND_QUIET);
    }
    let Some((stride, position, sprinting)) = footfall else {
        return;
    };
    let dust_every = if sprinting {
        theme::DUST_STRIDE * 0.5
    } else {
        theme::DUST_STRIDE
    };
    if stride - feel.dust_marker >= dust_every {
        feel.dust_marker = stride;
        burst(
            &mut bursts,
            BurstKind::Dust,
            position + Vec3::new(0.0, 0.1, 0.0),
        );
    }
    if stride - feel.step_marker >= theme::STEP_STRIDE {
        feel.step_marker = stride;
        cue_one_of(&mut cues, &mut hero.random, &STEP_CLIPS, theme::SOUND_FAINT);
    }
}

pub fn select_target(
    actions: Res<Actions<Ask>>,
    mut hero: ResMut<Hero>,
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
        let mut best_distance = theme::SELECT_RANGE;
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
        cue_one_of(
            &mut cues,
            &mut hero.random,
            &TARGET_CLIPS,
            theme::SOUND_QUIET,
        );
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
    mut hero: ResMut<Hero>,
    mut journal: ResMut<Journal>,
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
    let ability = &ABILITIES[CALLINGS[hero.class].abilities[slot]];
    let hero = &mut *hero;

    let mut monster_spots = Vec::new();
    monsters.for_each(|entity, (health, transform)| {
        if is_alive(health.0) {
            monster_spots.push((entity, transform.translation));
        }
    });

    let mut notice = None;
    let mut launches = Vec::new();
    let random = &mut hero.random;
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
                    let crit = random.f32() < theme::CRIT_CHANCE;
                    let damage = if crit {
                        power * theme::CRIT_MULTIPLIER
                    } else {
                        power
                    };
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
                            let crit = random.f32() < theme::CRIT_CHANCE;
                            let damage = if crit {
                                power * theme::CRIT_MULTIPLIER
                            } else {
                                power
                            };
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
                recovery.0 = theme::GLOBAL_COOLDOWN;
                mana.0 -= ability.mana;
            }
        },
    );

    if let Some(text) = notice {
        push_chat(&mut journal.chat, Channel::Combat, text.to_string());
        cue(&mut cues, SOUND_DENY, theme::SOUND_QUIET);
        return;
    }
    if launches.is_empty() {
        return;
    }
    feel.slot_flash[slot] = 1.0;
    cue(&mut cues, SOUND_SLOT, theme::SOUND_FAINT);

    for launch in launches {
        match launch {
            Launch::Bolt {
                target,
                origin,
                hit,
            } => {
                let frosty = hit.chill;
                let material = if frosty { FROST } else { FIRE };
                let bolt = prop(
                    &mut commands,
                    "Sphere",
                    origin,
                    Vec3::new(0.34, 0.34, 0.34),
                    material,
                );
                commands.set(bolt, Projectile(Some(target)));
                commands.set(bolt, crate::components::Carried(hit));
                if frosty {
                    commands.set(bolt, frost_trail(origin));
                    bolt_light(&mut commands, bolt, Vec3::new(0.5, 0.75, 1.0), 18.0);
                    burst(&mut bursts, BurstKind::FrostCast, origin);
                    cue(&mut cues, SOUND_FROST, theme::SOUND_PLAIN);
                } else {
                    commands.set(bolt, fire_trail(origin));
                    bolt_light(&mut commands, bolt, Vec3::new(1.0, 0.55, 0.2), 22.0);
                    burst(&mut bursts, BurstKind::Cast, origin);
                    cue(&mut cues, SOUND_FIREBALL, theme::SOUND_PLAIN);
                }
                if let Some(player) = hero.player {
                    strikers.for_entity(player, |(strike,)| strike.0 = theme::STRIKE_SECONDS);
                }
            }
            Launch::Slash { at, yaw } => {
                slash(&mut commands, at, yaw);
                cue_one_of(
                    &mut cues,
                    &mut hero.random,
                    &SWING_CLIPS,
                    theme::SOUND_PLAIN,
                );
                if let Some(player) = hero.player {
                    strikers.for_entity(player, |(strike,)| strike.0 = theme::STRIKE_SECONDS);
                }
            }
            Launch::Whirl { at } => {
                whirl(&mut commands, at);
                cue(&mut cues, SOUND_WHIRL, theme::SOUND_PLAIN);
                feel.trauma = (feel.trauma + 0.3).min(1.0);
                if let Some(player) = hero.player {
                    strikers.for_entity(player, |(strike,)| strike.0 = theme::STRIKE_SECONDS);
                }
            }
            Launch::Halo { at, caster } => {
                halo(&mut commands, at);
                heal_aura(&mut commands, caster);
                cue(&mut cues, SOUND_HEAL, theme::SOUND_PLAIN);
            }
            Launch::Sprint {
                caster,
                seconds,
                at,
            } => {
                sprint_trail(&mut commands, caster, seconds);
                burst(&mut bursts, BurstKind::Dust, at + Vec3::new(0.0, 0.2, 0.0));
                cue(&mut cues, SOUND_SPRINT, theme::SOUND_PLAIN);
            }
        }
    }
}

type Growers<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item Level,
        &'item PlateLevel,
        &'item mut MaxHealth,
        &'item mut Health,
        &'item mut MaxMana,
        &'item mut Mana,
        &'item mut Strength,
    ),
    With<Cooldowns>,
>;

pub fn grow(hero: Res<Hero>, mut players: Growers<'_, '_>) {
    let class = &CALLINGS[hero.class];
    players.for_each_mut(
        |_entity, (level, shown, max_health, health, max_mana, mana, strength)| {
            if level.0 == shown.0 {
                return;
            }
            let steps = (level.0 - 1) as f32;
            max_health.0 = class.health + theme::HEALTH_PER_LEVEL * steps;
            max_mana.0 = class.mana + theme::MANA_PER_LEVEL * steps;
            strength.0 = class.strength + theme::STRENGTH_PER_LEVEL * steps;
            health.0 = max_health.0;
            mana.0 = max_mana.0;
        },
    );
}
