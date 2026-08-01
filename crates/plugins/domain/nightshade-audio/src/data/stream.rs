use kira::Frame;
use kira::sound::{Sound, SoundData};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

const LONGEST_QUEUE: usize = 480_000;

#[derive(Clone)]
pub struct StreamHandle {
    queue: Arc<Mutex<VecDeque<Frame>>>,
    open: Arc<AtomicBool>,
    rate: u32,
}

impl StreamHandle {
    pub fn push_frames(&self, frames: &[Frame]) {
        let Ok(mut held) = self.queue.lock() else {
            return;
        };
        for frame in frames {
            if held.len() >= LONGEST_QUEUE {
                held.pop_front();
            }
            held.push_back(*frame);
        }
    }

    pub fn push_mono(&self, samples: &[f32]) {
        let frames: Vec<Frame> = samples
            .iter()
            .map(|sample| Frame {
                left: *sample,
                right: *sample,
            })
            .collect();
        self.push_frames(&frames);
    }

    pub fn push_interleaved(&self, samples: &[f32], channels: u32) {
        if channels <= 1 {
            self.push_mono(samples);
            return;
        }
        let step = channels as usize;
        let frames: Vec<Frame> = samples
            .chunks_exact(step)
            .map(|chunk| Frame {
                left: chunk[0],
                right: chunk[1],
            })
            .collect();
        self.push_frames(&frames);
    }

    pub fn waiting(&self) -> usize {
        self.queue.lock().map(|held| held.len()).unwrap_or(0)
    }

    pub fn rate(&self) -> u32 {
        self.rate
    }

    pub fn is_open(&self) -> bool {
        self.open.load(Ordering::Relaxed)
    }

    pub fn close(&self) {
        self.open.store(false, Ordering::Relaxed);
    }
}

pub struct StreamSoundData {
    queue: Arc<Mutex<VecDeque<Frame>>>,
    open: Arc<AtomicBool>,
    rate: u32,
}

impl StreamSoundData {
    pub fn new(rate: u32) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            open: Arc::new(AtomicBool::new(true)),
            rate: rate.max(1),
        }
    }

    pub fn handle(&self) -> StreamHandle {
        StreamHandle {
            queue: Arc::clone(&self.queue),
            open: Arc::clone(&self.open),
            rate: self.rate,
        }
    }
}

impl SoundData for StreamSoundData {
    type Error = ();
    type Handle = StreamHandle;

    fn into_sound(self) -> Result<(Box<dyn Sound>, Self::Handle), Self::Error> {
        let handle = self.handle();
        let sound = StreamSound {
            queue: self.queue,
            open: self.open,
            rate: self.rate as f64,
            held: Frame::ZERO,
            carried: 0.0,
        };
        Ok((Box::new(sound), handle))
    }
}

struct StreamSound {
    queue: Arc<Mutex<VecDeque<Frame>>>,
    open: Arc<AtomicBool>,
    rate: f64,
    held: Frame,
    carried: f64,
}

impl StreamSound {
    fn take(&mut self) -> Frame {
        let Ok(mut held) = self.queue.lock() else {
            return Frame::ZERO;
        };
        held.pop_front().unwrap_or(Frame::ZERO)
    }
}

impl Sound for StreamSound {
    fn process(&mut self, out: &mut [Frame], dt: f64, _info: &kira::info::Info<'_>) {
        let step = dt * self.rate;
        for frame in out.iter_mut() {
            self.carried += step;
            while self.carried >= 1.0 {
                self.held = self.take();
                self.carried -= 1.0;
            }
            *frame = self.held;
        }
    }

    fn finished(&self) -> bool {
        !self.open.load(Ordering::Relaxed)
    }
}
