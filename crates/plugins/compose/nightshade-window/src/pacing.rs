#[cfg(not(target_arch = "wasm32"))]
pub fn throttle_to_frame_rate_limit(resources: &mut nightshade::ecs::ResourceMap) {
    use web_time::Instant;

    let Some(target_fps) = resources
        .res::<nightshade_render::data::config::FramePacing>()
        .frame_rate_limit
    else {
        return;
    };
    if !target_fps.is_finite() || target_fps <= 0.0 {
        return;
    }
    let Some(last_frame_start) = resources
        .res::<nightshade_platform::contract::time::Time>()
        .last_frame_start_instant
    else {
        return;
    };

    let target_interval = std::time::Duration::from_secs_f32(1.0 / target_fps);
    let deadline = last_frame_start + target_interval;
    let busy_wait_floor = std::time::Duration::from_millis(1);

    let now = Instant::now();
    if now >= deadline {
        return;
    }
    let remaining = deadline - now;
    if remaining > busy_wait_floor {
        std::thread::sleep(remaining - busy_wait_floor);
    }
    while Instant::now() < deadline {
        std::hint::spin_loop();
    }
}

#[cfg(target_arch = "wasm32")]
pub fn throttle_to_frame_rate_limit(_resources: &mut nightshade::ecs::ResourceMap) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn sync_macos_auto_frame_rate_limit(resources: &mut nightshade::ecs::ResourceMap) {
    if !cfg!(target_os = "macos") {
        return;
    }
    let baseline = resources
        .res::<nightshade_render::data::config::FramePacing>()
        .auto_frame_rate_limit_baseline;
    if resources
        .res::<nightshade_render::data::config::FramePacing>()
        .vsync_enabled
    {
        if baseline.is_some() {
            if resources
                .res::<nightshade_render::data::config::FramePacing>()
                .frame_rate_limit
                == baseline
            {
                resources
                    .res_mut::<nightshade_render::data::config::FramePacing>()
                    .frame_rate_limit = None;
            }
            resources
                .res_mut::<nightshade_render::data::config::FramePacing>()
                .auto_frame_rate_limit_baseline = None;
        }
        return;
    }
    if baseline.is_some()
        || resources
            .res::<nightshade_render::data::config::FramePacing>()
            .frame_rate_limit
            .is_some()
    {
        return;
    }
    let Some(window_handle) = resources
        .res::<nightshade_platform::contract::window::Window>()
        .handle
        .clone()
    else {
        return;
    };
    let Some(target) = current_monitor_frame_rate_target(&window_handle) else {
        return;
    };
    resources
        .res_mut::<nightshade_render::data::config::FramePacing>()
        .frame_rate_limit = Some(target);
    resources
        .res_mut::<nightshade_render::data::config::FramePacing>()
        .auto_frame_rate_limit_baseline = Some(target);
}

#[cfg(target_arch = "wasm32")]
pub fn sync_macos_auto_frame_rate_limit(_resources: &mut nightshade::ecs::ResourceMap) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn refresh_macos_frame_rate_limit(
    resources: &mut nightshade::ecs::ResourceMap,
    window_handle: &std::sync::Arc<winit::window::Window>,
) {
    if !cfg!(target_os = "macos") {
        return;
    }
    if resources
        .res::<nightshade_render::data::config::FramePacing>()
        .vsync_enabled
    {
        return;
    }
    let baseline = resources
        .res::<nightshade_render::data::config::FramePacing>()
        .auto_frame_rate_limit_baseline;
    if baseline.is_none() {
        return;
    }
    if resources
        .res::<nightshade_render::data::config::FramePacing>()
        .frame_rate_limit
        != baseline
    {
        return;
    }
    let Some(target) = current_monitor_frame_rate_target(window_handle) else {
        return;
    };
    resources
        .res_mut::<nightshade_render::data::config::FramePacing>()
        .frame_rate_limit = Some(target);
    resources
        .res_mut::<nightshade_render::data::config::FramePacing>()
        .auto_frame_rate_limit_baseline = Some(target);
}

#[cfg(target_arch = "wasm32")]
pub fn refresh_macos_frame_rate_limit(
    _resources: &mut nightshade::ecs::ResourceMap,
    _window_handle: &std::sync::Arc<winit::window::Window>,
) {
}

#[cfg(not(target_arch = "wasm32"))]
fn current_monitor_frame_rate_target(
    window_handle: &std::sync::Arc<winit::window::Window>,
) -> Option<f32> {
    let monitor = window_handle.current_monitor()?;
    let millihertz = monitor.refresh_rate_millihertz()?;
    let refresh_hz = millihertz as f32 / 1000.0;
    if refresh_hz > 1.0 {
        Some(refresh_hz - 1.0)
    } else {
        None
    }
}

#[cfg(target_arch = "wasm32")]
pub fn wasm_capped_dpr(css_width: f64, css_height: f64, raw_dpr: f64) -> f64 {
    let smallest_css = css_width.min(css_height);
    let max_dpr = if smallest_css < 700.0 { 1.5 } else { 2.0 };
    raw_dpr.clamp(1.0, max_dpr)
}

#[cfg(target_arch = "wasm32")]
pub struct WasmViewport {
    pub css_width: f64,
    pub css_height: f64,
    pub raw_dpr: f64,
    pub capped_dpr: f64,
}

#[cfg(target_arch = "wasm32")]
impl WasmViewport {
    pub fn physical_size(&self) -> (u32, u32) {
        (
            (self.css_width * self.capped_dpr).round() as u32,
            (self.css_height * self.capped_dpr).round() as u32,
        )
    }

    pub fn pointer_scale(&self) -> f32 {
        (self.capped_dpr / self.raw_dpr) as f32
    }
}

#[cfg(target_arch = "wasm32")]
pub fn wasm_viewport(fallback_width: f64, fallback_height: f64) -> WasmViewport {
    let web_window = web_sys::window().unwrap();
    let raw_dpr = web_window.device_pixel_ratio().max(1.0);
    let css_width = web_window
        .inner_width()
        .ok()
        .and_then(|value| value.as_f64())
        .unwrap_or(fallback_width);
    let css_height = web_window
        .inner_height()
        .ok()
        .and_then(|value| value.as_f64())
        .unwrap_or(fallback_height);
    WasmViewport {
        css_width,
        css_height,
        raw_dpr,
        capped_dpr: wasm_capped_dpr(css_width, css_height, raw_dpr),
    }
}
