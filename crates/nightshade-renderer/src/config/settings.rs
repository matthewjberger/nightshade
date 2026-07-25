//! The render settings a program sets: shading and debug modes, the
//! `RenderSettings` block itself, its change signature, and the shading
//! resolution that turns settings into what a pass actually draws.

use super::*;
use serde::{Deserialize, Serialize};

/// Controls how the renderer treats the focused viewport when the
/// camera is in a non-`Always` update mode.
///
/// - `FocusedAlways` (default): the focused viewport (active camera, or
///   the camera tile under the mouse) re-renders every frame regardless
///   of its `ViewportUpdateMode`. Background tiles still respect their
///   own update mode.
/// - `Manual`: every viewport, focused or not, respects its own
///   `ViewportUpdateMode` strictly. Useful when the user wants
///   pixel-perfect control over which tiles render each frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ViewportFocusPolicy {
    /// Focused viewport re-renders every frame, background tiles keep their mode.
    #[default]
    FocusedAlways,
    /// Every viewport respects its own `ViewportUpdateMode`.
    Manual,
}

/// Debug visualization mode for PBR rendering components.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PbrDebugMode {
    /// Normal shading with no debug override.
    #[default]
    None,
    /// Albedo base color channel.
    BaseColor,
    /// Shading normal.
    Normal,
    /// Metallic channel.
    Metallic,
    /// Roughness channel.
    Roughness,
    /// Ambient occlusion channel.
    Occlusion,
    /// Emissive channel.
    Emissive,
    /// Fresnel term of the `PBR` `BRDF`.
    F,
    /// Geometry (visibility) term of the `PBR` `BRDF`.
    G,
    /// Normal distribution term of the `PBR` `BRDF`.
    D,
    /// Diffuse lighting contribution.
    Diffuse,
    /// Specular lighting contribution.
    Specular,
    /// Texture mip level shown as a rainbow ramp.
    MipRainbow,
}

impl PbrDebugMode {
    /// Every debug mode in menu order.
    pub const ALL: &'static [PbrDebugMode] = &[
        PbrDebugMode::None,
        PbrDebugMode::BaseColor,
        PbrDebugMode::Normal,
        PbrDebugMode::Metallic,
        PbrDebugMode::Roughness,
        PbrDebugMode::Occlusion,
        PbrDebugMode::Emissive,
        PbrDebugMode::F,
        PbrDebugMode::G,
        PbrDebugMode::D,
        PbrDebugMode::Diffuse,
        PbrDebugMode::Specular,
        PbrDebugMode::MipRainbow,
    ];

    /// Human-readable label for the mode.
    pub fn name(&self) -> &'static str {
        match self {
            PbrDebugMode::None => "None",
            PbrDebugMode::BaseColor => "Base Color",
            PbrDebugMode::Normal => "Normal",
            PbrDebugMode::Metallic => "Metallic",
            PbrDebugMode::Roughness => "Roughness",
            PbrDebugMode::Occlusion => "Occlusion",
            PbrDebugMode::Emissive => "Emissive",
            PbrDebugMode::F => "Fresnel (F)",
            PbrDebugMode::G => "Geometry (G)",
            PbrDebugMode::D => "Distribution (D)",
            PbrDebugMode::Diffuse => "Diffuse",
            PbrDebugMode::Specular => "Specular",
            PbrDebugMode::MipRainbow => "Mip Rainbow",
        }
    }

    /// Shader mode index passed to the fragment shader.
    pub fn as_u32(&self) -> u32 {
        match self {
            PbrDebugMode::None => 0,
            PbrDebugMode::BaseColor => 1,
            PbrDebugMode::Normal => 2,
            PbrDebugMode::Metallic => 3,
            PbrDebugMode::Roughness => 4,
            PbrDebugMode::Occlusion => 5,
            PbrDebugMode::Emissive => 6,
            PbrDebugMode::F => 7,
            PbrDebugMode::G => 8,
            PbrDebugMode::D => 9,
            PbrDebugMode::Diffuse => 10,
            PbrDebugMode::Specular => 11,
            PbrDebugMode::MipRainbow => 12,
        }
    }
}

