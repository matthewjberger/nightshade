#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Smoothing {
    pub mouse_sensitivity: f32,
    pub mouse_smoothness: f32,
    pub mouse_dpi_scale: f32,
    pub keyboard_smoothness: f32,
    pub gamepad_sensitivity: f32,
    pub gamepad_smoothness: f32,
    pub gamepad_deadzone: f32,
    pub smoothed_mouse_delta: nalgebra_glm::Vec2,
    pub smoothed_movement: nalgebra_glm::Vec3,
    pub smoothed_gamepad_input: nalgebra_glm::Vec2,
}

impl Default for Smoothing {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 0.5,
            mouse_smoothness: 0.05,
            mouse_dpi_scale: 1.0,
            keyboard_smoothness: 0.08,
            gamepad_sensitivity: 1.5,
            gamepad_smoothness: 0.06,
            gamepad_deadzone: 0.15,
            smoothed_mouse_delta: nalgebra_glm::vec2(0.0, 0.0),
            smoothed_movement: nalgebra_glm::vec3(0.0, 0.0, 0.0),
            smoothed_gamepad_input: nalgebra_glm::vec2(0.0, 0.0),
        }
    }
}
