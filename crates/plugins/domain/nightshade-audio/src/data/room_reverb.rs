use kira::Frame;
use kira::effect::{Effect, EffectBuilder};
use kira::info::Info;
use std::f32::consts::TAU;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

pub const GAIN: usize = 0;
pub const GAIN_LOW: usize = 1;
pub const GAIN_HIGH: usize = 2;
pub const DECAY: usize = 3;
pub const DECAY_LOW_RATIO: usize = 4;
pub const DECAY_HIGH_RATIO: usize = 5;
pub const DIFFUSION: usize = 6;
pub const DENSITY: usize = 7;
pub const EARLY_GAIN: usize = 8;
pub const EARLY_DELAY: usize = 9;
pub const EARLY_LEFT: usize = 10;
pub const EARLY_RIGHT: usize = 11;
pub const LATE_GAIN: usize = 12;
pub const LATE_DELAY: usize = 13;
pub const LATE_LEFT: usize = 14;
pub const LATE_RIGHT: usize = 15;
pub const ECHO_TIME: usize = 16;
pub const ECHO_DEPTH: usize = 17;
pub const SWAY_TIME: usize = 18;
pub const SWAY_DEPTH: usize = 19;
pub const AIR_HIGH: usize = 20;
pub const LOW_HERTZ: usize = 21;
pub const HIGH_HERTZ: usize = 22;
pub const SETTING_COUNT: usize = 23;

const LINES: usize = 8;
const DIFFUSERS: usize = 4;
const EARLY_TAPS: usize = 8;
const REFERENCE_RATE: f32 = 44_100.0;
const LONGEST_LINE: usize = 8192;
const LONGEST_DIFFUSER: usize = 1024;
const LONGEST_EARLY: usize = 16384;
const SWAY_SAMPLES: f32 = 12.0;
const TIGHTEST_FEEDBACK: f32 = 0.9995;

const LINE_LENGTHS: [usize; LINES] = [1553, 1861, 2053, 2251, 2399, 2593, 2731, 2879];
const DIFFUSER_LENGTHS: [usize; DIFFUSERS] = [353, 191, 113, 59];
const EARLY_TIMES: [f32; EARLY_TAPS] = [
    0.0043, 0.0097, 0.0151, 0.0203, 0.0271, 0.0339, 0.0417, 0.0503,
];
const EARLY_GAINS: [f32; EARLY_TAPS] = [1.0, -0.82, 0.71, -0.63, 0.54, -0.47, 0.39, -0.31];

pub struct RoomReverbBuilder {
    settings: Arc<[AtomicU32; SETTING_COUNT]>,
}

impl Default for RoomReverbBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RoomReverbBuilder {
    pub fn new() -> Self {
        let settings: [AtomicU32; SETTING_COUNT] =
            std::array::from_fn(|_| AtomicU32::new(0.0f32.to_bits()));
        let builder = Self {
            settings: Arc::new(settings),
        };
        let handle = RoomReverbHandle {
            settings: Arc::clone(&builder.settings),
        };
        for (slot, value) in [
            (GAIN, 1.0),
            (GAIN_LOW, 1.0),
            (GAIN_HIGH, 1.0),
            (DECAY, 1.5),
            (DECAY_LOW_RATIO, 1.0),
            (DECAY_HIGH_RATIO, 0.7),
            (DIFFUSION, 1.0),
            (DENSITY, 1.0),
            (EARLY_GAIN, 0.6),
            (EARLY_DELAY, 0.007),
            (EARLY_LEFT, 1.0),
            (EARLY_RIGHT, 1.0),
            (LATE_GAIN, 1.0),
            (LATE_DELAY, 0.011),
            (LATE_LEFT, 1.0),
            (LATE_RIGHT, 1.0),
            (ECHO_TIME, 0.25),
            (ECHO_DEPTH, 0.0),
            (SWAY_TIME, 0.25),
            (SWAY_DEPTH, 0.0),
            (AIR_HIGH, 0.994),
            (LOW_HERTZ, 250.0),
            (HIGH_HERTZ, 5000.0),
        ] {
            handle.set(slot, value);
        }
        builder
    }
}

