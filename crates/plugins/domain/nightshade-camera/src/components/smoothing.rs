#[derive(
    Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, enum2schema::Schema,
)]
pub struct Smoothing {
    pub mouse_sensitivity: f32,
    pub mouse_smoothness: f32,
    pub mouse_dpi_scale: f32,
    pub keyboard_smoothness: f32,
    pub gamepad_sensitivity: f32,
    pub gamepad_smoothness: f32,
    #[serde(default = "default_move_speed")]
    pub move_speed: f32,
    #[serde(default = "default_sprint_multiplier")]
    pub sprint_multiplier: f32,
    #[schema(type = "array", items = "number", len = 2)]
    pub smoothed_mouse_delta: nalgebra_glm::Vec2,
    #[schema(type = "array", items = "number", len = 3)]
    pub smoothed_movement: nalgebra_glm::Vec3,
    #[schema(type = "array", items = "number", len = 2)]
    pub smoothed_gamepad_input: nalgebra_glm::Vec2,
}

fn default_move_speed() -> f32 {
    20.0
}

fn default_sprint_multiplier() -> f32 {
    3.0
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
            move_speed: default_move_speed(),
            sprint_multiplier: default_sprint_multiplier(),
            smoothed_mouse_delta: nalgebra_glm::vec2(0.0, 0.0),
            smoothed_movement: nalgebra_glm::vec3(0.0, 0.0, 0.0),
            smoothed_gamepad_input: nalgebra_glm::vec2(0.0, 0.0),
        }
    }
}
