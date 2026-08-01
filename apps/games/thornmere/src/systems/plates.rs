use crate::commands::audio::cue;
use crate::commands::banner;
use crate::commands::chat::{random_fill, start_thread, system_line};
use crate::commands::effects::burst;
use crate::components::{
    Class, Cooldowns, Giver, GiverMark, Guild, Health, Level, Lift, Plate, PlateLevel, Rank,
    Target, Voice, Worth,
};
use crate::data::chatter::{DING, PLAYER_DING};
use crate::data::{
    DING_THREAD_CHANCE, ELITE, ELITE_PLATE_COLOR, MONSTER_PLATE_COLOR, Objective, PLATE_RANGE,
    QUESTS, SINK_DEPTH, SOUND_LEVEL, SOUND_LOUD,
};
use crate::queries::{giver_plate, is_alive, person_plate, player_plate};
use crate::resources::{Banners, BurstKind, Bursts, Cues, Speaker, ThornmereResources};
use crate::systems::quests::turn_in_giver;
use nightshade::prelude::*;
use nightshade_render::resources::outline::Outline;
use nightshade_scene::prelude::{LocalTransform, Name, Visibility};
use nightshade_text::prelude::{Text, TextState};

const SELECTED_PLATE_COLOR: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);
const GOLD: Vec4 = Vec4::new(1.0, 0.85, 0.30, 1.0);

pub fn follow(
    owners: Query<(&Plate, &Lift, &LocalTransform)>,
    players: Query<(&LocalTransform,), With<Cooldowns>>,
    mut plates: Query<(&mut LocalTransform, &mut Visibility), With<Text>>,
) {
    let mut eye = None;
    players.for_each(|_entity, (transform,)| eye = Some(transform.translation));
    let mut spots = Vec::new();
    owners.for_each(|_entity, (plate, lift, transform)| {
        let near =
            eye.is_none_or(|eye: Vec3| (transform.translation - eye).magnitude() < PLATE_RANGE);
        spots.push((
            plate.0,
            transform.translation + Vec3::new(0.0, lift.0, 0.0),
            near,
        ));
    });
    for (plate, position, near) in spots {
        plates.for_entity(plate, |(transform, visibility)| {
            transform.translation = position;
            if visibility.visible != near {
                visibility.visible = near;
            }
        });
    }
}

fn relabel_plate(
    text_state: &mut TextState,
    texts: &mut Query<(&mut Text,)>,
    plate: Entity,
    label: String,
) {
    texts.for_entity(plate, |(text,)| {
        text_state.cache.set_text(text.text_index, label.clone());
        text.dirty = true;
    });
}

type Owners<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item Plate,
        &'item Level,
        &'item PlateLevel,
        &'item Name,
        Option<&'item Guild>,
        Option<&'item Voice>,
        Option<&'item Class>,
        &'item LocalTransform,
    ),
>;

pub fn relabel(
    mut resources: ResMut<ThornmereResources>,
    mut text_state: ResMut<TextState>,
    mut cues: ResMut<Cues>,
    mut bursts: ResMut<Bursts>,
    mut banners: ResMut<Banners>,
    owners: Owners<'_, '_>,
    mut shown: Query<(&mut PlateLevel,)>,
    mut texts: Query<(&mut Text,)>,
) {
    let resources = &mut *resources;
    let mut changes = Vec::new();
    owners.for_each(
        |entity, (plate, level, shown_level, name, guild, voice, class, transform)| {
            if level.0 != shown_level.0 {
                changes.push((
                    entity,
                    plate.0,
                    level.0,
                    name.0.clone(),
                    guild.map(|guild| guild.0.clone()),
                    Speaker {
                        name: name.0.clone(),
                        voice: voice.map(|voice| voice.0).unwrap_or(1),
                        class: class.map(|class| class.0).unwrap_or(0),
                    },
                    transform.translation,
                ));
            }
        },
    );

    for (entity, plate, level, name, guild, speaker, position) in changes {
        shown.for_entity(entity, |(shown_level,)| shown_level.0 = level);
        match guild {
            Some(guild) => {
                relabel_plate(
                    &mut text_state,
                    &mut texts,
                    plate,
                    person_plate(&name, &guild, level),
                );
                burst(&mut bursts, BurstKind::LevelUp, position);
                if resources.random.f32() < DING_THREAD_CHANCE {
                    let mut fill = random_fill(
                        &mut resources.random,
                        &resources.chat.roster,
                        &resources.player_name,
                    );
                    fill.speakers[0] = speaker;
                    fill.level = level.to_string();
                    let thread = &DING[resources.random.usize(0..DING.len())];
                    start_thread(
                        &mut resources.chat,
                        &mut resources.random,
                        thread,
                        &fill,
                        0.0,
                    );
                }
            }
            None => {
                relabel_plate(
                    &mut text_state,
                    &mut texts,
                    plate,
                    player_plate(&name, level),
                );
                system_line(
                    &mut resources.chat,
                    format!("You have reached level {level}!"),
                );
                burst(&mut bursts, BurstKind::LevelUp, position);
                cue(&mut cues, SOUND_LEVEL, SOUND_LOUD);
                banner(
                    &mut banners,
                    format!("Level {level}"),
                    "Your strength grows".to_string(),
                    GOLD,
                );
                let mut fill = random_fill(
                    &mut resources.random,
                    &resources.chat.roster,
                    &resources.player_name,
                );
                fill.level = level.to_string();
                let thread = &PLAYER_DING[resources.random.usize(0..PLAYER_DING.len())];
                start_thread(
                    &mut resources.chat,
                    &mut resources.random,
                    thread,
                    &fill,
                    1.5,
                );
            }
        }
    }
}

