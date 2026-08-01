pub struct ShakeSettings {
    pub decay: f32,
    pub reach: f32,
    pub turn: f32,
    pub ceiling: f32,
}

impl Default for ShakeSettings {
    fn default() -> Self {
        Self {
            decay: 1.6,
            reach: 0.35,
            turn: 0.08,
            ceiling: 1.0,
        }
    }
}
