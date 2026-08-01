use crate::resources::{VoxelViewer, Watcher};
use nalgebra_glm::{Vec3, column};
use nightshade::prelude::*;
use nightshade_camera::prelude::Camera;
use nightshade_scene::prelude::GlobalTransform;

pub fn aim_the_viewer(
    watcher: Res<Watcher>,
    mut viewer: ResMut<VoxelViewer>,
    mut cameras: Query<(&Camera, &GlobalTransform)>,
) {
    let Some(camera) = watcher.camera else {
        return;
    };
    cameras.for_entity(camera, |(_, global)| {
        let eye = column(&global.0, 3);
        let forward = -column(&global.0, 2);
        viewer.eye = Vec3::new(eye.x, eye.y, eye.z);
        viewer.forward = Vec3::new(forward.x, forward.y, forward.z).normalize();
    });
}
