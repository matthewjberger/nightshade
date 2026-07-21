//! Graphics settings resources.

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

/// Quality level for depth of field effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DepthOfFieldQuality {
    /// 8 samples per pixel.
    Low,
    /// 16 samples per pixel.
    #[default]
    Medium,
    /// 32 samples per pixel.
    High,
}

impl DepthOfFieldQuality {
    /// Every quality level in ascending order.
    pub const ALL: &'static [DepthOfFieldQuality] = &[
        DepthOfFieldQuality::Low,
        DepthOfFieldQuality::Medium,
        DepthOfFieldQuality::High,
    ];

    /// Human-readable label for the quality level.
    pub fn name(&self) -> &'static str {
        match self {
            DepthOfFieldQuality::Low => "Low",
            DepthOfFieldQuality::Medium => "Medium",
            DepthOfFieldQuality::High => "High",
        }
    }

    /// Samples per pixel for this quality level.
    pub fn sample_count(&self) -> u32 {
        match self {
            DepthOfFieldQuality::Low => 8,
            DepthOfFieldQuality::Medium => 16,
            DepthOfFieldQuality::High => 32,
        }
    }
}

/// Depth of field post-processing settings.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DepthOfField {
    /// Whether DOF is active.
    pub enabled: bool,
    /// Distance to the focal plane in world units.
    pub focus_distance: f32,
    /// Range around focus distance that remains sharp.
    pub focus_range: f32,
    /// Maximum blur radius in pixels.
    pub max_blur_radius: f32,
    /// Brightness threshold for bokeh highlights.
    pub bokeh_threshold: f32,
    /// Intensity multiplier for bokeh highlights.
    pub bokeh_intensity: f32,
    /// Sample count quality level.
    pub quality: DepthOfFieldQuality,
    /// Debug visualization of circle of confusion.
    pub visualize_coc: bool,
    /// Enable tilt-shift miniature effect.
    pub tilt_shift_enabled: bool,
    /// Angle of the tilt-shift band in radians.
    pub tilt_shift_angle: f32,
    /// Vertical position of the sharp band center (-1 to 1).
    pub tilt_shift_center: f32,
    /// Blur strength outside the sharp band.
    pub tilt_shift_blur_amount: f32,
    /// Debug visualization of tilt-shift mask.
    pub visualize_tilt_shift: bool,
}

impl Default for DepthOfField {
    fn default() -> Self {
        Self {
            enabled: false,
            focus_distance: 10.0,
            focus_range: 5.0,
            max_blur_radius: 8.0,
            bokeh_threshold: 0.8,
            bokeh_intensity: 1.0,
            quality: DepthOfFieldQuality::Medium,
            visualize_coc: false,
            tilt_shift_enabled: false,
            tilt_shift_angle: 0.0,
            tilt_shift_center: 0.0,
            tilt_shift_blur_amount: 1.0,
            visualize_tilt_shift: false,
        }
    }
}

impl DepthOfField {
    /// Preset for close-up portraits with strong background blur.
    pub fn portrait() -> Self {
        Self {
            enabled: true,
            focus_distance: 3.0,
            focus_range: 1.5,
            max_blur_radius: 12.0,
            bokeh_threshold: 0.6,
            bokeh_intensity: 1.2,
            quality: DepthOfFieldQuality::High,
            visualize_coc: false,
            tilt_shift_enabled: false,
            tilt_shift_angle: 0.0,
            tilt_shift_center: 0.0,
            tilt_shift_blur_amount: 1.0,
            visualize_tilt_shift: false,
        }
    }

    /// Preset for cinematic medium-distance focus.
    pub fn cinematic() -> Self {
        Self {
            enabled: true,
            focus_distance: 8.0,
            focus_range: 4.0,
            max_blur_radius: 10.0,
            bokeh_threshold: 0.7,
            bokeh_intensity: 1.0,
            quality: DepthOfFieldQuality::Medium,
            visualize_coc: false,
            tilt_shift_enabled: false,
            tilt_shift_angle: 0.0,
            tilt_shift_center: 0.0,
            tilt_shift_blur_amount: 1.0,
            visualize_tilt_shift: false,
        }
    }

    /// Preset for extreme close-ups with a razor-thin focal plane.
    pub fn macro_shot() -> Self {
        Self {
            enabled: true,
            focus_distance: 0.5,
            focus_range: 0.2,
            max_blur_radius: 16.0,
            bokeh_threshold: 0.5,
            bokeh_intensity: 1.5,
            quality: DepthOfFieldQuality::High,
            visualize_coc: false,
            tilt_shift_enabled: false,
            tilt_shift_angle: 0.0,
            tilt_shift_center: 0.0,
            tilt_shift_blur_amount: 1.0,
            visualize_tilt_shift: false,
        }
    }

    /// Preset for distant scenes with a wide sharp range.
    pub fn landscape() -> Self {
        Self {
            enabled: true,
            focus_distance: 50.0,
            focus_range: 100.0,
            max_blur_radius: 4.0,
            bokeh_threshold: 0.9,
            bokeh_intensity: 0.5,
            quality: DepthOfFieldQuality::Low,
            visualize_coc: false,
            tilt_shift_enabled: false,
            tilt_shift_angle: 0.0,
            tilt_shift_center: 0.0,
            tilt_shift_blur_amount: 1.0,
            visualize_tilt_shift: false,
        }
    }

    /// Preset that enables the tilt-shift miniature band.
    pub fn tilt_shift() -> Self {
        Self {
            enabled: true,
            focus_distance: 10.0,
            focus_range: 5.0,
            max_blur_radius: 12.0,
            bokeh_threshold: 0.8,
            bokeh_intensity: 0.8,
            quality: DepthOfFieldQuality::Medium,
            visualize_coc: false,
            tilt_shift_enabled: true,
            tilt_shift_angle: 0.0,
            tilt_shift_center: 0.0,
            tilt_shift_blur_amount: 1.0,
            visualize_tilt_shift: false,
        }
    }
}

/// HDR to LDR tonemapping algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TonemapAlgorithm {
    /// Academy Color Encoding System (Narkowicz approximation).
    #[default]
    Aces,
    /// Fitted ACES (Stephen Hill) with AP1 color transforms.
    Aces2,
    /// Simple Reinhard curve.
    Reinhard,
    /// Extended Reinhard with white point.
    ReinhardExtended,
    /// Filmic curve from Uncharted 2.
    Uncharted2,
    /// AgX display transform.
    AgX,
    /// Neutral (minimal color shift).
    Neutral,
    /// No tonemapping (clamp only).
    None,
}

impl TonemapAlgorithm {
    /// Every tonemapping algorithm in menu order.
    pub const ALL: &'static [TonemapAlgorithm] = &[
        TonemapAlgorithm::Aces,
        TonemapAlgorithm::Aces2,
        TonemapAlgorithm::Reinhard,
        TonemapAlgorithm::ReinhardExtended,
        TonemapAlgorithm::Uncharted2,
        TonemapAlgorithm::AgX,
        TonemapAlgorithm::Neutral,
        TonemapAlgorithm::None,
    ];

    /// Shader algorithm index passed to the tonemapping pass.
    pub fn as_u32(&self) -> u32 {
        match self {
            TonemapAlgorithm::Aces => 0,
            TonemapAlgorithm::Aces2 => 7,
            TonemapAlgorithm::Reinhard => 1,
            TonemapAlgorithm::ReinhardExtended => 2,
            TonemapAlgorithm::Uncharted2 => 3,
            TonemapAlgorithm::AgX => 4,
            TonemapAlgorithm::Neutral => 5,
            TonemapAlgorithm::None => 6,
        }
    }

    /// Human-readable label for the algorithm.
    pub fn name(&self) -> &'static str {
        match self {
            TonemapAlgorithm::Aces => "ACES",
            TonemapAlgorithm::Aces2 => "ACES (Fitted)",
            TonemapAlgorithm::Reinhard => "Reinhard",
            TonemapAlgorithm::ReinhardExtended => "Reinhard Extended",
            TonemapAlgorithm::Uncharted2 => "Uncharted 2",
            TonemapAlgorithm::AgX => "AgX",
            TonemapAlgorithm::Neutral => "Neutral",
            TonemapAlgorithm::None => "None",
        }
    }
}

/// Pre-configured color grading style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ColorGradingPreset {
    /// Neutral default settings.
    #[default]
    Default,
    /// High saturation and brightness.
    Vibrant,
    /// Film-like with lifted blacks.
    Cinematic,
    /// Low saturation and contrast.
    Muted,
    /// Strong contrast with deep blacks.
    HighContrast,
    /// Orange-shifted warm tones.
    Warm,
    /// Blue-shifted cool tones.
    Cool,
    /// Faded vintage look.
    Retro,
    /// Nearly black and white.
    Desaturated,
    /// User-defined values.
    Custom,
}

