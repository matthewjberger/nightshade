use crate::contract::data::EmitterVoice;
use nightshade_ecs::Entity;

#[derive(Debug, Clone, Copy, Default)]
pub struct AcousticListener;

#[derive(Debug, Clone, Copy)]
pub struct AcousticEmitter {
    pub reach: f32,
    pub voice: EmitterVoice,
    pub sounding: bool,
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
    pub relative_reverb: bool,
    pub leech_of: Option<Entity>,
}

impl Default for AcousticEmitter {
    fn default() -> Self {
        Self {
            reach: 40.0,
            voice: EmitterVoice::Tone,
            sounding: true,
            hold: false,
            own_reverb: false,
            listener_reverb: true,
            reverb_rays: 0,
            reverb_bounces: 0,
            occlusion_rays: 0,
            occlusion_bounces: 0,
            permeation_rays: 0,
            permeation_bounces: 0,
            ambient_occlusion_rays: 0,
            ambient_occlusion_bounces: 0,
            ambient_permeation_rays: 0,
            ambient_permeation_bounces: 0,
            bucket_seconds: 0.0,
            seed: 0,
            clamped: true,
            relative_reverb: false,
            leech_of: None,
        }
    }
}

impl AcousticEmitter {
    pub fn new(reach: f32) -> Self {
        Self {
            reach,
            ..Default::default()
        }
    }

    pub fn with_voice(mut self, voice: EmitterVoice) -> Self {
        self.voice = voice;
        self
    }

    pub fn held(mut self) -> Self {
        self.hold = true;
        self
    }

    pub fn with_own_reverb(mut self) -> Self {
        self.own_reverb = true;
        self
    }

    pub fn alone(mut self) -> Self {
        self.listener_reverb = false;
        self
    }

    pub fn with_reverb_budget(mut self, rays: u32, bounces: u32) -> Self {
        self.reverb_rays = rays;
        self.reverb_bounces = bounces;
        self
    }

    pub fn with_occlusion_budget(mut self, rays: u32, bounces: u32) -> Self {
        self.occlusion_rays = rays;
        self.occlusion_bounces = bounces;
        self
    }

    pub fn with_permeation_budget(mut self, rays: u32, bounces: u32) -> Self {
        self.permeation_rays = rays;
        self.permeation_bounces = bounces;
        self
    }

    pub fn with_ambient_budget(mut self, rays: u32, bounces: u32) -> Self {
        self.ambient_occlusion_rays = rays;
        self.ambient_occlusion_bounces = bounces;
        self.ambient_permeation_rays = rays;
        self.ambient_permeation_bounces = bounces;
        self
    }

    pub fn with_echogram(mut self, bucket_seconds: f32) -> Self {
        self.bucket_seconds = bucket_seconds;
        self
    }

    pub fn with_seed(mut self, seed: u32) -> Self {
        self.seed = seed;
        self
    }

    pub fn unclamped(mut self) -> Self {
        self.clamped = false;
        self
    }

    pub fn as_reference(mut self) -> Self {
        self.relative_reverb = true;
        self
    }

    pub fn leeching(mut self, parent: Entity) -> Self {
        self.leech_of = Some(parent);
        self
    }
}
