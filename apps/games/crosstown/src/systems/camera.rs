use crate::data;
use crate::queries::{camera_limit, eye_at, look_at};
use crate::resources::Ring;
use nightshade::prelude::*;
use nightshade_camera::queries::camera_look::camera_look_rotation;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn follow(time: Res<Time>, mut ring: ResMut<Ring>, mut cameras: Query<(&mut LocalTransform,)>) {
    let (low, high) = camera_limit(ring.cleared, ring.gate_x, ring.end_x, ring.last_wave);
    let wanted = ring.feet.x.clamp(low, high);
    let share = (data::CAMERA_FOLLOW * time.delta_time).clamp(0.0, 1.0);
    ring.camera_x += (wanted - ring.camera_x) * share;
    let shake = ring.shake * (time.uptime_seconds as f32 * data::CAMERA_SHAKE_RATE).sin();
    let eye = eye_at(ring.camera_x, shake);
    let focus = look_at(ring.camera_x);
    let camera = ring.camera;
    cameras.for_entity(camera, |(transform,)| {
        transform.translation = eye;
        transform.rotation = camera_look_rotation(eye, focus, 0.0);
    });
}
