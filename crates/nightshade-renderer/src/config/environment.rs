//! What the scene sits in: fog, the day-night cycle, the atmosphere
//! presets, and the filtered image-based lighting views.

use serde::{Deserialize, Serialize};

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
#[cfg(feature = "wgpu")]
#[derive(Default)]
pub struct IblViews {
    /// View of the precomputed split-sum `BRDF` lookup texture.
    pub brdf_lut_view: Option<wgpu::TextureView>,
    /// View of the diffuse irradiance cubemap.
    pub irradiance_view: Option<wgpu::TextureView>,
    /// View of the prefiltered specular cubemap.
    pub prefiltered_view: Option<wgpu::TextureView>,
}

#[cfg(feature = "wgpu")]
impl Clone for IblViews {
    fn clone(&self) -> Self {
        Self {
            brdf_lut_view: self.brdf_lut_view.clone(),
            irradiance_view: self.irradiance_view.clone(),
            prefiltered_view: self.prefiltered_view.clone(),
        }
    }
}
