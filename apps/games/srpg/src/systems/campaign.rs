use crate::commands::audio::play;
use crate::commands::save::write_the_run;
use crate::commands::{
    add_experience, clear_highlights, lift_morale, log, look_at, open_chapter, reset_battle, speak,
    story, wake_the_camp, wound,
};
use crate::commands::{decor, effects, world};
use crate::commands::{edit_fighter, set_fighter, write_fighter};
use crate::components::{MaxHp, Slot, Stride};
use crate::data::Fighter;
use crate::data::{CAMP_SPOTS, CAMP_START, CHAPTERS, COMPANIONS, PROLOGUE, Place, Team, level_of};
use crate::queries::Fighters;
use crate::queries::menu::clicked;
use crate::queries::{hp_share, mustering, open_tile, roster};
use crate::resources::{
    Battle, Battlefield, CameraRig, Camp, Campaign, Highlights, Prompt, Screen, Sheet, Terrain,
    UiHandles,
};
use crate::tags::{Scenery, Upright};
use crate::theme;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_render::data::config::RendererState;
use nightshade_scene::prelude::*;
use nightshade_state::prelude::*;

nightshade::ecs::query_bundle! {
    pub struct CampArrival<'world, 'item> {
        pub shown: Query<'world, (&'item mut Visibility, &'item mut LocalTransform)>,
        pub leaders: Query<'world, (&'item Stride,)>,
        pub units: Query<'world, (&'item MaxHp,)>,
        pub scenery: Query<'world, (&'item LocalTransform,), WithTag<Scenery>>,
    }
}

nightshade::ecs::query_bundle! {
    pub struct MarchRows<'world, 'item> {
        pub avatars: Query<'world, (&'item Slot, &'item mut Visibility)>,
        pub leaders: Query<'world, (&'item Stride, &'item mut Visibility)>,
        pub scenery: Query<'world, (&'item LocalTransform,), WithTag<Scenery>>,
    }
}

pub fn raise_the_squad(sheet: Res<Sheet>, mut campaign: ResMut<Campaign>, mut commands: Commands) {
    campaign.supplies = theme::START_SUPPLIES;
    let chapter = campaign.chapter.min(CHAPTERS.len() - 1);
    let marching = campaign.march;
    open_chapter(&mut campaign, chapter);
    campaign.march = marching;
    let enlisted: Vec<usize> = CHAPTERS[..=chapter]
        .iter()
        .flat_map(|chapter| chapter.joins.iter().copied())
        .collect();
    let mut mustered = 0;
    for (slot, companion) in COMPANIONS.iter().enumerate() {
        let entity = decor::spawn_standing(
            &mut commands,
            &sheet,
            crate::queries::sheet::standing(&sheet, companion.class.tile(Team::Player)),
            Vec3::zeros(),
            Vec2::new(0.85, 0.55),
            Team::Player.shadow_tint(),
            0.0,
            1.0,
        );
        set_fighter(
            &mut commands,
            entity,
            &Fighter {
                name: companion.name,
                title: companion.title,
                class: companion.class,
                taste: companion.taste,
                mood: companion.mood,
                slot,
                facing: companion.facing,
                morale: theme::START_MORALE,
                joined: enlisted.contains(&slot),
                deployed: enlisted.contains(&slot) && mustered < CHAPTERS[chapter].deploy,
                ..Default::default()
            },
        );
        if enlisted.contains(&slot) && mustered < CHAPTERS[chapter].deploy {
            mustered += 1;
        }
        commands.set(entity, Visibility { visible: false });
    }
    let commander = decor::spawn_standing(
        &mut commands,
        &sheet,
        crate::queries::sheet::standing(&sheet, theme::COMMANDER_TILE + theme::BLUE_OFFSET),
        Vec3::zeros(),
        Vec2::new(0.85, 0.55),
        Team::Player.shadow_tint(),
        0.0,
        1.0,
    );
    commands.set(commander, Stride::default());
    commands.set(commander, crate::data::Facing::default());
    commands.set(commander, Visibility { visible: false });
}

