use crate::components::{Lift, Spot};
use crate::data;
use crate::queries::cylinder;
use crate::queries::town::lift_at;
use crate::resources::Town;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn stand_on_the_world(
    time: Res<Time>,
    town: Res<Town>,
    walkers: Query<(&Spot, &mut Lift, &mut LocalTransform)>,
) {
    let share = (data::STEP_SMOOTHING * time.delta_time).clamp(0.0, 1.0);
    walkers.for_each(|_entity, (spot, lift, transform)| {
        let ground = lift_at(&town, spot.0);
        lift.0 += (ground - lift.0) * share;
        transform.translation = cylinder::place(spot.0, lift.0);
        transform.rotation = cylinder::tilt(spot.0.y);
    });
}
