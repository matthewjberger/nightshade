use crate::theme;
use blocker_blocks::queries::step_holds;
use blocker_blocks::resources::VoxelWorld;
use blocker_figure::components::Home;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::ecs::Entity;
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::LocalTransform;

pub fn hold_the_crew_apart(
    time: Res<Time>,
    world: Res<VoxelWorld>,
    mut crowd: Query<(&mut LocalTransform,), With<Home>>,
) {
    let delta_time = time.delta_time.min(0.05);
    let mut places: Vec<(Entity, Vec3)> = Vec::new();
    crowd.for_each_mut(|entity, (transform,)| places.push((entity, transform.translation)));
    if places.len() < 2 {
        return;
    }
    let girth = theme::CREW_GIRTH;
    let stature = theme::CREW_STATURE;
    let reach = girth * 2.0;
    let mut pushes: Vec<Vec2> = vec![Vec2::zeros(); places.len()];
    for first in 0..places.len() {
        for second in (first + 1)..places.len() {
            let here = places[first].1;
            let there = places[second].1;
            if (here.y - there.y).abs() >= stature {
                continue;
            }
            let between = Vec2::new(there.x - here.x, there.z - here.z);
            let distance = between.magnitude();
            if distance >= reach {
                continue;
            }
            let away = if distance > 1.0e-4 {
                between / distance
            } else {
                let turn = (first * 7 + second) as f32;
                Vec2::new(turn.cos(), turn.sin())
            };
            let overlap = (reach - distance) * 0.5;
            pushes[first] -= away * overlap;
            pushes[second] += away * overlap;
        }
    }
    let share = (theme::CREW_NUDGE * delta_time).clamp(0.0, 1.0);
    let moves: Vec<(Entity, Vec2)> = places
        .iter()
        .zip(pushes.iter())
        .filter(|(_, push)| push.magnitude() > 1.0e-4)
        .map(|((entity, place), push)| {
            let ground = Vec2::new(place.x, place.z);
            (*entity, ground + push * share)
        })
        .collect();
    for (entity, wanted) in moves {
        crowd.for_entity(entity, |(transform,)| {
            let stand = transform.translation.y.floor() as i32;
            let ground = Vec2::new(transform.translation.x, transform.translation.z);
            if step_holds(&world, ground, wanted, stand) {
                transform.translation.x = wanted.x;
                transform.translation.z = wanted.y;
            }
        });
    }
}
