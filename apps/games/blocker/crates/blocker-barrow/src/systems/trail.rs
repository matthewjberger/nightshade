use crate::data::barrow::{
    CAMP_HAUNTS, CAMP_SPAN, CAMP_WIGHT, FLOOR_SPAN, MOUTH_CELL, WATCHMAN_CELL, WIGHT_CELL,
};
use crate::queries::{cell_place, nearest_seat, seat_center, seats_awake, stand_in_cell};
use crate::resources::Den;
use crate::theme;
use blocker_beasts::components::{Beast, Lair};
use blocker_blocks::resources::VoxelWorld;
use blocker_crew::commands::stand_a_person;
use blocker_figure::components::Eye;
use blocker_figure::queries::read_the_hero;
use blocker_figure::resources::Hero;
use blocker_quests::data::quests::{QUESTS, TRAIL_EAST, WIGHT_HUNT};
use blocker_quests::data::spots::{BARROW_DEEP, BARROW_MOUTH, PLACES, WATCH_LODGE, WOODS_CAMP};
use blocker_quests::resources::{Journal, Waypoint};
use blocker_screen::data::asks::BeastWanted;
use blocker_world::resources::{Bench, Generation};
use nalgebra_glm::{Vec2, Vec3};
use nightshade::ecs::system_param::EventWriter;
use nightshade::prelude::*;
use nightshade_health::{Health, queries::fallen};
use nightshade_random::Random;
use nightshade_random::commands::between;
use nightshade_text::prelude::TextState;

fn spot_place(bench: &Bench, generation: &Generation, seat: usize, place: usize) -> Option<Vec3> {
    let at = bench.seats.get(seat)?;
    match place {
        WATCH_LODGE => {
            let door = generation.guides.get(&guide_key(generation, at.city))?;
            Some(Vec3::new(
                door[0] as f32 + 0.5,
                door[1] as f32,
                door[2] as f32 + 0.5,
            ))
        }
        BARROW_MOUTH => Some(cell_place(bench, at.barrow, FLOOR_SPAN, MOUTH_CELL)),
        BARROW_DEEP => Some(cell_place(bench, at.barrow, FLOOR_SPAN, WIGHT_CELL)),
        WOODS_CAMP => {
            let center = seat_center(at.camp);
            Some(Vec3::new(
                center.x,
                blocker_world::queries::barrow_floor(bench, at.camp) as f32,
                center.y,
            ))
        }
        _ => None,
    }
}

fn guide_key(generation: &Generation, at: [i32; 2]) -> [i32; 2] {
    generation
        .guides
        .keys()
        .copied()
        .min_by_key(|village| (village[0] - at[0]).abs() + (village[1] - at[1]).abs())
        .unwrap_or(at)
}

fn wanted_place(journal: &Journal) -> Option<usize> {
    journal
        .active
        .iter()
        .find_map(|quest| QUESTS.get(*quest).and_then(|quest| quest.place))
}

pub fn aim_the_waypoint(
    hero: Res<Hero>,
    bench: Res<Bench>,
    generation: Res<Generation>,
    journal: Res<Journal>,
    mut waypoint: ResMut<Waypoint>,
    mut sighted: Query<(&Eye,)>,
) {
    let Some(eye) = read_the_hero(&hero, &mut sighted, |(eye,)| eye.0) else {
        return;
    };
    let seat = nearest_seat(&bench, eye);
    let Some(place) = wanted_place(&journal) else {
        waypoint.at = None;
        waypoint.name.clear();
        return;
    };
    waypoint.at = spot_place(&bench, &generation, seat, place);
    if waypoint.at.is_some() {
        waypoint.name = PLACES[place].name.to_string();
    }
}