pub fn marks(
    resources: Res<ThornmereResources>,
    mut text_state: ResMut<TextState>,
    givers: Query<(&Giver, &Plate, &GiverMark)>,
    mut stamps: Query<(&mut GiverMark,)>,
    mut texts: Query<(&mut Text,)>,
) {
    let quests = &resources.quests;
    let mut changes = Vec::new();
    givers.for_each(|entity, (giver, plate, mark)| {
        let complete_here = quests.active.iter().any(|state| {
            state.complete && turn_in_giver(state.quest) == giver.0
                || matches!(QUESTS[state.quest].objective, Objective::Deliver { giver: target } if target == giver.0)
                    && !state.complete
        });
        let available_here = QUESTS.iter().enumerate().any(|(index, quest)| {
            quest.giver == giver.0
                && !quests.finished[index]
                && !quests.active.iter().any(|state| state.quest == index)
        });
        let wanted = if complete_here {
            2
        } else if available_here {
            1
        } else {
            0
        };
        if wanted != mark.0 {
            changes.push((entity, plate.0, giver.0, wanted));
        }
    });
    for (entity, plate, giver, wanted) in changes {
        stamps.for_entity(entity, |(mark,)| mark.0 = wanted);
        relabel_plate(
            &mut text_state,
            &mut texts,
            plate,
            giver_plate(giver, wanted),
        );
    }
}

pub fn highlight(
    mut resources: ResMut<ThornmereResources>,
    mut outline: ResMut<Outline>,
    players: Query<(&Target,), With<Cooldowns>>,
    mut marked: Query<(&Plate, &Rank, &Health, &LocalTransform), With<Worth>>,
    mut texts: Query<(&mut Text,)>,
    mut plates: Query<(&mut LocalTransform,), With<Text>>,
    mut rings: Query<(&mut LocalTransform,), Without<Text>>,
) {
    let resources = &mut *resources;
    let mut target = None;
    players.for_each(|_entity, (chosen,)| target = chosen.0);

    let previous = resources.clock.shown_target;
    if previous != target {
        if let Some(previous) = previous {
            let mut plate = None;
            marked.for_entity(previous, |(marked, rank, _, _)| {
                plate = Some((marked.0, rank.0))
            });
            if let Some((plate, rank)) = plate {
                texts.for_entity(plate, |(text,)| {
                    text.properties.color = if rank >= ELITE {
                        ELITE_PLATE_COLOR
                    } else {
                        MONSTER_PLATE_COLOR
                    };
                    text.dirty = true;
                });
                plates.for_entity(plate, |(transform,)| {
                    transform.scale = Vec3::new(1.0, 1.0, 1.0);
                });
            }
        }
        if let Some(current) = target {
            let mut plate = None;
            marked.for_entity(current, |(marked, _, _, _)| plate = Some(marked.0));
            if let Some(plate) = plate {
                texts.for_entity(plate, |(text,)| {
                    text.properties.color = SELECTED_PLATE_COLOR;
                    text.dirty = true;
                });
                plates.for_entity(plate, |(transform,)| {
                    transform.scale = Vec3::new(1.25, 1.25, 1.25);
                });
            }
        }
        resources.clock.shown_target = target;
        outline.active_entity = target;
        let mut elite = false;
        if let Some(current) = target {
            marked.for_entity(current, |(_, rank, _, _)| elite = rank.0 >= ELITE);
        }
        outline.color = if elite {
            [1.0, 0.75, 0.25, 1.0]
        } else {
            [1.0, 0.35, 0.25, 1.0]
        };
    }
    if let Some(current) = target {
        let mut alive = true;
        marked.for_entity(current, |(_, _, health, _)| alive = is_alive(health.0));
        if !alive && outline.active_entity.is_some() {
            outline.active_entity = None;
        } else if alive && outline.active_entity.is_none() {
            outline.active_entity = target;
        }
    }

    let Some(ring) = resources.ring else {
        return;
    };
    let mut spot = None;
    if let Some(current) = target {
        marked.for_entity(current, |(_, _, health, transform)| {
            if is_alive(health.0) {
                spot = Some(transform.translation);
            }
        });
    }
    rings.for_entity(ring, |(transform,)| {
        transform.translation = match spot {
            Some(spot) => spot + Vec3::new(0.0, 0.06, 0.0),
            None => Vec3::new(0.0, -SINK_DEPTH, 0.0),
        };
    });
}
