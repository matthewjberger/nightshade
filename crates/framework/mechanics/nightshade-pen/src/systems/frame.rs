use crate::components::Pen;
use crate::queries::held;
use nightshade::prelude::*;
use nightshade_plane::PlayPlane;
use nightshade_plane::commands::move_across;
use nightshade_plane::queries::{flatten, raise};
use nightshade_scene::contract::LocalTransform;

pub fn keep_them_inside(plane: Res<PlayPlane>, bodies: Query<(&Pen, &mut LocalTransform)>) {
    bodies.for_each(|_entity, (pen, transform)| {
        let flat = flatten(plane.0, transform.translation);
        let step = raise(plane.0, held(pen.0, flat) - flat);
        move_across(plane.0, &mut transform.translation, step);
    });
}
