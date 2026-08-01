#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TetherEvent {
    Released { speed: f32 },
    Fell { speed: f32 },
}
