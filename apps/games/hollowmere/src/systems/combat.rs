use crate::commands::audio::{cue, cue_one_of};
use crate::commands::chat::{random_fill, start_thread};
use crate::commands::effects::burst;
use crate::commands::spells::requiem;
use crate::commands::{
    banner, flash, grant_experience, push_chat, record_hit, record_kill, teleport,
};
use crate::components::{
    Bounty, Chill, Class, Cooldowns, Dying, Experience, Flinch, Heading, Health, Home, Job, Kills,
    Level, Mana, MaxHealth, MaxMana, Rank, Rebirth, Recovery, Respawn, Rest, Rising, Sprint,
    Strength, Stride, Strike, Swing, Target, Task, Voice, Worth,
};
use crate::contract::Channel;
use crate::data::chatter::{BOSS_UP, DEATH, PLAYER_DEATH};
use crate::data::{
    COIN_CLIPS, CRIT_CLIPS, ELITE, HIT_CLIPS, HURT_CLIPS, KILL_CLIPS, PLACES, SOUND_DEATH,
    SOUND_ELITE_UP, SWING_CLIPS, TOWN,
};
use crate::queries::{coin_text, flat_distance, guessed_ground, is_alive, place_at, spot_of};
use crate::resources::{
    Banners, BurstKind, Bursts, Cues, Feel, Flashes, Generation, Hero, HitSource, Hits, Inventory,
    Journal, KillEvent, Places, Slain, Speaker, VoxelWorld,
};
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_random::prelude::*;
use nightshade_scene::prelude::*;
use std::collections::HashMap;

const PLAYER_HIT_COLOR: Vec4 = Vec4::new(1.0, 0.92, 0.35, 1.0);
const CRIT_COLOR: Vec4 = Vec4::new(1.0, 0.55, 0.15, 1.0);
const TAKEN_COLOR: Vec4 = Vec4::new(1.0, 0.30, 0.25, 1.0);
const OTHER_COLOR: Vec4 = Vec4::new(0.85, 0.85, 0.90, 0.85);
const HEAL_COLOR: Vec4 = Vec4::new(0.40, 1.0, 0.45, 1.0);
const GOLD_COLOR: Vec4 = Vec4::new(1.0, 0.85, 0.30, 1.0);
const DEATH_RED: Vec4 = Vec4::new(0.9, 0.25, 0.2, 1.0);

type Attackers<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item Target,
        &'item mut Swing,
        &'item Strength,
        &'item Health,
        &'item LocalTransform,
        Option<&'item Worth>,
        Option<&'item Cooldowns>,
    ),
>;

pub fn swings(
    time: Res<Time>,
    mut hero: ResMut<Hero>,
    mut hits: ResMut<Hits>,
    mut cues: ResMut<Cues>,
    victims: Query<(&Health, &LocalTransform), With<Target>>,
    mut attackers: Attackers<'_, '_>,
    mut strikers: Query<(&mut Strike,)>,
) {
    let delta_time = time.delta_time;
    let hero = &mut *hero;
    let player = hero.player;
    let random = &mut hero.random;

    let mut spots: HashMap<Entity, (f32, Vec3)> = HashMap::new();
    victims.for_each(|entity, (health, transform)| {
        spots.insert(entity, (health.0, transform.translation));
    });

    let mut swung = Vec::new();
    attackers.for_each_mut(
        |attacker, (target, swing, strength, health, transform, worth, cooldowns)| {
            swing.0 = (swing.0 - delta_time).max(0.0);
            if !is_alive(health.0) {
                return;
            }
            let Some(target_entity) = target.0 else {
                return;
            };
            let Some(&(target_health, spot)) = spots.get(&target_entity) else {
                return;
            };
            if !is_alive(target_health)
                || flat_distance(transform.translation, spot) > theme::MELEE_REACH
            {
                return;
            }
            if swing.0 > 0.0 {
                return;
            }
            swing.0 = theme::ATTACK_PERIOD;
            let source = if worth.is_some() {
                HitSource::Monster
            } else if cooldowns.is_some() {
                HitSource::Player
            } else {
                HitSource::Person
            };
            let crit = source == HitSource::Player && random.f32() < theme::CRIT_CHANCE;
            let damage = if crit {
                strength.0 * theme::CRIT_MULTIPLIER
            } else {
                strength.0
            };
            record_hit(
                &mut hits,
                attacker,
                target_entity,
                damage,
                crit,
                false,
                source,
            );
            swung.push((attacker, source, target_entity));
        },
    );
    for (attacker, source, target_entity) in swung {
        strikers.for_entity(attacker, |(strike,)| strike.0 = theme::STRIKE_SECONDS);
        let near_player = Some(attacker) == player || Some(target_entity) == player;
        if source == HitSource::Player {
            cue_one_of(&mut cues, random, &SWING_CLIPS, theme::SOUND_PLAIN);
        } else if near_player {
            cue_one_of(&mut cues, random, &SWING_CLIPS, theme::SOUND_QUIET);
        }
    }
}

