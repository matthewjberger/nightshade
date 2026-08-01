use crate::components::{
    Bone, Eye, Gait, Gaze, Head, Limb, Owner, Pace, Plate, Regard, Soar, Tilt,
};
use crate::queries::{angle_delta, plate_above, read_the_hero, soaring_pose};
use crate::resources::{Hero, Nameplates};
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::ecs::Entity;
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::{GlobalTransform, LocalTransform, Visibility};
use nightshade_text::prelude::Text;
use std::collections::HashMap;

pub fn swing_the_limbs(
    bodies: Query<(&Gait, &Pace, Option<&Soar>)>,
    mut limbs: Query<(&Owner, &Limb, Option<&Bone>, &mut LocalTransform)>,
) {
    let mut drivers: HashMap<Entity, (f32, f32, f32)> = HashMap::new();
    bodies.for_each(|entity, (gait, pace, soar)| {
        drivers.insert(
            entity,
            (gait.0, pace.0, soar.map(|soar| soar.0).unwrap_or(0.0)),
        );
    });
    if drivers.is_empty() {
        return;
    }
    limbs.for_each_mut(|_, (owner, limb, bone, transform)| {
        let Some(&(gait, pace, soar)) = drivers.get(&owner.0) else {
            return;
        };
        let swing =
            nalgebra_glm::quat_angle_axis((gait + limb.0).sin() * theme::SWING * pace, &Vec3::x());
        transform.rotation = match bone {
            Some(bone) if soar > 0.0 => {
                nalgebra_glm::quat_slerp(&swing, &soaring_pose(bone.0, gait), soar)
            }
            _ => swing,
        };
    });
}

pub fn follow_the_plates(
    active: Res<ActiveCamera>,
    nameplates: Res<Nameplates>,
    mut cameras: Query<(&GlobalTransform,)>,
    owners: Query<(&Plate, &LocalTransform)>,
    mut plates: Query<(&mut LocalTransform, &mut Visibility), With<Text>>,
) {
    let Some(camera) = active.0 else {
        return;
    };
    let mut eye = None;
    cameras.for_entity(camera, |(transform,)| eye = Some(transform.translation()));
    let Some(eye) = eye else {
        return;
    };
    let mut spots = Vec::new();
    owners.for_each(|_entity, (plate, transform)| {
        let near =
            nameplates.shown && (transform.translation - eye).magnitude() < theme::PLATE_RANGE;
        spots.push((plate.0, plate_above(transform.translation), near));
    });
    for (plate, position, near) in spots {
        plates.for_entity(plate, |(transform, visibility)| {
            transform.translation = position;
            if visibility.visible != near {
                visibility.visible = near;
            }
        });
    }
}

pub fn turn_the_heads(
    time: Res<Time>,
    hero: Res<Hero>,
    bodies: Query<(&LocalTransform, Option<&Regard>), With<Gait>>,
    mut heads: Query<(
        &Owner,
        &mut Head,
        &mut Tilt,
        &GlobalTransform,
        &mut LocalTransform,
    )>,
    mut sighted: Query<(&Eye, &Gaze)>,
) {
    let watched = read_the_hero(&hero, &mut sighted, |(eye, gaze)| (eye.0, gaze.0));
    let (watched_at, watched_gaze) = match watched {
        Some((eye, gaze)) => (Some(eye), gaze),
        None => (None, Vec3::zeros()),
    };
    let delta_time = time.delta_time.min(0.05);
    let share = (theme::HEAD_EASE * delta_time).clamp(0.0, 1.0);
    let mut facings: HashMap<Entity, (f32, Option<Vec3>)> = HashMap::new();
    bodies.for_each(|entity, (transform, regard)| {
        let forward =
            nalgebra_glm::quat_rotate_vec3(&transform.rotation, &Vec3::new(0.0, 0.0, 1.0));
        facings.insert(
            entity,
            (forward.x.atan2(forward.z), regard.map(|regard| regard.0)),
        );
    });
    if facings.is_empty() {
        return;
    }
    heads.for_each_mut(|_, (owner, head, tilt, global, transform)| {
        let Some(&(facing, regard)) = facings.get(&owner.0) else {
            return;
        };
        let place = global.translation();
        let (mut wanted_yaw, mut wanted_pitch) = (0.0, 0.0);
        if let Some(at) = regard.or(watched_at) {
            let offset = at - place;
            let flat = Vec3::new(offset.x, 0.0, offset.z);
            let reach = flat.magnitude();
            let looked_at = offset.magnitude() > 0.001
                && watched_gaze.magnitude() > 0.001
                && (-offset).normalize().dot(&watched_gaze.normalize()) > theme::HEAD_GAZE_COSINE;
            let noticed = regard.is_some()
                || offset.magnitude() < theme::HEAD_CLOSE
                || (offset.magnitude() < theme::HEAD_RANGE && looked_at);
            if noticed && (regard.is_some() || reach > theme::HEAD_NEAR) {
                let toward = flat.x.atan2(flat.z);
                wanted_yaw = angle_delta(facing, toward)
                    .clamp(-theme::HEAD_YAW_LIMIT, theme::HEAD_YAW_LIMIT);
                wanted_pitch = (-offset.y.atan2(reach))
                    .clamp(-theme::HEAD_PITCH_LIMIT, theme::HEAD_PITCH_LIMIT);
            }
        }
        head.0 += (wanted_yaw - head.0) * share;
        tilt.0 += (wanted_pitch - tilt.0) * share;
        transform.rotation = nalgebra_glm::quat_angle_axis(head.0, &Vec3::y())
            * nalgebra_glm::quat_angle_axis(tilt.0, &Vec3::x());
    });
}
