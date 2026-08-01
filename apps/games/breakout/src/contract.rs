#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BreakoutEvent {
    BrickBroken { points: u32 },
    BallLost,
}