/// PS1-style vertex snapping for retro rendering.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct VertexSnap {
    /// Grid resolution to snap vertices to (lower = more pixelated).
    pub resolution: [f32; 2],
}

impl Default for VertexSnap {
    fn default() -> Self {
        Self {
            resolution: [160.0, 120.0],
        }
    }
}

/// User-tunable rendering settings. This is the subset of former
/// `Graphics` state that materially controls how a scene is shaded and
/// is safe to persist. Carried in `RenderInputs::settings`.
#[derive(Clone, Serialize, Deserialize)]
pub struct RenderSettings {
    /// Software frame rate cap in frames per second. `None` lets the
    /// present mode pace the loop (typically vsync or unbounded under
    /// Mailbox). `Some(fps)` sleeps after each frame to hold the loop
    /// near the target rate.
    ///
    /// On macOS this defaults to `refresh_rate - 1` at window creation
    /// (and re-applies when the window moves to a monitor with a
    /// different refresh rate) when the value still matches what the
    /// engine auto-applied.
    pub frame_rate_limit: Option<f32>,
    /// Active skybox/environment map.
    pub atmosphere: Atmosphere,
    /// When false, the sky pass is skipped so the atmosphere does not render
    /// to the color buffer. IBL contributions are unaffected.
    pub show_sky: bool,
    /// Enable world render layer.
    pub render_layer_world_enabled: bool,
    /// Enable overlay render layer.
    pub render_layer_overlay_enabled: bool,
    /// When true, the world renders to the swapchain even when no viewport tile is requesting
    /// a camera. Default true, which is the normal "game" behaviour: the world fills the window
    /// and any retained UI overlays on top of it. UI-first apps (asset viewers, gallery demos)
    /// can set this to false so an idle frame skips all scene passes and only paints the UI.
    pub render_world_to_swapchain: bool,
    /// Background clear color.
    pub clear_color: [f32; 4],
    /// Override UI scale factor (None = automatic).
    pub ui_scale: Option<f32>,
    /// Disable all lighting calculations.
    pub unlit_mode: bool,
    /// PS1-style vertex snapping settings.
    pub vertex_snap: Option<VertexSnap>,
    /// PS1-style affine texture mapping.
    pub affine_texture_mapping: bool,
    /// Anisotropic filtering clamp for material textures (1 = disabled,
    /// max 16). Default 16 for crisp PBR textures at grazing angles.
    pub material_anisotropy_filtering: u16,
    /// Distance fog settings.
    pub fog: Option<Fog>,
    /// Color grading and tonemapping.
    pub color_grading: ColorGrading,
    /// Enable bloom post-processing.
    pub bloom_enabled: bool,
    /// Bloom intensity multiplier.
    pub bloom_intensity: f32,
    /// Brightness threshold for bloom extraction (pixels below this luminance are excluded).
    pub bloom_threshold: f32,
    /// Soft-knee width around `bloom_threshold` (0 = hard cut).
    pub bloom_knee: f32,
    /// Upsample blur radius in normalized UV units.
    pub bloom_filter_radius: f32,
    /// Depth of field settings.
    pub depth_of_field: DepthOfField,
    /// Enable screen-space ambient occlusion.
    pub ssao_enabled: bool,
    /// SSAO sample radius in world units.
    pub ssao_radius: f32,
    /// SSAO depth bias to reduce self-occlusion.
    pub ssao_bias: f32,
    /// SSAO darkening intensity.
    pub ssao_intensity: f32,
    /// Number of `SSAO` samples per pixel.
    pub ssao_sample_count: u32,
    /// Depth difference above which the `SSAO` blur stops crossing an edge.
    pub ssao_blur_depth_threshold: f32,
    /// Normal-similarity exponent guiding the `SSAO` blur across edges.
    pub ssao_blur_normal_power: f32,
    /// Enable screen-space global illumination.
    pub ssgi_enabled: bool,
    /// `SSGI` gather radius in world units.
    pub ssgi_radius: f32,
    /// `SSGI` bounce-light intensity.
    pub ssgi_intensity: f32,
    /// Maximum ray-march steps per `SSGI` sample.
    pub ssgi_max_steps: u32,
    /// Enable screen-space reflections.
    pub ssr_enabled: bool,
    /// Maximum ray-march steps per `SSR` ray.
    pub ssr_max_steps: u32,
    /// Depth thickness a `SSR` ray treats as a valid hit, in world units.
    pub ssr_thickness: f32,
    /// Maximum `SSR` ray distance in world units.
    pub ssr_max_distance: f32,
    /// `SSR` ray-march step length in texels.
    pub ssr_stride: f32,
    /// Screen-edge fraction where `SSR` reflections begin to fade.
    pub ssr_fade_start: f32,
    /// Screen-edge fraction where `SSR` reflections fully fade out.
    pub ssr_fade_end: f32,
    /// `SSR` reflection intensity.
    pub ssr_intensity: f32,
    /// Global ambient light color and intensity.
    pub ambient_light: [f32; 4],
    /// Enables temporal antialiasing. On by default; it resolves screen-space
    /// sampling noise (soft shadows and the like) that otherwise bands on smooth
    /// surfaces. When off, the temporal resolve passes the image through
    /// unchanged and camera jitter is disabled.
    pub taa_enabled: bool,
    /// Temporal resolve blend toward the current frame each frame. Lower keeps
    /// more history (steadier, more ghost-prone), higher favors the current
    /// frame (sharper, noisier).
    pub taa_blend: f32,
    /// Post-resolve sharpening strength applied to the displayed image only.
    pub taa_sharpness: f32,
    /// Master switch for the water surface pass. The pass also no-ops when no
    /// `Water` entities exist, so this only matters when bodies are present.
    pub water_enabled: bool,
    /// When true, the swapchain runs in `Fifo` (vsync). When false, the
    /// renderer picks the lowest-latency present mode the adapter
    /// supports. The renderer compares this against the active swapchain
    /// configuration each frame and reconfigures live. No restart required.
    pub vsync_enabled: bool,
    /// Per-camera render-resolution multiplier. Clamped to `[0.25, 4.0]` at use.
    pub render_scale: f32,
    /// IBL blend factor for interpolating between snapshot cubemaps (0.0 - 1.0).
    pub ibl_blend_factor: f32,
}

