use crate::components::Hook;
use crate::data::{EYE_OFFSET, HAND_ACROSS, HAND_DOWN, ROPE_THICKNESS};
use crate::resources::TetherResources;
use nightshade::prelude::*;
use nightshade_camera::queries::camera_look::camera_look_rotation;
use nightshade_scene::prelude::{LocalTransform, Visibility};

pub fn draw_the_line(
    resources: Res<TetherResources>,
    mut commands: Commands,
    mut parts: Query<(&LocalTransform, Option<&Hook>, Option<&Visibility>)>,
) {
    let Some(rope) = resources.scene.rope else {
        return;
    };
    let Some(runner) = resources.scene.runner else {
        return;
    };
    let Some(camera) = resources.scene.camera else {
        return;
    };

    let mut body = None;
    let mut anchor = None;
    parts.for_entity(runner, |(transform, runner_anchor, _)| {
        body = Some(transform.translation);
        anchor = runner_anchor.map(|anchor| anchor.0);
    });

    let mut drawn = false;
    parts.for_entity(rope, |(_, _, visibility)| {
        drawn = visibility
            .map(|visibility| visibility.visible)
            .unwrap_or(false);
    });

    let shown = anchor.is_some();
    if drawn != shown {
        commands.set(rope, Visibility { visible: shown });
    }

    let (Some(anchor), Some(body)) = (anchor, body) else {
        return;
    };

    let mut hand = None;
    parts.for_entity(camera, |(transform, _, _)| {
        let right = nalgebra_glm::quat_rotate_vec3(&transform.rotation, &Vec3::x());
        let up = nalgebra_glm::quat_rotate_vec3(&transform.rotation, &Vec3::y());
        hand = Some(body + Vec3::new(0.0, EYE_OFFSET, 0.0) + right * HAND_ACROSS - up * HAND_DOWN);
    });
    let Some(hand) = hand else {
        return;
    };

    let span = anchor - hand;
    let length = span.norm();
    if length <= f32::EPSILON {
        return;
    }

    let along = camera_look_rotation(hand, anchor, 0.0)
        * nalgebra_glm::quat_angle_axis(-std::f32::consts::FRAC_PI_2, &Vec3::x());

    commands.set(
        rope,
        LocalTransform {
            translation: hand + span * 0.5,
            rotation: along,
            scale: Vec3::new(ROPE_THICKNESS, length, ROPE_THICKNESS),
        },
    );
}
