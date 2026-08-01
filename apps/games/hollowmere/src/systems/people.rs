use crate::commands::chat::{random_fill, start_thread};
use crate::commands::random::{random_line, random_place_spot, stylize};
use crate::commands::{Walk, grant_experience, place_on_ground, teleport, walk_toward};
use crate::components::{
    Class, Experience, Goal, Heading, Health, Hunt, Job, Kills, Level, Rest, Speed, Stride, Target,
    Task, Voice, Worth,
};
use crate::contract::{Channel, ChatLine};
use crate::data::chatter::{CAMP_LINES, LOGOFF, LOGON, RETURN_LINES, TURN_IN_LINES};
use crate::data::{GIVERS, HUNTS, PLACES, TOWN};
use crate::queries::chat::fill_text;
use crate::queries::{
    flat_distance, guessed_ground, heading_toward, hunt_for_level, is_alive, offline_fraction,
    spot_of,
};
use crate::resources::{Generation, Hero, Journal, Places, Speaker, VoxelWorld};
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_random::prelude::*;
use nightshade_render::data::config::RendererState;
use nightshade_scene::prelude::*;
use std::collections::HashMap;

const TARGET_CHOICES: usize = 3;
const PATIENCE_SECONDS: f32 = 3.0;

#[derive(Clone, Copy)]
enum Announcement {
    Camp,
    Return,
    TurnIn,
}

type Walkers<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item Speed,
        &'item mut Task,
        &'item mut Rest,
        &'item mut Goal,
        &'item mut Target,
        &'item mut Heading,
        &'item mut Stride,
        &'item mut LocalTransform,
    ),
>;

type Progress<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item mut Kills,
        &'item mut Level,
        &'item mut Experience,
        &'item mut Hunt,
        &'item Name,
        &'item Voice,
    ),
    With<Task>,
>;

fn travel(
    world: &VoxelWorld,
    transform: &mut LocalTransform,
    heading: &mut Heading,
    stride: &mut Stride,
    rest: &mut Rest,
    goal: Vec3,
    speed: f32,
    delta_time: f32,
) -> bool {
    match walk_toward(world, transform, heading, stride, goal, speed, delta_time) {
        Walk::Arrived => true,
        Walk::Moved => {
            rest.0 = 0.0;
            false
        }
        Walk::Blocked => {
            rest.0 += delta_time;
            if rest.0 > PATIENCE_SECONDS {
                rest.0 = 0.0;
                let toward = Vec3::new(
                    goal.x - transform.translation.x,
                    0.0,
                    goal.z - transform.translation.z,
                );
                let hop = if toward.magnitude() > 3.0 {
                    toward.normalize() * 3.0
                } else {
                    toward
                };
                let spot = transform.translation + hop + Vec3::new(0.0, 2.0, 0.0);
                teleport(world, transform, heading, stride, spot);
            }
            false
        }
    }
}