type Victims<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item mut Health,
        &'item MaxHealth,
        &'item mut Target,
        &'item mut Chill,
        &'item mut Flinch,
        &'item LocalTransform,
        &'item Name,
    ),
>;

pub fn land(
    mut hero: ResMut<Hero>,
    mut journal: ResMut<Journal>,
    mut inventory: ResMut<Inventory>,
    places: Res<Places>,
    mut hits: ResMut<Hits>,
    mut slain: ResMut<Slain>,
    mut flashes: ResMut<Flashes>,
    mut bursts: ResMut<Bursts>,
    mut cues: ResMut<Cues>,
    mut feel: ResMut<Feel>,
    mut commands: Commands,
    mut names: Query<(&Name,)>,
    mut rewards: Query<(&Worth, &Bounty, &Rank, &Home)>,
    mut victims: Victims<'_, '_>,
    mut killers: Query<(&mut Kills, &mut Level, &mut Experience)>,
) {
    let hero = &mut *hero;
    let player = hero.player;

    for hit in std::mem::take(&mut hits.pending) {
        let mut outcome = None;
        victims.for_entity(
            hit.target,
            |(health, max_health, target, chill, flinch, transform, name)| {
                if !is_alive(health.0) {
                    return;
                }
                if hit.source == HitSource::Heal {
                    let before = health.0;
                    health.0 = (health.0 - hit.damage).min(max_health.0);
                    let healed = health.0 - before;
                    outcome = Some((
                        false,
                        transform.translation,
                        format!("+{}", healed.round() as i32),
                        HEAL_COLOR,
                        name.0.clone(),
                        theme::FLOATING_POP,
                    ));
                    return;
                }
                health.0 -= hit.damage;
                flinch.0 = theme::FLINCH_SECONDS;
                if target.0.is_none() {
                    target.0 = Some(hit.attacker);
                }
                if hit.chill {
                    chill.0 = theme::CHILL_SECONDS;
                }
                let text = if hit.crit {
                    format!("{}!", hit.damage.round() as i32)
                } else {
                    format!("{}", hit.damage.round() as i32)
                };
                let color = match hit.source {
                    HitSource::Player if hit.crit => CRIT_COLOR,
                    HitSource::Player => PLAYER_HIT_COLOR,
                    HitSource::Monster if Some(hit.target) == player => TAKEN_COLOR,
                    _ => OTHER_COLOR,
                };
                let pop = if hit.crit {
                    theme::CRIT_POP
                } else {
                    theme::FLOATING_POP
                };
                outcome = Some((
                    !is_alive(health.0),
                    transform.translation,
                    text,
                    color,
                    name.0.clone(),
                    pop,
                ));
            },
        );

        let Some((killed, position, text, color, victim_name, pop)) = outcome else {
            continue;
        };
        let strike_point = position + Vec3::new(0.0, 1.1, 0.0);
        flash(&mut flashes, position, text, color, pop);

        let by_player = Some(hit.attacker) == player;
        let on_player = Some(hit.target) == player;
        if by_player || on_player {
            if !feel.combat_shown {
                feel.combat_shown = true;
                push_chat(
                    &mut journal.chat,
                    Channel::Combat,
                    "You enter combat.".to_string(),
                );
            }
            feel.combat = theme::COMBAT_LINGER;
        }

        match hit.source {
            HitSource::Heal => {
                burst(
                    &mut bursts,
                    BurstKind::Heal,
                    position + Vec3::new(0.0, 0.6, 0.0),
                );
            }
            HitSource::Player => {
                if hit.chill {
                    burst(&mut bursts, BurstKind::Frost, strike_point);
                } else if hit.crit {
                    burst(&mut bursts, BurstKind::Crit, strike_point);
                    cue_one_of(&mut cues, &mut hero.random, &CRIT_CLIPS, theme::SOUND_LOUD);
                    feel.trauma = (feel.trauma + theme::CRIT_TRAUMA).min(1.0);
                } else {
                    burst(&mut bursts, BurstKind::Hit, strike_point);
                    cue_one_of(&mut cues, &mut hero.random, &HIT_CLIPS, theme::SOUND_PLAIN);
                    feel.trauma = (feel.trauma + theme::HIT_TRAUMA).min(1.0);
                }
            }
            HitSource::Monster if on_player => {
                burst(&mut bursts, BurstKind::Hit, strike_point);
                cue_one_of(&mut cues, &mut hero.random, &HURT_CLIPS, theme::SOUND_LOUD);
                feel.trauma = (feel.trauma + theme::HURT_TRAUMA).min(1.0);
                feel.hurt = 1.0;
            }
            _ => {
                burst(&mut bursts, BurstKind::Hit, strike_point);
            }
        }

        if !killed {
            continue;
        }

        let mut reward = None;
        rewards.for_entity(hit.target, |(worth, bounty, rank, home)| {
            reward = Some((worth.0, bounty.0, rank.0, place_at(&places, home.0)));
        });
        let Some((experience, coins, rank, place)) = reward else {
            continue;
        };
        let elite = rank >= ELITE;

        let mut killer_name = String::new();
        names.for_entity(hit.attacker, |(name,)| killer_name = name.0.clone());
        killers.for_entity(hit.attacker, |(kills, level, tally)| {
            kills.0 += 1;
            grant_experience(level, tally, experience);
        });
        burst(
            &mut bursts,
            if elite {
                BurstKind::EliteKill
            } else {
                BurstKind::Kill
            },
            strike_point,
        );
        requiem(&mut commands, position, elite);
        if by_player {
            inventory.coins += coins;
            push_chat(
                &mut journal.chat,
                Channel::Combat,
                format!(
                    "{victim_name} dies. You gain {experience} experience and loot {}.",
                    coin_text(coins)
                ),
            );
            flash(
                &mut flashes,
                position + Vec3::new(0.4, 0.5, 0.0),
                format!("+{}", coin_text(coins)),
                GOLD_COLOR,
                theme::FLOATING_POP,
            );
            burst(&mut bursts, BurstKind::Coins, position);
            cue_one_of(&mut cues, &mut hero.random, &KILL_CLIPS, theme::SOUND_PLAIN);
            cue_one_of(&mut cues, &mut hero.random, &COIN_CLIPS, theme::SOUND_QUIET);
            feel.trauma = (feel.trauma
                + if elite {
                    theme::ELITE_KILL_TRAUMA
                } else {
                    theme::KILL_TRAUMA
                })
            .min(1.0);
        }
        record_kill(
            &mut slain,
            KillEvent {
                prey: victim_name,
                rank,
                place,
                by_player,
                killer_name,
            },
        );
    }

    if feel.combat <= 0.0 && feel.combat_shown {
        feel.combat_shown = false;
        push_chat(
            &mut journal.chat,
            Channel::Combat,
            "You leave combat.".to_string(),
        );
    }
}