/// Debug visualization toggles. Carried in `RenderInputs::debug_draw`.
#[derive(Clone)]
pub struct DebugDraw {
    /// Show debug grid overlay.
    pub show_grid: bool,
    /// Show bounding volumes for all entities.
    pub show_bounding_volumes: bool,
    /// Show bounding volume for selected entity only.
    pub show_selected_bounding_volume: bool,
    /// Show vertex normal debug lines.
    pub show_normals: bool,
    /// Length of normal debug lines.
    pub normal_line_length: f32,
    /// Color for normal debug lines.
    pub normal_line_color: [f32; 4],
    /// PBR debug visualization mode.
    pub pbr_debug_mode: PbrDebugMode,
    /// Show animated debug stripes on textures.
    pub texture_debug_stripes: bool,
    /// Animation speed for debug stripes.
    pub texture_debug_stripes_speed: f32,
    /// Show the raw `SSAO` buffer instead of the shaded scene.
    pub ssao_visualization: bool,
    /// When false, material textures sample at mip 0 with no anisotropy,
    /// reproducing the unfiltered look for comparison.
    pub material_filtering: bool,
    /// When true, the meshlet resolve tints every pixel by the cluster it
    /// rasterized from instead of shading it, exposing the decomposition the
    /// bake produced.
    pub meshlet_cluster_visualization: bool,
    /// Screen-space error, in pixels, a meshlet cluster may introduce before
    /// the level of detail it belongs to is rejected for a finer one. One pixel
    /// is where the seam between levels stops being visible. Lower holds a finer
    /// cut everywhere, which is how the raster is put under load without
    /// changing the scene; higher trades visible error for fewer clusters.
    pub meshlet_lod_error_threshold: f32,
    /// When true, every meshlet cull picks its cut against the active camera's
    /// frustum and distance rather than the camera being rendered. The active
    /// camera is unchanged, since the frozen view is its own; any other camera
    /// then draws exactly the clusters the active camera kept, from wherever it
    /// stands, so its frustum and level of detail show as geometry in space.
    pub meshlet_freeze_cull_to_active: bool,
    /// When true, the meshlet cull drops clusters the previous frame's depth
    /// already covered, tested against a pyramid of that depth. On by default,
    /// since occlusion is part of what makes virtualized geometry affordable;
    /// off is for seeing what it removes.
    pub meshlet_occlusion_culling: bool,
}

