bitflags::bitflags! {
    #[derive(Default, Debug, Clone, Copy)]
    pub struct MouseState: u16 {
        const LEFT_CLICKED = 0b0000_0000_0001;
        const MIDDLE_CLICKED = 0b0000_0000_0010;
        const RIGHT_CLICKED = 0b0000_0000_0100;
        const MOVED = 0b0000_0000_1000;
        const SCROLLED = 0b0000_0001_0000;
        const LEFT_JUST_PRESSED = 0b0000_0010_0000;
        const MIDDLE_JUST_PRESSED = 0b0000_0100_0000;
        const RIGHT_JUST_PRESSED = 0b0000_1000_0000;
        const LEFT_JUST_RELEASED = 0b0001_0000_0000;
        const MIDDLE_JUST_RELEASED = 0b0010_0000_0000;
        const RIGHT_JUST_RELEASED = 0b0100_0000_0000;
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Mouse {
    pub state: MouseState,
    pub position: nalgebra_glm::Vec2,
    pub position_delta: nalgebra_glm::Vec2,
    pub raw_mouse_delta: nalgebra_glm::Vec2,
    pub wheel_delta: nalgebra_glm::Vec2,
    pub position_initialized: bool,
}

impl Mouse {
    pub fn end_frame(&mut self) {
        if self.state.contains(MouseState::SCROLLED) {
            self.wheel_delta = nalgebra_glm::Vec2::new(0.0, 0.0);
        }
        self.state.set(MouseState::SCROLLED, false);
        if self.state.contains(MouseState::MOVED) {
            self.position_delta = nalgebra_glm::Vec2::new(0.0, 0.0);
        }
        self.state.set(MouseState::MOVED, false);
        self.raw_mouse_delta = nalgebra_glm::Vec2::new(0.0, 0.0);
        self.state.set(MouseState::LEFT_JUST_PRESSED, false);
        self.state.set(MouseState::MIDDLE_JUST_PRESSED, false);
        self.state.set(MouseState::RIGHT_JUST_PRESSED, false);
        self.state.set(MouseState::LEFT_JUST_RELEASED, false);
        self.state.set(MouseState::MIDDLE_JUST_RELEASED, false);
        self.state.set(MouseState::RIGHT_JUST_RELEASED, false);
    }
}
