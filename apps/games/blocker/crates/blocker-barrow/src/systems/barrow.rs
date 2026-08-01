use crate::components::Kept;
use crate::data::barrow::{
    BARROW_SINK, COUNTING_LINES, FLOOR_SPAN, HAUNT_CELLS, HOARD_CELL, HOARD_LAYER, KEPT_CELL,
    WAITING_CELL,
};
use crate::data::hoard::HOARD;
use crate::queries::{
    capital_name, cell_place, nearest_barrow, seat_center, seats_awake, stand_in_cell,
};
use crate::resources::{Counting, Den};
use crate::theme;
use blocker_beasts::components::{Beast, Lair};
use blocker_blocks::data::blocks::BlockKind;
use blocker_blocks::queries::block_at;
use blocker_blocks::resources::VoxelWorld;
use blocker_crew::commands::stand_a_person;
use blocker_figure::components::Eye;
use blocker_figure::data::characters::ROTTEN_LOOK;
use blocker_figure::queries::read_the_hero;
use blocker_figure::resources::Hero;
use blocker_pocket::commands::{chest_slots, stock_the_slots};
use blocker_screen::data::asks::BeastWanted;
use blocker_screen::data::notice::Narration;
use blocker_world::data::world::places::BARROWS;
use blocker_world::resources::Bench;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::ecs::system_param::EventWriter;
use nightshade::prelude::*;
use nightshade_health::{Health, queries::fallen};
use nightshade_platform::prelude::Time;
use nightshade_random::Random;
use nightshade_random::commands::between;
use nightshade_text::prelude::TextState;

pub fn stock_the_barrows(
    hero: Res<Hero>,
    bench: Res<Bench>,
    world: Res<VoxelWorld>,
    mut den: ResMut<Den>,
    mut random: ResMut<Random>,
    mut text_state: ResMut<TextState>,
    mut asks: EventWriter<BeastWanted>,
    mut commands: Commands,
    mut haunts: Query<(&Beast, &Lair, &Health)>,
    mut sighted: Query<(&Eye,)>,
) {
    let Some(eye) = read_the_hero(&hero, &mut sighted, |(eye,)| eye.0) else {
        return;
    };
    for (index, at) in seats_awake(&bench, eye, |seat| seat.barrow) {
        let center = seat_center(at);
        let mut walkers = 0usize;
        haunts.for_each_mut(|_, (beast, lair, health)| {
            if fallen(health)
                || (lair.0 - center).magnitude() > theme::BARROW_REACH
                || beast.0 != blocker_beasts::data::beasts::ZOMBIE
            {
                return;
            }
            walkers += 1;
        });
        let cell = HAUNT_CELLS[nightshade_random::commands::index(&mut random, HAUNT_CELLS.len())];
        if walkers < theme::BARROW_HAUNTS
            && let Some(place) = stand_in_cell(&world, &bench, at, FLOOR_SPAN, cell)
        {
            asks.send(BeastWanted {
                place,
                heading: between(&mut random, 0.0, std::f32::consts::TAU),
                kind: blocker_beasts::data::beasts::ZOMBIE,
                lair: Vec2::new(place.x, place.z),
            });
        }
        if !den.kept.contains(&index)
            && let Some(place) = stand_in_cell(&world, &bench, at, FLOOR_SPAN, KEPT_CELL)
        {
            den.kept.insert(index);
            stand_the_keeper(&mut text_state, &mut commands, index, place);
        }
    }
}

fn stand_the_keeper(
    text_state: &mut TextState,
    commands: &mut Commands,
    index: usize,
    place: Vec3,
) {
    let body = stand_a_person(
        text_state,
        commands,
        place,
        ROTTEN_LOOK,
        blocker_crew::data::people::KEPT_PERSONA,
        theme::FIGURE_PACE,
        theme::KEEPER_SCALE,
    );
    commands.set(body, Kept(index));
}

pub fn stand_the_waiting(
    hero: Res<Hero>,
    bench: Res<Bench>,
    world: Res<VoxelWorld>,
    mut den: ResMut<Den>,
    mut text_state: ResMut<TextState>,
    mut commands: Commands,
    mut sighted: Query<(&Eye,)>,
) {
    let Some(eye) = read_the_hero(&hero, &mut sighted, |(eye,)| eye.0) else {
        return;
    };
    for (index, at) in seats_awake(&bench, eye, |seat| seat.barrow) {
        if den.waiting.contains(&index) {
            continue;
        }
        let Some(place) = stand_in_cell(&world, &bench, at, FLOOR_SPAN, WAITING_CELL) else {
            continue;
        };
        den.waiting.insert(index);
        stand_a_person(
            &mut text_state,
            &mut commands,
            place,
            theme::WAITING_LOOK,
            blocker_crew::data::people::WAITING_PERSONA,
            theme::FIGURE_PACE,
            blocker_figure::data::rig::FIGURE_SCALE,
        );
    }
}