#[derive(Clone)]
pub struct RoomReverbHandle {
    settings: Arc<[AtomicU32; SETTING_COUNT]>,
}

impl RoomReverbHandle {
    pub fn set(&self, slot: usize, value: f32) {
        if slot < SETTING_COUNT {
            self.settings[slot].store(value.to_bits(), Ordering::Relaxed);
        }
    }

    fn read(&self, slot: usize) -> f32 {
        f32::from_bits(self.settings[slot].load(Ordering::Relaxed))
    }
}

impl EffectBuilder for RoomReverbBuilder {
    type Handle = RoomReverbHandle;

    fn build(self) -> (Box<dyn Effect>, Self::Handle) {
        let handle = RoomReverbHandle {
            settings: Arc::clone(&self.settings),
        };
        let effect = RoomReverb {
            reading: handle.clone(),
            sample_rate: REFERENCE_RATE,
            lines: std::array::from_fn(|_| Line::default()),
            diffusers: std::array::from_fn(|_| Diffuser::default()),
            early: vec![0.0; LONGEST_EARLY],
            early_written: 0,
            early_tone: Tone::default(),
            late_tone: Tone::default(),
            swayed: 0.0,
            echoed: 0.0,
        };
        (Box::new(effect), handle)
    }
}

#[derive(Default, Clone, Copy)]
struct Pole {
    held: f32,
}

impl Pole {
    fn pass(&mut self, input: f32, coefficient: f32) -> f32 {
        self.held += (input - self.held) * coefficient;
        self.held
    }
}

#[derive(Default)]
struct Tone {
    low: Pole,
    high: Pole,
    air: Pole,
}

impl Tone {
    fn shape(&mut self, input: f32, shape: &Shaping) -> f32 {
        let under = self.low.pass(input, shape.low_pole);
        let shelved = input + (shape.gain_low - 1.0) * under;
        let below = self.high.pass(shelved, shape.high_pole);
        let toned = below + (shelved - below) * shape.gain_high;
        self.air.pass(toned, shape.air_pole)
    }
}

struct Shaping {
    low_pole: f32,
    high_pole: f32,
    air_pole: f32,
    gain_low: f32,
    gain_high: f32,
}

struct Line {
    samples: Vec<f32>,
    written: usize,
    damping: Pole,
    lifting: Pole,
}

impl Default for Line {
    fn default() -> Self {
        Self {
            samples: vec![0.0; LONGEST_LINE],
            written: 0,
            damping: Pole::default(),
            lifting: Pole::default(),
        }
    }
}

struct Diffuser {
    samples: Vec<f32>,
    written: usize,
}

impl Default for Diffuser {
    fn default() -> Self {
        Self {
            samples: vec![0.0; LONGEST_DIFFUSER],
            written: 0,
        }
    }
}

struct RoomReverb {
    reading: RoomReverbHandle,
    sample_rate: f32,
    lines: [Line; LINES],
    diffusers: [Diffuser; DIFFUSERS],
    early: Vec<f32>,
    early_written: usize,
    early_tone: Tone,
    late_tone: Tone,
    swayed: f32,
    echoed: f32,
}

fn plain_pole(hertz: f32, sample_rate: f32) -> f32 {
    (1.0 - (-TAU * hertz / sample_rate).exp()).clamp(0.0, 1.0)
}

fn pole_reaching(target: f32, hertz: f32, sample_rate: f32) -> f32 {
    let wanted = target.clamp(0.001, 0.999);
    let turn = (TAU * hertz / sample_rate).min(std::f32::consts::PI);
    let squared = wanted * wanted;
    let along = 1.0 - squared;
    if along.abs() < 1.0e-6 {
        return 1.0;
    }
    let across = 2.0 * (squared * turn.cos() - 1.0);
    let under = across * across - 4.0 * along * along;
    if under < 0.0 {
        return 1.0;
    }
    let held = (-across - under.sqrt()) / (2.0 * along);
    1.0 - held.clamp(0.0, 0.9999)
}