pub fn dress_the_title(
    sheet: Res<Sheet>,
    mut terrain_state: ResMut<Terrain>,
    mut rig: ResMut<CameraRig>,
    mut commands: Commands,
) {
    let commands = &mut commands;
    let sheet = *sheet;
    let plan = CHAPTERS[0].plan;
    let mut terrain = std::mem::take(&mut *terrain_state);
    world::build(&mut terrain, Place::Field, plan);
    world::spawn_blocks(commands, &terrain, Place::Field);
    world::spawn_water(commands, &terrain);
    world::spawn_sea(commands);
    decor::build(commands, &mut terrain, &sheet, Place::Field, plan);
    effects::spawn_fireflies(commands, &terrain, &theme::FIREFLY_SPOTS);
    let center = crate::queries::terrain::top(&terrain, terrain.columns / 2, terrain.rows / 2);
    *terrain_state = terrain;
    rig.focus = center;
    look_at(&mut rig, center);
    rig.distance_index = theme::TITLE_DISTANCE;
}

pub fn follow_the_shade(
    mut bodies: Query<(&Visibility,), WithTag<Upright>>,
    shades: Query<(&crate::components::Shade, &mut Visibility)>,
) {
    let mut shown = Vec::new();
    bodies.for_each_mut(|entity, (visibility,)| shown.push((entity, visibility.visible)));
    shades.for_each(|_entity, (shade, visibility)| {
        let Some((_, visible)) = shown.iter().find(|(owner, _)| *owner == shade.owner) else {
            return;
        };
        if visibility.visible != *visible {
            visibility.visible = *visible;
        }
    });
}

pub fn open_the_camp(
    sheet: Res<Sheet>,
    battlefield: Res<Battlefield>,
    mut terrain_state: ResMut<Terrain>,
    mut campaign: ResMut<Campaign>,
    mut renderer_state: ResMut<RendererState>,
    mut camp: ResMut<Camp>,
    mut rig: ResMut<CameraRig>,
    mut highlights: ResMut<Highlights>,
    mut commands: Commands,
    mut avatars: crate::queries::Fighters,
    mut rows: CampArrival<'_, '_>,
) {
    let commands = &mut commands;
    if !campaign.arrival {
        return;
    }
    campaign.arrival = false;
    rows.units
        .for_each_mut(|entity, _| commands.despawn_recursive(entity));

    let sheet = *sheet;
    world::raze(commands, &mut rows.scenery);
    let mut terrain = std::mem::take(&mut *terrain_state);
    let plan = CHAPTERS[campaign.chapter.min(CHAPTERS.len() - 1)].plan;
    world::build(&mut terrain, Place::Camp, plan);
    world::spawn_blocks(commands, &terrain, Place::Camp);
    world::spawn_water(commands, &terrain);
    world::spawn_sea(commands);
    decor::build(commands, &mut terrain, &sheet, Place::Camp, plan);
    effects::spawn_fireflies(commands, &terrain, &theme::FIREFLY_SPOTS);

    let chapter_index = campaign.chapter;
    let chapter = &CHAPTERS[chapter_index.min(CHAPTERS.len() - 1)];
    let mut mustered = 0;
    let mut owners = Vec::new();
    let mut taken = Vec::new();
    let mut arrivals = Vec::new();
    let enlisted: Vec<usize> = CHAPTERS[..=chapter_index.min(CHAPTERS.len() - 1)]
        .iter()
        .flat_map(|chapter| chapter.joins.iter().copied())
        .collect();
    let mut placements: Vec<(Entity, bool, Option<Vec3>)> = Vec::new();
    for (entity, row) in crate::queries::all_fighters(&mut avatars) {
        let mut fighter = row;
        if fighter.fallen || !enlisted.contains(&fighter.slot) {
            fighter.joined = false;
            fighter.deployed = false;
            write_fighter(&mut avatars, entity, &fighter);
            placements.push((entity, false, None));
            continue;
        }
        if !fighter.joined {
            fighter.joined = true;
            arrivals.push(fighter.name);
        }
        wake_the_camp(&mut fighter);
        fighter.deployed = fighter.wounded == 0 && mustered < chapter.deploy;
        if fighter.deployed {
            mustered += 1;
        }
        let spot = CAMP_SPOTS[fighter.slot.min(CAMP_SPOTS.len() - 1)];
        let (column, row_index) = open_tile(&terrain, spot.0, spot.1, &taken);
        taken.push((column, row_index));
        fighter.column = column;
        fighter.row = row_index;
        write_fighter(&mut avatars, entity, &fighter);
        placements.push((
            entity,
            true,
            Some(crate::queries::terrain::stand(&terrain, column, row_index)),
        ));
        owners.push(entity);
    }

    let start_tile = open_tile(&terrain, CAMP_START.0, CAMP_START.1, &taken);
    let start = crate::queries::terrain::stand(&terrain, start_tile.0, start_tile.1);
    rows.leaders.for_each_mut(|entity, _| {
        placements.push((entity, true, Some(start)));
        owners.push(entity);
    });
    for (entity, visible, at) in placements {
        rows.shown.for_entity(entity, |(visibility, transform)| {
            visibility.visible = visible;
            if let Some(at) = at {
                transform.translation = at;
            }
        });
    }
    *terrain_state = terrain;

    let hour = battlefield.fixed_hour.unwrap_or(theme::CAMP_HOUR);
    renderer_state.day_night.hour = hour;
    camp.feet = start;
    camp.prompt = Prompt::Nothing;
    camp.confirm = false;
    camp.cancel = false;
    rig.focus = start;
    look_at(&mut rig, start);
    rig.distance_index = theme::CAMP_DISTANCE;
    rig.pitch_index = theme::CAMP_PITCH;
    clear_highlights(&mut highlights);
    story(
        &mut campaign,
        format!("Chapter {}: {}", chapter_index + 1, chapter.name),
    );
    story(&mut campaign, chapter.intro);
    story(
        &mut campaign,
        format!(
            "{}.  {}",
            chapter.objective.name(),
            chapter.objective.detail()
        ),
    );
    for name in arrivals {
        story(&mut campaign, format!("{name} reports to the camp."));
    }
    if !campaign.briefed {
        campaign.briefed = true;
        let mut scene: Vec<(String, String)> = Vec::new();
        if chapter_index > 0 {
            let last = &CHAPTERS[chapter_index - 1];
            scene.extend(
                last.debrief
                    .iter()
                    .map(|(who, says)| (who.to_string(), says.to_string())),
            );
        }
        scene.extend(
            chapter
                .brief
                .iter()
                .map(|(who, says)| (who.to_string(), says.to_string())),
        );
        speak(&mut campaign, scene);
    }
}

