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
    projection[(0, 2)] += taa_jitter[0];
    projection[(1, 2)] += taa_jitter[1];
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
