use crate::queries::is_solid_at;
use crate::resources::{Framing, Player, VoxelWorld};
use crate::theme;
use nalgebra_glm::{IVec3, Vec3, quat_angle_axis, quat_slerp};
use nightshade::prelude::*;
use nightshade_camera::prelude::Camera;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::LocalTransform;
use nightshade_sprite::prelude::Sprite;

pub fn stage_the_talk(
    time: Res<Time>,
    player: Res<Player>,
    world: Res<VoxelWorld>,
    mut framing: ResMut<Framing>,
    mut people: Query<(&mut LocalTransform, &mut Sprite)>,
    mut cameras: Query<(&Camera, &mut LocalTransform)>,
) {
    let Some(subject) = framing.subject else {
        return;
    };
    let delta_time = time.delta_time.min(0.05);
    let wanted = if framing.talking { 1.0 } else { 0.0 };
    let step = theme::TALK_BLEND_RATE * delta_time;
    framing.blend += (wanted - framing.blend).clamp(-step, step);
    framing.phase += delta_time;

    let head = framing.base + Vec3::new(0.0, theme::TALK_HEAD, 0.0);
    let mut flat = head - player.eye;
    flat.y = 0.0;
    let flat = if flat.norm_squared() > 1.0e-4 {
        flat.normalize()
    } else {
        Vec3::new(0.0, 0.0, 1.0)
    };
    let aside = Vec3::new(flat.z, 0.0, -flat.x);
    let focus = player.eye + (head - player.eye) * theme::TALK_FOCUS;
    let mut seat = focus - flat * theme::TALK_BACK
        + aside * theme::TALK_SIDE
        + Vec3::new(0.0, theme::TALK_LIFT, 0.0);
    for _ in 0..6 {
        if !is_solid_at(&world, block_of(seat)) {
            break;
        }
        seat += (focus - seat) * 0.25;
    }
    let mut view = focus - seat;
    if view.norm() < theme::TALK_CLEARANCE {
        view = flat;
    }
    let view = view.normalize();
    let right = Vec3::new(-view.z, 0.0, view.x);

    if framing.talking {
        let bounce = (framing.phase * theme::TALK_BOUNCE_RATE).sin().abs() * theme::TALK_BOUNCE;
        let sway = (framing.phase * theme::TALK_SWAY_RATE).sin() * theme::TALK_SWAY;
        let stand = framing.base + aside * sway + Vec3::new(0.0, bounce, 0.0);
        let looks_left = (player.eye - head).dot(&right) < 0.0;
        people.for_entity(subject, |(transform, sprite)| {
            transform.translation = stand;
            sprite.flip_x = looks_left;
        });
    } else {
        let stand = framing.base;
        people.for_entity(subject, |(transform, _)| {
            transform.translation = stand;
        });
    }

    let eased = framing.blend * framing.blend * (3.0 - 2.0 * framing.blend);
    let seated =
        quat_angle_axis(look_yaw(view), &Vec3::y()) * quat_angle_axis(look_pitch(view), &Vec3::x());
    let held = quat_angle_axis(player.yaw, &Vec3::y()) * quat_angle_axis(player.pitch, &Vec3::x());
    let position = player.eye + (seat - player.eye) * eased;
    let rotation = quat_slerp(&held, &seated, eased).normalize();
    if let Some(camera) = player.camera {
        cameras.for_entity(camera, |(_, transform)| {
            transform.translation = position;
            transform.rotation = rotation;
        });
    }

    if !framing.talking && framing.blend <= 1.0e-3 {
        framing.blend = 0.0;
        framing.subject = None;
    }
}

fn block_of(position: Vec3) -> IVec3 {
    IVec3::new(
        position.x.floor() as i32,
        position.y.floor() as i32,
        position.z.floor() as i32,
    )
}

fn look_yaw(direction: Vec3) -> f32 {
    (-direction.x).atan2(-direction.z)
}

fn look_pitch(direction: Vec3) -> f32 {
    direction.y.clamp(-1.0, 1.0).asin()
}