pub fn walk_the_trail(
    hero: Res<Hero>,
    bench: Res<Bench>,
    generation: Res<Generation>,
    mut journal: ResMut<Journal>,
    mut sighted: Query<(&Eye,)>,
) {
    let Some(eye) = read_the_hero(&hero, &mut sighted, |(eye,)| eye.0) else {
        return;
    };
    let seat = nearest_seat(&bench, eye);
    for (index, place) in PLACES.iter().enumerate() {
        if journal.seen.contains(&index) {
            continue;
        }
        let Some(at) = spot_place(&bench, &generation, seat, index) else {
            continue;
        };
        if (at - eye).magnitude() < place.reach {
            journal.record_seen(index);
        }
    }
}

pub fn stock_the_camps(
    hero: Res<Hero>,
    bench: Res<Bench>,
    world: Res<VoxelWorld>,
    journal: Res<Journal>,
    mut random: ResMut<Random>,
    mut asks: EventWriter<BeastWanted>,
    mut haunts: Query<(&Beast, &Lair, &Health)>,
    mut sighted: Query<(&Eye,)>,
) {
    if !journal.active.contains(&TRAIL_EAST) && !journal.active.contains(&WIGHT_HUNT) {
        return;
    }
    let Some(eye) = read_the_hero(&hero, &mut sighted, |(eye,)| eye.0) else {
        return;
    };
    for (_, at) in seats_awake(&bench, eye, |seat| seat.camp) {
        let center = seat_center(at);
        let mut walkers = 0usize;
        let mut wight = false;
        haunts.for_each_mut(|_, (beast, lair, health)| {
            if fallen(health) || (lair.0 - center).magnitude() > theme::BARROW_REACH {
                return;
            }
            if beast.0 == blocker_beasts::data::beasts::WIGHT {
                wight = true;
            } else if beast.0 == blocker_beasts::data::beasts::ZOMBIE {
                walkers += 1;
            }
        });
        if !wight
            && journal.finished.contains(&TRAIL_EAST)
            && let Some(place) = stand_in_cell(&world, &bench, at, CAMP_SPAN, CAMP_WIGHT)
        {
            asks.send(BeastWanted {
                place,
                heading: between(&mut random, 0.0, std::f32::consts::TAU),
                kind: blocker_beasts::data::beasts::WIGHT,
                lair: Vec2::new(place.x, place.z),
            });
        }
        let cell = CAMP_HAUNTS[nightshade_random::commands::index(&mut random, CAMP_HAUNTS.len())];
        if walkers < CAMP_HAUNTS.len()
            && let Some(place) = stand_in_cell(&world, &bench, at, CAMP_SPAN, cell)
        {
            asks.send(BeastWanted {
                place,
                heading: between(&mut random, 0.0, std::f32::consts::TAU),
                kind: blocker_beasts::data::beasts::ZOMBIE,
                lair: Vec2::new(place.x, place.z),
            });
        }
    }
}

pub fn free_the_watchman(
    hero: Res<Hero>,
    bench: Res<Bench>,
    world: Res<VoxelWorld>,
    journal: Res<Journal>,
    mut den: ResMut<Den>,
    mut text_state: ResMut<TextState>,
    mut commands: Commands,
    mut sighted: Query<(&Eye,)>,
) {
    if journal.slain_count(blocker_beasts::data::beasts::WIGHT) == 0 {
        return;
    }
    let Some(eye) = read_the_hero(&hero, &mut sighted, |(eye,)| eye.0) else {
        return;
    };
    for (index, at) in seats_awake(&bench, eye, |seat| seat.camp) {
        if den.freed.contains(&index) {
            continue;
        }
        let Some(place) = stand_in_cell(&world, &bench, at, CAMP_SPAN, WATCHMAN_CELL) else {
            continue;
        };
        den.freed.insert(index);
        stand_a_person(
            &mut text_state,
            &mut commands,
            place,
            theme::WATCHMAN_LOOK,
            blocker_crew::data::people::WATCHMAN_PERSONA,
            theme::FIGURE_PACE,
            blocker_figure::data::rig::FIGURE_SCALE,
        );
    }
}
