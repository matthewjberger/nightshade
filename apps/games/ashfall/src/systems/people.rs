use crate::components::Alive;
use crate::data::rules::PERSON_FALLEN_HEIGHT;
use crate::data::theme::{PERSON_REACH, PERSON_TURN};
use crate::resources::AshfallResources;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn attend(
    time: Res<Time>,
    resources: Res<AshfallResources>,
    people: Query<(&Alive, &mut LocalTransform)>,
) {
    if !resources.screen.is_in_town() {
        return;
    }
    let here = resources.player.at;
    let share = (time.delta_time * PERSON_TURN).min(1.0);
    people.for_each(|_entity, (alive, transform)| {
        if !alive.0 {
            transform.rotation =
                nalgebra_glm::quat_angle_axis(-std::f32::consts::FRAC_PI_2, &Vec3::x());
            transform.translation.y = PERSON_FALLEN_HEIGHT;
            return;
        }
        let toward = Vec3::new(
            here.x - transform.translation.x,
            0.0,
            here.z - transform.translation.z,
        );
        if toward.magnitude() > PERSON_REACH * 1.6 || toward.magnitude() < 0.05 {
            return;
        }
        let wanted = nalgebra_glm::quat_angle_axis(toward.x.atan2(toward.z), &Vec3::y());
        transform.rotation = approach(transform.rotation, wanted, share);
    });
}

fn approach(from: Quat, to: Quat, share: f32) -> Quat {
    let to = if nalgebra_glm::quat_dot(&from, &to) < 0.0 {
        -to
    } else {
        to
    };
    nalgebra_glm::quat_normalize(&(from + (to - from) * share))
}
