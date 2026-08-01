use crate::queries::base_height;
use crate::resources::Terrain;
use crate::theme;
use nightshade::prelude::*;
use nightshade_camera::components::Camera;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_scene::prelude::LocalTransform;

pub fn keep_above_the_ground(
    active: Res<ActiveCamera>,
    terrain: Res<Terrain>,
    cameras: Query<(&mut LocalTransform, &Camera)>,
) {
    let watching = active.0;
    let noises = &terrain.noises;
    cameras.for_each(|entity, (transform, _camera)| {
        if Some(entity) != watching {
            return;
        }
        let ground = base_height(noises, transform.translation.x, transform.translation.z)
            + theme::OVERHANG_REACH
            + theme::CAMERA_CLEARANCE;
        if transform.translation.y < ground {
            transform.translation.y = ground;
        }
    });
}
