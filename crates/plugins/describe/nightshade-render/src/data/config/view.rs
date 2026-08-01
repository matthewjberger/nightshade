#[derive(Clone, Debug)]
pub struct RenderView {
    pub view: nalgebra_glm::Mat4,
    pub projection: nalgebra_glm::Mat4,
    pub view_projection: nalgebra_glm::Mat4,
    pub inverse_view: nalgebra_glm::Mat4,
    pub inverse_projection: nalgebra_glm::Mat4,
    pub inverse_view_projection: nalgebra_glm::Mat4,
    pub camera_position: nalgebra_glm::Vec3,
    pub camera_right: nalgebra_glm::Vec3,
    pub camera_up: nalgebra_glm::Vec3,
    pub frustum_planes: [nalgebra_glm::Vec4; 6],
    pub oblique_depth: bool,
    pub z_near: f32,
    pub z_far: f32,
    pub y_fov_rad: f32,
    pub aspect: f32,
    pub orthographic: Option<(f32, f32)>,
    pub screen_size: (u32, u32),
    pub constrained_aspect: Option<f32>,
}

impl RenderView {
    pub fn projection_params(&self) -> CameraProjectionParams {
        CameraProjectionParams {
            z_near: self.z_near,
            z_far: self.z_far,
            y_fov_rad: self.y_fov_rad,
            aspect: self.aspect,
            orthographic: self.orthographic,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DynamicRenderState {
    pub visible: u32,
    pub morph_weights: [f32; 8],
    pub transform: nalgebra_glm::Mat4,
    pub culling_mask: u32,
    pub is_overlay: u32,
    pub render_layer: u32,
}

pub fn reverse_z_perspective(
    y_fov_rad: f32,
    aspect: f32,
    z_near: f32,
    z_far: f32,
) -> nalgebra_glm::Mat4 {
    let focal_length = 1.0 / (y_fov_rad / 2.0).tan();
    nalgebra_glm::Mat4::new(
        focal_length / aspect,
        0.0,
        0.0,
        0.0,
        0.0,
        focal_length,
        0.0,
        0.0,
        0.0,
        0.0,
        z_near / (z_far - z_near),
        z_near * z_far / (z_far - z_near),
        0.0,
        0.0,
        -1.0,
        0.0,
    )
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ViewportRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl ViewportRect {
    pub fn contains(&self, screen_pos: nalgebra_glm::Vec2) -> bool {
        screen_pos.x >= self.x
            && screen_pos.x <= self.x + self.width
            && screen_pos.y >= self.y
            && screen_pos.y <= self.y + self.height
    }

    pub fn to_local(&self, screen_pos: nalgebra_glm::Vec2) -> nalgebra_glm::Vec2 {
        nalgebra_glm::Vec2::new(screen_pos.x - self.x, screen_pos.y - self.y)
    }
}

use super::{EffectiveShading, ViewportUpdateMode};
use nightshade::ecs::Entity;

#[derive(Clone, Copy)]
pub struct CameraMatricesInputs {
    pub view: nalgebra_glm::Mat4,
    pub projection: nalgebra_glm::Mat4,
    pub cull_projection: nalgebra_glm::Mat4,
    pub camera_position: nalgebra_glm::Vec3,
}

#[derive(Clone, Copy)]
pub struct CameraProjectionParams {
    pub z_near: f32,
    pub z_far: f32,
    pub y_fov_rad: f32,
    pub aspect: f32,
    pub orthographic: Option<(f32, f32)>,
}

impl Default for CameraProjectionParams {
    fn default() -> Self {
        Self {
            z_near: 0.1,
            z_far: 1000.0,
            y_fov_rad: std::f32::consts::FRAC_PI_4,
            aspect: 1.78,
            orthographic: None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct CameraTarget {
    pub size: (u32, u32),
    pub host_owned: bool,
}

#[derive(Clone)]
pub struct CameraFrameInputs {
    pub entity: Entity,
    pub matrices: Option<CameraMatricesInputs>,
    pub projection: CameraProjectionParams,
    pub constrained_aspect: Option<f32>,
    pub effective_shading: EffectiveShading,
    pub update_mode: ViewportUpdateMode,
    pub world_transform: Option<nalgebra_glm::Mat4>,
    pub target: Option<CameraTarget>,
}

#[derive(Default)]
pub struct RenderViews {
    pub active: Option<CameraFrameInputs>,
    pub dispatched: Vec<CameraFrameInputs>,
    pub fallback_shading: EffectiveShading,
}

impl RenderViews {
    pub fn view_for(&self, entity: Entity) -> Option<&CameraFrameInputs> {
        self.dispatched
            .iter()
            .chain(self.active.iter())
            .find(|view| view.entity == entity)
    }

    pub fn active_entity(&self) -> Option<Entity> {
        self.active.as_ref().map(|view| view.entity)
    }
}

impl CameraFrameInputs {
    pub fn perspective(
        entity: Entity,
        view: nalgebra_glm::Mat4,
        position: nalgebra_glm::Vec3,
        projection: CameraProjectionParams,
        effective_shading: super::EffectiveShading,
    ) -> Self {
        Self {
            entity,
            matrices: Some(CameraMatricesInputs {
                view,
                projection: super::reverse_z_perspective(
                    projection.y_fov_rad,
                    projection.aspect,
                    projection.z_near,
                    projection.z_far,
                ),
                cull_projection: super::reverse_z_perspective(
                    projection.y_fov_rad,
                    projection.aspect,
                    projection.z_near,
                    projection.z_far,
                ),
                camera_position: position,
            }),
            projection,
            constrained_aspect: None,
            effective_shading,
            update_mode: super::ViewportUpdateMode::Always,
            world_transform: None,
            target: None,
        }
    }
}

pub fn build_render_view(
    camera: &CameraFrameInputs,
    taa_jitter: [f32; 2],
    screen_size: (u32, u32),
) -> Option<RenderView> {
    let matrices = camera.matrices.as_ref()?;
    let view = matrices.view;
    let mut projection = matrices.projection;
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
    let frustum_planes =
        crate::data::bounding_volume::extract_frustum_planes(&(matrices.cull_projection * view));
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
        oblique_depth: matrices.cull_projection != matrices.projection,
        z_near: camera.projection.z_near,
        z_far: camera.projection.z_far,
        y_fov_rad: camera.projection.y_fov_rad,
        aspect: camera.projection.aspect,
        orthographic: camera.projection.orthographic,
        screen_size,
        constrained_aspect: camera.constrained_aspect,
    })
}
