use crate::commands::{Standing, float_the_player, set_the_player_down};
use crate::resources::Resume;
use crate::theme;
use blocker_blocks::queries::{column_of, column_settled};
use blocker_blocks::resources::VoxelWorld;
use blocker_figure::resources::Hero;
use blocker_world::queries::city_gate;
use blocker_world::resources::{Arrival, Bench, Draft, Generation};
use nightshade::prelude::*;

pub fn place_player(
    draft: Res<Draft>,
    resume: Res<Resume>,
    hero: Res<Hero>,
    mut generation: ResMut<Generation>,
    mut bench: ResMut<Bench>,
    mut bodies: Standing<'_, '_>,
) {
    let Some(body) = hero.0 else {
        return;
    };
    if resume.0.is_none() {
        generation.seed = draft.seed;
    }
    bench.reseed(generation.seed);
    let gate = city_gate(&bench, 0);
    float_the_player(&mut bodies, body, &mut generation, resume.0, gate);
}

pub fn stand_the_player(
    world: Res<VoxelWorld>,
    resume: Res<Resume>,
    hero: Res<Hero>,
    mut arrival: ResMut<Arrival>,
    mut generation: ResMut<Generation>,
    mut bodies: Standing<'_, '_>,
) {
    let Some(body) = hero.0 else {
        return;
    };
    let Some(spawn) = generation.spawn else {
        return;
    };
    if arrival.stood {
        return;
    }
    let center = column_of(spawn.0, spawn.1);
    let radius = theme::ARRIVAL_RADIUS;
    let mut wanted = 0;
    let mut landed = 0;
    for offset_z in -radius..=radius {
        for offset_x in -radius..=radius {
            if offset_x * offset_x + offset_z * offset_z > radius * radius {
                continue;
            }
            wanted += 1;
            if column_settled(&world, [center[0] + offset_x, center[1] + offset_z]) {
                landed += 1;
            }
        }
    }
    arrival.wanted = wanted;
    arrival.landed = landed;
    if landed >= wanted {
        set_the_player_down(&mut bodies, body, &world, &mut generation, resume.0);
        arrival.stood = true;
    }
}