impl ColorGradingPreset {
    /// Every color grading preset in menu order.
    pub const ALL: &'static [ColorGradingPreset] = &[
        ColorGradingPreset::Default,
        ColorGradingPreset::Vibrant,
        ColorGradingPreset::Cinematic,
        ColorGradingPreset::Muted,
        ColorGradingPreset::HighContrast,
        ColorGradingPreset::Warm,
        ColorGradingPreset::Cool,
        ColorGradingPreset::Retro,
        ColorGradingPreset::Desaturated,
        ColorGradingPreset::Custom,
    ];

    /// Human-readable label for the preset.
    pub fn name(&self) -> &'static str {
        match self {
            ColorGradingPreset::Default => "Default",
            ColorGradingPreset::Vibrant => "Vibrant",
            ColorGradingPreset::Cinematic => "Cinematic",
            ColorGradingPreset::Muted => "Muted",
            ColorGradingPreset::HighContrast => "High Contrast",
            ColorGradingPreset::Warm => "Warm",
            ColorGradingPreset::Cool => "Cool",
            ColorGradingPreset::Retro => "Retro",
            ColorGradingPreset::Desaturated => "Desaturated",
            ColorGradingPreset::Custom => "Custom",
        }
    }

    /// Expands the preset into a full `ColorGrading` value.
    pub fn to_color_grading(&self) -> ColorGrading {
        let preset = *self;
        match self {
            ColorGradingPreset::Default => ColorGrading {
                preset,
                ..ColorGrading::default()
            },
            ColorGradingPreset::Vibrant => ColorGrading {
                saturation: 1.3,
                brightness: 0.02,
                contrast: 1.1,
                vibrance: 0.4,
                preset,
                ..ColorGrading::default()
            },
            ColorGradingPreset::Cinematic => ColorGrading {
                gamma: 2.4,
                saturation: 0.9,
                brightness: -0.02,
                contrast: 1.15,
                preset,
                ..ColorGrading::default()
            },
            ColorGradingPreset::Muted => ColorGrading {
                saturation: 0.7,
                contrast: 0.9,
                tonemap_algorithm: TonemapAlgorithm::Reinhard,
                preset,
                ..ColorGrading::default()
            },
            ColorGradingPreset::HighContrast => ColorGrading {
                saturation: 1.1,
                contrast: 1.4,
                preset,
                ..ColorGrading::default()
            },
            ColorGradingPreset::Warm => ColorGrading {
                gamma: 2.1,
                saturation: 1.1,
                brightness: 0.03,
                contrast: 1.05,
                preset,
                ..ColorGrading::default()
            },
            ColorGradingPreset::Cool => ColorGrading {
                gamma: 2.3,
                saturation: 0.95,
                brightness: -0.01,
                contrast: 1.05,
                tonemap_algorithm: TonemapAlgorithm::Neutral,
                preset,
                ..ColorGrading::default()
            },
            ColorGradingPreset::Retro => ColorGrading {
                gamma: 2.0,
                saturation: 0.8,
                brightness: 0.05,
                contrast: 1.2,
                tonemap_algorithm: TonemapAlgorithm::Reinhard,
                preset,
                ..ColorGrading::default()
            },
            ColorGradingPreset::Desaturated => ColorGrading {
                saturation: 0.3,
                preset,
                ..ColorGrading::default()
            },
            ColorGradingPreset::Custom => ColorGrading::default(),
        }
    }
}

/// Color grading and tonemapping settings.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ColorGrading {
    /// Linear exposure multiplier applied before tonemapping. Use small values
    /// (e.g. 0.001) when the scene uses physical glTF light units (lux/candela).
    pub exposure: f32,
    /// Exposure compensation in EV stops applied on top of `exposure` and
    /// auto-exposure. Multiplies the final color by `2^ev`.
    pub exposure_compensation_ev: f32,
    /// When true, derive exposure each frame from average HDR scene luminance
    /// and use `exposure` as a manual compensation multiplier on top.
    pub auto_exposure: bool,
    /// Target luminance the auto-exposure tries to map to middle gray
    /// (typical 0.18 = perceptual middle gray).
    pub auto_exposure_target: f32,
    /// Adaptation speed for the auto-exposure smoothing (1/seconds). 1.0 lerps
    /// ~63% toward the new target each second; 5.0 is snappy; 0.2 is cinematic.
    pub auto_exposure_rate: f32,
    /// Lower bound on the exposure multiplier resolved from auto-exposure,
    /// expressed in EV stops relative to `auto_exposure_target` mapping to
    /// middle gray. Prevents night-vision overshoot in dark scenes.
    pub auto_exposure_min_ev: f32,
    /// Upper bound on the exposure multiplier resolved from auto-exposure,
    /// expressed in EV stops. Prevents specular hot-spots from crushing
    /// bright scenes.
    pub auto_exposure_max_ev: f32,
    /// Gamma correction exponent (typically 2.2).
    pub gamma: f32,
    /// Color saturation multiplier (1.0 = neutral).
    pub saturation: f32,
    /// Brightness offset (-1 to 1).
    pub brightness: f32,
    /// Contrast multiplier (1.0 = neutral).
    pub contrast: f32,
    /// Vibrance: saturation that spares already-saturated colors (0 = off).
    pub vibrance: f32,
    /// Vignette darkening strength at the frame edges (0 = off).
    pub vignette_intensity: f32,
    /// Normalized radius where the vignette begins (0 at center, 1 at corner).
    pub vignette_radius: f32,
    /// Falloff width of the vignette past its radius.
    pub vignette_smoothness: f32,
    /// Chromatic aberration strength: per-channel radial offset scaled by the
    /// squared distance from the center (0 = off).
    pub chromatic_aberration: f32,
    /// Blend weight for the 3D color grading lookup table (0 = off, 1 = full).
    /// Upload the table itself with a `SetColorLut` render command.
    pub color_lut_weight: f32,
    /// HDR tonemapping algorithm.
    pub tonemap_algorithm: TonemapAlgorithm,
    /// Active preset (Custom if manually adjusted).
    pub preset: ColorGradingPreset,
}

impl Default for ColorGrading {
    fn default() -> Self {
        Self {
            exposure: 1.0,
            exposure_compensation_ev: 0.0,
            auto_exposure: false,
            auto_exposure_target: 0.18,
            auto_exposure_rate: 1.5,
            auto_exposure_min_ev: -3.0,
            auto_exposure_max_ev: 5.0,
            gamma: 2.2,
            saturation: 1.0,
            brightness: 0.0,
            contrast: 1.0,
            vibrance: 0.0,
            vignette_intensity: 0.0,
            vignette_radius: 0.6,
            vignette_smoothness: 0.4,
            chromatic_aberration: 0.0,
            color_lut_weight: 0.0,
            tonemap_algorithm: TonemapAlgorithm::Aces,
            preset: ColorGradingPreset::Default,
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

/// Distance-based fog settings.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Fog {
    /// Fog color (linear RGB).
    pub color: [f32; 3],
    /// Distance where fog begins. For exponential modes this is the offset
    /// where fog starts accumulating.
    pub start: f32,
    /// Distance where linear fog is fully opaque. For exponential modes this
    /// is the distance at which fog is nearly opaque (drives the density).
    pub end: f32,
    /// How fog density grows with distance.
    #[serde(default)]
    pub mode: FogMode,
}

/// How fog density accumulates with distance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FogMode {
    /// Linear ramp between `start` and `end`.
    #[default]
    Linear,
    /// Exponential falloff (`1 - exp(-d)`), softer near the camera.
    Exponential,
    /// Exponential-squared falloff (`1 - exp(-d*d)`), a tighter band.
    ExponentialSquared,
}

impl FogMode {
    /// Shader mode index. 0 is reserved for "fog disabled".
    pub fn as_u32(self) -> u32 {
        match self {
            FogMode::Linear => 1,
            FogMode::Exponential => 2,
            FogMode::ExponentialSquared => 3,
        }
    }

