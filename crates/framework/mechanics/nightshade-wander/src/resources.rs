pub struct WanderSettings {
    pub reach: f32,
    pub speed: f32,
    pub arrival: f32,
    pub rest_low: f32,
    pub rest_high: f32,
}

impl Default for WanderSettings {
    fn default() -> Self {
        Self {
            reach: 6.0,
            speed: 1.4,
            arrival: 0.2,
            rest_low: 0.5,
            rest_high: 3.0,
        }
    }
}