pub fn hear_the_counting(
    hero: Res<Hero>,
    time: Res<Time>,
    bench: Res<Bench>,
    journal: Res<blocker_quests::resources::Journal>,
    mut counting: ResMut<Counting>,
    mut random: ResMut<Random>,
    mut lines: EventWriter<Narration>,
    mut sighted: Query<(&Eye,)>,
) {
    if journal
        .finished
        .contains(&blocker_quests::data::quests::WIGHT_HUNT)
    {
        return;
    }
    let Some(eye) = read_the_hero(&hero, &mut sighted, |(eye,)| eye.0) else {
        return;
    };
    if nearest_barrow(&bench, eye, theme::BARROW_REACH).is_none() {
        if counting.0 != theme::COUNT_GAP {
            counting.0 = theme::COUNT_GAP;
        }
        return;
    }
    counting.0 -= time.delta_time.min(0.05);
    if counting.0 > 0.0 {
        return;
    }
    counting.0 = between(&mut random, theme::COUNT_GAP, theme::COUNT_GAP * 2.0);
    let line =
        COUNTING_LINES[nightshade_random::commands::index(&mut random, COUNTING_LINES.len())];
    lines.send(Narration(line.to_string()));
}

pub fn name_the_barrow(
    hero: Res<Hero>,
    bench: Res<Bench>,
    mut den: ResMut<Den>,
    mut lines: EventWriter<Narration>,
    mut sighted: Query<(&Eye,)>,
) {
    let Some(eye) = read_the_hero(&hero, &mut sighted, |(eye,)| eye.0) else {
        return;
    };
    let Some(index) = nearest_barrow(&bench, eye, theme::BARROW_INSIDE) else {
        return;
    };
    let barrow = &BARROWS[index];
    if !den.named.insert(index) {
        return;
    }
    lines.send(Narration(format!("You step into {}.", barrow.name)));
    lines.send(Narration(
        "The air down here is cold and it does not move.".to_string(),
    ));
}

pub fn point_to_the_watch(
    hero: Res<Hero>,
    bench: Res<Bench>,
    journal: Res<blocker_quests::resources::Journal>,
    mut den: ResMut<Den>,
    mut lines: EventWriter<Narration>,
    mut sighted: Query<(&Eye,)>,
) {
    let starter = blocker_quests::data::quests::STARTER;
    if journal.active.contains(&starter) || journal.finished.contains(&starter) {
        return;
    }
    let Some(eye) = read_the_hero(&hero, &mut sighted, |(eye,)| eye.0) else {
        return;
    };
    for (index, seat) in bench.seats.iter().enumerate() {
        let offset = Vec2::new(eye.x - seat.city[0] as f32, eye.z - seat.city[1] as f32);
        if offset.magnitude() > theme::WATCH_CALL || !den.pointed.insert(index) {
            continue;
        }
        let giver = &blocker_quests::data::trade::GIVERS[blocker_quests::data::trade::WATCH_GIVER];
        lines.send(Narration(format!(
            "{} is asking after you in {}. Her lodge stands inside the east wall, the long one with the shutters open.",
            giver.name,
            capital_name(index)
        )));
        lines.send(Narration(format!(
            "Word in the market is that the watch lost people in {}, out past the south-east gate.",
            BARROWS[index].name
        )));
    }
}

pub fn fill_the_hoard(
    hero: Res<Hero>,
    bench: Res<Bench>,
    world: Res<VoxelWorld>,
    mut den: ResMut<Den>,
    mut chests: ResMut<blocker_pocket::resources::Chests>,
    mut sighted: Query<(&Eye,)>,
) {
    let Some(eye) = read_the_hero(&hero, &mut sighted, |(eye,)| eye.0) else {
        return;
    };
    for (index, at) in seats_awake(&bench, eye, |seat| seat.barrow) {
        if den.hoarded.contains(&index) {
            continue;
        }
        let place = cell_place(&bench, at, FLOOR_SPAN, HOARD_CELL);
        let block = [
            place.x.floor() as i32,
            place.y.floor() as i32 - BARROW_SINK + HOARD_LAYER,
            place.z.floor() as i32,
        ];
        if block_at(
            &world,
            nalgebra_glm::IVec3::new(block[0], block[1], block[2]),
        ) != BlockKind::Chest
        {
            continue;
        }
        den.hoarded.insert(index);
        let slots = chest_slots(&mut chests, block, blocker_world::data::world::CHEST_SLOTS);
        stock_the_slots(slots, &HOARD);
    }
}