    /// Build from a mode index, falling back to `Linear` for unknown values.
    pub fn from_u32(value: u32) -> Self {
        match value {
            2 => FogMode::Exponential,
            3 => FogMode::ExponentialSquared,
            _ => FogMode::Linear,
        }
    }
}

impl Default for Fog {
    fn default() -> Self {
        Self {
            color: [0.5, 0.5, 0.55],
            start: 2.0,
            end: 15.0,
            mode: FogMode::Linear,
        }
    }
}

/// Day/night driver state: current hour, cycle speed, and the sun entity.
#[derive(Clone)]
pub struct DayNightState {
    /// Time of day in hours (0 to 24).
    pub hour: f32,
    /// Hours advanced per second when `auto_cycle` is on.
    pub speed: f32,
    /// When true, the driver advances `hour` each frame.
    pub auto_cycle: bool,
    /// The directional light driven as the sun, if any.
    pub sun_entity: Option<nightshade_ecs::Entity>,
    /// Hours the renderer bakes image based lighting snapshots at when the
    /// day/night hour starts changing, so lighting blends smoothly between them.
    pub ibl_snapshot_hours: Vec<f32>,
}

impl Default for DayNightState {
    fn default() -> Self {
        Self {
            hour: 12.0,
            speed: 0.0,
            auto_cycle: false,
            sun_entity: None,
            ibl_snapshot_hours: vec![0.0, 4.0, 7.0, 10.0, 14.0, 17.0, 20.0],
        }
    }
}

/// Skybox and environment map selection.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, enum2schema::Schema,
)]
#[schema(string_enum)]
pub enum Atmosphere {
    /// Solid background color (no skybox).
    #[default]
    None,
    /// Procedural clear sky gradient.
    Sky,
    /// Procedural sky with volumetric clouds.
    CloudySky,
    /// Procedural starfield.
    Space,
    /// Procedural nebula with stars.
    Nebula,
    /// Procedural sunset gradient.
    Sunset,
    /// Procedural day/night cycle driven by hour parameter.
    DayNight,
    /// HDR environment cubemap.
    Hdr,
    /// Debug: HDR mip level 0.
    HdrMip0,
    /// Debug: HDR mip level 1.
    HdrMip1,
    /// Debug: HDR mip level 2.
    HdrMip2,
    /// Debug: HDR mip level 3.
    HdrMip3,
    /// Debug: HDR mip level 4.
    HdrMip4,
    /// Debug: Diffuse irradiance map.
    Irradiance,
    /// Debug: Prefiltered specular mip 0.
    PrefilterMip0,
    /// Debug: Prefiltered specular mip 1.
    PrefilterMip1,
    /// Debug: Prefiltered specular mip 2.
    PrefilterMip2,
    /// Debug: Prefiltered specular mip 3.
    PrefilterMip3,
    /// Debug: Prefiltered specular mip 4.
    PrefilterMip4,
}

impl Atmosphere {
    /// Every atmosphere option in menu order.
    pub const ALL: &'static [Atmosphere] = &[
        Atmosphere::None,
        Atmosphere::Sky,
        Atmosphere::CloudySky,
        Atmosphere::Space,
        Atmosphere::Nebula,
        Atmosphere::Sunset,
        Atmosphere::DayNight,
        Atmosphere::Hdr,
        Atmosphere::HdrMip0,
        Atmosphere::HdrMip1,
        Atmosphere::HdrMip2,
        Atmosphere::HdrMip3,
        Atmosphere::HdrMip4,
        Atmosphere::Irradiance,
        Atmosphere::PrefilterMip0,
        Atmosphere::PrefilterMip1,
        Atmosphere::PrefilterMip2,
        Atmosphere::PrefilterMip3,
        Atmosphere::PrefilterMip4,
    ];

    /// Mip level a debug mip variant selects, 0 for non-debug options.
    pub fn mip_level(&self) -> f32 {
        match self {
            Atmosphere::HdrMip0 | Atmosphere::PrefilterMip0 => 0.0,
            Atmosphere::HdrMip1 | Atmosphere::PrefilterMip1 => 1.0,
            Atmosphere::HdrMip2 | Atmosphere::PrefilterMip2 => 2.0,
            Atmosphere::HdrMip3 | Atmosphere::PrefilterMip3 => 3.0,
            Atmosphere::HdrMip4 | Atmosphere::PrefilterMip4 => 4.0,
            _ => 0.0,
        }
    }

    /// The next atmosphere in `ALL`, wrapping around at the end.
    pub fn next(self) -> Self {
        let all = Self::ALL;
        let current_index = all.iter().position(|&a| a == self).unwrap_or(0);
        let next_index = (current_index + 1) % all.len();
        all[next_index]
    }

    /// The previous atmosphere in `ALL`, wrapping around at the start.
    pub fn previous(self) -> Self {
        let all = Self::ALL;
        let current_index = all.iter().position(|&a| a == self).unwrap_or(0);
        let prev_index = if current_index == 0 {
            all.len() - 1
        } else {
            current_index - 1
        };
        all[prev_index]
    }

    /// True for the procedurally generated skyboxes.
    pub fn is_procedural(&self) -> bool {
        matches!(
            self,
            Atmosphere::Sky
                | Atmosphere::CloudySky
                | Atmosphere::Space
                | Atmosphere::Nebula
                | Atmosphere::Sunset
                | Atmosphere::DayNight
        )
    }

    /// Shader cubemap-type index for procedural skyboxes, `None` otherwise.
    pub fn as_procedural_cubemap_type(&self) -> Option<u32> {
        match self {
            Atmosphere::Sky => Some(0),
            Atmosphere::CloudySky => Some(1),
            Atmosphere::Space => Some(2),
            Atmosphere::Nebula => Some(3),
            Atmosphere::Sunset => Some(4),
            Atmosphere::DayNight => Some(5),
            _ => None,
        }
    }
}

/// Texture views for the image-based lighting inputs the lit passes bind.
#[derive(Default)]
pub struct IblViews {
    /// View of the precomputed split-sum `BRDF` lookup texture.
    pub brdf_lut_view: Option<wgpu::TextureView>,
    /// View of the diffuse irradiance cubemap.
    pub irradiance_view: Option<wgpu::TextureView>,
    /// View of the prefiltered specular cubemap.
    pub prefiltered_view: Option<wgpu::TextureView>,
}

impl Clone for IblViews {
    fn clone(&self) -> Self {
        Self {
            brdf_lut_view: self.brdf_lut_view.clone(),
            irradiance_view: self.irradiance_view.clone(),
            prefiltered_view: self.prefiltered_view.clone(),
        }
    }
}

/// One level-of-detail step: a mesh and the screen size below which it is used.
#[derive(Clone)]
pub struct MeshLodLevel {
    /// Registered name of the mesh this level draws.
    pub mesh_name: String,
    /// Minimum on-screen size in pixels for this level to remain active.
    pub min_screen_pixels: f32,
}

/// A base mesh and its ordered chain of lower-detail replacements.
#[derive(Clone)]
pub struct MeshLodChain {
    /// Registered name of the full-detail mesh.
    pub base_mesh: String,
    /// Lower-detail levels, ordered from highest to lowest detail.
    pub levels: Vec<MeshLodLevel>,
}

/// GPU uniform layout for the configurable post-process effects pass.
/// 38 packed `f32`s mapped 1:1 to the WGSL uniform binding in
/// `crates/nightshade/src/render/wgpu/shaders/effects.wgsl`. The
/// `EffectsPass` writes this buffer each frame from
/// `Graphics::effects.uniforms` plus the live frame time.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct EffectsUniforms {
    /// Elapsed time in seconds, driving animated effects.
    pub time: f32,
    /// Per-channel radial color separation strength.
    pub chromatic_aberration: f32,
    /// Sinusoidal screen-space wave distortion strength.
    pub wave_distortion: f32,
    /// Hue shift amount applied across the frame.
    pub color_shift: f32,
    /// Kaleidoscope mirror segment count (0 disables).
    pub kaleidoscope_segments: f32,
    /// CRT scanline overlay strength.
    pub crt_scanlines: f32,
    /// Edge vignette darkening strength.
    pub vignette: f32,
    /// Plasma color-field overlay intensity.
    pub plasma_intensity: f32,
    /// Digital glitch displacement strength.
    pub glitch_intensity: f32,
    /// Mirror-reflection mode selector.
    pub mirror_mode: f32,
    /// Color inversion amount (0 to 1).
    pub invert: f32,
    /// Static hue rotation in radians.
    pub hue_rotation: f32,
    /// Raymarch scene selector, see `RaymarchMode`.
    pub raymarch_mode: f32,
    /// Blend weight of the raymarched layer over the scene.
    pub raymarch_blend: f32,
    /// Film grain noise strength.
    pub film_grain: f32,
    /// Unsharp-mask sharpening strength.
    pub sharpen: f32,
    /// Pixelation block size (0 disables).
    pub pixelate: f32,
    /// Color posterization level count (0 disables).
    pub color_posterize: f32,
    /// Radial motion blur strength.
    pub radial_blur: f32,
    /// Tunnel raymarch scroll speed.
    pub tunnel_speed: f32,
    /// Fractal raymarch iteration count.
    pub fractal_iterations: f32,
    /// Bloom-style glow intensity.
    pub glow_intensity: f32,
    /// Screen shake displacement amount.
    pub screen_shake: f32,
    /// Pulsing zoom amount.
    pub zoom_pulse: f32,
    /// Anime speed-line overlay strength.
    pub speed_lines: f32,
    /// Color grade preset selector, see `ColorGradeMode`.
    pub color_grade_mode: f32,
    /// VHS tape distortion strength.
    pub vhs_distortion: f32,
    /// Lens flare intensity.
    pub lens_flare: f32,
    /// Edge-detection glow strength.
    pub edge_glow: f32,
    /// Saturation multiplier (1 is neutral).
    pub saturation: f32,
    /// Warp-speed streak effect strength.
    pub warp_speed: f32,
    /// Concentric pulse ring strength.
    pub pulse_rings: f32,
    /// Heat-haze distortion strength.
    pub heat_distortion: f32,
    /// Matrix digital-rain overlay strength.
    pub digital_rain: f32,
    /// Strobe flash strength.
    pub strobe: f32,
    /// Animated color cycling speed.
    pub color_cycle_speed: f32,
    /// Frame feedback (trails) blend amount.
    pub feedback_amount: f32,
    /// ASCII-art rendering mode strength.
    pub ascii_mode: f32,
}