pub fn read_the_prologue(
    handles: Res<UiHandles>,
    frame: Res<nightshade_ui::RetainedUiFrame>,
    camp: Res<Camp>,
    mut campaign: ResMut<Campaign>,
    mut next: ResMut<NextState<Screen>>,
) {
    if clicked(&frame, handles.prologue_skip) {
        next.pending = Some(Screen::Camp);
        return;
    }
    let forward = camp.confirm || clicked(&frame, handles.prologue_next);
    if !forward {
        return;
    }
    let page = campaign.page + 1;
    if page >= PROLOGUE.len() {
        next.pending = Some(Screen::Camp);
        return;
    }
    campaign.page = page;
}

pub fn march_out(
    sheet: Res<Sheet>,
    mut terrain_state: ResMut<Terrain>,
    mut campaign: ResMut<Campaign>,
    mut battlefield: ResMut<Battlefield>,
    mut renderer_state: ResMut<RendererState>,
    mut battle: ResMut<Battle>,
    mut highlights: ResMut<Highlights>,
    mut rig: ResMut<CameraRig>,
    mut next: ResMut<NextState<Screen>>,
    mut commands: Commands,
    mut rows: MarchRows<'_, '_>,
) {
    let commands = &mut commands;
    if !campaign.march {
        return;
    }
    campaign.march = false;
    let mut owners = Vec::new();
    rows.avatars.for_each_mut(|entity, (_, visibility)| {
        visibility.visible = false;
        owners.push(entity);
    });
    rows.leaders.for_each_mut(|entity, (_, visibility)| {
        visibility.visible = false;
        owners.push(entity);
    });

    let sheet = *sheet;
    world::raze(commands, &mut rows.scenery);
    let mut terrain = std::mem::take(&mut *terrain_state);
    let plan = CHAPTERS[campaign.chapter.min(CHAPTERS.len() - 1)].plan;
    world::build(&mut terrain, Place::Field, plan);
    world::spawn_blocks(commands, &terrain, Place::Field);
    world::spawn_water(commands, &terrain);
    world::spawn_sea(commands);
    decor::build(commands, &mut terrain, &sheet, Place::Field, plan);
    effects::spawn_fireflies(commands, &terrain, &theme::FIREFLY_SPOTS);
    let center = crate::queries::terrain::top(&terrain, terrain.columns / 2, terrain.rows / 2);
    *terrain_state = terrain;

    let hour = battlefield.fixed_hour.unwrap_or(theme::BATTLE_HOUR);
    renderer_state.day_night.hour = hour;
    reset_battle(&mut battle);
    clear_highlights(&mut highlights);
    rig.focus = center;
    look_at(&mut rig, center);
    rig.distance_index = 1;
    battlefield.spawn_pending = true;
    next.pending = Some(Screen::Battle);
}

