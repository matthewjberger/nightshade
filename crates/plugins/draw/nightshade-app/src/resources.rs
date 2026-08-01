#[derive(Clone, Copy)]
pub struct Opening {
    pub title: &'static str,
    pub capture_frame: u64,
    pub vsync: Option<bool>,
    pub frame_rate_limit: Option<f32>,
    pub size: Option<(u32, u32)>,
}

impl Default for Opening {
    fn default() -> Self {
        Self {
            title: "nightshade",
            capture_frame: nightshade_cli::data::DEFAULT_CAPTURE_FRAME,
            vsync: None,
            frame_rate_limit: None,
            size: None,
        }
    }
}

impl Opening {
    pub fn titled(title: &'static str) -> Self {
        Self {
            title,
            ..Default::default()
        }
    }

    pub fn settling_at(mut self, frame: u64) -> Self {
        self.capture_frame = frame;
        self
    }

    pub fn without_vsync(mut self) -> Self {
        self.vsync = Some(false);
        self
    }

    pub fn limited_to(mut self, frames_a_second: f32) -> Self {
        self.frame_rate_limit = Some(frames_a_second);
        self
    }

    pub fn pacing(mut self, frame_rate_limit: Option<f32>) -> Self {
        self.frame_rate_limit = frame_rate_limit;
        self
    }

    pub fn sized(mut self, size: (u32, u32)) -> Self {
        self.size = Some(size);
        self
    }
}
