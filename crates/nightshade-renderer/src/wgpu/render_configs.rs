//! The renderer's per-frame input: one owned structure the engine driver
//! composes from the world before rendering and restores afterward. The
//! large snapshot state is moved in and out rather than cloned, so handing
//! it to the render graph costs nothing, and the graph and passes are
//! monomorphized on this concrete type with no engine coupling.

use nightshade_ecs::Entity;
use std::collections::HashMap;

/// The retained-UI content for the current frame.
pub struct UiFrameInputs {
    /// Solid-color rectangles to draw this frame.
    pub rects: Vec<crate::ui_data::UiRect>,
    /// The source entity for each rectangle, index-aligned with `rects`.
    pub rect_entities: Vec<Option<Entity>>,
    /// Textured image quads to draw this frame.
    pub images: Vec<crate::ui_data::UiImage>,
    /// Glyph-mesh instances to draw this frame.
    pub text_meshes: Vec<crate::ui_data::UiTextInstance>,
    /// Allocator assigning render-target slots to UI content this frame.
    pub render_slots: crate::ui_data::RenderSlotAllocator,
    /// Clear color for the UI layer, or `None` to leave it transparent.
    pub background_color: Option<nalgebra_glm::Vec4>,
}

/// The state of the view being rendered right now: the active camera, frame
/// timing, and the viewport layout the compose pass targets. The renderer
/// updates this between camera dispatches within a frame.
pub struct ViewInputs {
    /// The camera driving this view, or `None` for the fallback path.
    pub active_camera: Option<Entity>,
    /// Milliseconds elapsed since startup, for time-driven effects.
    pub uptime_milliseconds: u64,
    /// Seconds elapsed since the previous frame.
    pub delta_time: f32,
    /// The view's target size in pixels, or `None` when unset.
    pub viewport_size: Option<(u32, u32)>,
    /// Per-camera sub-rectangle within the viewport for tiled multi-camera
    /// layouts.
    pub camera_tile_rects: HashMap<Entity, crate::config::ViewportRect>,
    /// Which tile this render iteration targets in a multi-camera layout.
    pub camera_tile_render_iteration: u32,
    /// The rectangle the compose pass writes to this frame.
    pub active_viewport_rect: Option<crate::config::ViewportRect>,
}

/// One camera's unjittered matrices, captured before the frame.
#[derive(Clone, Copy)]
pub struct CameraMatricesInputs {
    /// World-to-view matrix.
    pub view: nalgebra_glm::Mat4,
    /// View-to-clip matrix, reverse-Z as the passes expect.
    pub projection: nalgebra_glm::Mat4,
    /// The camera's world-space position.
    pub camera_position: nalgebra_glm::Vec3,
}

