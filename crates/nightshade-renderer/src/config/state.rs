//! `RendererState`, the renderer's own per-frame bookkeeping that no
//! other layer writes.

use super::*;

/// Runtime-only renderer bookkeeping that apps generally do not edit
/// directly. Carried in `RenderInputs::scene`.
#[derive(Clone)]
pub struct RendererState {
    /// Internal bookkeeping for the macOS frame-rate auto-default.
    pub auto_frame_rate_limit_baseline: Option<f32>,
    /// Sub-pixel offset applied to the active-camera projection each frame for
    /// temporal antialiasing, expressed in normalized device coordinates. The
    /// renderer advances it along a low-discrepancy sequence so the temporal
    /// resolve accumulates a supersampled image.
    pub taa_jitter: [f32; 2],
    /// Active-camera view-projection from the current frame, unjittered, used to
    /// derive screen-space motion vectors.
    pub view_projection: [[f32; 4]; 4],
    /// Active-camera view-projection from the previous frame, unjittered.
    pub prev_view_projection: [[f32; 4]; 4],
    /// Enable GPU frustum culling.
    pub gpu_culling_enabled: bool,
    /// Enable GPU hi-z occlusion culling for the mesh pass. Also gates the
    /// machinery that feeds it: the depth prepass, the hi-z pyramid build,
    /// and the second cull dispatch. Defaults off when the renderer publishes
    /// a Metal profile, because Apple's tile based GPUs already eliminate
    /// hidden opaque surfaces in hardware and the prepass costs a full extra
    /// geometry rasterization there. The `cull occlusion on` shell command
    /// re-enables it at runtime.
    pub occlusion_culling_enabled: bool,
    /// Enable the GPU-driven batch table build (classification + combo table on
    /// the GPU). When false the CPU builds the batch tables.
    pub gpu_batching_enabled: bool,
    /// On-screen size in pixels below which an entity is culled.
    pub min_screen_pixel_size: f32,
    /// Current letterbox amount (0-1).
    pub letterbox_amount: f32,
    /// Target letterbox amount for animation.
    pub letterbox_target: f32,
    /// Day/night cycle driver state.
    pub day_night: DayNightState,
    /// Registered level-of-detail chains, keyed by base mesh.
    pub mesh_lod_chains: Vec<MeshLodChain>,
    /// View-local shading derived for the camera the renderer is currently
    /// executing the graph against. Renderer writes this at the start of
    /// each per-camera render. Passes read from this for view-local decisions.
    pub active_view: EffectiveShading,
    /// Camera state for the view the renderer is currently executing the graph
    /// against, extracted once per dispatch. `None` before the first render or
    /// when there is no active camera. Passes read this instead of calling
    /// `query_active_camera_matrices`.
    pub render_view: Option<RenderView>,
    /// The active camera's view, held for every camera in the frame, so a pass
    /// can decide against the active camera rather than the one being rendered.
    /// The meshlet cull reads this to freeze its cut to the active camera while
    /// another camera draws that cut from elsewhere. `None` unless something asks
    /// for it, so an ordinary frame carries no extra view.
    pub frozen_cull_view: Option<RenderView>,
    /// Per-entity dynamic render state (transform, visibility, morph weights) for
    /// skinned meshes, extracted once per frame so the skinned pass reads it here
    /// instead of scanning the ECS. Static meshes live in the scene world.
    pub render_skinned_dynamic_state:
        std::collections::HashMap<nightshade_ecs::Entity, DynamicRenderState>,
    /// Scene materials resolved once per frame by the engine. Render passes read
    /// materials here instead of walking the material registry themselves.
    pub render_materials: RenderMaterials,
    /// Per-entity mesh name for skinned meshes, extracted once per frame so the
    /// skinned pass resolves geometry from its own registry without reading the
    /// `RenderMesh` component. Static meshes live in the scene world instead.
    pub render_skinned_mesh_names: std::collections::HashMap<nightshade_ecs::Entity, String>,
    /// The renderer's meshlet asset cache. Placements live in the scene world
    /// as [`MeshletPlacement`] components; this holds only the baked assets they
    /// reference, kept warm across frames.
    #[cfg(feature = "meshlet")]
    pub meshlet_assets: MeshletAssetCache,
    /// Bumped by the scene sync whenever a meshlet placement was written or
    /// retired, so the meshlet pass can gate its instance rebuild on a counter
    /// rather than reading a quarter of a million placements back every frame
    /// to find that none of them moved.
    #[cfg(feature = "meshlet")]
    pub meshlet_placements_generation: u64,
    /// The color the outline pass draws the selection outline with, resolved
    /// once per frame. The covered entities carry the
    /// [`crate::render_world::SelectionOutline`] tag in the scene world.
    pub render_selection_outline_color: [f32; 4],
    /// Shadow-casting entities collected once per frame, split by draw kind, so
    /// the shadow pass enumerates occluders here instead of walking the ECS.
    pub render_shadow_casters: RenderShadowCasters,
    /// Skinned-mesh entities collected once per frame in query order, so the
    /// skinned-mesh pass enumerates draws here instead of walking the ECS.
    pub render_skinned_meshes: Vec<nightshade_ecs::Entity>,
    /// When set, the mesh pass derives its cull frustum planes from this
    /// view-projection instead of the active camera's, pinning culling to a
    /// fixed viewpoint.
    pub culling_camera_view_projection: Option<nalgebra_glm::Mat4>,
    /// Skinning palette resolved once per frame, so the skinned-mesh and shadow
    /// passes read bone data here instead of walking Skin and bone transforms.
    pub render_skinning: RenderSkinning,
    /// Skinned-animation state resolved once per frame by the engine, so the
    /// skinned-mesh compute driver builds its GPU buffers without reading ECS
    /// animation, skin, parent, or transform components.
    pub render_animation: SkinnedAnimationSnapshot,
    /// Scene lighting for the current dispatch, collected once by the renderer.
    /// `None` before the first render. Lit passes read this instead of scanning
    /// the ECS for lights themselves.
    #[cfg(feature = "wgpu")]
    pub render_lighting: Option<RenderLighting>,
    /// Bumped whenever a skinned entity's object state changed (visibility,
    /// morph weights, transform, bounds), so the skinned pass can gate its
    /// instance rebuild on a counter.
    pub skinned_generation: u64,
    /// Monotonic version bumped by the renderer whenever
    /// `render_settings_signature` changes between frames.
    pub settings_version: u64,
    /// Controls whether the focused viewport overrides its
    /// `ViewportUpdateMode` and re-renders every frame.
    pub focus_policy: ViewportFocusPolicy,
    /// Frame-time budget that scales optional post-process sample counts.
    pub adaptive_sampling: AdaptiveSamplingState,
    /// Post-process effects pass settings. The `EffectsPass` reads this each frame.
    pub effects: EffectsState,
    /// The adapter the renderer selected, published once the renderer has a
    /// device. `None` until the first rendered frame.
    pub gpu_profile: Option<GpuProfile>,
}