/// Hash of every setting that materially affects what a rendered tile
/// looks like, spanning the render and debug buckets plus the resolved
/// selection-outline coverage. Fields that
/// only affect performance, window chrome, or day/night driver state are
/// excluded. The renderer compares this signature against the previous
/// frame's value and bumps `RendererState::settings_version` when they differ.
pub fn render_settings_signature(
    render: &RenderSettings,
    debug: &DebugDraw,
    outline_ids: &[u32],
    outline_color: [f32; 4],
) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    outline_ids.hash(&mut hasher);
    for component in outline_color {
        component.to_bits().hash(&mut hasher);
    }
    debug.show_grid.hash(&mut hasher);
    debug.show_bounding_volumes.hash(&mut hasher);
    debug.show_normals.hash(&mut hasher);
    debug.normal_line_length.to_bits().hash(&mut hasher);
    for component in debug.normal_line_color {
        component.to_bits().hash(&mut hasher);
    }
    (render.atmosphere as u32).hash(&mut hasher);
    render.show_sky.hash(&mut hasher);
    render.render_layer_world_enabled.hash(&mut hasher);
    render.render_layer_overlay_enabled.hash(&mut hasher);
    render.render_world_to_swapchain.hash(&mut hasher);
    for component in render.clear_color {
        component.to_bits().hash(&mut hasher);
    }
    render.unlit_mode.hash(&mut hasher);
    render.vertex_snap.is_some().hash(&mut hasher);
    if let Some(ref snap) = render.vertex_snap {
        for component in snap.resolution {
            component.to_bits().hash(&mut hasher);
        }
    }
    render.affine_texture_mapping.hash(&mut hasher);
    render.fog.is_some().hash(&mut hasher);
    if let Some(ref fog) = render.fog {
        for component in fog.color {
            component.to_bits().hash(&mut hasher);
        }
        fog.start.to_bits().hash(&mut hasher);
        fog.end.to_bits().hash(&mut hasher);
    }
    let cg = &render.color_grading;
    cg.exposure.to_bits().hash(&mut hasher);
    cg.exposure_compensation_ev.to_bits().hash(&mut hasher);
    cg.auto_exposure.hash(&mut hasher);
    cg.auto_exposure_target.to_bits().hash(&mut hasher);
    cg.auto_exposure_rate.to_bits().hash(&mut hasher);
    cg.auto_exposure_min_ev.to_bits().hash(&mut hasher);
    cg.auto_exposure_max_ev.to_bits().hash(&mut hasher);
    cg.gamma.to_bits().hash(&mut hasher);
    cg.saturation.to_bits().hash(&mut hasher);
    cg.brightness.to_bits().hash(&mut hasher);
    cg.contrast.to_bits().hash(&mut hasher);
    (cg.tonemap_algorithm as u32).hash(&mut hasher);
    render.bloom_enabled.hash(&mut hasher);
    render.bloom_intensity.to_bits().hash(&mut hasher);
    render.bloom_threshold.to_bits().hash(&mut hasher);
    render.bloom_knee.to_bits().hash(&mut hasher);
    render.bloom_filter_radius.to_bits().hash(&mut hasher);
    let dof = &render.depth_of_field;
    dof.enabled.hash(&mut hasher);
    dof.focus_distance.to_bits().hash(&mut hasher);
    dof.focus_range.to_bits().hash(&mut hasher);
    dof.max_blur_radius.to_bits().hash(&mut hasher);
    dof.bokeh_threshold.to_bits().hash(&mut hasher);
    dof.bokeh_intensity.to_bits().hash(&mut hasher);
    (dof.quality as u32).hash(&mut hasher);
    dof.visualize_coc.hash(&mut hasher);
    dof.tilt_shift_enabled.hash(&mut hasher);
    dof.tilt_shift_angle.to_bits().hash(&mut hasher);
    dof.tilt_shift_center.to_bits().hash(&mut hasher);
    dof.tilt_shift_blur_amount.to_bits().hash(&mut hasher);
    dof.visualize_tilt_shift.hash(&mut hasher);
    render.ssao_enabled.hash(&mut hasher);
    render.ssao_radius.to_bits().hash(&mut hasher);
    render.ssao_bias.to_bits().hash(&mut hasher);
    render.ssao_intensity.to_bits().hash(&mut hasher);
    render.ssao_sample_count.hash(&mut hasher);
    debug.ssao_visualization.hash(&mut hasher);
    render.ssao_blur_depth_threshold.to_bits().hash(&mut hasher);
    render.ssao_blur_normal_power.to_bits().hash(&mut hasher);
    render.ssgi_enabled.hash(&mut hasher);
    render.ssgi_radius.to_bits().hash(&mut hasher);
    render.ssgi_intensity.to_bits().hash(&mut hasher);
    render.ssgi_max_steps.hash(&mut hasher);
    render.ssr_enabled.hash(&mut hasher);
    render.ssr_max_steps.hash(&mut hasher);
    render.ssr_thickness.to_bits().hash(&mut hasher);
    render.ssr_max_distance.to_bits().hash(&mut hasher);
    render.ssr_stride.to_bits().hash(&mut hasher);
    render.ssr_fade_start.to_bits().hash(&mut hasher);
    render.ssr_fade_end.to_bits().hash(&mut hasher);
    render.ssr_intensity.to_bits().hash(&mut hasher);
    for component in render.ambient_light {
        component.to_bits().hash(&mut hasher);
    }
    debug.pbr_debug_mode.as_u32().hash(&mut hasher);
    debug.texture_debug_stripes.hash(&mut hasher);
    debug
        .texture_debug_stripes_speed
        .to_bits()
        .hash(&mut hasher);
    debug.material_filtering.hash(&mut hasher);
    debug.meshlet_cluster_visualization.hash(&mut hasher);
    debug
        .meshlet_lod_error_threshold
        .to_bits()
        .hash(&mut hasher);
    render.taa_enabled.hash(&mut hasher);
    render.render_scale.to_bits().hash(&mut hasher);
    render.ibl_blend_factor.to_bits().hash(&mut hasher);
    hasher.finish()
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            frame_rate_limit: None,
            atmosphere: Atmosphere::None,
            show_sky: true,
            render_layer_world_enabled: true,
            render_layer_overlay_enabled: true,
            render_world_to_swapchain: true,
            clear_color: [0.0, 0.0, 0.0, 1.0],
            ui_scale: None,
            unlit_mode: false,
            vertex_snap: None,
            affine_texture_mapping: false,
            material_anisotropy_filtering: 16,
            fog: None,
            color_grading: ColorGrading::default(),
            bloom_enabled: true,
            bloom_intensity: 0.04,
            bloom_threshold: 1.0,
            bloom_knee: 0.5,
            bloom_filter_radius: 0.005,
            depth_of_field: DepthOfField::default(),
            ssao_enabled: false,
            ssao_radius: 0.5,
            ssao_bias: 0.025,
            ssao_intensity: 1.0,
            ssao_sample_count: 64,
            ssao_blur_depth_threshold: 0.005,
            ssao_blur_normal_power: 8.0,
            ssgi_enabled: false,
            ssgi_radius: 2.0,
            ssgi_intensity: 1.0,
            ssgi_max_steps: 16,
            ssr_enabled: false,
            ssr_max_steps: 64,
            ssr_thickness: 0.3,
            ssr_max_distance: 50.0,
            ssr_stride: 1.0,
            ssr_fade_start: 0.8,
            ssr_fade_end: 1.0,
            ssr_intensity: 1.0,
            ambient_light: [0.1, 0.1, 0.1, 1.0],
            taa_enabled: true,
            taa_blend: 0.12,
            taa_sharpness: 0.5,
            water_enabled: true,
            vsync_enabled: true,
            render_scale: 1.0,
            ibl_blend_factor: 0.0,
        }
    }
}

