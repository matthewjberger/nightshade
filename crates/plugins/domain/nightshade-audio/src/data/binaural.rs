use kira::Frame;
use kira::effect::{Effect, EffectBuilder};
use kira::info::Info;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

const HEAD_RADIUS: f32 = 0.0875;
const SOUND_SPEED: f32 = 343.0;
const LONGEST_DELAY: usize = 192;
const REFERENCE_RATE: f32 = 44_100.0;
const OPEN_SHADOW: f32 = 1.0;
const CLOSED_SHADOW: f32 = 0.16;
const FAR_EAR_QUIET: f32 = 0.55;
const FOLLOW: f32 = 0.0015;

pub struct BinauralBuilder {
    aim: Arc<AtomicU32>,
}

impl Default for BinauralBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl BinauralBuilder {
    pub fn new() -> Self {
        Self {
            aim: Arc::new(AtomicU32::new(0.0f32.to_bits())),
        }
    }
}

pub struct BinauralHandle {
    aim: Arc<AtomicU32>,
}

impl BinauralHandle {
    pub fn set_aim(&self, radians: f32) {
        self.aim.store(radians.to_bits(), Ordering::Relaxed);
    }
}

impl EffectBuilder for BinauralBuilder {
    type Handle = BinauralHandle;

    fn build(self) -> (Box<dyn Effect>, Self::Handle) {
        let handle = BinauralHandle {
            aim: Arc::clone(&self.aim),
        };
        let effect = Binaural {
            aim: self.aim,
            held: 0.0,
            line: [0.0; LONGEST_DELAY],
            written: 0,
            left_shadow: 0.0,
            right_shadow: 0.0,
            sample_rate: 44_100.0,
        };
        (Box::new(effect), handle)
    }
}

struct Binaural {
    aim: Arc<AtomicU32>,
    held: f32,
    line: [f32; LONGEST_DELAY],
    written: usize,
    left_shadow: f32,
    right_shadow: f32,
    sample_rate: f32,
}

impl Binaural {
    fn read_back(&self, samples: f32) -> f32 {
        let span = samples.clamp(0.0, (LONGEST_DELAY - 2) as f32);
        let whole = span.floor() as usize;
        let part = span - whole as f32;
        let first = (self.written + LONGEST_DELAY - whole) % LONGEST_DELAY;
        let second = (first + LONGEST_DELAY - 1) % LONGEST_DELAY;
        self.line[first] * (1.0 - part) + self.line[second] * part
    }
}

impl Effect for Binaural {
    fn init(&mut self, sample_rate: u32, _internal_buffer_size: usize) {
        self.sample_rate = sample_rate as f32;
    }

    fn on_change_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate as f32;
    }

    fn process(&mut self, input: &mut [Frame], _dt: f64, _info: &Info<'_>) {
        let wanted = f32::from_bits(self.aim.load(Ordering::Relaxed));
        let rates = REFERENCE_RATE / self.sample_rate;
        let follow = 1.0 - (1.0 - FOLLOW).powf(rates);
        for frame in input.iter_mut() {
            self.held += turn_toward(self.held, wanted) * follow;
            self.held = wrapped(self.held);
            let aside = self.held.sin();
            let away = aside.abs();

            let turned = self.held.abs();
            let ahead = if turned > std::f32::consts::FRAC_PI_2 {
                std::f32::consts::PI - turned
            } else {
                turned
            };
            let lag = (HEAD_RADIUS / SOUND_SPEED) * (ahead + away) * self.sample_rate;

            let mono = (frame.left + frame.right) * 0.5;
            self.written = (self.written + 1) % LONGEST_DELAY;
            self.line[self.written] = mono;

            let near = self.read_back(0.0);
            let far = self.read_back(lag);

            let shadow =
                1.0 - (1.0 - (OPEN_SHADOW - (OPEN_SHADOW - CLOSED_SHADOW) * away)).powf(rates);
            let quiet = 1.0 - (1.0 - FAR_EAR_QUIET) * away;

            if aside >= 0.0 {
                self.right_shadow = near;
                self.left_shadow += (far - self.left_shadow) * shadow;
                frame.left = self.left_shadow * quiet;
                frame.right = self.right_shadow;
            } else {
                self.left_shadow = near;
                self.right_shadow += (far - self.right_shadow) * shadow;
                frame.left = self.left_shadow;
                frame.right = self.right_shadow * quiet;
            }
        }
    }
}

fn wrapped(angle: f32) -> f32 {
    if angle > std::f32::consts::PI {
        angle - std::f32::consts::TAU
    } else if angle < -std::f32::consts::PI {
        angle + std::f32::consts::TAU
    } else {
        angle
    }
}

fn turn_toward(held: f32, wanted: f32) -> f32 {
    let mut apart = wanted - held;
    while apart > std::f32::consts::PI {
        apart -= std::f32::consts::TAU;
    }
    while apart < -std::f32::consts::PI {
        apart += std::f32::consts::TAU;
    }
    apart
}
