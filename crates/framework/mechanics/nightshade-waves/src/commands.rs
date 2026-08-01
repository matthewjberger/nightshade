use crate::resources::WaveProgress;

pub fn start(progress: &mut WaveProgress) {
    progress.running = true;
}

pub fn stop(progress: &mut WaveProgress) {
    progress.running = false;
}

pub fn rewind(progress: &mut WaveProgress) {
    progress.wave = 0;
    progress.sent = 0;
    progress.alive = 0;
    progress.wait = 0.0;
    progress.rest = 0.0;
    progress.running = false;
    progress.resting = false;
}

pub fn one_fell(progress: &mut WaveProgress) {
    progress.alive = progress.alive.saturating_sub(1);
}

pub fn hurry(progress: &mut WaveProgress) -> f32 {
    if !progress.resting {
        return 0.0;
    }
    let saved = progress.rest.max(0.0);
    progress.rest = 0.0;
    saved
}