type Mobs<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item mut Health,
        &'item MaxHealth,
        &'item Home,
        &'item mut Respawn,
        &'item Rebirth,
        &'item Rank,
        &'item mut Target,
        &'item mut LocalTransform,
    ),
>;

type FallenPeople<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item mut Health,
        &'item MaxHealth,
        &'item mut Task,
        &'item mut Rest,
        &'item mut Target,
        &'item Heading,
        &'item Stride,
        &'item mut LocalTransform,
    ),
>;

type FallenPlayers<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item mut Health,
        &'item MaxHealth,
        &'item mut Mana,
        &'item mut Target,
        &'item Heading,
        &'item Stride,
        &'item mut LocalTransform,
    ),
    With<Cooldowns>,
>;

pub fn deaths(
    time: Res<Time>,
    world: Res<VoxelWorld>,
    generation: Res<Generation>,
    places: Res<Places>,
    mut hero: ResMut<Hero>,
    mut journal: ResMut<Journal>,
    mut cues: ResMut<Cues>,
    mut bursts: ResMut<Bursts>,
    mut banners: ResMut<Banners>,
    mut feel: ResMut<Feel>,
    mut names: Query<(&Name,)>,
    mut voices: Query<(&Name, &Voice, &Class)>,
    mut dying: Query<(&mut Dying, &mut Rising)>,
    mut mobs: Mobs<'_, '_>,
    mut people: FallenPeople<'_, '_>,
    mut players: FallenPlayers<'_, '_>,
) {
    let delta_time = time.delta_time;
    let hero = &mut *hero;
    let journal = &mut *journal;
    let graveyard = spot_of(&places, TOWN, theme::GRAVEYARD_OFFSET);

    let mut risen = Vec::new();
    let mut fell = Vec::new();
    mobs.for_each_mut(
        |entity, (health, max_health, home, respawn, rebirth, rank, target, transform)| {
            if is_alive(health.0) {
                return;
            }
            if respawn.0 <= 0.0 {
                respawn.0 = rebirth.0;
                target.0 = None;
                fell.push(entity);
                return;
            }
            respawn.0 -= delta_time;
            if respawn.0 <= 0.0 {
                respawn.0 = 0.0;
                health.0 = max_health.0;
                transform.translation = home.0;
                risen.push((entity, rank.0 >= ELITE, place_at(&places, home.0), home.0));
            }
        },
    );
    for entity in fell {
        dying.for_entity(entity, |(dying, _)| dying.0 = theme::DYING_SECONDS);
    }
    for (entity, elite, place, home) in risen {
        dying.for_entity(entity, |(_, rising)| rising.0 = theme::RISING_SECONDS);
        burst(
            &mut bursts,
            BurstKind::Puff,
            home + Vec3::new(0.0, 0.5, 0.0),
        );
        if !elite {
            continue;
        }
        cue(&mut cues, SOUND_ELITE_UP, theme::SOUND_QUIET);
        let mut fill = random_fill(&mut hero.random, &journal.chat.roster, &hero.name);
        names.for_entity(entity, |(name,)| fill.monster = name.0.clone());
        if let Some(place) = place {
            fill.place = PLACES[place].name.to_string();
        }
        let thread = &BOSS_UP[hero.random.usize(0..BOSS_UP.len())];
        start_thread(&mut journal.chat, &mut hero.random, thread, &fill, 1.5);
    }

    let mut fallen = Vec::new();
    people.for_each_mut(
        |entity, (health, max_health, task, rest, target, heading, stride, transform)| {
            if is_alive(health.0) || task.0 == Job::Offline {
                return;
            }
            let killer = target.0;
            health.0 = max_health.0;
            task.0 = Job::Resting;
            rest.0 = between(
                &mut hero.random,
                theme::REST_MIN_SECONDS,
                theme::REST_MAX_SECONDS,
            );
            target.0 = None;
            let spot = around(&mut hero.random, graveyard, 1.0, 3.0);
            teleport(
                &world,
                transform,
                heading,
                stride,
                guessed_ground(&generation, spot),
            );
            fallen.push((entity, killer));
        },
    );
    for (entity, killer) in fallen {
        if hero.random.f32() > theme::DEATH_THREAD_CHANCE {
            continue;
        }
        let mut speaker = None;
        voices.for_entity(entity, |(name, voice, class)| {
            speaker = Some(Speaker {
                name: name.0.clone(),
                voice: voice.0,
                class: class.0,
            });
        });
        let mut fill = random_fill(&mut hero.random, &journal.chat.roster, &hero.name);
        if let Some(speaker) = speaker {
            fill.speakers[0] = speaker;
        }
        if let Some(killer) = killer {
            names.for_entity(killer, |(name,)| fill.monster = name.0.clone());
        }
        let thread = &DEATH[hero.random.usize(0..DEATH.len())];
        start_thread(&mut journal.chat, &mut hero.random, thread, &fill, 1.0);
    }

    let mut died = None;
    players.for_each_mut(
        |_entity, (health, max_health, mana, target, heading, stride, transform)| {
            if is_alive(health.0) {
                return;
            }
            died = Some(target.0);
            health.0 = max_health.0;
            mana.0 = 0.0;
            target.0 = None;
            teleport(
                &world,
                transform,
                heading,
                stride,
                guessed_ground(&generation, graveyard + Vec2::new(0.0, 4.0)),
            );
        },
    );
    if let Some(killer) = died {
        let mut monster = "something".to_string();
        if let Some(killer) = killer {
            names.for_entity(killer, |(name,)| monster = name.0.clone());
        }
        push_chat(
            &mut journal.chat,
            Channel::System,
            format!("You have died. {monster} did it. You wake at the graveyard."),
        );
        feel.grey = 1.0;
        feel.trauma = 1.0;
        cue(&mut cues, SOUND_DEATH, theme::SOUND_LOUD);
        banner(
            &mut banners,
            "You have died".to_string(),
            format!("{monster} was too much. The graveyard has you."),
            DEATH_RED,
        );
        let mut fill = random_fill(&mut hero.random, &journal.chat.roster, &hero.name);
        fill.monster = monster;
        let thread = &PLAYER_DEATH[hero.random.usize(0..PLAYER_DEATH.len())];
        start_thread(&mut journal.chat, &mut hero.random, thread, &fill, 2.0);
    }
}

