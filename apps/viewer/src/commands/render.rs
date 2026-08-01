use nightshade::prelude::*;
use nightshade_camera::components::ViewportShading;
use nightshade_scene::contract::LocalTransform;

use crate::data::ATMOSPHERE_LABELS;
use crate::queries::render::{atmosphere, fog_of, shading_mode};

pub fn set_shading_mode(
    stage: &mut crate::resources::Stage<'_>,
    storage: &mut nightshade::ecs::Storage,
    active_camera: Option<Entity>,
    index: usize,
) {
    stage.viewer.view.shading = index;
    let camera = stage.viewer.camera_input.viewer_camera.or(active_camera);
    if let Some(camera) = camera {
        storage.set(
            camera,
            ViewportShading {
                mode: shading_mode(index),
                ..Default::default()
            },
        );
    }
}

pub fn set_atmosphere(stage: &mut crate::resources::Stage<'_>, index: usize) {
    let hdr = index + 1 == ATMOSPHERE_LABELS.len();
    let hdr_loaded = stage.viewer.view.hdr_loaded;
    if hdr && !hdr_loaded {
        return;
    }
    stage.viewer.view.atmosphere = index;
    stage.settings.atmosphere = atmosphere(index);
}

pub fn write_fog(stage: &mut crate::resources::Stage<'_>) {
    let fog = fog_of(stage.viewer);
    let settings = &mut *stage.settings;
    if let Some(held) = settings.fog.as_mut() {
        *held = fog;
    }
}

pub fn point_sun(storage: &mut nightshade::ecs::Storage, sun: Entity, yaw: f32, pitch: f32) {
    if let Some(transform) = storage.get_mut::<LocalTransform>(sun) {
        transform.rotation = nalgebra_glm::quat_angle_axis(yaw.to_radians(), &Vec3::y())
            * nalgebra_glm::quat_angle_axis(pitch.to_radians(), &Vec3::x());
    }
}
