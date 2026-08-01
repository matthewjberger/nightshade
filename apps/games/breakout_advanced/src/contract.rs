#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BreakoutAdvancedEvent {
    BrickChipped { at: [f32; 2], armor: u32 },
    BrickBroken { at: [f32; 2], points: u32 },
    PaddleHit { at: [f32; 2], offset: f32 },
    WallHit { at: [f32; 2] },
    BallLost,
    LevelCleared { level: usize, bonus: u32 },
}
