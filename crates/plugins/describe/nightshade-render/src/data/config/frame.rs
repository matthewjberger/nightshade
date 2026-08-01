use super::view::CameraFrameInputs;
use nightshade::ecs::Entity;
use std::collections::HashMap;

pub struct ViewInputs {
    pub active_camera: Option<Entity>,
    pub uptime_milliseconds: u64,
    pub delta_time: f32,
    pub viewport_size: Option<(u32, u32)>,
    pub scene_target_size: Option<(u32, u32)>,
    pub camera_tile_rects: HashMap<Entity, super::ViewportRect>,
    pub camera_tile_render_iteration: u32,
    pub active_viewport_rect: Option<super::ViewportRect>,
}

pub enum RendererCommand {
    LoadHdrSkybox {
        hdr_data: Vec<u8>,
    },
    ReloadTexture {
        name: String,
        rgba_data: Vec<u8>,
        width: u32,
        height: u32,
    },
    SetColorLut {
        data: Vec<u8>,
    },
    CaptureScreenshot {
        path: Option<std::path::PathBuf>,
        max_dimension: Option<u32>,
    },
}

#[derive(Clone, Copy, Debug)]
pub struct RenderLine {
    pub start: nalgebra_glm::Vec3,
    pub end: nalgebra_glm::Vec3,
    pub color: nalgebra_glm::Vec4,
    pub entity_id: u32,
    pub depth_mode: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct RenderBoundingVolume {
    pub center: nalgebra_glm::Vec3,
    pub half_extents: nalgebra_glm::Vec3,
    pub orientation: nalgebra_glm::Vec4,
    pub transform: nalgebra_glm::Mat4,
    pub color: nalgebra_glm::Vec4,
}

#[derive(Clone, Copy, Debug)]
pub struct RenderNormal {
    pub position: nalgebra_glm::Vec3,
    pub normal: nalgebra_glm::Vec3,
    pub transform: nalgebra_glm::Mat4,
    pub color: nalgebra_glm::Vec4,
    pub length: f32,
}

#[derive(Default)]
pub struct LinesFrameInputs {
    pub lines: Vec<RenderLine>,
    pub bounding_volumes: Option<Vec<RenderBoundingVolume>>,
    pub normals: Option<Vec<RenderNormal>>,
}

pub struct FrameInputs {
    pub commands: Vec<RendererCommand>,
    pub cameras: Vec<CameraFrameInputs>,
    pub active_camera_frame: Option<CameraFrameInputs>,
    pub fallback_shading: super::EffectiveShading,
    pub pick_request: Option<(u32, u32)>,
    pub has_skinned_meshes: bool,
    pub dirty_world_spheres: Vec<(nalgebra_glm::Vec3, f32)>,
    pub global_dirty_signal: bool,
    pub mouse_position: nalgebra_glm::Vec2,
    pub force_render_cameras: std::collections::HashSet<Entity>,
    pub debug_lines: LinesFrameInputs,
    pub background_clear_color: Option<nalgebra_glm::Vec4>,
}

#[derive(Default)]
pub struct FrameOutputs {
    pub viewport_texture_sizes: Vec<(u32, u32)>,
    pub force_render_cleared: Vec<Entity>,
    pub frame_executed: bool,
}

pub struct ViewFrame {
    pub camera: CameraFrameInputs,
    pub viewport: (u32, u32),
    pub uptime_milliseconds: u64,
    pub delta_time: f32,
}
