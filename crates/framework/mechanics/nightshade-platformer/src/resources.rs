pub struct PlatformerSettings {
    pub coyote: f32,
    pub buffer: f32,
}

impl Default for PlatformerSettings {
    fn default() -> Self {
        Self {
            coyote: 0.12,
            buffer: 0.12,
        }
    }
}
