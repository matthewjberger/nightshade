use nightshade::prelude::*;
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::contract::PanOrbitCamera;
use nightshade_platform::prelude::*;

use crate::commands::camera::{frame_bounds, settle_for_preview};
use crate::queries::camera::{model_bounds, selection_bounds};
use crate::resources::{ViewerArguments, ViewerResources};

const TURNTABLE_RADIANS_PER_SECOND: f32 = std::f32::consts::TAU / 20.0;

pub fn update_camera(
    mut viewer: ResMut<ViewerResources>,
    time: Res<Time>,
    active_camera: Res<ActiveCamera>,
    hierarchy: Res<nightshade::ecs::HierarchyIndex>,
    arguments: Res<ViewerArguments>,
    storage: &mut nightshade::ecs::Storage,
) {
    let viewer = &mut *viewer;
    let Some(camera) = active_camera.0 else {
        return;
    };

    if std::mem::take(&mut viewer.camera_input.frame_requested) {
        let bounds = match viewer.selection.selected {
            Some(selected) => selection_bounds(viewer, &hierarchy, storage, selected),
            None => model_bounds(viewer, storage),
        };
        if let Some(bounds) = bounds {
            frame_bounds(camera, storage, bounds);
            if arguments.preview {
                settle_for_preview(camera, storage);
            }
        }
    }

    if viewer.camera_input.turntable
        && let Some(orbit) = storage.get_mut::<PanOrbitCamera>(camera)
    {
        orbit.target_yaw += TURNTABLE_RADIANS_PER_SECOND * time.delta_time;
    }
}
