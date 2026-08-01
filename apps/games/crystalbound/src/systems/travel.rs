use crate::commands::{show_toast, travel};
use crate::components::Facing;
use crate::queries::{arrival, portal_at};
use crate::resources::{Field, Toast};
use nalgebra_glm::Vec2;
use nightshade::prelude::*;
use nightshade_scene::prelude::*;

pub fn cross_the_threshold(
    mut field: ResMut<Field>,
    mut toast: ResMut<Toast>,
    walkers: Query<(&LocalTransform,), With<Facing>>,
) {
    if field.pending.is_some() {
        return;
    }
    let mut here = None;
    walkers.for_each(|_entity, (transform,)| {
        here = Some(Vec2::new(transform.translation.x, transform.translation.y));
    });
    let Some(here) = here else {
        return;
    };
    let Some(to) = portal_at(&field, here) else {
        field.armed = true;
        return;
    };
    if !field.armed {
        return;
    }
    let landing = arrival(field.place, to);
    travel(&mut field, to, landing);
    show_toast(&mut toast, to.name());
}
