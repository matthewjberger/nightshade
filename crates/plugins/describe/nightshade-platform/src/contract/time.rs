#[derive(Clone)]
pub struct Time {
    pub frames_per_second: f32,
    pub delta_time: f32,
    pub raw_delta_time: f32,
    pub time_speed: f32,
    pub paused: bool,
    pub last_frame_start_instant: Option<web_time::Instant>,
    pub current_frame_start_instant: Option<web_time::Instant>,
    pub frame_counter: u32,
    pub uptime_milliseconds: u64,
    pub uptime_seconds: f64,
    pub host_delta_time: Option<f32>,
}

impl Default for Time {
    fn default() -> Self {
        Self {
            frames_per_second: 0.0,
            delta_time: 0.0,
            raw_delta_time: 0.0,
            time_speed: 1.0,
            paused: false,
            last_frame_start_instant: None,
            current_frame_start_instant: None,
            frame_counter: 0,
            uptime_milliseconds: 0,
            uptime_seconds: 0.0,
            host_delta_time: None,
        }
    }
}
