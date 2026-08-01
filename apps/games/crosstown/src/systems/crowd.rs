use crate::components::{Lift, Pose, Solid};
use crate::data;
use crate::resources::Ring;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

const DEPTH_SCALE: f32 = data::BODY_RADIUS / data::BODY_DEPTH;

fn apart(mine: Vec3, theirs: Vec3, span: f32) -> Option<Vec2> {
    let across = Vec2::new(mine.x - theirs.x, (mine.z - theirs.z) * DEPTH_SCALE);
    let distance = across.magnitude();
    if distance >= span {
        return None;
    }
    let overlap = span - distance;
    let direction = if distance > 0.0001 {
        across / distance
    } else {
        Vec2::new(1.0, 0.0)
    };
    Some(direction * overlap)
}

pub fn part(time: Res<Time>, ring: Res<Ring>, mut bodies: Bodies) {
    if ring.hit_stop > 0.0 || time.delta_time <= 0.0 {
        return;
    }
    let mut spots: Vec<(Entity, Vec3, f32)> = Vec::new();
    bodies
        .spots
        .for_each_mut(|entity, (transform, solid, lift)| {
            if lift.0 > 0.1 {
                return;
            }
            spots.push((entity, transform.translation, solid.0));
        });
    if spots.len() < 2 {
        return;
    }
    bodies.act.for_each_mut(|entity, (transform, solid, lift)| {
        if lift.0 > 0.1 {
            return;
        }
        let mut push = Vec2::zeros();
        for (other, at, radius) in &spots {
            if *other == entity {
                continue;
            }
            if let Some(step) = apart(transform.translation, *at, solid.0 + radius) {
                push += step;
            }
        }
        if push == Vec2::zeros() {
            return;
        }
        transform.translation.x += push.x * data::CROWD_PUSH;
        transform.translation.z += push.y * data::CROWD_PUSH / DEPTH_SCALE;
        transform.translation.z = transform
            .translation
            .z
            .clamp(-data::LANE_HALF, data::LANE_HALF);
    });
}

fn out_of(mine: Vec3, theirs: Vec3, span: f32, depth: f32) -> Option<Vec2> {
    let across = Vec2::new(mine.x - theirs.x, mine.z - theirs.z);
    let along_span = span - across.x.abs();
    let along_depth = depth - across.y.abs();
    if along_span <= 0.0 || along_depth <= 0.0 {
        return None;
    }
    if along_span < along_depth {
        Some(Vec2::new(
            along_span * if across.x < 0.0 { -1.0 } else { 1.0 },
            0.0,
        ))
    } else {
        Some(Vec2::new(
            0.0,
            along_depth * if across.y < 0.0 { -1.0 } else { 1.0 },
        ))
    }
}

pub fn clear(
    ring: Res<Ring>,
    blocks: Query<(&LocalTransform, &Solid), Without<Pose>>,
    movers: Query<(&mut LocalTransform, &Solid, &Lift), With<Pose>>,
) {
    if ring.hit_stop > 0.0 {
        return;
    }
    let mut walls: Vec<(Vec3, f32)> = Vec::new();
    blocks.for_each(|_entity, (transform, solid)| {
        walls.push((transform.translation, solid.0));
    });
    if walls.is_empty() {
        return;
    }
    movers.for_each(|_entity, (transform, solid, lift)| {
        if lift.0 > 0.5 {
            return;
        }
        for (at, radius) in &walls {
            let span = solid.0 + radius;
            let depth = data::BODY_DEPTH + radius * 0.8;
            let Some(step) = out_of(transform.translation, *at, span, depth) else {
                continue;
            };
            transform.translation.x += step.x;
            transform.translation.z += step.y;
        }
        transform.translation.z = transform
            .translation
            .z
            .clamp(-data::LANE_HALF, data::LANE_HALF);
    });
}

pub type BodySpots<'world, 'item> =
    Query<'world, (&'item LocalTransform, &'item Solid, &'item Lift), With<Pose>>;

pub type BodyAct<'world, 'item> =
    Query<'world, (&'item mut LocalTransform, &'item Solid, &'item Lift), With<Pose>>;

nightshade::ecs::query_bundle! {
    pub struct Bodies<'world, 'item> {
        pub spots: BodySpots<'world, 'item>,
        pub act: BodyAct<'world, 'item>,
    }
}
