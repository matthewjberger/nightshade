use crate::commands::{pick_a_spot, rest_a_while};
use crate::components::{Home, Resting, Roaming, Wandering};
use crate::queries::arrived;
use crate::resources::WanderSettings;
use nightshade::prelude::*;
use nightshade_plane::PlayPlane;
use nightshade_plane::commands::move_across;
use nightshade_plane::queries::across;
use nightshade_platform::contract::time::Time;
use nightshade_random::Random;
use nightshade_scene::contract::LocalTransform;

pub fn wander(
    time: Res<Time>,
    settings: Res<WanderSettings>,
    mut random: ResMut<Random>,
    plane: Res<PlayPlane>,
    walkers: Query<(
        &mut LocalTransform,
        &Home,
        &mut Wandering,
        &mut Resting,
        &Roaming,
    )>,
) {
    let delta = time.delta_time;
    walkers.for_each(|_entity, (transform, home, wandering, resting, roaming)| {
        if resting.0 > 0.0 {
            resting.0 = (resting.0 - delta).max(0.0);
            if resting.0 <= 0.0 {
                pick_a_spot(plane.0, wandering, home, &settings, &mut random);
            }
            return;
        }
        let position = transform.translation;
        if arrived(plane.0, position, wandering, &settings) {
            rest_a_while(resting, &settings, &mut random);
            return;
        }
        let reach = across(plane.0, position, wandering.0).max(f32::EPSILON);
        let speed = settings.speed * roaming.0.max(0.0);
        let step = (speed * delta).min(reach) / reach;
        move_across(
            plane.0,
            &mut transform.translation,
            (wandering.0 - position) * step,
        );
    });
}
