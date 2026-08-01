pub const HEAD_CENTER: f32 = 0.9;

pub const MARK_HEIGHT: f32 = 2.6;

pub const MARK_YELLOW: [f32; 4] = [1.0, 0.86, 0.12, 1.0];

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum MarkState {
    #[default]
    Unset,
    Hidden,
    Offer,
    Ready,
    Pending,
}
