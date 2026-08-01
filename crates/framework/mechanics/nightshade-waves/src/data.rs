#[derive(Clone, Copy, Default)]
pub struct Wave {
    pub count: u32,
    pub spacing: f32,
    pub rest: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Arrival {
    pub wave: usize,
    pub index: u32,
    pub last: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct WaveCleared {
    pub wave: usize,
}
