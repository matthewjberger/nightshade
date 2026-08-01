#[derive(Default, Clone, Debug)]
pub struct ImeState {
    pub preedit: Option<(String, Option<(usize, usize)>)>,
    pub committed_this_frame: Vec<String>,
    pub allowed: bool,
    pub applied: bool,
}

#[derive(Default, Clone)]
pub struct Keyboard {
    pub keystates: std::collections::HashMap<winit::keyboard::KeyCode, winit::event::ElementState>,
    pub frame_chars: Vec<char>,
    pub frame_keys: Vec<(winit::keyboard::KeyCode, bool)>,
    pub just_pressed_keys: std::collections::HashSet<winit::keyboard::KeyCode>,
    pub just_released_keys: std::collections::HashSet<winit::keyboard::KeyCode>,
    pub ime: ImeState,
}

impl Keyboard {
    pub fn is_key_pressed(&self, keycode: winit::keyboard::KeyCode) -> bool {
        self.keystates
            .get(&keycode)
            .is_some_and(|state| *state == winit::event::ElementState::Pressed)
    }

    pub fn just_pressed(&self, keycode: winit::keyboard::KeyCode) -> bool {
        self.just_pressed_keys.contains(&keycode)
    }

    pub fn just_released(&self, keycode: winit::keyboard::KeyCode) -> bool {
        self.just_released_keys.contains(&keycode)
    }

    pub fn clear_frame_input(&mut self) {
        self.frame_chars.clear();
        self.frame_keys.clear();
        self.ime.committed_this_frame.clear();
    }

    pub fn end_frame(&mut self) {
        self.just_pressed_keys.clear();
        self.just_released_keys.clear();
        self.frame_chars.clear();
        self.frame_keys.clear();
    }
}
