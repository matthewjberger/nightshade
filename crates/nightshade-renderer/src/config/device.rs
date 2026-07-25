//! The GPU the renderer got and how hard to push it: adapter profile,
//! backend, performance target, and the adaptive sampling that tracks it.

/// The GPU class the renderer ended up on, mirrored from `wgpu::DeviceType`
/// so apps can pick quality presets without depending on wgpu directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GpuDeviceType {
    /// Unknown or unclassified device.
    #[default]
    Other,
    /// Integrated GPU sharing system memory.
    IntegratedGpu,
    /// Discrete GPU with dedicated memory.
    DiscreteGpu,
    /// Virtualized or paravirtual GPU.
    VirtualGpu,
    /// Software (CPU) rasterizer.
    Cpu,
}

/// The graphics backend the renderer is running on. `WebGl` is the constrained
/// browser fallback and is the strongest signal that a mobile-class budget is
/// appropriate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GpuBackend {
    /// Unknown or unclassified backend.
    #[default]
    Other,
    /// Vulkan.
    Vulkan,
    /// Apple Metal.
    Metal,
    /// Direct3D 12.
    Dx12,
    /// Desktop OpenGL.
    Gl,
    /// Native WebGPU.
    WebGpu,
    /// WebGL browser fallback.
    WebGl,
}

/// A snapshot of the adapter the renderer selected, published into
/// `RendererState` so game code can adapt quality and controls to the device.
#[derive(Debug, Clone, Default)]
pub struct GpuProfile {
    /// Graphics backend the renderer is running on.
    pub backend: GpuBackend,
    /// Class of the selected GPU.
    pub device_type: GpuDeviceType,
    /// Adapter name reported by the driver.
    pub name: String,
}

impl GpuProfile {
    /// True when the renderer fell back to the browser WebGL path, the clearest
    /// indication of a constrained mobile-class GPU budget.
    pub fn is_webgl(&self) -> bool {
        self.backend == GpuBackend::WebGl
    }

    /// True when the adapter is an integrated, virtual, or software device, the
    /// classes that benefit from a reduced render budget.
    pub fn is_low_power(&self) -> bool {
        matches!(
            self.device_type,
            GpuDeviceType::IntegratedGpu | GpuDeviceType::VirtualGpu | GpuDeviceType::Cpu
        )
    }
}

/// Frame-time budget the adaptive sampler drives optional effect quality toward.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PerformanceTarget {
    /// No budget, effects run at full quality.
    #[default]
    Unbounded,
    /// Target 60 frames per second.
    Interactive,
    /// Target 30 frames per second.
    Balanced,
    /// Target 15 frames per second, favoring quality.
    Quality,
}

impl PerformanceTarget {
    /// Target frame time in milliseconds, `None` when unbounded.
    pub fn target_frame_time_ms(self) -> Option<f32> {
        match self {
            PerformanceTarget::Unbounded => None,
            PerformanceTarget::Interactive => Some(1000.0 / 60.0),
            PerformanceTarget::Balanced => Some(1000.0 / 30.0),
            PerformanceTarget::Quality => Some(1000.0 / 15.0),
        }
    }
}

/// Frame-time driven scaling of optional post-process sample counts.
#[derive(Debug, Clone)]
pub struct AdaptiveSamplingState {
    /// Frame-time budget the scaler drives toward.
    pub target: PerformanceTarget,
    /// Exponentially smoothed frame time in milliseconds.
    pub frame_time_rolling_ms: f32,
    /// Frames accumulated into the rolling average.
    pub frame_time_sample_count: u32,
    /// Current `SSAO` sample-count scale (0.25 to 1.0).
    pub ssao_sample_scale: f32,
    /// Current `SSGI` sample-count scale (0.25 to 1.0).
    pub ssgi_sample_scale: f32,
    /// Current `SSR` step-count scale (0.25 to 1.0).
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
    /// Folds a frame time into the rolling average and nudges the sample
    /// scales toward the target budget.
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
