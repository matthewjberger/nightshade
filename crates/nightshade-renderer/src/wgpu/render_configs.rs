//! The renderer's per-frame input: one owned structure the engine driver
//! composes from the world before rendering and restores afterward. The
//! large snapshot state is moved in and out rather than cloned, so handing
//! it to the render graph costs nothing, and the graph and passes are
//! monomorphized on this concrete type with no engine coupling.

use freecs::Entity;
use std::collections::HashMap;

/// The retained-UI content for the current frame.
pub struct UiFrameInputs {
    pub rects: Vec<crate::ui_data::UiRect>,
    pub rect_entities: Vec<Option<Entity>>,
    pub images: Vec<crate::ui_data::UiImage>,
    pub text_meshes: Vec<crate::ui_data::UiTextInstance>,
    pub render_slots: crate::ui_data::RenderSlotAllocator,
    pub background_color: Option<nalgebra_glm::Vec4>,
}

/// The state of the view being rendered right now: the active camera, frame
/// timing, and the viewport layout the compose pass targets. The renderer
/// updates this between camera dispatches within a frame.
pub struct ViewInputs {
    pub active_camera: Option<Entity>,
    pub uptime_milliseconds: u64,
    pub delta_time: f32,
    pub viewport_size: Option<(u32, u32)>,
    pub camera_tile_rects: HashMap<Entity, crate::config::ViewportRect>,
    pub camera_tile_render_iteration: u32,
    pub active_viewport_rect: Option<crate::config::ViewportRect>,
}

/// One camera's unjittered matrices, captured before the frame.
#[derive(Clone, Copy)]
pub struct CameraMatricesInputs {
    pub view: nalgebra_glm::Mat4,
    pub projection: nalgebra_glm::Mat4,
    pub camera_position: nalgebra_glm::Vec3,
}

/// One camera's resolved projection parameters, captured before the frame.
/// `aspect` already reflects the camera's override or the window fallback.
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

/// Everything the renderer needs to dispatch one camera, captured from the
/// scene before the frame: matrices, projection parameters, the resolved
/// per-camera shading, the viewport update policy, and the camera's world
/// transform for dirtiness checks.
#[derive(Clone)]
pub struct CameraFrameInputs {
    pub entity: Entity,
    pub matrices: Option<CameraMatricesInputs>,
    pub projection: CameraProjectionParams,
    pub constrained_aspect: Option<f32>,
    pub effective_shading: crate::config::EffectiveShading,
    pub update_mode: crate::config::ViewportUpdateMode,
    pub world_transform: Option<nalgebra_glm::Mat4>,
}

