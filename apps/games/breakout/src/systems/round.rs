use crate::components::{HalfExtent, Points};
use crate::contract::BreakoutEvent;
use crate::data;
use crate::queries;
use crate::resources::RoundState;
use crate::systems::setup::{brick_material, spawn_box};
use nalgebra_glm::Vec2;
use nightshade::prelude::*;

pub fn rebuild(
    mut round: ResMut<RoundState>,
    mut commands: Commands,
    mut standing: Query<(&Points,)>,
) {
    if !round.rebuild_wanted {
        return;
    }

    let mut previous = Vec::new();
    standing.for_each_mut(|entity, _| previous.push(entity));
    for entity in previous {
        commands.despawn(entity);
    }

    let half_extent = Vec2::new(data::BRICK_HALF_WIDTH, data::BRICK_HALF_HEIGHT);
    for row in 0..data::BRICK_ROWS {
        let points = queries::row_points(row);
        for column in 0..data::BRICK_COLUMNS {
            let entity = spawn_box(
                &mut commands,
                queries::brick_center(column, row),
                half_extent,
                &brick_material(row),
            );
            commands.set(entity, HalfExtent(half_extent));
            commands.set(entity, Points(points));
        }
    }

    round.bricks_standing = data::BRICK_COLUMNS * data::BRICK_ROWS;
    round.rebuild_wanted = false;
}

pub fn settle(events: EventReader<BreakoutEvent>, mut round: ResMut<RoundState>) {
    for event in &events {
        match event {
            BreakoutEvent::BrickBroken { points } => {
                crate::commands::score_points(&mut round, *points);
            }
            BreakoutEvent::BallLost => {
                crate::commands::lose_life(&mut round);
            }
        }
    }
}