fn hadamard(values: &mut [f32; LINES]) {
    let mut span = 1;
    while span < LINES {
        let mut start = 0;
        while start < LINES {
            for index in start..start + span {
                let first = values[index];
                let second = values[index + span];
                values[index] = first + second;
                values[index + span] = first - second;
            }
            start += span * 2;
        }
        span *= 2;
    }
    let scale = 1.0 / (LINES as f32).sqrt();
    for value in values.iter_mut() {
        *value *= scale;
    }
}

impl RoomReverb {
    fn line_length(&self, index: usize) -> usize {
        let scaled = LINE_LENGTHS[index] as f32 * self.sample_rate / REFERENCE_RATE;
        (scaled as usize).clamp(16, LONGEST_LINE - 2)
    }

    fn diffuser_length(&self, index: usize) -> usize {
        let scaled = DIFFUSER_LENGTHS[index] as f32 * self.sample_rate / REFERENCE_RATE;
        (scaled as usize).clamp(8, LONGEST_DIFFUSER - 2)
    }

    fn early_at(&self, seconds: f32) -> f32 {
        let samples = (seconds * self.sample_rate) as usize;
        let span = samples.min(LONGEST_EARLY - 1);
        let read = (self.early_written + LONGEST_EARLY - span) % LONGEST_EARLY;
        self.early[read]
    }
}

impl Effect for RoomReverb {
    fn init(&mut self, sample_rate: u32, _internal_buffer_size: usize) {
        self.sample_rate = sample_rate as f32;
    }