pub fn finish_the_mission(
    mut battle: ResMut<Battle>,
    mut campaign: ResMut<Campaign>,
    mut highlights: ResMut<Highlights>,
    mut next: ResMut<NextState<Screen>>,
    mut commands: Commands,
    mut units: crate::queries::Units,
    mut fighters: Fighters<'_, '_>,
) {
    let commands = &mut commands;
    let outcome = battle.outcome;
    let (Some(outcome), false) = (outcome, campaign.settled) else {
        return;
    };
    campaign.settled = true;
    let roster = roster(&mut units);
    let squad = mustering(&mut fighters);
    let won = outcome == Team::Player;
    let chapter_index = campaign.chapter;
    let chapter = &CHAPTERS[chapter_index.min(CHAPTERS.len() - 1)];

    let mut report = Vec::new();
    let mut survivors: usize = 0;
    let mut lines = Vec::new();
    for (entity, fighter) in &squad {
        if !fighter.deployed {
            continue;
        }
        let living = roster
            .iter()
            .find(|(_, unit)| unit.fighter == Some(*entity))
            .map(|(_, unit)| unit.clone());
        match living {
            Some(unit) => {
                survivors += 1;
                let gained = unit.merit + if won { theme::MERIT_SURVIVE } else { 0 };
                let before = level_of(fighter.experience);
                let after = level_of(fighter.experience + gained);
                let hurt = hp_share(&unit) < theme::WOUND_SHARE;
                lines.push((*entity, true, gained, unit.kills, before, after, hurt));
            }
            None => lines.push((*entity, false, 0, 0, 0, 0, false)),
        }
    }
    for (entity, alive, gained, kills, before, after, hurt) in lines {
        let mut name = "";
        edit_fighter(&mut fighters, entity, |fighter| {
            name = fighter.name;
            fighter.battles += 1;
            if alive {
                add_experience(fighter, gained);
                fighter.kills += kills;
                lift_morale(
                    fighter,
                    if won {
                        theme::MORALE_VICTORY
                    } else {
                        -theme::MORALE_VICTORY
                    },
                );
                if hurt {
                    wound(fighter, theme::WOUND_DAYS);
                }
            } else {
                fighter.fallen = true;
                fighter.deployed = false;
            }
        });
        if !alive {
            report.push(format!("{name} does not come back."));
        } else if hurt {
            report.push(format!("{name} comes back hurt and needs rest."));
        } else if after > before {
            report.push(format!("{name} earns {gained} and reaches level {after}."));
        } else {
            report.push(format!("{name} earns {gained} experience."));
        }
    }

    if survivors == 0 {
        campaign.over = true;
        report.push("The squad is gone. There is nobody left to march.".to_string());
    }
    if won {
        let reward = chapter.reward + survivors as i32 * theme::SUPPLY_PER_SURVIVOR;
        campaign.supplies += reward;
        report.push(format!("The quartermaster books {reward} supplies."));
    }
    clear_highlights(&mut highlights);
    play(
        commands,
        if won {
            theme::SOUND_WIN
        } else {
            theme::SOUND_LOSS
        },
        theme::SOUND_LOUD,
    );
    campaign.report = report;
    story(
        &mut campaign,
        if won {
            chapter.victory.to_string()
        } else {
            "The squad falls back to camp.".to_string()
        },
    );
    log(&mut battle, "The field is quiet.");
    next.pending = Some(Screen::Over);
}

pub fn keep_the_run(mut campaign: ResMut<Campaign>, mut fighters: Fighters<'_, '_>) {
    if !campaign.keep {
        return;
    }
    campaign.keep = false;
    write_the_run(&campaign, &mut fighters);
}