pub fn keep_the_loop(
    time: Res<Time>,
    world: Res<VoxelWorld>,
    generation: Res<Generation>,
    places: Res<Places>,
    mut hero: ResMut<Hero>,
    mut events: EventWriter<ChatLine>,
    facts: Query<(&Health, &Hunt, &Kills), With<Task>>,
    mobs: Query<(&Health, &LocalTransform, &Name), With<Worth>>,
    mut walkers: Walkers<'_, '_>,
    mut progress: Progress<'_, '_>,
) {
    let delta_time = time.delta_time;
    let random = &mut hero.random;

    let mut prey: Vec<(Entity, Vec3, String)> = Vec::new();
    mobs.for_each(|entity, (health, transform, name)| {
        if is_alive(health.0) {
            prey.push((entity, transform.translation, name.0.clone()));
        }
    });

    let mut known: HashMap<Entity, (bool, usize, u32)> = HashMap::new();
    facts.for_each(|entity, (health, hunt, kills)| {
        known.insert(entity, (is_alive(health.0), hunt.0, kills.0));
    });

    let mut announcements: Vec<(Entity, Announcement)> = Vec::new();
    walkers.for_each_mut(
        |entity, (speed, task, rest, goal, target, heading, stride, transform)| {
            let Some(&(alive, hunt, kills)) = known.get(&entity) else {
                return;
            };
            if !alive || task.0 == Job::Offline {
                return;
            }
            let hunt_data = &HUNTS[hunt];
            match task.0 {
                Job::Resting => {
                    rest.0 -= delta_time;
                    if rest.0 <= 0.0 {
                        task.0 = Job::ToCamp;
                        goal.0 = guessed_ground(
                            &generation,
                            random_place_spot(random, &places, hunt_data.place, 0.8),
                        );
                        if random.f32() < theme::CAMP_LINE_CHANCE {
                            announcements.push((entity, Announcement::Camp));
                        }
                    }
                }
                Job::ToCamp => {
                    if travel(
                        &world, transform, heading, stride, rest, goal.0, speed.0, delta_time,
                    ) {
                        task.0 = Job::Hunting;
                    }
                }
                Job::Hunting => {
                    if kills >= theme::SIM_QUEST_KILLS {
                        task.0 = Job::ToGiver;
                        let giver = &GIVERS[hunt_data.giver];
                        let giver_spot = spot_of(&places, giver.place, giver.offset);
                        goal.0 = guessed_ground(&generation, around(random, giver_spot, 1.5, 3.5));
                        target.0 = None;
                        if random.f32() < theme::RETURN_LINE_CHANCE {
                            announcements.push((entity, Announcement::Return));
                        }
                        return;
                    }
                    let current = target.0.and_then(|chosen| {
                        prey.iter()
                            .find(|(mob, _, _)| *mob == chosen)
                            .map(|(_, spot, _)| *spot)
                    });
                    let spot = match current {
                        Some(spot) => spot,
                        None => {
                            let position = transform.translation;
                            let mut nearest: Vec<(Entity, Vec3, f32)> = prey
                                .iter()
                                .filter(|(_, _, name)| name == hunt_data.prey)
                                .map(|(mob, spot, _)| (*mob, *spot, flat_distance(position, *spot)))
                                .collect();
                            if nearest.is_empty() {
                                if travel(
                                    &world, transform, heading, stride, rest, goal.0, speed.0,
                                    delta_time,
                                ) {
                                    goal.0 = guessed_ground(
                                        &generation,
                                        random_place_spot(random, &places, hunt_data.place, 0.8),
                                    );
                                }
                                return;
                            }
                            nearest.sort_by(|first, second| first.2.total_cmp(&second.2));
                            let choice = random.usize(0..nearest.len().min(TARGET_CHOICES));
                            let (mob, spot, _) = nearest[choice];
                            target.0 = Some(mob);
                            spot
                        }
                    };
                    if flat_distance(transform.translation, spot) > theme::MELEE_REACH * 0.8 {
                        travel(
                            &world, transform, heading, stride, rest, spot, speed.0, delta_time,
                        );
                    } else {
                        heading.0 = heading_toward(transform.translation, spot);
                        place_on_ground(&world, transform, heading, stride);
                    }
                }
                Job::ToGiver => {
                    if travel(
                        &world, transform, heading, stride, rest, goal.0, speed.0, delta_time,
                    ) {
                        task.0 = Job::Resting;
                        rest.0 = between(random, theme::REST_MIN_SECONDS, theme::REST_MAX_SECONDS);
                        announcements.push((entity, Announcement::TurnIn));
                    }
                }
                Job::Offline => {}
            }
        },
    );

    for (entity, announcement) in announcements {
        let mut line = None;
        progress.for_entity(entity, |(kills, level, experience, hunt, name, voice)| {
            let hunt_data = &HUNTS[hunt.0];
            let mut fill = random_fill(random, &[], "");
            fill.monster = hunt_data.prey.to_string();
            fill.place = PLACES[hunt_data.place].name.to_string();
            fill.quest = format!("{} hunting", hunt_data.prey);
            let text = match announcement {
                Announcement::Camp => Some(random_line(random, &CAMP_LINES)),
                Announcement::Return => Some(random_line(random, &RETURN_LINES)),
                Announcement::TurnIn => {
                    kills.0 = 0;
                    grant_experience(level, experience, theme::SIM_QUEST_EXPERIENCE);
                    hunt.0 = hunt_for_level(level.0);
                    if random.f32() < theme::TURN_IN_LINE_CHANCE {
                        Some(random_line(random, &TURN_IN_LINES))
                    } else {
                        None
                    }
                }
            };
            if let Some(text) = text {
                line = Some((
                    name.0.clone(),
                    stylize(&fill_text(&text, &fill), voice.0, random),
                ));
            }
        });
        if let Some((speaker, text)) = line {
            events.send(ChatLine {
                channel: Channel::General,
                speaker: Some(speaker),
                text,
            });
        }
    }
}

