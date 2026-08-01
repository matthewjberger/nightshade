use crate::components::{Armor, Grow, HalfExtent, Points};
use crate::data;
use crate::queries;
use crate::resources::{BreakoutAdvancedResources, Screen};
use crate::systems::setup::{brick_material, spawn_box};
use nalgebra_glm::Vec2;
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui::prelude::ui_set_text;

pub fn rebuild(
    mut resources: ResMut<BreakoutAdvancedResources>,
    mut commands: Commands,
    mut standing: Query<(&Points,)>,
) {
    if !resources.round.rebuild_wanted {
        return;
    }

    let mut previous = Vec::new();
    standing.for_each_mut(|entity, _| previous.push(entity));
    for entity in previous {
        commands.despawn(entity);
    }

    let level = resources.round.level;
    let half_extent = Vec2::new(data::BRICK_HALF_WIDTH, data::BRICK_HALF_HEIGHT);
    let mut placed = 0.0_f32;
    for row in 0..data::BRICK_ROWS {
        let points = queries::row_points(row);
        for column in 0..data::BRICK_COLUMNS {
            let Some(armor) = queries::brick_armor(level, column, row) else {
                continue;
            };
            let entity = spawn_box(
                &mut commands,
                queries::brick_center(column, row),
                half_extent,
                &brick_material(row, armor),
            );
            commands.set(entity, HalfExtent(half_extent));
            commands.set(entity, Points(points));
            commands.set(entity, Armor(armor));
            commands.set(
                entity,
                Grow(data::GROW_SECONDS + placed * data::GROW_STAGGER),
            );
            if let Some(transform) = commands.staged_mut::<LocalTransform>(entity) {
                transform.scale = Vec3::zeros();
            }
            commands.restage::<LocalTransform>(entity);
            placed += 1.0;
        }
    }

    resources.round.bricks_standing = queries::level_brick_count(level);
    resources.round.rebuild_wanted = false;
}

pub fn tick(time: Res<Time>, mut resources: ResMut<BreakoutAdvancedResources>) {
    resources.round.level_time += time.delta_time;
}

pub fn watch(
    mut resources: ResMut<BreakoutAdvancedResources>,
    screen: Res<State<Screen>>,
    mut next: ResMut<NextState<Screen>>,
    mut commands: Commands,
) {
    if screen.current != Screen::Playing {
        return;
    }
    let Some(outcome) = resources.round.outcome else {
        return;
    };

    let handles = resources.ui;
    ui_set_text(
        &mut commands,
        handles.over_heading,
        queries::outcome_heading(outcome),
    );
    ui_set_text(
        &mut commands,
        handles.over_summary,
        &queries::outcome_summary(&resources.round),
    );
    resources.round.outcome = None;
    next.pending = Some(Screen::Over);
}
