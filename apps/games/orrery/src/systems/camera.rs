use crate::data;
use crate::queries;
use crate::resources::{Mode, OrreryResources, View};
use nightshade::prelude::*;
use nightshade_camera::components::Camera;
use nightshade_graphics::prelude::*;
use nightshade_platform::prelude::*;

pub fn place(
    time: Res<Time>,
    resources: Res<OrreryResources>,
    mut cameras: Query<(&Camera, &mut LocalTransform)>,
) {
    let camera_entity = resources.scene.camera;
    if camera_entity == Entity::default() {
        return;
    }

    let pose = resources.pose;
    let mode = resources.flight.mode;
    let view = resources.flight.view;
    let look = resources.look;
    let blend = (data::CAMERA_LAG * time.delta_time).clamp(0.0, 1.0);

    cameras.for_entity(camera_entity, |(_, transform)| match mode {
        Mode::Flying => {
            let back = nalgebra_glm::quat_rotate_vec3(&pose.ship_rotation, &Vec3::z());
            let up = nalgebra_glm::quat_rotate_vec3(&pose.ship_rotation, &Vec3::y());
            match view {
                View::Chase => {
                    let goal =
                        pose.ship_position + back * data::CHASE_DISTANCE + up * data::CHASE_HEIGHT;
                    transform.translation += (goal - transform.translation) * blend;
                    let facing =
                        queries::basis_rotation(pose.ship_position - transform.translation, up);
                    transform.rotation = queries::blend_rotation(transform.rotation, facing, blend);
                }
                View::Cockpit => {
                    transform.translation = pose.ship_position - back * data::COCKPIT_FORWARD
                        + up * data::COCKPIT_HEIGHT;
                    transform.rotation = pose.ship_rotation;
                }
            }
        }
        Mode::OnFoot => {
            let up = pose.walker_up;
            let (forward, _) = queries::tangent_frame(up, look.yaw);
            let aimed =
                queries::normalized_or(forward * look.pitch.cos() + up * look.pitch.sin(), forward);
            transform.translation =
                pose.walker_position - aimed * data::FOOT_DISTANCE + up * data::FOOT_HEIGHT;
            transform.rotation = queries::basis_rotation(aimed, up);
        }
    });
}