impl Default for EffectsUniforms {
    fn default() -> Self {
        Self {
            time: 0.0,
            chromatic_aberration: 0.0,
            wave_distortion: 0.0,
            color_shift: 0.0,
            kaleidoscope_segments: 0.0,
            crt_scanlines: 0.0,
            vignette: 0.0,
            plasma_intensity: 0.0,
            glitch_intensity: 0.0,
            mirror_mode: 0.0,
            invert: 0.0,
            hue_rotation: 0.0,
            raymarch_mode: 0.0,
            raymarch_blend: 0.0,
            film_grain: 0.0,
            sharpen: 0.0,
            pixelate: 0.0,
            color_posterize: 0.0,
            radial_blur: 0.0,
            tunnel_speed: 1.0,
            fractal_iterations: 4.0,
            glow_intensity: 0.0,
            screen_shake: 0.0,
            zoom_pulse: 0.0,
            speed_lines: 0.0,
            color_grade_mode: 0.0,
            vhs_distortion: 0.0,
            lens_flare: 0.0,
            edge_glow: 0.0,
            saturation: 1.0,
            warp_speed: 0.0,
            pulse_rings: 0.0,
            heat_distortion: 0.0,
            digital_rain: 0.0,
            strobe: 0.0,
            color_cycle_speed: 1.0,
            feedback_amount: 0.0,
            ascii_mode: 0.0,
        }
    }
}

/// Raymarched scene the effects pass can composite over the frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RaymarchMode {
    /// No raymarched layer.
    Off = 0,
    /// Scrolling tunnel.
    Tunnel = 1,
    /// Iterated fractal field.
    Fractal = 2,
    /// Mandelbulb distance field.
    Mandelbulb = 3,
    /// Swirling plasma vortex.
    PlasmaVortex = 4,
    /// Geometric primitive scene.
    Geometric = 5,
}

/// Maps a shader mode index to `RaymarchMode`, falling back to `Off`.
impl From<u32> for RaymarchMode {
    fn from(value: u32) -> Self {
        match value {
            1 => RaymarchMode::Tunnel,
            2 => RaymarchMode::Fractal,
            3 => RaymarchMode::Mandelbulb,
            4 => RaymarchMode::PlasmaVortex,
            5 => RaymarchMode::Geometric,
            _ => RaymarchMode::Off,
        }
    }
}

/// Stylized color grade the effects pass can apply.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorGradeMode {
    /// No grade.
    None = 0,
    /// Neon cyberpunk palette.
    Cyberpunk = 1,
    /// Warm sunset palette.
    Sunset = 2,
    /// Grayscale.
    Grayscale = 3,
    /// Sepia tone.
    Sepia = 4,
    /// Green Matrix tint.
    Matrix = 5,
    /// Hot-metal gradient.
    HotMetal = 6,
}

/// Maps a shader mode index to `ColorGradeMode`, falling back to `None`.
impl From<u32> for ColorGradeMode {
    fn from(value: u32) -> Self {
        match value {
            1 => ColorGradeMode::Cyberpunk,
            2 => ColorGradeMode::Sunset,
            3 => ColorGradeMode::Grayscale,
            4 => ColorGradeMode::Sepia,
            5 => ColorGradeMode::Matrix,
            6 => ColorGradeMode::HotMetal,
            _ => ColorGradeMode::None,
        }
    }
}

/// Post-process effects pass settings. Held in
/// `Graphics::effects` as plain data; the `EffectsPass` reads it from
/// `RendererState::effects` each frame and uploads
/// `uniforms` (with `time`, and optionally `hue_rotation`, overridden
/// for the current frame) to the GPU.
#[derive(Clone, Debug)]
pub struct EffectsState {
    /// Effect parameter values uploaded to the GPU each frame.
    pub uniforms: EffectsUniforms,
    /// When false, the effects pass is skipped.
    pub enabled: bool,
    /// When true, `hue_rotation` advances automatically each frame.
    pub animate_hue: bool,
}

impl Default for EffectsState {
    fn default() -> Self {
        Self {
            uniforms: EffectsUniforms::default(),
            enabled: true,
            animate_hue: false,
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

/// The GPU class the renderer ended up on, mirrored from `wgpu::DeviceType`
/// so apps can pick quality presets without depending on wgpu directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GpuDeviceType {
    /// Unknown or unclassified device.
    #[default]
    Other,
    /// Integrated GPU sharing system memory.
    IntegratedGpu,
    /// Discrete GPU with dedicated memory.
    DiscreteGpu,
    /// Virtualized or paravirtual GPU.
    VirtualGpu,
    /// Software (CPU) rasterizer.
    Cpu,
}

/// The graphics backend the renderer is running on. `WebGl` is the constrained
/// browser fallback and is the strongest signal that a mobile-class budget is
/// appropriate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GpuBackend {
    /// Unknown or unclassified backend.
    #[default]
    Other,
    /// Vulkan.
    Vulkan,
    /// Apple Metal.
    Metal,
    /// Direct3D 12.
    Dx12,
    /// Desktop OpenGL.
    Gl,
    /// Native WebGPU.
    WebGpu,
    /// WebGL browser fallback.
    WebGl,
}

/// A snapshot of the adapter the renderer selected, published into
/// `RendererState` so game code can adapt quality and controls to the device.
#[derive(Debug, Clone, Default)]
pub struct GpuProfile {
    /// Graphics backend the renderer is running on.
    pub backend: GpuBackend,
    /// Class of the selected GPU.
    pub device_type: GpuDeviceType,
    /// Adapter name reported by the driver.
    pub name: String,
}

impl GpuProfile {
    /// True when the renderer fell back to the browser WebGL path, the clearest
    /// indication of a constrained mobile-class GPU budget.
    pub fn is_webgl(&self) -> bool {
        self.backend == GpuBackend::WebGl
    }

