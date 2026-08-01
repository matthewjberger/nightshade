pub struct FramePacing {
    pub frame_rate_limit: Option<f32>,
    pub vsync_enabled: bool,
    pub auto_frame_rate_limit_baseline: Option<f32>,
}

impl Default for FramePacing {
    fn default() -> Self {
        Self {
            frame_rate_limit: None,
            vsync_enabled: true,
            auto_frame_rate_limit_baseline: None,
        }
    }
}
