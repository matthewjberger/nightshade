use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DepthOfFieldQuality {
    Low,
    #[default]
    Medium,
    High,
}

impl DepthOfFieldQuality {
    pub const ALL: &'static [DepthOfFieldQuality] = &[
        DepthOfFieldQuality::Low,
        DepthOfFieldQuality::Medium,
        DepthOfFieldQuality::High,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            DepthOfFieldQuality::Low => "Low",
            DepthOfFieldQuality::Medium => "Medium",
            DepthOfFieldQuality::High => "High",
        }
    }

    pub fn sample_count(&self) -> u32 {
        match self {
            DepthOfFieldQuality::Low => 8,
            DepthOfFieldQuality::Medium => 16,
            DepthOfFieldQuality::High => 32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DepthOfField {
    pub enabled: bool,
    pub focus_distance: f32,
    pub focus_range: f32,
    pub max_blur_radius: f32,
    pub bokeh_threshold: f32,
    pub bokeh_intensity: f32,
    pub quality: DepthOfFieldQuality,
    pub visualize_coc: bool,
    pub tilt_shift_enabled: bool,
    pub tilt_shift_angle: f32,
    pub tilt_shift_center: f32,
    pub tilt_shift_blur_amount: f32,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TonemapAlgorithm {
    #[default]
    Aces,
    Aces2,
    Reinhard,
    ReinhardExtended,
    Uncharted2,
    AgX,
    Neutral,
    None,
}

impl TonemapAlgorithm {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ColorGradingPreset {
    #[default]
    Default,
    Vibrant,
    Cinematic,
    Muted,
    HighContrast,
    Warm,
    Cool,
    Retro,
    Desaturated,
    Custom,
}

impl ColorGradingPreset {
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ColorGrading {
    pub exposure: f32,
    pub exposure_compensation_ev: f32,
    pub auto_exposure: bool,
    pub auto_exposure_target: f32,
    pub auto_exposure_rate: f32,
    pub auto_exposure_min_ev: f32,
    pub auto_exposure_max_ev: f32,
    pub gamma: f32,
    pub saturation: f32,
    pub brightness: f32,
    pub contrast: f32,
    pub vibrance: f32,
    pub vignette_intensity: f32,
    pub vignette_radius: f32,
    pub vignette_smoothness: f32,
    pub chromatic_aberration: f32,
    pub color_lut_weight: f32,
    pub tonemap_algorithm: TonemapAlgorithm,
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
