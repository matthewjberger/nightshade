use crate::data::METER_SPEED_RANGE;
use crate::theme::{RETICLE_HELD, RETICLE_IDLE, RETICLE_READY, SPEED_BANDS};
use nightshade::prelude::Vec4;
use nightshade_platform::contract::input::{Input, InputMode};
use nightshade_platform::contract::window::Window;

pub fn in_control(window: &Window, input: &Input) -> bool {
    window.cursor_locked || input.input_mode == InputMode::Gamepad
}

pub fn speed_color(speed: f32) -> Vec4 {
    SPEED_BANDS
        .iter()
        .find(|(floor, _)| speed >= *floor)
        .map(|(_, color)| *color)
        .unwrap_or(SPEED_BANDS[SPEED_BANDS.len() - 1].1)
}

pub fn reticle_color(attached: bool, aiming: bool) -> Vec4 {
    if attached {
        RETICLE_HELD
    } else if aiming {
        RETICLE_READY
    } else {
        RETICLE_IDLE
    }
}

pub fn meter_fraction(speed: f32) -> f32 {
    (speed / METER_SPEED_RANGE).clamp(0.0, 1.0)
}