/// A render-side command drained from the caller's command queue for this
/// frame. Path-based commands are resolved to bytes before they reach the
/// renderer.
pub enum RendererCommand {
    UploadUiImageLayer {
        layer: u32,
        rgba_data: Vec<u8>,
        width: u32,
        height: u32,
    },
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

/// Debug line geometry gathered for this frame. `None` for the bounding
/// volume or normal sets means the corresponding overlay is disabled and the
/// pass buffers are cleared.
#[derive(Default)]
pub struct LinesFrameInputs {
    pub lines: Vec<crate::wgpu::passes::geometry::lines::GpuLineData>,
    pub bounding_volumes: Option<Vec<crate::wgpu::passes::geometry::lines::GpuBoundingVolumeData>>,
    pub normals: Option<Vec<crate::wgpu::passes::geometry::lines::GpuNormalData>>,
}

/// One frame's orchestration snapshot: everything the caller captured for
/// this frame beyond the persistent scene state. Consumed by the frame
/// dispatch, never read by passes.
pub struct FrameInputs {
    /// Render commands drained from the caller's queue for this frame.
    pub commands: Vec<RendererCommand>,
    /// The cameras to dispatch this frame, in dispatch order.
    pub cameras: Vec<CameraFrameInputs>,
    /// The active camera's frame data, for the previous-frame reprojection
    /// matrices and the no-viewport render path.
    pub active_camera_frame: Option<CameraFrameInputs>,
    /// The shading used when no camera drives the view.
    pub fallback_shading: crate::config::EffectiveShading,
    /// A pending GPU pick request as (screen_x, screen_y), consumed by the
    /// main viewport camera's dispatch.
    pub pick_request: Option<(u32, u32)>,
    /// Whether any skinned meshes exist this frame.
    pub has_skinned_meshes: bool,
    /// The frame's mesh dirty state, applied to the passes once before the
    /// first graph execution.
    pub mesh_frame_state: Option<crate::mesh_state::MeshRenderStateInner>,
    /// World-space bounding spheres of entities that changed this frame,
    /// used for per-viewport dirtiness culling.
    pub dirty_world_spheres: Vec<(nalgebra_glm::Vec3, f32)>,
    /// True when dirty work cannot be localized to a frustum and every
    /// cached viewport must re-render.
    pub global_dirty_signal: bool,
    /// The pointer position, for focus-driven viewport update policies.
    pub mouse_position: nalgebra_glm::Vec2,
    /// Cameras whose next render is forced; the renderer reports which ones
    /// it cleared through [`FrameOutputs`].
    pub force_render_cameras: std::collections::HashSet<Entity>,
    /// Debug line geometry gathered for this frame.
    pub debug_lines: LinesFrameInputs,
}

/// State the renderer hands back after a frame: writes that belong to the
/// caller's world, produced instead of performed because the renderer never
/// sees the world.
#[derive(Default)]
pub struct FrameOutputs {
    pub viewport_texture_sizes: Vec<(u32, u32)>,
    pub force_render_cleared: Vec<Entity>,
    /// Whether the frame actually rendered. False when the surface was
    /// occluded, timed out, or lost, so callers keep their previous
    /// viewport state instead of applying this frame's empty outputs.
    pub frame_executed: bool,
}

/// Everything the renderer consumes for one frame.
pub struct RenderInputs {
    /// User-facing render settings: post-processing, atmosphere, layers.
    pub settings: crate::config::RenderSettings,
    /// Debug visualization toggles.
    pub debug_draw: crate::config::DebugDraw,
    /// Editor selection state for outlines and culling overrides.
    pub editor_selection: crate::config::EditorSelection,
    /// The per-frame scene snapshot every pass draws from.
    pub scene: crate::config::RendererState,
    /// Image-based-lighting texture views captured by the sky pass.
    pub ibl_views: crate::config::IblViews,
    /// Spotlight shadow atlas slot assignment for this frame.
    pub shadow_atlas: crate::wgpu::passes::shadow_depth::atlas::SpotlightAtlasAssignment,
    /// Global wind driving the cloth simulation.
    pub wind: crate::wind::Wind,
    /// Named mesh geometry with dirty tracking.
    pub mesh_cache: crate::mesh_cache::MeshCache,
    /// GPU texture registry and lifecycle state.
    pub texture_cache: crate::wgpu::texture_cache::TextureCache,
    /// Particle textures decoded and waiting for upload.
    pub pending_particle_textures: Vec<crate::particles::ParticleTextureUpload>,
    /// The retained-UI content for the current frame.
    pub ui: UiFrameInputs,
    /// The view being rendered right now.
    pub view: ViewInputs,
    /// Identity of the world being rendered, keying per-world GPU state.
    pub world_id: u64,
    /// The frame's orchestration snapshot: cameras, commands, and dirt.
    pub frame: FrameInputs,
    #[cfg(feature = "grass")]
    pub grass: crate::grass_config::GrassSettings,
    #[cfg(feature = "terrain")]
    pub terrain: crate::terrain_config::TerrainSettings,
    #[cfg(feature = "terrain")]
    pub terrain_render: crate::terrain_config::TerrainRenderState,
}