    fn on_change_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate as f32;
    }

    fn process(&mut self, input: &mut [Frame], _dt: f64, _info: &Info<'_>) {
        let gain = self.reading.read(GAIN);
        let decay = self.reading.read(DECAY).max(0.05);
        let low_ratio = self.reading.read(DECAY_LOW_RATIO).clamp(0.1, 1.0);
        let high_ratio = self.reading.read(DECAY_HIGH_RATIO).clamp(0.1, 1.0);
        let diffusion = self.reading.read(DIFFUSION).clamp(0.0, 1.0);
        let density = self.reading.read(DENSITY).clamp(0.0, 1.0);
        let early_gain = self.reading.read(EARLY_GAIN);
        let early_delay = self.reading.read(EARLY_DELAY).clamp(0.0, 0.3);
        let early_left = self.reading.read(EARLY_LEFT).clamp(0.0, 2.0);
        let early_right = self.reading.read(EARLY_RIGHT).clamp(0.0, 2.0);
        let late_gain = self.reading.read(LATE_GAIN);
        let late_delay = self.reading.read(LATE_DELAY).clamp(0.0, 0.3);
        let late_left = self.reading.read(LATE_LEFT).clamp(0.0, 2.0);
        let late_right = self.reading.read(LATE_RIGHT).clamp(0.0, 2.0);
        let echo_time = self.reading.read(ECHO_TIME).clamp(0.05, 2.0);
        let echo_depth = self.reading.read(ECHO_DEPTH).clamp(0.0, 1.0);
        let sway_time = self.reading.read(SWAY_TIME).clamp(0.04, 4.0);
        let sway_depth = self.reading.read(SWAY_DEPTH).clamp(0.0, 1.0);

        let low_hertz = self.reading.read(LOW_HERTZ).clamp(20.0, 2000.0);
        let high_hertz = self.reading.read(HIGH_HERTZ).clamp(1000.0, 20000.0);
        let shape = Shaping {
            low_pole: plain_pole(low_hertz, self.sample_rate),
            high_pole: plain_pole(high_hertz, self.sample_rate),
            air_pole: pole_reaching(
                self.reading.read(AIR_HIGH).clamp(0.1, 1.0),
                high_hertz,
                self.sample_rate,
            ),
            gain_low: self.reading.read(GAIN_LOW).clamp(0.0, 1.0),
            gain_high: self.reading.read(GAIN_HIGH).clamp(0.0, 1.0),
        };

        let used = ((density * LINES as f32).round() as usize).clamp(4, LINES);
        let diffusing = ((diffusion * DIFFUSERS as f32).round() as usize).min(DIFFUSERS);
        let allpass = 0.5 + 0.25 * diffusion;

        let mut feedback = [0.0f32; LINES];
        let mut damping = [0.0f32; LINES];
        let mut lifting = [0.0f32; LINES];
        for index in 0..LINES {
            let length = self.line_length(index) as f32;
            let held = 10.0f32.powf(-3.0 * length / (self.sample_rate * decay));
            feedback[index] = held.clamp(0.0, TIGHTEST_FEEDBACK);
            let quicker_high =
                10.0f32.powf(-3.0 * length / (self.sample_rate * decay * high_ratio));
            damping[index] = pole_reaching(
                quicker_high / held.max(1.0e-6),
                high_hertz,
                self.sample_rate,
            );
            let quicker_low = 10.0f32.powf(-3.0 * length / (self.sample_rate * decay * low_ratio));
            lifting[index] = (quicker_low / held.max(1.0e-6)).clamp(0.0, 1.0);
        }
        let low_pole = shape.low_pole;

        let sway_step = TAU / (sway_time * self.sample_rate);
        let echo_step = TAU / (echo_time * self.sample_rate);

        for frame in input.iter_mut() {
            let mono = (frame.left + frame.right) * 0.5;

            self.early_written = (self.early_written + 1) % LONGEST_EARLY;
            self.early[self.early_written] = mono;

            let mut early = 0.0;
            for tap in 0..EARLY_TAPS {
                early += self.early_at(early_delay + EARLY_TIMES[tap]) * EARLY_GAINS[tap];
            }
            early *= early_gain / EARLY_TAPS as f32;

            let mut spread = self.early_at(late_delay);
            for index in 0..diffusing {
                let length = self.diffuser_length(index);
                let diffuser = &mut self.diffusers[index];
                let read = (diffuser.written + LONGEST_DIFFUSER - length) % LONGEST_DIFFUSER;
                let held = diffuser.samples[read];
                let written = spread + held * allpass;
                diffuser.written = (diffuser.written + 1) % LONGEST_DIFFUSER;
                diffuser.samples[diffuser.written] = written;
                spread = held - written * allpass;
            }

            self.swayed += sway_step;
            if self.swayed > TAU {
                self.swayed -= TAU;
            }
            let wander = self.swayed.sin() * sway_depth * SWAY_SAMPLES;

            let mut taken = [0.0f32; LINES];
            for (index, held) in taken.iter_mut().enumerate().take(used) {
                let length = self.line_length(index);
                let offset = (length as f32 - wander).clamp(4.0, (LONGEST_LINE - 2) as f32);
                let whole = offset.floor() as usize;
                let part = offset - whole as f32;
                let line = &self.lines[index];
                let first = (line.written + LONGEST_LINE - whole) % LONGEST_LINE;
                let second = (first + LONGEST_LINE - 1) % LONGEST_LINE;
                *held = line.samples[first] * (1.0 - part) + line.samples[second] * part;
            }

            let mut mixed = taken;
            hadamard(&mut mixed);

            let mut late = 0.0;
            for index in 0..used {
                let line = &mut self.lines[index];
                let damped = line.damping.pass(mixed[index], damping[index]);
                let under = line.lifting.pass(damped, low_pole);
                let shaped = damped + (lifting[index] - 1.0) * under;
                line.written = (line.written + 1) % LONGEST_LINE;
                line.samples[line.written] = spread + shaped * feedback[index];
                late += taken[index];
            }
            late *= late_gain / used as f32;

            self.echoed += echo_step;
            if self.echoed > TAU {
                self.echoed -= TAU;
            }
            let flutter = 1.0 - echo_depth * 0.5 * (1.0 - self.echoed.sin());

            let early_out = self.early_tone.shape(early, &shape) * gain;
            let late_out = self.late_tone.shape(late * flutter, &shape) * gain;

            frame.left = early_out * early_left + late_out * late_left;
            frame.right = early_out * early_right + late_out * late_right;
        }
    }
}
