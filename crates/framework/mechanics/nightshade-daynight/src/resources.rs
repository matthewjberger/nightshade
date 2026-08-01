pub struct DaySettings {
    pub hours_a_second: f32,
    pub dawn: f32,
    pub dusk: f32,
}

impl Default for DaySettings {
    fn default() -> Self {
        Self {
            hours_a_second: 0.05,
            dawn: 6.0,
            dusk: 19.0,
        }
    }
}
