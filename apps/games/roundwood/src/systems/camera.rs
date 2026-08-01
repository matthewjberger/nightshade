use crate::data;
use crate::queries::camera::{eye, look};
use crate::queries::cylinder;
use crate::resources::{Rig, Village};
use nalgebra_glm::Vec2;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn follow_the_neighbor(
    time: Res<Time>,
    village: Res<Village>,
    mut rig: ResMut<Rig>,
    mut movers: Query<(&mut LocalTransform,)>,
) {
    let share = (data::CAMERA_FOLLOW * time.delta_time).clamp(0.0, 1.0);
    let delta = cylinder::town_delta(rig.focus, village.feet);
    rig.focus = Vec2::new(
        rig.focus.x + delta.x * share,
        cylinder::wrap(rig.focus.y + delta.y * share),
    );
    rig.lift += (village.lift - rig.lift) * share;
    let focus = rig.focus;
    let lift = rig.lift;

    if let Some(camera) = village.camera {
        movers.for_entity(camera, |(transform,)| {
            transform.translation = eye(focus, lift);
            transform.rotation = look(focus, lift);
        });
    }
    if let Some(turntable) = village.turntable {
        movers.for_entity(turntable, |(transform,)| {
            transform.rotation = cylinder::spin(focus.y);
        });
    }
}