/// One camera's resolved projection parameters, captured before the frame.
/// `aspect` already reflects the camera's override or the window fallback.
#[derive(Clone, Copy)]
pub struct CameraProjectionParams {
    /// Near clip distance in world units.
    pub z_near: f32,
    /// Far clip distance in world units.
    pub z_far: f32,
    /// Vertical field of view in radians.
    pub y_fov_rad: f32,
    /// Width-over-height ratio, already resolved from override or window.
    pub aspect: f32,
    /// Orthographic half-width and half-height, or `None` for perspective.
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

/// Where a camera's image goes and how large it is, when the camera says so
/// itself rather than inheriting the window's size.
#[derive(Clone, Copy)]
pub struct CameraTarget {
    /// Render resolution in pixels.
    pub size: (u32, u32),
    /// Whether the host binds the texture before each frame instead of the
    /// renderer allocating one. A host-owned target is somewhere the renderer
    /// cannot allocate, and its frame is not going through the window.
    pub host_owned: bool,
}

/// Everything the renderer needs to dispatch one camera, captured from the
/// scene before the frame: matrices, projection parameters, the resolved
/// per-camera shading, the viewport update policy, and the camera's world
/// transform for dirtiness checks.
#[derive(Clone)]
pub struct CameraFrameInputs {
    /// The camera entity this dispatch renders.
    pub entity: Entity,
    /// The camera's unjittered matrices, or `None` when not yet resolved.
    pub matrices: Option<CameraMatricesInputs>,
    /// The camera's resolved projection parameters.
    pub projection: CameraProjectionParams,
    /// Aspect the viewport is constrained to, or `None` to fill.
    pub constrained_aspect: Option<f32>,
    /// The per-camera shading resolved from settings.
    pub effective_shading: crate::config::EffectiveShading,
    /// When this camera's viewport re-renders.
    pub update_mode: crate::config::ViewportUpdateMode,
    /// The camera's world transform, for dirtiness checks.
    pub world_transform: Option<nalgebra_glm::Mat4>,
    /// The camera's own render target, when it declares one. Cameras without
    /// one are sized from their viewport tile or the window, as before.
    pub target: Option<CameraTarget>,
}

/// A render-side command drained from the caller's command queue for this
/// frame. Path-based commands are resolved to bytes before they reach the
/// renderer.
pub enum RendererCommand {
    /// Upload RGBA pixels into one layer of the UI image array.
    UploadUiImageLayer {
        /// Destination array layer.
        layer: u32,
        /// Tightly packed RGBA8 pixels.
        rgba_data: Vec<u8>,
        /// Image width in pixels.
        width: u32,
        /// Image height in pixels.
        height: u32,
    },
    /// Replace the skybox from an equirectangular HDR image.
    LoadHdrSkybox {
        /// Encoded HDR image bytes.
        hdr_data: Vec<u8>,
    },
    /// Replace a named texture's pixels in the cache.
    ReloadTexture {
        /// The cache key of the texture to replace.
        name: String,
        /// Tightly packed RGBA8 pixels.
        rgba_data: Vec<u8>,
        /// Image width in pixels.
        width: u32,
        /// Image height in pixels.
        height: u32,
    },
    /// Set the color-grading lookup table used by post-processing.
    SetColorLut {
        /// Encoded LUT image bytes.
        data: Vec<u8>,
    },
    /// Capture the next frame to disk or memory.
    CaptureScreenshot {
        /// Output file path, or `None` to keep the capture in memory.
        path: Option<std::path::PathBuf>,
        /// Longest-edge pixel cap, or `None` for full resolution.
        max_dimension: Option<u32>,
    },
}

/// One debug line segment in world space. The caller composes these; the
/// lines pass converts them to its GPU layout, so the contract carries no
/// GPU-repr type.
#[derive(Clone, Copy, Debug)]
pub struct RenderLine {
    /// Segment start in world space.
    pub start: nalgebra_glm::Vec3,
    /// Segment end in world space.
    pub end: nalgebra_glm::Vec3,
    /// Line color as linear RGBA.
    pub color: nalgebra_glm::Vec4,
    /// Owning entity id for picking.
    pub entity_id: u32,
    /// Depth test mode: 0 depth-tested, 1 always on top.
    pub depth_mode: u32,
}

/// One oriented bounding volume to expand into wireframe lines. Mirror of the
/// lines pass's GPU box input, without the GPU padding.
#[derive(Clone, Copy, Debug)]
pub struct RenderBoundingVolume {
    /// Box center in local space.
    pub center: nalgebra_glm::Vec3,
    /// Half-size along each local axis.
    pub half_extents: nalgebra_glm::Vec3,
    /// Orientation quaternion as (x, y, z, w).
    pub orientation: nalgebra_glm::Vec4,
    /// World transform applied to the box.
    pub transform: nalgebra_glm::Mat4,
    /// Line color as linear RGBA.
    pub color: nalgebra_glm::Vec4,
}

/// One vertex normal to expand into a line segment. Mirror of the lines pass's
/// GPU normal input, without the GPU padding.
#[derive(Clone, Copy, Debug)]
pub struct RenderNormal {
    /// Normal origin in local space.
    pub position: nalgebra_glm::Vec3,
    /// Normal direction in local space.
    pub normal: nalgebra_glm::Vec3,
    /// World transform applied to the origin and direction.
    pub transform: nalgebra_glm::Mat4,
    /// Line color as linear RGBA.
    pub color: nalgebra_glm::Vec4,
    /// Length of the drawn normal line.
    pub length: f32,
}

/// Debug line geometry gathered for this frame. `None` for the bounding
/// volume or normal sets means the corresponding overlay is disabled and the
/// pass buffers are cleared.
#[derive(Default)]
pub struct LinesFrameInputs {
    /// Explicit debug line segments to draw.
    pub lines: Vec<RenderLine>,
    /// Bounding-volume wireframes, or `None` when that overlay is off.
    pub bounding_volumes: Option<Vec<RenderBoundingVolume>>,
    /// Vertex-normal lines, or `None` when that overlay is off.
    pub normals: Option<Vec<RenderNormal>>,
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
    /// The egui overlay's tessellated output for this frame, or `None` when
    /// nothing was drawn. Carried here so egui flows through the frame inputs
    /// like every other input rather than mutating the renderer through a side
    /// channel; the frame driver applies it before the graph executes.
    #[cfg(feature = "egui")]
    pub egui: Option<EguiFrameInputs>,
}

/// The egui overlay's tessellated output for one frame. A renderer-owned
/// wrapper over the egui payload so the frame inputs, not a bare function call
/// on the renderer, carry the overlay across the boundary.
#[cfg(feature = "egui")]
pub struct EguiFrameInputs {
    /// Texture atlas deltas egui produced this frame.
    pub textures_delta: egui::TexturesDelta,
    /// The overlay's device pixel ratio.
    pub pixels_per_point: f32,
    /// The tessellated clipped primitives to draw.
    pub paint_jobs: Vec<egui::ClippedPrimitive>,
}

/// State the renderer hands back after a frame: writes that belong to the
/// caller's world, produced instead of performed because the renderer never
/// sees the world.
#[derive(Default)]
pub struct FrameOutputs {
    /// The pixel size of each viewport texture the frame produced.
    pub viewport_texture_sizes: Vec<(u32, u32)>,
    /// Cameras whose forced-render flag the frame consumed.
    pub force_render_cleared: Vec<Entity>,
    /// Whether the frame actually rendered. False when the surface was
    /// occluded, timed out, or lost, so callers keep their previous
    /// viewport state instead of applying this frame's empty outputs.
    pub frame_executed: bool,
}

/// Everything the renderer consumes for one frame.
pub struct RenderInputs {
    /// The scene as the renderer's own ECS world (see [`crate::render_world`]).
    /// Meshes, instances, lights, decals, water, and emitters live here as
    /// components; a host spawns into it or composes it as a member world.
    /// Skinned meshes and the material table stay on [`Self::scene`].
    pub scene_world: nightshade_ecs::dynamic::DynWorld,
    /// User-facing render settings: post-processing, atmosphere, layers.
    pub settings: crate::config::RenderSettings,
    /// Debug visualization toggles.
    pub debug_draw: crate::config::DebugDraw,
    /// The per-frame scene snapshot every pass draws from.
    pub scene: crate::config::RendererState,
    /// Image-based-lighting texture views. Renderer-owned: the frame driver
    /// swaps the persisted views in at frame start, so callers compose this
    /// field as `Default::default()`.
    pub ibl_views: crate::config::IblViews,
    /// Spotlight shadow atlas slot assignment. Renderer-computed at frame
    /// start from the scene's lights, so callers compose this field as
    /// `Default::default()`.
    pub shadow_atlas: crate::wgpu::passes::shadow_depth::atlas::SpotlightAtlasAssignment,
    /// Global wind driving the cloth simulation.
    pub wind: crate::wind::Wind,
    /// Named mesh geometry with dirty tracking.
    pub mesh_cache: crate::mesh_cache::MeshCache,
    /// CPU texture registry: id/refcount/name bookkeeping the host owns.
    pub texture_cache: crate::wgpu::texture_cache::TextureCache,
    /// Live GPU textures. Renderer-owned: the frame driver swaps its persisted
    /// store in at frame start, so callers compose this field as
    /// `Default::default()`.
    pub texture_store: crate::wgpu::texture_cache::TextureStore,
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
    /// Grass rendering settings.
    #[cfg(feature = "grass")]
    pub grass: crate::grass_config::GrassSettings,
    /// Terrain rendering settings.
    #[cfg(feature = "terrain")]
    pub terrain: crate::terrain_config::TerrainSettings,
    /// Per-frame terrain render state.
    #[cfg(feature = "terrain")]
    pub terrain_render: crate::terrain_config::TerrainRenderState,
}

impl CameraFrameInputs {
    /// Builds a perspective camera for one view, computing the reverse-Z
    /// projection the passes expect from `projection`. Resolve
    /// `effective_shading` with [`crate::config::EffectiveShading::from_settings`].
    pub fn perspective(
        entity: Entity,
        view: nalgebra_glm::Mat4,
        position: nalgebra_glm::Vec3,
        projection: CameraProjectionParams,
        effective_shading: crate::config::EffectiveShading,
    ) -> Self {
        Self {
            entity,
            matrices: Some(CameraMatricesInputs {
                view,
                projection: crate::config::reverse_z_perspective(
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
            update_mode: crate::config::ViewportUpdateMode::Always,
            world_transform: None,
            target: None,
        }
    }
}

/// The camera, viewport, and timing for one single-view frame.
pub struct ViewFrame {
    /// The camera to render.
    pub camera: CameraFrameInputs,
    /// The target size in pixels.
    pub viewport: (u32, u32),
    /// Milliseconds elapsed since startup.
    pub uptime_milliseconds: u64,
    /// Seconds elapsed since the previous frame.
    pub delta_time: f32,
}

impl RenderInputs {
    /// Assembles the inputs for a single-view frame from the host-owned state,
    /// filling the renderer's bookkeeping fields with defaults that rebuild the
    /// scene every frame. This is the simplest correct path for a host not
    /// tracking incremental deltas, and it does work proportional to the scene
    /// size every frame, so it suits modest scenes rather than large ones. The
    /// returned struct's public fields (`ui`, `pending_particle_textures`,
    /// `frame.commands`) can be set afterward.
    ///
    /// Most hosts drive this through [`SingleViewHost`] and
    /// [`crate::wgpu::frame::render_single_view`] rather than calling it
    /// directly.
    pub fn single_view(
        scene_world: nightshade_ecs::dynamic::DynWorld,
        scene: crate::config::RendererState,
        mesh_cache: crate::mesh_cache::MeshCache,
        texture_cache: crate::wgpu::texture_cache::TextureCache,
        settings: crate::config::RenderSettings,
        debug_draw: crate::config::DebugDraw,
        frame: ViewFrame,
    ) -> Self {
        let ViewFrame {
            camera,
            viewport,
            uptime_milliseconds,
            delta_time,
        } = frame;
        let shading = camera.effective_shading;
        let active_camera = camera.entity;
        let has_skinned_meshes = !scene.render_skinned_meshes.is_empty();
        Self {
            scene_world,
            settings,
            debug_draw,
            scene,
            ibl_views: Default::default(),
            shadow_atlas: Default::default(),
            wind: crate::wind::Wind::default(),
            mesh_cache,
            texture_cache,
            texture_store: Default::default(),
            pending_particle_textures: Vec::new(),
            ui: UiFrameInputs {
                rects: Vec::new(),
                rect_entities: Vec::new(),
                images: Vec::new(),
                text_meshes: Vec::new(),
                render_slots: Default::default(),
                background_color: None,
            },
            view: ViewInputs {
                active_camera: Some(active_camera),
                uptime_milliseconds,
                delta_time,
                viewport_size: Some(viewport),
                camera_tile_rects: HashMap::new(),
                camera_tile_render_iteration: 0,
                active_viewport_rect: None,
            },
            world_id: 0,
            frame: FrameInputs {
                commands: Vec::new(),
                cameras: Vec::new(),
                active_camera_frame: Some(camera),
                fallback_shading: shading,
                pick_request: None,
                has_skinned_meshes,
                mesh_frame_state: Some(crate::mesh_state::MeshRenderStateInner {
                    full_rebuild_needed: true,
                    ..Default::default()
                }),
                dirty_world_spheres: Vec::new(),
                global_dirty_signal: true,
                mouse_position: nalgebra_glm::Vec2::zeros(),
                force_render_cameras: std::collections::HashSet::new(),
                debug_lines: LinesFrameInputs::default(),
                #[cfg(feature = "egui")]
                egui: None,
            },
            #[cfg(feature = "grass")]
            grass: Default::default(),
            #[cfg(feature = "terrain")]
            terrain: Default::default(),
            #[cfg(feature = "terrain")]
            terrain_render: Default::default(),
        }
    }
}

/// The renderer-facing state a single-view host owns across frames: the scene
/// world of entities, the renderer state, the geometry and texture caches, the
/// frame's settings, and the debug-draw toggles. Build one, spawn into
/// `scene_world`, then drive each frame with
/// [`crate::wgpu::frame::render_single_view`]. The host's state moves into the
/// frame's [`RenderInputs`] and back out, never copied.
///
/// A field added here must also be threaded through [`compose_single_view`] and
/// [`restore_single_view`]; the latter rebuilds the whole struct, so a field
/// left out of it fails to compile rather than silently resetting to default.
pub struct SingleViewHost {
    /// The scene as the renderer's ECS world.
    pub scene_world: nightshade_ecs::dynamic::DynWorld,
    /// The persistent per-frame scene snapshot.
    pub state: crate::config::RendererState,
    /// Named mesh geometry with dirty tracking.
    pub mesh_cache: crate::mesh_cache::MeshCache,
    /// GPU texture registry and lifecycle state.
    pub texture_cache: crate::wgpu::texture_cache::TextureCache,
    /// User-facing render settings.
    pub settings: crate::config::RenderSettings,
    /// Debug visualization toggles.
    pub debug_draw: crate::config::DebugDraw,
}

/// Moves `host`'s persistent state into the inputs for one single-view frame,
/// leaving `host`'s fields empty until [`restore_single_view`] puts them back.
/// Set the returned struct's public fields (`ui`, `pending_particle_textures`,
/// `frame.commands`) before rendering if the host needs them.
pub fn compose_single_view(host: &mut SingleViewHost, frame: ViewFrame) -> RenderInputs {
    RenderInputs::single_view(
        std::mem::replace(
            &mut host.scene_world,
            nightshade_ecs::dynamic::DynWorld::new(),
        ),
        std::mem::take(&mut host.state),
        std::mem::take(&mut host.mesh_cache),
        std::mem::take(&mut host.texture_cache),
        std::mem::take(&mut host.settings),
        std::mem::take(&mut host.debug_draw),
        frame,
    )
}

/// Returns the state [`compose_single_view`] moved out to `host` after the frame
/// has finished with it.
pub fn restore_single_view(host: &mut SingleViewHost, inputs: RenderInputs) {
    *host = SingleViewHost {
        scene_world: inputs.scene_world,
        state: inputs.scene,
        mesh_cache: inputs.mesh_cache,
        texture_cache: inputs.texture_cache,
        settings: inputs.settings,
        debug_draw: inputs.debug_draw,
    };
}
