pub mod camera_look;
pub mod viewport_shading;
use crate::components::{
    Camera, ConstrainedAspect, OrthographicCamera, PerspectiveCamera, Projection,
    ProjectionOverride,
};
use nalgebra_glm::{Mat4, Vec3, Vec4};
use nightshade_ecs::Entity;
use nightshade_ecs::QueryRef;
use nightshade_scene::contract::GlobalTransform;

pub type CameraRows<'world> = QueryRef<
    'world,
    (
        &'world Camera,
        &'world GlobalTransform,
        Option<&'world ProjectionOverride>,
        Option<&'world ConstrainedAspect>,
    ),
>;

#[derive(Clone, Copy)]
pub struct CameraRow {
    pub camera: Camera,
    pub global_transform: GlobalTransform,
    pub projection_override: Option<ProjectionOverride>,
    pub constrained_aspect: Option<ConstrainedAspect>,
}

pub fn camera_row(cameras: CameraRows<'_>, entity: Entity) -> Option<CameraRow> {
    cameras.iter().find(|(row, _)| *row == entity).map(
        |(_, (camera, global_transform, projection_override, constrained_aspect))| CameraRow {
            camera: *camera,
            global_transform: *global_transform,
            projection_override: projection_override.copied(),
            constrained_aspect: constrained_aspect.copied(),
        },
    )
}

#[derive(Clone)]
pub struct CameraMatrices {
    pub camera_position: Vec3,
    pub projection: Mat4,
    pub cull_projection: Mat4,
    pub view: Mat4,
}

#[derive(Clone, Copy, Default)]
pub struct AspectFallbacks {
    pub tile: Option<f32>,
    pub window: Option<f32>,
}

pub enum ResolvedProjection {
    Override(ProjectionOverride),
    Perspective(PerspectiveCamera, f32),
    Orthographic(OrthographicCamera, f32),
    None,
}

fn viewport_aspect(row: &CameraRow, fallbacks: AspectFallbacks) -> Option<f32> {
    row.constrained_aspect
        .map(|constrained| constrained.0)
        .or(fallbacks.tile)
        .or(fallbacks.window)
}

pub fn resolve_projection(
    row: Option<&CameraRow>,
    fallbacks: AspectFallbacks,
) -> ResolvedProjection {
    let Some(row) = row else {
        return ResolvedProjection::None;
    };
    if let Some(override_projection) = row.projection_override {
        return ResolvedProjection::Override(override_projection);
    }
    match &row.camera.projection {
        Projection::Perspective(perspective) => {
            let aspect_ratio = row
                .constrained_aspect
                .map(|constrained| constrained.0)
                .or(perspective.aspect_ratio)
                .or(fallbacks.tile)
                .or(fallbacks.window)
                .unwrap_or(16.0 / 9.0);
            ResolvedProjection::Perspective(*perspective, aspect_ratio)
        }
        Projection::Orthographic(orthographic) => {
            let aspect_ratio = viewport_aspect(row, fallbacks).unwrap_or(16.0 / 9.0);
            ResolvedProjection::Orthographic(*orthographic, aspect_ratio)
        }
    }
}

pub fn query_camera_matrices(
    row: &CameraRow,
    fallbacks: AspectFallbacks,
) -> Option<CameraMatrices> {
    let camera_position = row.global_transform.translation();
    let forward = row.global_transform.forward_vector();
    let up = row.global_transform.up_vector();
    let target = camera_position + forward;

    let (projection, cull_projection) = match resolve_projection(Some(row), fallbacks) {
        ResolvedProjection::Override(held) => (
            held.matrix,
            nightshade_render::queries::projection::reverse_z_perspective(
                held.y_fov_rad,
                held.aspect,
                held.z_near,
                held.z_far,
            ),
        ),
        ResolvedProjection::Perspective(perspective, aspect_ratio) => {
            let matrix = perspective.matrix_with_aspect(aspect_ratio);
            (matrix, matrix)
        }
        ResolvedProjection::Orthographic(orthographic, aspect_ratio) => {
            let matrix = orthographic.matrix_with_aspect(aspect_ratio);
            (matrix, matrix)
        }
        ResolvedProjection::None => return None,
    };

    let view = nalgebra_glm::look_at(&camera_position, &target, &up);

    Some(CameraMatrices {
        camera_position,
        projection,
        cull_projection,
        view,
    })
}

pub fn query_camera_tile_aspect_ratio(
    viewport: &nightshade_render::resources::viewport::Viewport,
    entity: Entity,
) -> Option<f32> {
    let rect = viewport.camera_tile_rects.get(&entity)?;
    if rect.height <= 0.0 {
        return None;
    }
    Some(rect.width / rect.height)
}

pub fn query_window_aspect_ratio(
    window: &nightshade_platform::contract::window::Window,
) -> Option<f32> {
    let (width, height) = window.cached_viewport_size?;
    Some(width as f32 / height.max(1) as f32)
}

pub fn aspect_fallbacks(
    viewport: &nightshade_render::resources::viewport::Viewport,
    window: &nightshade_platform::contract::window::Window,
    entity: Entity,
) -> AspectFallbacks {
    AspectFallbacks {
        tile: query_camera_tile_aspect_ratio(viewport, entity),
        window: query_window_aspect_ratio(window),
    }
}

#[derive(Clone)]
pub struct CameraFrustumCorners {
    pub near_top_left: Vec3,
    pub near_top_right: Vec3,
    pub near_bottom_left: Vec3,
    pub near_bottom_right: Vec3,
    pub far_top_left: Vec3,
    pub far_top_right: Vec3,
    pub far_bottom_left: Vec3,
    pub far_bottom_right: Vec3,
}

pub fn query_camera_frustum(
    row: &CameraRow,
    fallbacks: AspectFallbacks,
) -> Option<CameraFrustumCorners> {
    let matrices = query_camera_matrices(row, fallbacks)?;
    let view_proj = matrices.projection * matrices.view;
    let inv_view_proj = view_proj.try_inverse()?;

    let near_z = 1.0_f32;
    let far_z = 0.0_f32;

    Some(CameraFrustumCorners {
        near_top_left: unproject_ndc(&inv_view_proj, Vec3::new(-1.0, 1.0, near_z)),
        near_top_right: unproject_ndc(&inv_view_proj, Vec3::new(1.0, 1.0, near_z)),
        near_bottom_left: unproject_ndc(&inv_view_proj, Vec3::new(-1.0, -1.0, near_z)),
        near_bottom_right: unproject_ndc(&inv_view_proj, Vec3::new(1.0, -1.0, near_z)),
        far_top_left: unproject_ndc(&inv_view_proj, Vec3::new(-1.0, 1.0, far_z)),
        far_top_right: unproject_ndc(&inv_view_proj, Vec3::new(1.0, 1.0, far_z)),
        far_bottom_left: unproject_ndc(&inv_view_proj, Vec3::new(-1.0, -1.0, far_z)),
        far_bottom_right: unproject_ndc(&inv_view_proj, Vec3::new(1.0, -1.0, far_z)),
    })
}

fn unproject_ndc(inv_view_proj: &Mat4, ndc: Vec3) -> Vec3 {
    let clip = inv_view_proj * Vec4::new(ndc.x, ndc.y, ndc.z, 1.0);
    Vec3::new(clip.x / clip.w, clip.y / clip.w, clip.z / clip.w)
}