pub fn mend(time: Res<Time>, mut resting: Query<(&mut Health, &MaxHealth, &Target)>) {
    let delta_time = time.delta_time;
    resting.for_each_mut(|_entity, (health, max_health, target)| {
        if target.0.is_some() || !is_alive(health.0) || health.0 >= max_health.0 {
            return;
        }
        health.0 = (health.0 + max_health.0 * theme::MEND_RATE * delta_time).min(max_health.0);
    });
}

pub fn effects(
    time: Res<Time>,
    mut players: Query<(
        &mut Cooldowns,
        &mut Recovery,
        &mut Sprint,
        &mut Mana,
        &MaxMana,
        &Target,
    )>,
    mut chilled: Query<(&mut Chill,)>,
) {
    let delta_time = time.delta_time;
    players.for_each_mut(
        |_entity, (cooldowns, recovery, sprint, mana, max_mana, target)| {
            for cooldown in cooldowns.0.iter_mut() {
                *cooldown = (*cooldown - delta_time).max(0.0);
            }
            recovery.0 = (recovery.0 - delta_time).max(0.0);
            sprint.0 = (sprint.0 - delta_time).max(0.0);
            let regen = if target.0.is_some() {
                theme::MANA_REGEN
            } else {
                theme::REST_MANA_REGEN
            };
            mana.0 = (mana.0 + regen * delta_time).min(max_mana.0);
        },
    );
    chilled.for_each_mut(|_entity, (chill,)| {
        chill.0 = (chill.0 - delta_time).max(0.0);
    });
}

pub fn forget(mut slain: ResMut<Slain>) {
    slain.events.clear();
}
