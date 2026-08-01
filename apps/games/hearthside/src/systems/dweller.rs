use crate::components::{Cell, Facing, Feeds, FillRate, Needs, Route, Seat, Using};
use crate::data;
use crate::queries;
use crate::resources::{Grid, Models};
use nightshade::prelude::*;
use nightshade_animation::contract::AnimationPlayer;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;

pub fn decide(
    grid: Res<Grid>,
    pieces: Query<(&Feeds, &Cell)>,
    dwellers: Query<(&Needs, &LocalTransform, &mut Route, &mut Using)>,
) {
    let mut servers: Vec<(Entity, data::Need, usize)> = Vec::new();
    pieces.for_each(|entity, (feeds, cell)| {
        servers.push((entity, feeds.0, cell.0));
    });
    if servers.is_empty() {
        return;
    }

    dwellers.for_each(|_entity, (needs, transform, route, using)| {
        if !route.0.is_empty() {
            return;
        }
        if let Some(held) = using.0 {
            let still_wanted = servers.iter().any(|(entity, need, _)| {
                *entity == held && needs.0[data::need_index(*need)] < data::NEED_SATISFIED
            });
            if still_wanted {
                return;
            }
            using.0 = None;
        }

        let Some(here) = queries::point_to_cell(transform.translation) else {
            return;
        };

        for wanted in queries::needs_by_urgency(needs) {
            if needs.0[data::need_index(wanted)] >= data::NEED_SATISFIED {
                continue;
            }
            let mut best: Option<(Entity, usize, Vec<usize>)> = None;
            for (entity, need, cell) in &servers {
                if *need != wanted {
                    continue;
                }
                let Some(found) = queries::find_route(&grid, here, *cell) else {
                    continue;
                };
                if best
                    .as_ref()
                    .is_none_or(|(_, _, shortest)| found.len() < shortest.len())
                {
                    best = Some((*entity, *cell, found));
                }
            }
            if let Some((entity, _, found)) = best {
                using.0 = Some(entity);
                route.0 = found
                    .iter()
                    .skip(1)
                    .take(found.len().saturating_sub(2))
                    .map(|index| queries::cell_center(*index))
                    .collect();
                return;
            }
        }
    });
}

pub fn walk(time: Res<Time>, dwellers: Query<(&mut Route, &mut Facing, &mut LocalTransform)>) {
    let delta_time = time.delta_time;
    dwellers.for_each(|_entity, (route, facing, transform)| {
        let Some(goal) = route.0.first().copied() else {
            return;
        };
        let mut apart = goal - transform.translation;
        apart.y = 0.0;
        let distance = apart.magnitude();
        if distance <= data::DWELLER_REACH {
            route.0.remove(0);
            return;
        }
        let step = (data::DWELLER_SPEED * delta_time).min(distance);
        transform.translation += apart / distance * step;
        facing.0 = apart.x.atan2(apart.z);
    });
}

pub fn use_piece(
    time: Res<Time>,
    rates: Query<(&Feeds, &FillRate, &Cell)>,
    dwellers: Query<(&Route, &Using, &LocalTransform, &mut Needs, &mut Facing)>,
) {
    let delta_time = time.delta_time;
    let mut fed: Vec<(Entity, data::Need, f32, usize)> = Vec::new();
    rates.for_each(|entity, (feeds, rate, cell)| {
        fed.push((entity, feeds.0, rate.0, cell.0));
    });
    if fed.is_empty() {
        return;
    }

    dwellers.for_each(|_entity, (route, using, transform, needs, facing)| {
        if !route.0.is_empty() {
            return;
        }
        let Some(held) = using.0 else {
            return;
        };
        let Some((_, need, rate, cell)) = fed.iter().find(|(entity, _, _, _)| *entity == held)
        else {
            return;
        };
        let slot = data::need_index(*need);
        needs.0[slot] = (needs.0[slot] + rate * delta_time).min(data::NEED_FULL);
        facing.0 = queries::facing_toward(transform.translation, queries::cell_center(*cell));
    });
}

pub fn turn(time: Res<Time>, dwellers: Query<(&Facing, &mut LocalTransform)>) {
    let delta_time = time.delta_time;
    dwellers.for_each(|_entity, (facing, transform)| {
        let wanted = nalgebra_glm::quat_angle_axis(facing.0, &Vec3::y());
        let blend = (data::DWELLER_TURN * delta_time).clamp(0.0, 1.0);
        transform.rotation = nalgebra_glm::quat_slerp(&transform.rotation, &wanted, blend);
    });
}

pub fn animate(
    models: Res<Models>,
    seats: Query<(&Seat,)>,
    dwellers: Query<(&Route, &Using, &mut AnimationPlayer)>,
) {
    let mut sitters: Vec<Entity> = Vec::new();
    seats.for_each(|entity, (seat,)| {
        if seat.0 {
            sitters.push(entity);
        }
    });

    dwellers.for_each(|_entity, (route, using, player)| {
        let wanted = if !route.0.is_empty() {
            models.clip_walk
        } else if using.0.is_some_and(|held| sitters.contains(&held)) {
            models.clip_sit
        } else {
            models.clip_idle
        };
        let Some(index) = wanted else {
            return;
        };
        if player.current_clip == Some(index) {
            return;
        }
        player.looping = true;
        player.blend_to(index, data::CLIP_BLEND);
    });
}
