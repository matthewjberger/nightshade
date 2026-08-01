use crate::data::Wave;

#[derive(Default)]
pub struct WavePlan(pub Vec<Wave>);

#[derive(Default)]
pub struct WaveProgress {
    pub wave: usize,
    pub sent: u32,
    pub alive: u32,
    pub wait: f32,
    pub rest: f32,
    pub running: bool,
    pub resting: bool,
    pub pace: f32,
}