impl Default for RendererState {
    fn default() -> Self {
        Self {
            auto_frame_rate_limit_baseline: None,
            taa_jitter: [0.0, 0.0],
            frozen_cull_view: None,
            view_projection: nalgebra_glm::Mat4::identity().into(),
            prev_view_projection: nalgebra_glm::Mat4::identity().into(),
            gpu_culling_enabled: true,
            occlusion_culling_enabled: true,
            gpu_batching_enabled: true,
            min_screen_pixel_size: 0.0,
            letterbox_amount: 0.0,
            letterbox_target: 0.0,
            day_night: DayNightState::default(),
            mesh_lod_chains: Vec::new(),
            active_view: EffectiveShading::default(),
            render_view: None,
            render_skinned_dynamic_state: std::collections::HashMap::new(),
            render_materials: RenderMaterials::default(),
            render_skinned_mesh_names: std::collections::HashMap::new(),
            #[cfg(feature = "meshlet")]
            meshlet_assets: MeshletAssetCache::default(),
            #[cfg(feature = "meshlet")]
            meshlet_placements_generation: 0,
            render_selection_outline_color: [1.0, 0.45, 0.0, 1.0],
            render_shadow_casters: RenderShadowCasters::default(),
            render_skinned_meshes: Vec::new(),
            culling_camera_view_projection: None,
            render_skinning: RenderSkinning::default(),
            render_animation: SkinnedAnimationSnapshot::default(),
            #[cfg(feature = "wgpu")]
            render_lighting: None,
            skinned_generation: 0,
            settings_version: 0,
            focus_policy: ViewportFocusPolicy::default(),
            adaptive_sampling: AdaptiveSamplingState::default(),
            effects: EffectsState::default(),
            gpu_profile: None,
        }
    }
}
