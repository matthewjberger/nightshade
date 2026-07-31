//! Per-camera [`RenderView`] construction from the frame's camera snapshot.

use crate::config::RenderView;
use crate::wgpu::passes;
use crate::wgpu::render_configs::CameraFrameInputs;

/// Builds the resolved view a camera dispatch renders with: jittered
/// projection, derived inverse matrices, frustum planes, and the camera's
/// projection parameters. Returns `None` when the camera has no matrices.
pub fn build_render_view(
    camera: &CameraFrameInputs,
    taa_jitter: [f32; 2],
    screen_size: (u32, u32),
) -> Option<RenderView> {
    let matrices = camera.matrices.as_ref()?;
    let view = matrices.view;
    let mut projection = matrices.projection;
    // The jitter is a constant offset in normalized device space, and clip is
    // divided by w to get there, so it has to be added as a multiple of the row
    // that produces w. A perspective projection takes w off the view depth, and
    // adding straight to the depth column happens to cancel against that; an
    // orthographic one leaves w at one, so the same write instead offsets every
    // fragment by its own distance from the camera and the whole scene shakes
    // as the sequence advances. Going through the w row is the one form that is
    // right for both.
    for column in 0..4 {
        let weight = projection[(3, column)];
        projection[(0, column)] += taa_jitter[0] * weight;
        projection[(1, column)] += taa_jitter[1] * weight;
    }
    let view_projection = projection * view;
    let inverse_view = nalgebra_glm::inverse(&view);
    let inverse_projection = nalgebra_glm::inverse(&projection);
    let inverse_view_projection = nalgebra_glm::inverse(&view_projection);
    let camera_right = nalgebra_glm::vec3(
        inverse_view[(0, 0)],
        inverse_view[(1, 0)],
        inverse_view[(2, 0)],
    );
    let camera_up = nalgebra_glm::vec3(
        inverse_view[(0, 1)],
        inverse_view[(1, 1)],
        inverse_view[(2, 1)],
    );
    let frustum_planes = passes::geometry::extract_frustum_planes(&view_projection);
    Some(RenderView {
        view,
        projection,
        view_projection,
        inverse_view,
        inverse_projection,
        inverse_view_projection,
        camera_position: matrices.camera_position,
        camera_right,
        camera_up,
        frustum_planes,
        z_near: camera.projection.z_near,
        z_far: camera.projection.z_far,
        y_fov_rad: camera.projection.y_fov_rad,
        aspect: camera.projection.aspect,
        orthographic: camera.projection.orthographic,
        screen_size,
        constrained_aspect: camera.constrained_aspect,
    })
}
