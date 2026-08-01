use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const POLL_SYSTEM: &str = "nightshade_gamepad::systems::input::gamepad_input_system";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GamepadButton {
    South,
    East,
    West,
    North,
    LeftBumper,
    RightBumper,
    LeftTrigger,
    RightTrigger,
    Select,
    Start,
    Mode,
    LeftThumb,
    RightThumb,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
}

pub const STICK_DEADZONE: f32 = 0.15;

pub fn deadzoned(value: f32) -> f32 {
    if value.abs() < STICK_DEADZONE {
        return 0.0;
    }
    let scaled = (value.abs() - STICK_DEADZONE) / (1.0 - STICK_DEADZONE);
    scaled.min(1.0) * value.signum()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
    LeftTrigger,
    RightTrigger,
}

#[derive(Default)]
pub struct Gamepad {
    pub connected: bool,
    pub axes: Vec<(GamepadAxis, f32)>,
    pub held: HashSet<GamepadButton>,
    pub just_pressed: HashSet<GamepadButton>,
}

impl Gamepad {
    pub fn axis(&self, axis: GamepadAxis) -> f32 {
        self.axes
            .iter()
            .find(|(name, _)| *name == axis)
            .map(|(_, value)| *value)
            .unwrap_or(0.0)
    }

    pub fn pressed(&self, button: GamepadButton) -> bool {
        self.held.contains(&button)
    }

    pub fn just_pressed(&self, button: GamepadButton) -> bool {
        self.just_pressed.contains(&button)
    }

    pub fn clear(&mut self) {
        self.connected = false;
        self.axes.clear();
        self.held.clear();
        self.just_pressed.clear();
    }
}
