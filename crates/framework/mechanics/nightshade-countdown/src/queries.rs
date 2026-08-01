use crate::resources::Remaining;

pub fn seconds(remaining: &Remaining) -> f32 {
    remaining.0
}

pub fn gone(remaining: &Remaining) -> bool {
    remaining.0 <= 0.0
}
