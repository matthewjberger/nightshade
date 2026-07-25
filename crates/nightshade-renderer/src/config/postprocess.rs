//! Settings and GPU-facing state for the screen-space effects: depth of
//! field, tonemapping, color grading, and the packed effects uniform the
//! post-process passes read.

use serde::{Deserialize, Serialize};

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