type Sleepers<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item mut Task,
        &'item mut Rest,
        &'item mut Target,
        &'item Heading,
        &'item Stride,
        &'item mut LocalTransform,
        &'item Name,
        &'item Voice,
    ),
>;

pub fn population(
    time: Res<Time>,
    world: Res<VoxelWorld>,
    generation: Res<Generation>,
    places: Res<Places>,
    renderer_state: Res<RendererState>,
    mut hero: ResMut<Hero>,
    mut journal: ResMut<Journal>,
    mut classes: Query<(&Class,)>,
    mut people: Sleepers<'_, '_>,
) {
    let journal = &mut *journal;
    journal.clock.population_timer -= time.delta_time;
    if journal.clock.population_timer > 0.0 {
        return;
    }
    journal.clock.population_timer = theme::POPULATION_STEP_SECONDS;

    let hour = renderer_state.day_night.hour;
    let wanted = (theme::POPULATION as f32 * (1.0 - offline_fraction(hour))).round() as usize;
    let board = spot_of(&places, TOWN, GIVERS[0].offset);

    let mut online = Vec::new();
    let mut offline = Vec::new();
    people.for_each_mut(|entity, (task, _, _, _, _, _, _, _)| {
        if task.0 == Job::Offline {
            offline.push(entity);
        } else {
            online.push(entity);
        }
    });
    journal.clock.online = online.len();

    let hero = &mut *hero;
    let hero_name = hero.name.clone();
    let random = &mut hero.random;
    let mut announcement = None;
    if online.len() > wanted && !online.is_empty() {
        let chosen = online[random.usize(0..online.len())];
        people.for_entity(
            chosen,
            |(task, _, target, heading, stride, transform, name, voice)| {
                task.0 = Job::Offline;
                target.0 = None;
                teleport(
                    &world,
                    transform,
                    heading,
                    stride,
                    guessed_ground(&generation, board),
                );
                transform.translation.y -= theme::SINK_DEPTH;
                announcement = Some((name.0.clone(), voice.0, false, chosen));
            },
        );
        journal.clock.online -= 1;
    } else if online.len() < wanted && !offline.is_empty() {
        let chosen = offline[random.usize(0..offline.len())];
        people.for_entity(
            chosen,
            |(task, rest, target, heading, stride, transform, name, voice)| {
                task.0 = Job::Resting;
                rest.0 = between(random, theme::REST_MIN_SECONDS, theme::REST_MAX_SECONDS);
                target.0 = None;
                let spot = around(random, board, 2.0, 5.0);
                teleport(
                    &world,
                    transform,
                    heading,
                    stride,
                    guessed_ground(&generation, spot),
                );
                announcement = Some((name.0.clone(), voice.0, true, chosen));
            },
        );
        journal.clock.online += 1;
    }

    if let Some((name, voice, arrived, chosen)) = announcement
        && random.f32() < theme::LOG_LINE_CHANCE
    {
        let mut class = 0;
        classes.for_entity(chosen, |(known,)| class = known.0);
        let mut fill = random_fill(random, &journal.chat.roster, &hero_name);
        fill.speakers[0] = Speaker { name, voice, class };
        let threads = if arrived { LOGON } else { LOGOFF };
        let thread = &threads[random.usize(0..threads.len())];
        start_thread(&mut journal.chat, random, thread, &fill, 0.5);
    }
}