impl Default for DebugDraw {
    fn default() -> Self {
        Self {
            show_grid: false,
            show_bounding_volumes: false,
            show_selected_bounding_volume: false,
            show_normals: false,
            normal_line_length: 0.1,
            normal_line_color: [0.0, 1.0, 0.0, 1.0],
            pbr_debug_mode: PbrDebugMode::None,
            texture_debug_stripes: false,
            texture_debug_stripes_speed: 100.0,
            ssao_visualization: false,
            material_filtering: true,
            meshlet_cluster_visualization: false,
            meshlet_lod_error_threshold: 1.0,
            meshlet_freeze_cull_to_active: false,
            meshlet_occlusion_culling: true,
        }
    }
}

/// Neutral grey the flat-shading debug mode paints surfaces with.
pub const FLAT_SHADING_COLOR: nalgebra_glm::Vec4 = nalgebra_glm::Vec4::new(0.72, 0.72, 0.72, 1.0);

/// Update mode for a camera's viewport tile.
///
/// - `Always`: re-render every frame regardless of dirty state.
/// - `WhenVisible`: same as `Always` for now (any dispatched camera is
///   treated as visible). Reserved for future visibility/focus tracking.
/// - `WhenDirty`: re-render only when the camera moved, when first becoming
///   visible (no cached frame yet), or when the frame's coarse dirty signal
///   indicates a transform/scene mutation. Otherwise blit the cached
///   viewport texture from the previous render.
/// - `Once`: render exactly the first time the camera is visible, then
///   reuse the cached texture forever.
/// - `Disabled`: never render after the first frame; the cached texture
///   is whatever the first render produced (or zero-initialized memory if
///   the camera was never rendered).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ViewportUpdateMode {
    /// Re-render every frame.
    #[default]
    Always,
    /// Re-render every frame while the camera is dispatched.
    WhenVisible,
    /// Re-render only on camera or scene change, else blit the cached frame.
    WhenDirty,
    /// Render once on first visibility, then reuse the cached frame.
    Once,
    /// Never render after the first frame.
    Disabled,
}

