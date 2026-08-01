use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Fog {
    pub color: [f32; 3],
    pub start: f32,
    pub end: f32,
    pub mode: FogMode,
    pub volumetric: bool,
    pub density: f32,
    pub anisotropy: f32,
    pub height: f32,
    pub height_falloff: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FogMode {
    #[default]
    Linear,
    Exponential,
    ExponentialSquared,
}

impl FogMode {
    pub fn as_u32(self) -> u32 {
        match self {
            FogMode::Linear => 1,
            FogMode::Exponential => 2,
            FogMode::ExponentialSquared => 3,
        }
    }

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
            volumetric: false,
            density: 0.0,
            anisotropy: 0.3,
            height: 0.0,
            height_falloff: 0.0,
        }
    }
}

#[derive(Clone)]
pub struct DayNightState {
    pub hour: f32,
    pub speed: f32,
    pub auto_cycle: bool,
    pub sun_entity: Option<nightshade::ecs::Entity>,
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

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, enum2schema::Schema,
)]
#[schema(string_enum)]
pub enum Atmosphere {
    #[default]
    None,
    Sky,
    CloudySky,
    Space,
    Nebula,
    Sunset,
    DayNight,
    Hdr,
    HdrMip0,
    HdrMip1,
    HdrMip2,
    HdrMip3,
    HdrMip4,
    Irradiance,
    PrefilterMip0,
    PrefilterMip1,
    PrefilterMip2,
    PrefilterMip3,
    PrefilterMip4,
}

impl Atmosphere {
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

    pub fn next(self) -> Self {
        let all = Self::ALL;
        let current_index = all.iter().position(|&a| a == self).unwrap_or(0);
        let next_index = (current_index + 1) % all.len();
        all[next_index]
    }

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
