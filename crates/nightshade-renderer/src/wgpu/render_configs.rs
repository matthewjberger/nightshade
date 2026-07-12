//! The renderer's per-frame input: one owned structure the engine driver
//! composes from the world before rendering and restores afterward. The
//! large snapshot state is moved in and out rather than cloned, so handing
//! it to the render graph costs nothing, and the graph and passes are
//! monomorphized on this concrete type with no engine coupling.

use freecs::Entity;
use std::collections::HashMap;

/// The retained-UI content for the current frame.
pub struct UiFrameInputs {
    pub rects: Vec<crate::wgpu::passes::geometry::UiRect>,
    pub rect_entities: Vec<Option<Entity>>,
    pub images: Vec<crate::wgpu::passes::geometry::UiImage>,
    pub text_meshes: Vec<crate::ui_data::UiTextInstance>,
    pub render_slots: crate::ui_data::RenderSlotAllocator,
    pub background_color: Option<nalgebra_glm::Vec4>,
}

/// The state of the view being rendered right now: the active camera, frame
/// timing, and the viewport layout the compose pass targets. The driver
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
    #[cfg(feature = "grass")]
    pub grass: crate::grass_config::GrassSettings,
    #[cfg(feature = "terrain")]
    pub terrain: crate::terrain_config::TerrainSettings,
    #[cfg(feature = "terrain")]
    pub terrain_render: crate::terrain_config::TerrainRenderState,
}