    /// True when the adapter is an integrated, virtual, or software device, the
    /// classes that benefit from a reduced render budget.
    pub fn is_low_power(&self) -> bool {
        matches!(
            self.device_type,
            GpuDeviceType::IntegratedGpu | GpuDeviceType::VirtualGpu | GpuDeviceType::Cpu
        )
    }
}

/// Per-dispatch camera state the renderer extracts once at the start of each
/// per-camera render and every pass reads, replacing scattered
/// `query_active_camera_matrices` calls and their per-pass re-derivations.
/// `projection` carries the TAA jitter, matching `query_active_camera_matrices`.
#[derive(Clone, Debug)]
pub struct RenderView {
    /// World-to-view matrix.
    pub view: nalgebra_glm::Mat4,
    /// View-to-clip projection, carrying the `TAA` jitter.
    pub projection: nalgebra_glm::Mat4,
    /// Combined world-to-clip matrix.
    pub view_projection: nalgebra_glm::Mat4,
    /// View-to-world matrix.
    pub inverse_view: nalgebra_glm::Mat4,
    /// Clip-to-view matrix.
    pub inverse_projection: nalgebra_glm::Mat4,
    /// Clip-to-world matrix.
    pub inverse_view_projection: nalgebra_glm::Mat4,
    /// Camera position in world space.
    pub camera_position: nalgebra_glm::Vec3,
    /// Camera right axis in world space.
    pub camera_right: nalgebra_glm::Vec3,
    /// Camera up axis in world space.
    pub camera_up: nalgebra_glm::Vec3,
    /// Six world-space frustum planes for culling.
    pub frustum_planes: [nalgebra_glm::Vec4; 6],
    /// Near clip distance in world units.
    pub z_near: f32,
    /// Far clip distance in world units.
    pub z_far: f32,
    /// Vertical field of view in radians.
    pub y_fov_rad: f32,
    /// Aspect ratio (width over height).
    pub aspect: f32,
    /// Orthographic half-width and half-height, `None` for perspective.
    pub orthographic: Option<(f32, f32)>,
    /// Render target size in pixels.
    pub screen_size: (u32, u32),
    /// Fixed aspect the view is letterboxed to, if constrained.
    pub constrained_aspect: Option<f32>,
}

/// Scene lighting the renderer collects once per dispatch and every lit pass
/// reads, replacing per-pass `collect_lights`, `collect_area_lights`,
/// `calculate_cascade_shadows`, and `query_sun` calls. The base light list is
/// carried without per-pass shadow-index application; each pass still resolves
/// its own spotlight, point, and cookie indices against its texture arrays.
#[cfg(feature = "wgpu")]
#[derive(Clone, Default)]
pub struct RenderLighting {
    /// GPU-ready light records for every non-area light.
    pub lights_data: Vec<crate::wgpu::passes::geometry::projection::LightData>,
    /// Number of directional lights in the scene.
    pub num_directional_lights: u32,
    /// Primary directional light direction in world space.
    pub directional_light_direction: [f32; 4],
    /// True when at least one directional light is present.
    pub has_directional_light: bool,
    /// Entity to index map into `lights_data`.
    pub entity_to_lights_index: std::collections::HashMap<nightshade_ecs::Entity, usize>,
    /// Shadow view-projection for each directional cascade.
    pub cascade_view_projections: [[[f32; 4]; 4]; crate::wgpu::passes::NUM_SHADOW_CASCADES],
    /// World-space diameter each cascade covers.
    pub cascade_diameters: [f32; crate::wgpu::passes::NUM_SHADOW_CASCADES],
    /// View-space far split distance for each cascade.
    pub cascade_split_distances: [f32; crate::wgpu::passes::NUM_SHADOW_CASCADES],
    /// Directional light view-projection for single-map shadows.
    pub light_view_projection: [[f32; 4]; 4],
    /// Depth bias applied when sampling shadow maps.
    pub shadow_bias: f32,
    /// Normal-scaled bias applied when sampling shadow maps.
    pub shadow_normal_bias: f32,
    /// Angular size of the directional light for soft shadows.
    pub directional_light_size: f32,
    /// Shadow enable flag, 1.0 on and 0.0 off.
    pub shadows_enabled: f32,
    /// GPU-ready records for every area light.
    pub area_lights_data: Vec<crate::wgpu::passes::geometry::projection::AreaLightData>,
    /// Entity to index map into `area_lights_data`.
    pub area_entity_to_index: std::collections::HashMap<nightshade_ecs::Entity, usize>,
    /// Resolved sun direction in world space.
    pub sun_direction: nalgebra_glm::Vec3,
    /// Resolved sun color.
    pub sun_color: nalgebra_glm::Vec3,
}

/// Per-entity dynamic render state the renderer refreshes each frame without a
/// full rebuild: whether the object is visible and its current morph weights.
#[derive(Clone, Copy, Debug)]
pub struct DynamicRenderState {
    /// Visibility flag, 1 visible and 0 hidden.
    pub visible: u32,
    /// Active morph target weights.
    pub morph_weights: [f32; 8],
    /// World transform.
    pub transform: nalgebra_glm::Mat4,
    /// Culling layer bitmask tested against camera masks.
    pub culling_mask: u32,
    /// Overlay flag, 1 for overlay-layer entities.
    pub is_overlay: u32,
    /// Render layer index.
    pub render_layer: u32,
}

/// Expanded per-instance data for one instanced-mesh entity, snapshotted once
/// per frame so the renderer builds instanced draws without reading the
/// `InstancedMesh` component.
#[derive(Clone, Default)]
pub struct InstancedObjectData {
    /// Per-instance world matrices.
    pub world_models: Vec<[[f32; 4]; 4]>,
    /// Per-instance normal matrices.
    pub world_normals: Vec<[[f32; 4]; 3]>,
    /// Per-instance local transforms relative to the parent.
    pub local_matrices: Vec<nalgebra_glm::Mat4>,
    /// Per-instance color tints.
    pub custom_tints: Vec<[f32; 4]>,
    /// Registered name of the instanced mesh.
    pub mesh_name: String,
    /// Render layer index.
    pub render_layer: u32,
    /// Visibility flag, 1 visible and 0 hidden.
    pub visible: u32,
    /// World transform of the instancing entity's parent.
    pub parent_transform: nalgebra_glm::Mat4,
}

/// A single animation channel flattened to GPU-ready arrays, produced by the
/// engine so the renderer never clones keyframe data per frame.
#[derive(Clone)]
pub struct SkinnedChannelData {
    /// Animated property selector (translation, rotation, scale, or weights).
    pub property: u32,
    /// Interpolation mode (step, linear, or cubic spline).
    pub interpolation: u32,
    /// Keyframe timestamps in seconds.
    pub input: Vec<f32>,
    /// Keyframe output values, one entry per timestamp.
    pub values: Vec<[f32; 4]>,
    /// Output values consumed per keyframe.
    pub stride: u32,
}

/// One joint of a skinned skeleton in the renderer's dispatch order, with its
/// rest pose and resolved animation channels for the current clip, the
/// cross-fade source clip, and each animation layer.
#[derive(Clone)]
pub struct SkinnedJointData {
    /// Index of this joint within the skeleton.
    pub local_index: u32,
    /// Parent joint index, `None` for a root joint.
    pub parent_local: Option<u32>,
    /// Rest-pose translation.
    pub rest_translation: [f32; 3],
    /// Rest-pose rotation quaternion.
    pub rest_rotation: [f32; 4],
    /// Rest-pose scale.
    pub rest_scale: [f32; 3],
    /// Animation channels for each active base-pose clip, in weight-blend order.
    pub pose_channels: Vec<Vec<SkinnedChannelData>>,
    /// Animation channels for each additional animation layer.
    pub layer_channels: Vec<Vec<SkinnedChannelData>>,
    /// Reference-pose channels per layer for additive blending, empty for
    /// override layers.
    pub reference_channels: Vec<Vec<SkinnedChannelData>>,
}

/// A two-bone IK chain resolved to local joint indices. The world-space target,
/// pole, and weight live in the per-frame `ik_runtime` map so moving the target
/// does not rebuild the static payload.
#[derive(Clone)]
pub struct SkinnedIkChainData {
    /// Index of this chain in the player's chain list, for the runtime lookup.
    pub chain_index: u32,
    /// Local joint index of the chain root.
    pub root_local: u32,
    /// Local joint index of the middle joint.
    pub mid_local: u32,
    /// Local joint index of the tip joint.
    pub tip_local: u32,
    /// Whether the chain uses a pole target to bias the bend.
    pub has_pole: bool,
}

/// An N-bone IK chain resolved to local joint indices, ordered root to tip, for
/// the GPU cyclic-coordinate-descent solver. Target, weight, and limits live in
/// the per-frame `ik_multi_runtime` map.
#[derive(Clone)]
pub struct SkinnedMultiIkChainData {
    /// Index of this chain in the player's multi-chain list, for the runtime lookup.
    pub chain_index: u32,
    /// Local joint indices ordered from root to tip.
    pub joint_locals: Vec<u32>,
}

/// A look-at constraint resolved to local joint indices, ordered base to tip.
/// The local forward axis is static; the target, weight, and cone limit live in
/// the per-frame `aim_runtime` map.
#[derive(Clone)]
pub struct SkinnedAimConstraintData {
    /// Index of this constraint in the player's aim list, for the runtime lookup.
    pub chain_index: u32,
    /// Local joint indices ordered from the base of the chain to the tip.
    pub joint_locals: Vec<u32>,
    /// The bones' local forward axis.
    pub forward: [f32; 3],
}

/// A spring-bone chain resolved to local joint indices, ordered root to tip,
/// with each joint's rest direction and bone length. The dynamics parameters
/// live in the per-frame `spring_runtime` map.
#[derive(Clone)]
pub struct SkinnedSpringChainData {
    /// Index of this chain in the player's spring list, for the runtime lookup.
    pub chain_index: u32,
    /// Local joint indices ordered root to tip.
    pub joint_locals: Vec<u32>,
    /// Per joint local rest direction from its parent; the first entry is unused.
    pub rest_axes: Vec<[f32; 3]>,
    /// Per joint length from its parent; the first entry is unused.
    pub bone_lengths: Vec<f32>,
}

/// One animated skeleton resolved by the engine: its skin and player entities,
/// depth-ordered joints, and how many animation layers the player exposes.
#[derive(Clone)]
pub struct SkinnedSkeletonData {
    /// Entity carrying the skin.
    pub skin_entity: nightshade_ecs::Entity,
    /// Entity carrying the animation player.
    pub player_entity: nightshade_ecs::Entity,
    /// The entity above the skeleton's root joints, captured at build time so
    /// the per-frame armature-root refresh is one transform read instead of a
    /// joint-parent walk.
    pub armature_parent: Option<nightshade_ecs::Entity>,
    /// Number of joints in the skeleton.
    pub joint_count: u32,
    /// Number of active base-pose clips the player blends.
    pub active_count: u32,
    /// Number of animation layers the player exposes.
    pub layer_count: u32,
    /// Blend mode per layer: `0` override, `1` additive.
    pub layer_modes: Vec<u32>,
    /// Local joint index whose translation is locked to rest so root motion
    /// drives the entity instead of the rendered pose, `None` when disabled.
    pub root_motion_local_index: Option<u32>,
    /// Two-bone IK chains solved on the GPU pose after blending.
    pub ik_chains: Vec<SkinnedIkChainData>,
    /// Multi-bone IK chains solved on the GPU with cyclic coordinate descent.
    pub ik_multi_chains: Vec<SkinnedMultiIkChainData>,
    /// Look-at constraints solved on the GPU.
    pub aim_constraints: Vec<SkinnedAimConstraintData>,
    /// Spring-bone chains simulated on the GPU.
    pub spring_chains: Vec<SkinnedSpringChainData>,
    /// Joints in the renderer's depth-first dispatch order.
    pub joints_ordered: Vec<SkinnedJointData>,
}

/// Skinning palette resolved once per frame by the engine so the skinned-mesh
/// and shadow passes build their GPU buffers without reading Skin components or
/// bone transforms. The static layout (`cache`) is rebuilt only when the skinned
/// set changes, bumping `static_generation`; `bone_transforms` refreshes every
/// frame.
#[derive(Default, Clone)]
pub struct RenderSkinning {
    /// Static skinning layout, rebuilt only when the skinned set changes.
    pub cache: crate::skinning::SkinningCache,
    /// Current bone matrices for every skinned joint.
    pub bone_transforms: Vec<nalgebra_glm::Mat4>,
    /// Bumped whenever `cache` is rebuilt.
    pub static_generation: u64,
    /// Bumped whenever any bone transform changed since the last sync, so
    /// consumers can gate uploads on a counter instead of comparing the
    /// whole palette.
    pub bone_transforms_generation: u64,
}

/// Animation state resolved once per frame by the engine so the skinned-mesh
/// compute driver builds its GPU buffers without reading ECS animation, skin,
/// parent, or transform components. The heavy `skeletons` payload is rebuilt
/// only when the engine's animation gate sees a change, bumping `signature`;
/// the runtime maps refresh every frame.
#[derive(Clone, Default)]
pub struct SkinnedAnimationSnapshot {
    /// Bumped when the heavy `skeletons` payload is rebuilt.
    pub signature: u64,
    /// Resolved skeletons, rebuilt only when the animation gate sees a change.
    pub skeletons: Vec<SkinnedSkeletonData>,
    /// Per active base-pose clip runtime state (time and weight), keyed by player
    /// and active-clip index.
    pub pose_runtime: std::collections::HashMap<(nightshade_ecs::Entity, usize), [f32; 2]>,
    /// Per-layer runtime state (time and weight), keyed by player and layer.
    pub layer_runtime: std::collections::HashMap<(nightshade_ecs::Entity, usize), [f32; 2]>,
    /// Per IK chain runtime, keyed by player and chain index: target xyz, pole
    /// xyz, weight, and enabled flag (`1.0` when the chain solves this frame).
    pub ik_runtime: std::collections::HashMap<(nightshade_ecs::Entity, usize), [f32; 8]>,
    /// Per multi-bone IK chain runtime: target xyz, weight, enabled, max angle,
    /// iterations, pad.
    pub ik_multi_runtime: std::collections::HashMap<(nightshade_ecs::Entity, usize), [f32; 8]>,
    /// Per aim constraint runtime: target xyz, weight, enabled, cone angle, pad.
    pub aim_runtime: std::collections::HashMap<(nightshade_ecs::Entity, usize), [f32; 8]>,
    /// Per spring chain runtime: gravity xyz, stiffness, damping, enabled, delta
    /// time, pad.
    pub spring_runtime: std::collections::HashMap<(nightshade_ecs::Entity, usize), [f32; 8]>,
    /// Current armature-root world transform for each player.
    pub armature_roots: std::collections::HashMap<nightshade_ecs::Entity, nalgebra_glm::Mat4>,
}

/// A resolved material for the renderer: the engine-side material data plus its
/// resolved texture ids. The renderer converts this to GPU material data with
/// its own bindless texture-layer map. Material id `N` maps to `entries[N - 1]`;
/// id 0 is the built-in default material.
#[derive(Clone, Default)]
pub struct RenderMaterialEntry {
    /// Engine-side material parameters.
    pub material: crate::material::Material,
    /// Resolved texture ids for the material's texture slots.
    pub texture_ids: crate::material::MaterialTextureIds,
}

/// The scene's materials resolved once per frame by the engine, so the render
/// passes read materials here instead of walking the material registry and each
/// entity's `MaterialRef` themselves.
#[derive(Clone, Default)]
pub struct RenderMaterials {
    /// Resolved materials, where id `N` is `entries[N - 1]`.
    pub entries: Vec<RenderMaterialEntry>,
    /// The set of live material ids (`entries` index + 1). Used to check whether
    /// a material an entity references is already known to the GPU-side cache.
    pub material_ids: std::collections::HashSet<u32>,
    /// Ids of materials with a blended or dithered alpha mode.
    pub transparent_ids: std::collections::HashSet<u32>,
    /// Ids of materials with an alpha-mask cutout mode.
    pub mask_ids: std::collections::HashSet<u32>,
    /// Ids of materials that render both faces.
    pub double_sided_ids: std::collections::HashSet<u32>,
    /// Bumped whenever the table changes, so consumers can gate their material
    /// conversions on a counter.
    pub generation: u64,
}

impl RenderMaterials {
    /// Adds a material and returns its id. Material id `N` is `entries[N - 1]`;
    /// id 0 is the renderer's built-in default. Updates the live / transparent /
    /// mask / double-sided id sets from the material and bumps `generation`.
    pub fn add(&mut self, material: crate::material::Material) -> u32 {
        self.add_textured(material, crate::material::MaterialTextureIds::default())
    }

