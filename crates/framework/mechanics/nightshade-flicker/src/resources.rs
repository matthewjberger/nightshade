pub struct FlickerSettings {
    pub depth: f32,
    pub dropout_chance: f32,
    pub dropout_scale: f32,
    pub response: f32,
    pub dark_below: f32,
}

impl Default for FlickerSettings {
    fn default() -> Self {
        Self {
            depth: 0.3,
            dropout_chance: 0.03,
            dropout_scale: 0.15,
            response: 18.0,
            dark_below: 0.4,
        }
    }
}
