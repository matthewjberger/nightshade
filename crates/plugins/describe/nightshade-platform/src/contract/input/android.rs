#[derive(Debug, Clone)]
pub enum AndroidGamepadEvent {
    Button {
        device: i32,
        keycode: u32,
        pressed: bool,
    },
    Axes {
        device: i32,
        values: Vec<(u32, f32)>,
    },
}

#[derive(Default)]
pub struct AndroidGamepadEvents {
    pub events: Vec<AndroidGamepadEvent>,
}

impl AndroidGamepadEvents {
    pub fn end_frame(&mut self) {
        self.events.clear();
    }
}

#[derive(Default)]
pub struct AndroidKeyboard {
    pub combining_accent: Option<char>,
}
