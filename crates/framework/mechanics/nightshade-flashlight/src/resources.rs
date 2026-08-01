pub struct FlashlightSettings {
    pub full: f32,
    pub low: f32,
    pub brownout: f32,
}

impl Default for FlashlightSettings {
    fn default() -> Self {
        Self {
            full: 100.0,
            low: 22.0,
            brownout: 0.45,
        }
    }
}