    /// Like [`RenderMaterials::add`], with resolved texture ids.
    pub fn add_textured(
        &mut self,
        material: crate::material::Material,
        texture_ids: crate::material::MaterialTextureIds,
    ) -> u32 {
        use crate::material::AlphaMode;
        let transparent = matches!(material.alpha_mode, AlphaMode::Blend | AlphaMode::Dither);
        let mask = matches!(material.alpha_mode, AlphaMode::Mask);
        let double_sided = material.double_sided;
        self.entries.push(RenderMaterialEntry {
            material,
            texture_ids,
        });
        let id = self.entries.len() as u32;
        self.material_ids.insert(id);
        if transparent {
            self.transparent_ids.insert(id);
        }
        if mask {
            self.mask_ids.insert(id);
        }
        if double_sided {
            self.double_sided_ids.insert(id);
        }
        self.generation += 1;
        id
    }
}

/// One scene light with its world transform, snapshotted once per frame so the
/// lighting, shadow, and projection passes read lights here instead of walking
/// the ECS themselves.
/// A light's kind, mirrored from the `LightType` component enum so the render
/// passes do not read that type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RenderLightType {
    /// Infinitely distant light with parallel rays.
    #[default]
    Directional,
    /// Omnidirectional point light.
    Point,
    /// Cone-shaped spotlight.
    Spot,
    /// Shaped area light.
    Area,
}

/// An area light's shape, mirrored from the `AreaLightShape` component enum so
/// the render passes do not read that type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RenderAreaLightShape {
    /// Flat rectangle.
    #[default]
    Rectangle,
    /// Flat disk.
    Disk,
    /// Sphere.
    Sphere,
    /// Capsule-like tube.
    Tube,
}

