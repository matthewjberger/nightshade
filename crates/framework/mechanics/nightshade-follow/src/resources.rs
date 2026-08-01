pub struct FollowSettings {
    pub rate: f32,
    pub snap: Option<f32>,
    pub aim: bool,
}

impl Default for FollowSettings {
    fn default() -> Self {
        Self {
            rate: 6.0,
            snap: None,
            aim: false,
        }
    }
}