/// Sentinel `flat_color` used to tell the mesh fragment shaders that the
/// camera is in wireframe mode and that the surface should be discarded.
/// `flat_color.a >= 2.0` is impossible for a regular color and is reserved
/// for this purpose.
pub const WIREFRAME_SENTINEL_COLOR: nalgebra_glm::Vec4 =
    nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 2.0);

/// Per-camera shading the render passes read, resolved from `RenderSettings`
/// and `DebugDraw` plus any per-camera overrides.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffectiveShading {
    /// Disable all lighting for this view.
    pub unlit_mode: bool,
    /// Enable bloom for this view.
    pub bloom_enabled: bool,
    /// Enable `SSAO` for this view.
    pub ssao_enabled: bool,
    /// Enable `SSGI` for this view.
    pub ssgi_enabled: bool,
    /// Enable `SSR` for this view.
    pub ssr_enabled: bool,
    /// Draw vertex normal debug lines.
    pub show_normals: bool,
    /// Draw bounding volume debug outlines.
    pub show_bounding_volumes: bool,
    /// Render surfaces as wireframe.
    pub show_wireframe: bool,
    /// Draw the selection outline for tagged entities.
    pub selection_outline_enabled: bool,
    /// Flat debug color painted over surfaces, `None` for normal shading.
    pub flat_shading_color: Option<nalgebra_glm::Vec4>,
    /// Render the shadow depth passes for this view.
    pub shadow_depth_enabled: bool,
    /// Run the debug line pass for this view.
    pub lines_enabled: bool,
    /// Color grading and tonemapping for this view.
    pub color_grading: ColorGrading,
    /// Depth of field settings for this view.
    pub depth_of_field: DepthOfField,
    /// Bloom intensity multiplier.
    pub bloom_intensity: f32,
    /// Bloom luminance extraction threshold.
    pub bloom_threshold: f32,
    /// Soft-knee width around the bloom threshold.
    pub bloom_knee: f32,
    /// Bloom upsample blur radius in normalized UV units.
    pub bloom_filter_radius: f32,
    /// `SSAO` sample radius in world units.
    pub ssao_radius: f32,
    /// `SSAO` depth bias.
    pub ssao_bias: f32,
    /// `SSAO` darkening intensity.
    pub ssao_intensity: f32,
    /// `SSAO` samples per pixel.
    pub ssao_sample_count: u32,
    /// Show the raw `SSAO` buffer.
    pub ssao_visualization: bool,
    /// Depth threshold guiding the `SSAO` blur across edges.
    pub ssao_blur_depth_threshold: f32,
    /// Normal-similarity exponent guiding the `SSAO` blur.
    pub ssao_blur_normal_power: f32,
    /// `SSGI` gather radius in world units.
    pub ssgi_radius: f32,
    /// `SSGI` bounce-light intensity.
    pub ssgi_intensity: f32,
    /// Maximum ray-march steps per `SSGI` sample.
    pub ssgi_max_steps: u32,
    /// Maximum ray-march steps per `SSR` ray.
    pub ssr_max_steps: u32,
    /// Depth thickness a `SSR` ray treats as a hit, in world units.
    pub ssr_thickness: f32,
    /// Maximum `SSR` ray distance in world units.
    pub ssr_max_distance: f32,
    /// `SSR` step length in texels.
    pub ssr_stride: f32,
    /// Screen-edge fraction where `SSR` reflections begin to fade.
    pub ssr_fade_start: f32,
    /// Screen-edge fraction where `SSR` reflections fully fade.
    pub ssr_fade_end: f32,
    /// `SSR` reflection intensity.
    pub ssr_intensity: f32,
    /// Global ambient light color and intensity.
    pub ambient_light: [f32; 4],
    /// Bitmask of "culling layers" this view will render. Compared
    /// against each entity's `CullingMask`
    /// at render-collection time; an entity is skipped when
    /// `(entity_layers & culling_mask) == 0`. Defaults to `!0` (all
    /// layers) so cameras without a `CameraCullingMask` component
    /// continue to render every entity.
    pub culling_mask: u32,
    /// Resolved fog state for this view. `None` disables fog; `Some`
    /// uses the contained Fog parameters. Inherits from `Graphics::fog`
    /// unless the camera carries a `CameraEnvironment` override.
    pub fog: Option<Fog>,
    /// Resolved atmosphere selection for this view, used by the sky
    /// pass and the IBL bracket selection. Defaults to
    /// `Graphics::atmosphere` and can be overridden per camera via
    /// `CameraEnvironment`.
    pub atmosphere: Atmosphere,
}

