#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GpuDeviceType {
    #[default]
    Other,
    IntegratedGpu,
    DiscreteGpu,
    VirtualGpu,
    Cpu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GpuBackend {
    #[default]
    Other,
    Vulkan,
    Metal,
    Dx12,
    Gl,
    WebGpu,
    WebGl,
}

#[derive(Debug, Clone, Default)]
pub struct GpuProfile {
    pub backend: GpuBackend,
    pub device_type: GpuDeviceType,
    pub name: String,
}

impl GpuProfile {
    pub fn is_webgl(&self) -> bool {
        self.backend == GpuBackend::WebGl
    }

    pub fn is_low_power(&self) -> bool {
        matches!(
            self.device_type,
            GpuDeviceType::IntegratedGpu | GpuDeviceType::VirtualGpu | GpuDeviceType::Cpu
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PerformanceTarget {
    #[default]
    Unbounded,
    Interactive,
    Balanced,
    Quality,
}

impl PerformanceTarget {
    pub fn target_frame_time_ms(self) -> Option<f32> {
        match self {
            PerformanceTarget::Unbounded => None,
            PerformanceTarget::Interactive => Some(1000.0 / 60.0),
            PerformanceTarget::Balanced => Some(1000.0 / 30.0),
            PerformanceTarget::Quality => Some(1000.0 / 15.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AdaptiveSamplingState {
    pub target: PerformanceTarget,
    pub frame_time_rolling_ms: f32,
    pub frame_time_sample_count: u32,
    pub ssao_sample_scale: f32,
    pub ssgi_sample_scale: f32,
    pub ssr_step_scale: f32,
}

impl Default for AdaptiveSamplingState {
    fn default() -> Self {
        Self {
            target: PerformanceTarget::Unbounded,
            frame_time_rolling_ms: 16.67,
            frame_time_sample_count: 0,
            ssao_sample_scale: 1.0,
            ssgi_sample_scale: 1.0,
            ssr_step_scale: 1.0,
        }
    }
}

impl AdaptiveSamplingState {
    pub fn record_frame_time(&mut self, frame_time_ms: f32) {
        const WINDOW: u32 = 100;
        if self.frame_time_sample_count == 0 {
            self.frame_time_rolling_ms = frame_time_ms;
            self.frame_time_sample_count = 1;
        } else {
            let alpha = 1.0 / (self.frame_time_sample_count.min(WINDOW) as f32 + 1.0);
            self.frame_time_rolling_ms =
                self.frame_time_rolling_ms * (1.0 - alpha) + frame_time_ms * alpha;
            self.frame_time_sample_count = (self.frame_time_sample_count + 1).min(WINDOW);
        }

        let Some(target_ms) = self.target.target_frame_time_ms() else {
            self.ssao_sample_scale = 1.0;
            self.ssgi_sample_scale = 1.0;
            self.ssr_step_scale = 1.0;
            return;
        };

        let error = self.frame_time_rolling_ms - target_ms;
        let sensitivity = 0.002;
        let max_delta = 0.05;
        let adjustment = (error * sensitivity).clamp(-max_delta, max_delta);

        self.ssao_sample_scale = (self.ssao_sample_scale - adjustment).clamp(0.25, 1.0);
        self.ssgi_sample_scale = (self.ssgi_sample_scale - adjustment).clamp(0.25, 1.0);
        self.ssr_step_scale = (self.ssr_step_scale - adjustment).clamp(0.25, 1.0);
    }
}
