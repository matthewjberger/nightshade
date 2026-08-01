use nalgebra_glm::Vec3;
use nightshade_ecs::Entity;

pub const AIR: u8 = 0;
pub const MAX_SIDE: u32 = 128;
pub const MAX_EMITTERS: usize = 32;
pub const MAX_ECHOES: usize = 1024;
pub const MAX_RAY_STEPS: usize = 640;
pub const MAX_MATERIALS: usize = 8;
pub const STONE: u8 = 1;
pub const TIMBER: u8 = 2;
pub const CLOTH: u8 = 3;
pub const GLASS: u8 = 4;
pub const LOW_BAND_HERTZ: f32 = 300.0;
pub const HIGH_BAND_HERTZ: f32 = 4000.0;
pub const AIR_SOAK_HIGH: f32 = 0.006018;
pub const ECHO_BUCKETS: usize = 32;
pub const BUCKET_SECONDS: f32 = 0.05;
pub const SOUND_SPEED: f32 = 343.0;

#[derive(Debug, Clone, Copy)]
pub struct MaterialAcoustics {
    pub absorb_low: f32,
    pub absorb_high: f32,
    pub crossing_low: f32,
    pub crossing_high: f32,
    pub scatter: f32,
    pub flat_low: f32,
    pub flat_high: f32,
    pub solid: f32,
}

const NOTHING: MaterialAcoustics = MaterialAcoustics {
    absorb_low: 0.0,
    absorb_high: 0.0,
    crossing_low: 1.0e6,
    crossing_high: 1.0e6,
    scatter: 0.0,
    flat_low: 0.0,
    flat_high: 0.0,
    solid: 1.0,
};

pub const MATERIALS: [MaterialAcoustics; MAX_MATERIALS] = [
    NOTHING,
    MaterialAcoustics {
        absorb_low: 0.03,
        absorb_high: 0.02,
        crossing_low: 0.10,
        crossing_high: 0.05,
        scatter: 0.05,
        flat_low: 0.10,
        flat_high: 0.25,
        solid: 1.0,
    },
    MaterialAcoustics {
        absorb_low: 0.08,
        absorb_high: 0.12,
        crossing_low: 0.80,
        crossing_high: 0.35,
        scatter: 0.15,
        flat_low: 0.15,
        flat_high: 0.35,
        solid: 1.0,
    },
    MaterialAcoustics {
        absorb_low: 0.20,
        absorb_high: 0.55,
        crossing_low: 1.60,
        crossing_high: 0.60,
        scatter: 0.40,
        flat_low: 0.20,
        flat_high: 0.50,
        solid: 1.0,
    },
    MaterialAcoustics {
        absorb_low: 0.02,
        absorb_high: 0.03,
        crossing_low: 1.0e6,
        crossing_high: 1.0e6,
        scatter: 0.02,
        flat_low: 0.10,
        flat_high: 0.25,
        solid: 0.0,
    },
    NOTHING,
    NOTHING,
    NOTHING,
];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RayKind {
    #[default]
    Clear,
    Blocked,
    Bounce,
    Home,
    Found,
}

impl RayKind {
    pub fn from_code(code: f32) -> Self {
        match code as u32 {
            1 => Self::Blocked,
            2 => Self::Bounce,
            3 => Self::Home,
            5 => Self::Found,
            _ => Self::Clear,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RayStep {
    pub start: Vec3,
    pub end: Vec3,
    pub from_travel: f32,
    pub to_travel: f32,
    pub kind: RayKind,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EchoPoint {
    pub place: Vec3,
    pub strength: f32,
    pub voice: EmitterVoice,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum EmitterVoice {
    #[default]
    Tone,
    Step,
    Blast,
}

impl EmitterVoice {
    pub fn from_code(code: u32) -> Self {
        match code {
            1 => Self::Step,
            2 => Self::Blast,
            _ => Self::Tone,
        }
    }

    pub fn code(self) -> u32 {
        match self {
            Self::Tone => 0,
            Self::Step => 1,
            Self::Blast => 2,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EmitterPlace {
    pub entity: Entity,
    pub position: Vec3,
    pub reach: f32,
    pub voice: EmitterVoice,
    pub hold: bool,
    pub own_reverb: bool,
    pub listener_reverb: bool,
    pub reverb_rays: u32,
    pub reverb_bounces: u32,
    pub occlusion_rays: u32,
    pub occlusion_bounces: u32,
    pub permeation_rays: u32,
    pub permeation_bounces: u32,
    pub ambient_occlusion_rays: u32,
    pub ambient_occlusion_bounces: u32,
    pub ambient_permeation_rays: u32,
    pub ambient_permeation_bounces: u32,
    pub bucket_seconds: f32,
    pub seed: u32,
    pub clamped: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SoundPath {
    pub listener_reverb: bool,
    pub direct: f32,
    pub gain_low: f32,
    pub gain_high: f32,
    pub travel: f32,
    pub through: f32,
    pub around: f32,
}

impl SoundPath {
    pub fn cutoff(&self, low_band: f32, high_band: f32) -> f32 {
        let ratio = (self.gain_high / self.gain_low.max(0.01)).clamp(0.0, 0.999);
        if ratio <= 0.0 {
            return low_band;
        }
        (high_band * ratio / (1.0 - ratio * ratio).sqrt()).clamp(low_band, 20000.0)
    }
}

pub fn air_gain_high(settings: &crate::contract::resources::AcousticsSettings) -> f32 {
    air_gain_at(
        settings.high_band,
        settings.temperature,
        settings.humidity,
        settings.pressure,
    )
}

fn air_gain_at(hertz: f32, celsius: f32, humidity: f32, pascals: f32) -> f32 {
    let kelvin = celsius + 273.15;
    let relative_pressure = (pascals / 101_325.0).max(1.0e-3);
    let warmth = kelvin / 293.15;
    let saturated = 10.0f32.powf(4.6151 - 6.8346 * (273.16 / kelvin).powf(1.261));
    let molar = (humidity.clamp(0.0, 1.0) * 100.0) * saturated / relative_pressure;
    let oxygen = relative_pressure * (24.0 + 4.04e4 * molar * (0.02 + molar) / (0.391 + molar));
    let nitrogen = relative_pressure
        * warmth.powf(-0.5)
        * (9.0 + 280.0 * molar * (-4.17 * (warmth.powf(-1.0 / 3.0) - 1.0)).exp());
    let squared = hertz * hertz;
    let relaxing = 0.01275 * (-2239.1 / kelvin).exp() / (oxygen + squared / oxygen)
        + 0.1068 * (-3352.0 / kelvin).exp() / (nitrogen + squared / nitrogen);
    let decibels = 8.686
        * squared
        * (1.84e-11 * warmth.sqrt() / relative_pressure + warmth.powf(-2.5) * relaxing);
    10.0f32.powf(-decibels / 20.0)
}