impl EffectiveShading {
    /// Derives the per-camera shading the render passes actually read from the
    /// user-facing [`RenderSettings`] and [`DebugDraw`]. A host sets
    /// `RenderSettings` and lets the renderer resolve this, rather than filling
    /// every field by hand.
    pub fn from_settings(render: &RenderSettings, debug: &DebugDraw) -> Self {
        Self {
            unlit_mode: render.unlit_mode,
            bloom_enabled: render.bloom_enabled,
            ssao_enabled: render.ssao_enabled,
            ssgi_enabled: render.ssgi_enabled,
            ssr_enabled: render.ssr_enabled,
            show_normals: debug.show_normals,
            show_bounding_volumes: debug.show_bounding_volumes,
            show_wireframe: false,
            selection_outline_enabled: false,
            flat_shading_color: None,
            shadow_depth_enabled: true,
            lines_enabled: debug.show_normals || debug.show_bounding_volumes,
            color_grading: render.color_grading,
            depth_of_field: render.depth_of_field,
            bloom_intensity: render.bloom_intensity,
            bloom_threshold: render.bloom_threshold,
            bloom_knee: render.bloom_knee,
            bloom_filter_radius: render.bloom_filter_radius,
            ssao_radius: render.ssao_radius,
            ssao_bias: render.ssao_bias,
            ssao_intensity: render.ssao_intensity,
            ssao_sample_count: render.ssao_sample_count,
            ssao_visualization: debug.ssao_visualization,
            ssao_blur_depth_threshold: render.ssao_blur_depth_threshold,
            ssao_blur_normal_power: render.ssao_blur_normal_power,
            ssgi_radius: render.ssgi_radius,
            ssgi_intensity: render.ssgi_intensity,
            ssgi_max_steps: render.ssgi_max_steps,
            ssr_max_steps: render.ssr_max_steps,
            ssr_thickness: render.ssr_thickness,
            ssr_max_distance: render.ssr_max_distance,
            ssr_stride: render.ssr_stride,
            ssr_fade_start: render.ssr_fade_start,
            ssr_fade_end: render.ssr_fade_end,
            ssr_intensity: render.ssr_intensity,
            ambient_light: render.ambient_light,
            culling_mask: !0,
            fog: render.fog,
            atmosphere: render.atmosphere,
        }
    }
}