/// A light's render parameters snapshotted once per frame from the `Light`
/// component, so the lighting, shadow, and projection passes read them here
/// instead of that component.
#[derive(Clone, Default)]
pub struct RenderLightData {
    /// Kind of light.
    pub light_type: RenderLightType,
    /// Linear RGB color.
    pub color: nalgebra_glm::Vec3,
    /// Brightness multiplier.
    pub intensity: f32,
    /// Reach distance in world units for point and spot lights.
    pub range: f32,
    /// Spotlight inner cone half-angle in radians.
    pub inner_cone_angle: f32,
    /// Spotlight outer cone half-angle in radians.
    pub outer_cone_angle: f32,
    /// Whether the light casts shadows.
    pub cast_shadows: bool,
    /// Depth bias applied when sampling this light's shadow map.
    pub shadow_bias: f32,
    /// Shadow map edge length in texels.
    pub shadow_resolution: u32,
    /// Maximum shadow distance in world units.
    pub shadow_distance: f32,
    /// Optional projected cookie texture name.
    pub cookie_texture: Option<String>,
    /// Area light shape.
    pub area_shape: RenderAreaLightShape,
    /// Area light width in world units.
    pub area_width: f32,
    /// Area light height in world units.
    pub area_height: f32,
    /// Area light radius in world units.
    pub area_radius: f32,
    /// Whether the area light emits from both faces.
    pub area_two_sided: bool,
    /// Optional emissive texture name for the area light.
    pub area_emissive_texture: Option<String>,
    /// Normal-scaled bias applied when sampling this light's shadow map.
    pub shadow_normal_bias: f32,
    /// Shadow penumbra softness.
    pub shadow_softness: f32,
}

/// One scene light with its world transform, snapshotted once per frame so the
/// lighting, shadow, and projection passes read lights here instead of walking
/// the ECS themselves.
#[derive(Clone)]
pub struct RenderLight {
    /// The scene entity this light belongs to.
    pub entity: nightshade_ecs::Entity,
    /// The light's render parameters.
    pub light: RenderLightData,
    /// World transform, whose forward axis is read as `-column2`.
    pub transform: nalgebra_glm::Mat4,
}

fn base_light_data(
    light_type: RenderLightType,
    color: nalgebra_glm::Vec3,
    intensity: f32,
    range: f32,
) -> RenderLightData {
    RenderLightData {
        light_type,
        color,
        intensity,
        range,
        inner_cone_angle: 0.0,
        outer_cone_angle: std::f32::consts::FRAC_PI_4,
        cast_shadows: false,
        shadow_bias: 0.0015,
        shadow_resolution: 2048,
        shadow_distance: 100.0,
        cookie_texture: None,
        area_shape: RenderAreaLightShape::Rectangle,
        area_width: 1.0,
        area_height: 1.0,
        area_radius: 0.5,
        area_two_sided: false,
        area_emissive_texture: None,
        shadow_normal_bias: 0.02,
        shadow_softness: 1.0,
    }
}

/// A world transform whose forward axis (which the passes read as `-column2`)
/// is `direction`, positioned at `position`.
fn direction_transform(
    position: nalgebra_glm::Vec3,
    direction: nalgebra_glm::Vec3,
) -> nalgebra_glm::Mat4 {
    let forward = direction.normalize();
    let world_up = if forward.y.abs() > 0.99 {
        nalgebra_glm::vec3(0.0, 0.0, 1.0)
    } else {
        nalgebra_glm::vec3(0.0, 1.0, 0.0)
    };
    let right = forward.cross(&world_up).normalize();
    let up = right.cross(&forward);
    nalgebra_glm::Mat4::new(
        right.x, up.x, -forward.x, position.x, right.y, up.y, -forward.y, position.y, right.z,
        up.z, -forward.z, position.z, 0.0, 0.0, 0.0, 1.0,
    )
}

impl RenderLight {
    /// A shadow-casting directional light (a sun) pointing along `direction`.
    pub fn directional(
        entity: nightshade_ecs::Entity,
        direction: nalgebra_glm::Vec3,
        color: nalgebra_glm::Vec3,
        intensity: f32,
    ) -> Self {
        let mut light = base_light_data(RenderLightType::Directional, color, intensity, 0.0);
        light.cast_shadows = true;
        Self {
            entity,
            light,
            transform: direction_transform(nalgebra_glm::Vec3::zeros(), direction),
        }
    }

    /// A point light at `position` reaching out to `range`.
    pub fn point(
        entity: nightshade_ecs::Entity,
        position: nalgebra_glm::Vec3,
        color: nalgebra_glm::Vec3,
        intensity: f32,
        range: f32,
    ) -> Self {
        Self {
            entity,
            light: base_light_data(RenderLightType::Point, color, intensity, range),
            transform: nalgebra_glm::translation(&position),
        }
    }

    /// A spot light at `position` aimed along `direction`, with `[inner, outer]`
    /// cone half-angles in radians.
    pub fn spot(
        entity: nightshade_ecs::Entity,
        position: nalgebra_glm::Vec3,
        direction: nalgebra_glm::Vec3,
        color: nalgebra_glm::Vec3,
        intensity: f32,
        range: f32,
        cone: [f32; 2],
    ) -> Self {
        let mut light = base_light_data(RenderLightType::Spot, color, intensity, range);
        light.inner_cone_angle = cone[0];
        light.outer_cone_angle = cone[1];
        Self {
            entity,
            light,
            transform: direction_transform(position, direction),
        }
    }
}

/// A projected decal's render parameters snapshotted once per frame from the
/// `Decal` component, so the decal pass reads them here instead of that
/// component.
#[derive(Clone, Default)]
pub struct RenderDecalData {
    /// Optional base color texture name.
    pub texture: Option<String>,
    /// Optional emissive texture name.
    pub emissive_texture: Option<String>,
    /// Emissive brightness multiplier.
    pub emissive_strength: f32,
    /// Tint color multiplied into the decal.
    pub color: [f32; 4],
    /// Decal footprint size in world units.
    pub size: nalgebra_glm::Vec2,
    /// Projection depth along the decal's forward axis in world units.
    pub depth: f32,
    /// Maximum angle between surface and decal normals before the decal fades.
    pub normal_threshold: f32,
    /// Distance from the projector where fading begins.
    pub fade_start: f32,
    /// Distance from the projector where the decal fully fades out.
    pub fade_end: f32,
}

/// A water body's render parameters snapshotted once per frame from the `Water`
/// component, so the water pass reads them here instead of that component.
#[derive(Clone, Debug, Default)]
pub struct RenderWaterData {
    /// Half the water plane's size in world units.
    pub half_extents: nalgebra_glm::Vec2,
    /// Mesh grid subdivision count.
    pub tessellation: u32,
    /// Gerstner wave height in world units.
    pub wave_amplitude: f32,
    /// Gerstner wave crest steepness.
    pub wave_steepness: f32,
    /// Distance between wave crests in world units.
    pub wave_length: f32,
    /// Wave travel speed.
    pub wave_speed: f32,
    /// Wave travel direction in radians.
    pub wave_direction_radians: f32,
    /// Linear color in shallow water.
    pub shallow_color: [f32; 3],
    /// Linear color in deep water.
    pub deep_color: [f32; 3],
    /// Depth over which color fades from shallow to deep, in world units.
    pub depth_fade_distance: f32,
    /// Distance from shoreline edges where foam appears, in world units.
    pub edge_foam_distance: f32,
    /// Overall foam coverage amount.
    pub foam_amount: f32,
    /// Linear foam color.
    pub foam_color: [f32; 3],
    /// Surface roughness.
    pub roughness: f32,
    /// Fresnel falloff exponent.
    pub fresnel_power: f32,
    /// Reflection blend strength.
    pub reflection_strength: f32,
    /// Refraction blend strength.
    pub refraction_strength: f32,
    /// Specular highlight strength.
    pub specular_strength: f32,
}

/// One water body with its world transform, snapshotted once per frame so the
/// water pass reads it here instead of walking `WATER | GLOBAL_TRANSFORM`.
#[derive(Clone)]
pub struct RenderWater {
    /// The scene entity this water body belongs to.
    pub entity: nightshade_ecs::Entity,
    /// The water body's render parameters.
    pub water: RenderWaterData,
    /// World transform.
    pub transform: nalgebra_glm::Mat4,
}

/// Which cloth particles anchor to the entity transform, mirrored from the
/// `ClothPinning` component enum so the cloth pass does not read that type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RenderClothPinning {
    /// Pin the entire top row of particles.
    #[default]
    TopRow,
    /// Pin only the two top corner particles.
    TopCorners,
    /// Pin nothing, the cloth falls freely.
    None,
}

