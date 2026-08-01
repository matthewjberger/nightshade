use nightshade::prelude::*;
use nightshade_scene::contract::{GlobalTransform, LocalTransform};

use crate::components::{Distance, DistanceLimits, Focus, Pitch, Yaw};
use crate::resources::{DemoInput, DemoRuntime};

const DRAG_TO_RADIANS: f32 = 0.006;
const WHEEL_TO_DISTANCE: f32 = 0.35;
const PITCH_LIMIT: f32 = 1.45;

pub fn orbit(
    demo_input: Res<DemoInput>,
    runtime: Res<DemoRuntime>,
    cameras: Query<(
        &Focus,
        &mut Yaw,
        &mut Pitch,
        &mut Distance,
        &DistanceLimits,
        &mut LocalTransform,
        &mut GlobalTransform,
    )>,
) {
    let Some((demo, aim)) = demo_input.stages.iter().find(|(_, aim)| {
        aim.hovered && (aim.drag.x != 0.0 || aim.drag.y != 0.0 || aim.wheel != 0.0)
    }) else {
        return;
    };
    let Some(turning) = runtime.cameras.get(demo).copied() else {
        return;
    };
    let drag = aim.drag;
    let wheel = aim.wheel;

    cameras.for_each(
        |entity, (focus, camera_yaw, camera_pitch, distance, limits, transform, global)| {
            if entity != turning {
                return;
            }
            camera_yaw.0 -= drag.x * DRAG_TO_RADIANS;
            camera_pitch.0 =
                (camera_pitch.0 + drag.y * DRAG_TO_RADIANS).clamp(-PITCH_LIMIT, PITCH_LIMIT);
            distance.0 = (distance.0 - wheel * WHEEL_TO_DISTANCE).clamp(limits.0, limits.1);

            let yaw = nalgebra_glm::quat_angle_axis(camera_yaw.0, &Vec3::y());
            let pitch = nalgebra_glm::quat_angle_axis(-camera_pitch.0, &Vec3::x());
            let rotation = yaw * pitch;
            let offset =
                nalgebra_glm::quat_rotate_vec3(&rotation, &Vec3::new(0.0, 0.0, distance.0));

            transform.translation = focus.0 + offset;
            transform.rotation = rotation;
            *global = GlobalTransform(transform.as_matrix());
        },
    );
}