impl Default for EffectiveShading {
    fn default() -> Self {
        Self {
            unlit_mode: false,
            bloom_enabled: true,
            ssao_enabled: false,
            ssgi_enabled: false,
            ssr_enabled: false,
            show_normals: false,
            show_bounding_volumes: false,
            show_wireframe: false,
            selection_outline_enabled: false,
            flat_shading_color: None,
            shadow_depth_enabled: true,
            lines_enabled: false,
            color_grading: ColorGrading::default(),
            depth_of_field: DepthOfField::default(),
            bloom_intensity: 0.5,
            bloom_threshold: 1.0,
            bloom_knee: 0.5,
            bloom_filter_radius: 0.005,
            ssao_radius: 0.5,
            ssao_bias: 0.025,
            ssao_intensity: 1.0,
            ssao_sample_count: 64,
            ssao_visualization: false,
            ssao_blur_depth_threshold: 0.005,
            ssao_blur_normal_power: 8.0,
            ssgi_radius: 2.0,
            ssgi_intensity: 1.0,
            ssgi_max_steps: 16,
            ssr_max_steps: 64,
            ssr_thickness: 0.3,
            ssr_max_distance: 50.0,
            ssr_stride: 1.0,
            ssr_fade_start: 0.8,
            ssr_fade_end: 1.0,
            ssr_intensity: 1.0,
            ambient_light: [0.1, 0.1, 0.1, 1.0],
            culling_mask: !0,
            fog: None,
            atmosphere: Atmosphere::None,
        }
    }
}
