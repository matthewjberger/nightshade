/// Input smoothing parameters for camera movement.
///
/// Applies exponential smoothing to input devices for fluid camera control.
/// Lower smoothness values = more responsive, higher values = smoother but laggier.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Smoothing {
    /// Mouse look speed multiplier.
    pub mouse_sensitivity: f32,
    /// Mouse input smoothing factor (0 = instant, 1 = no change).
    pub mouse_smoothness: f32,
    /// DPI scaling factor for mouse input.
    pub mouse_dpi_scale: f32,
    /// Keyboard movement smoothing factor.
    pub keyboard_smoothness: f32,
    /// Gamepad stick look speed multiplier.
    pub gamepad_sensitivity: f32,
    /// Gamepad input smoothing factor.
    pub gamepad_smoothness: f32,
    /// Gamepad stick deadzone threshold.
    pub gamepad_deadzone: f32,
    /// Smoothed mouse delta (internal state).
    pub smoothed_mouse_delta: nalgebra_glm::Vec2,
    /// Smoothed movement direction (internal state).
    pub smoothed_movement: nalgebra_glm::Vec3,
    /// Smoothed gamepad input (internal state).
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