/// A cloth body's simulation parameters, held as a scene-world component on
/// the cloth entity so the cloth pass queries them alongside the entity's
/// [`crate::render_world::Transform`] and [`crate::render_world::MeshName`].
/// Carries every field so the pass's config-change comparison still detects
/// rebuilds and resets.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct RenderClothData {
    /// Particle grid column count.
    pub columns: u32,
    /// Particle grid row count.
    pub rows: u32,
    /// Cloth width in world units.
    pub width: f32,
    /// Cloth height in world units.
    pub height: f32,
    /// Which particles anchor to the entity transform.
    pub pinning: RenderClothPinning,
    /// Constraint stiffness.
    pub stiffness: f32,
    /// Velocity damping factor.
    pub damping: f32,
    /// Simulation substeps per frame.
    pub substeps: u32,
    /// Constraint solver iterations per substep.
    pub solver_iterations: u32,
    /// Gravity acceleration applied to particles.
    pub gravity: nalgebra_glm::Vec3,
    /// Response strength to scene wind.
    pub wind_response: f32,
    /// Ground collision plane height, `None` to disable.
    pub ground_height: Option<f32>,
    /// Texture repeat count across the cloth.
    pub texture_tiling: nalgebra_glm::Vec2,
    /// Counter bumped to force a simulation reset.
    pub reset_epoch: u32,
}

/// The skinned shadow-casting entities collected once per frame, so the shadow
/// pass enumerates skinned occluders from here. Static and instanced casters
/// carry the [`crate::render_world::ShadowCaster`] tag in the scene world.
#[derive(Clone, Default)]
pub struct RenderShadowCasters {
    /// Skinned entities that cast shadows.
    pub skinned: Vec<nightshade_ecs::Entity>,
}

/// One visible 3D text entity, held as a scene-world component with its world
/// transform, resolved character count, and per-character colors, so the text
/// pass queries it instead of reading the `Text`, `GlobalTransform`, and
/// `TextCharacterColors` components. Only visible text with a cached mesh
/// carries this component.
#[derive(Clone, Default)]
pub struct RenderText {
    /// Cached glyph mesh for the text.
    pub mesh: crate::text_data::TextMesh,
    /// Base fill color.
    pub color: nalgebra_glm::Vec4,
    /// Outline color.
    pub outline_color: nalgebra_glm::Vec4,
    /// Outline width in signed-distance-field units.
    pub outline_width: f32,
    /// Edge smoothing width for the signed distance field.
    pub smoothing: f32,
    /// Whether the text faces the camera.
    pub billboard: bool,
    /// World transform.
    pub transform: nalgebra_glm::Mat4,
    /// Number of drawn characters.
    pub char_count: usize,
    /// Optional per-character color overrides.
    pub character_colors: Option<Vec<Option<nalgebra_glm::Vec4>>>,
}

/// A renderable's bounding volume reduced to the data the renderer needs:
/// the local-space sphere center and radius. Snapshotted once per frame from
/// the `BoundingVolume` component so the renderer never reads that component.
#[derive(Clone, Copy, Debug, Default)]
pub struct RenderBounds {
    /// Bounding sphere center in local space.
    pub center: nalgebra_glm::Vec3,
    /// Bounding sphere radius in local space.
    pub sphere_radius: f32,
}

/// One placement of a baked meshlet mesh in the scene, held as a scene-world
/// component on the meshlet entity. `asset_id` keys the baked asset in the
/// renderer's meshlet asset cache; the mesh's root bvh node is resolved by the
/// meshlet pass from its own upload ledger, so it is not carried here.
#[derive(Clone, Copy, Debug, Default)]
pub struct MeshletPlacement {
    /// World transform for this placement.
    pub transform: nalgebra_glm::Mat4,
    /// Key of the baked asset in the renderer's meshlet asset cache.
    pub asset_id: u64,
    /// The material this places, as the registry index plus one, with zero the
    /// built-in default grey. Resolved from the registry where the placement is
    /// written rather than looked up in the pass against a material table: the
    /// registry is the source of truth and is already current there, so the id
    /// never trails the table's separate rebuild, which is what left every mesh
    /// grey the frame the table lagged the collection.
    pub material_id: u32,
}

/// The renderer's cache of baked meshlet assets, keyed by asset id. Persists
/// across frames like the mesh and texture caches: the engine inserts an asset
/// the first time a [`MeshletPlacement`] references it, and the meshlet pass
/// uploads any asset it has not seen before.
#[cfg(feature = "meshlet")]
pub type MeshletAssetCache =
    std::collections::HashMap<u64, std::sync::Arc<crate::meshlet::asset::MeshletMesh>>;

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

/// Frame-time budget the adaptive sampler drives optional effect quality toward.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PerformanceTarget {
    /// No budget, effects run at full quality.
    #[default]
    Unbounded,
    /// Target 60 frames per second.
    Interactive,
    /// Target 30 frames per second.
    Balanced,
    /// Target 15 frames per second, favoring quality.
    Quality,
}

impl PerformanceTarget {
    /// Target frame time in milliseconds, `None` when unbounded.
    pub fn target_frame_time_ms(self) -> Option<f32> {
        match self {
            PerformanceTarget::Unbounded => None,
            PerformanceTarget::Interactive => Some(1000.0 / 60.0),
            PerformanceTarget::Balanced => Some(1000.0 / 30.0),
            PerformanceTarget::Quality => Some(1000.0 / 15.0),
        }
    }
}

/// Frame-time driven scaling of optional post-process sample counts.
#[derive(Debug, Clone)]
pub struct AdaptiveSamplingState {
    /// Frame-time budget the scaler drives toward.
    pub target: PerformanceTarget,
    /// Exponentially smoothed frame time in milliseconds.
    pub frame_time_rolling_ms: f32,
    /// Frames accumulated into the rolling average.
    pub frame_time_sample_count: u32,
    /// Current `SSAO` sample-count scale (0.25 to 1.0).
    pub ssao_sample_scale: f32,
    /// Current `SSGI` sample-count scale (0.25 to 1.0).
    pub ssgi_sample_scale: f32,
    /// Current `SSR` step-count scale (0.25 to 1.0).
    pub ssr_step_scale: f32,
}

impl Default for AdaptiveSamplingState {
    fn default() -> Self {
        Self {
            target: PerformanceTarget::Unbounded,
            frame_time_rolling_ms: 16.67,
            frame_time_sample_count: 0,
            ssao_sample_scale: 1.0,
            ssgi_sample_scale: 1.0,
            ssr_step_scale: 1.0,
        }
    }
}

impl AdaptiveSamplingState {
    /// Folds a frame time into the rolling average and nudges the sample
    /// scales toward the target budget.
    pub fn record_frame_time(&mut self, frame_time_ms: f32) {
        const WINDOW: u32 = 100;
        if self.frame_time_sample_count == 0 {
            self.frame_time_rolling_ms = frame_time_ms;
            self.frame_time_sample_count = 1;
        } else {
            let alpha = 1.0 / (self.frame_time_sample_count.min(WINDOW) as f32 + 1.0);
            self.frame_time_rolling_ms =
                self.frame_time_rolling_ms * (1.0 - alpha) + frame_time_ms * alpha;
            self.frame_time_sample_count = (self.frame_time_sample_count + 1).min(WINDOW);
        }

        let Some(target_ms) = self.target.target_frame_time_ms() else {
            self.ssao_sample_scale = 1.0;
            self.ssgi_sample_scale = 1.0;
            self.ssr_step_scale = 1.0;
            return;
        };

        let error = self.frame_time_rolling_ms - target_ms;
        let sensitivity = 0.002;
        let max_delta = 0.05;
        let adjustment = (error * sensitivity).clamp(-max_delta, max_delta);

        self.ssao_sample_scale = (self.ssao_sample_scale - adjustment).clamp(0.25, 1.0);
        self.ssgi_sample_scale = (self.ssgi_sample_scale - adjustment).clamp(0.25, 1.0);
        self.ssr_step_scale = (self.ssr_step_scale - adjustment).clamp(0.25, 1.0);
    }
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

/// The reverse-Z perspective matrix the render passes expect (near maps to
/// depth 1, far to depth 0). Use this instead of a stock perspective, whose
/// depth convention will not line up with the depth the passes read.
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

/// A camera tile's rectangle in screen pixels.
#[derive(Debug, Clone, Copy, Default)]
pub struct ViewportRect {
    /// Left edge in screen pixels.
    pub x: f32,
    /// Top edge in screen pixels.
    pub y: f32,
    /// Width in screen pixels.
    pub width: f32,
    /// Height in screen pixels.
    pub height: f32,
}

impl ViewportRect {
    /// True when `screen_pos` lies within the rectangle.
    pub fn contains(&self, screen_pos: nalgebra_glm::Vec2) -> bool {
        screen_pos.x >= self.x
            && screen_pos.x <= self.x + self.width
            && screen_pos.y >= self.y
            && screen_pos.y <= self.y + self.height
    }

    /// Converts a screen position into coordinates relative to the top-left.
    pub fn to_local(&self, screen_pos: nalgebra_glm::Vec2) -> nalgebra_glm::Vec2 {
        nalgebra_glm::Vec2::new(screen_pos.x - self.x, screen_pos.y - self.y)
    }
}
