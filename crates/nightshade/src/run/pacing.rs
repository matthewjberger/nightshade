use crate::ecs::world::World;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn throttle_to_frame_rate_limit(world: &mut World) {
    use web_time::Instant;

    let Some(target_fps) = world
        .expect_resource_mut::<crate::render::config::RenderSettings>()
        .frame_rate_limit
    else {
        return;
    };
    if !target_fps.is_finite() || target_fps <= 0.0 {
        return;
    }
    let Some(last_frame_start) = world
        .expect_resource_mut::<crate::ecs::time::Time>()
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
pub(crate) fn throttle_to_frame_rate_limit(_world: &mut World) {}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn sync_macos_auto_frame_rate_limit(world: &mut World) {
    if !cfg!(target_os = "macos") {
        return;
    }
    let baseline = world
        .resources
        .renderer_state
        .auto_frame_rate_limit_baseline;
    if world
        .expect_resource_mut::<crate::render::config::RenderSettings>()
        .vsync_enabled
    {
        if baseline.is_some() {
            if world
                .expect_resource_mut::<crate::render::config::RenderSettings>()
                .frame_rate_limit
                == baseline
            {
                world
                    .expect_resource_mut::<crate::render::config::RenderSettings>()
                    .frame_rate_limit = None;
            }
            world
                .resources
                .renderer_state
                .auto_frame_rate_limit_baseline = None;
        }
        return;
    }
    if baseline.is_some()
        || world
            .expect_resource_mut::<crate::render::config::RenderSettings>()
            .frame_rate_limit
            .is_some()
    {
        return;
    }
    let Some(window_handle) = world
        .expect_resource_mut::<crate::ecs::window::resources::Window>()
        .handle
        .clone()
    else {
        return;
    };
    let Some(target) = current_monitor_frame_rate_target(&window_handle) else {
        return;
    };
    world
        .expect_resource_mut::<crate::render::config::RenderSettings>()
        .frame_rate_limit = Some(target);
    world
        .resources
        .renderer_state
        .auto_frame_rate_limit_baseline = Some(target);
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn sync_macos_auto_frame_rate_limit(_world: &mut World) {}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn refresh_macos_frame_rate_limit(
    world: &mut World,
    window_handle: &std::sync::Arc<winit::window::Window>,
) {
    if !cfg!(target_os = "macos") {
        return;
    }
    if world
        .expect_resource_mut::<crate::render::config::RenderSettings>()
        .vsync_enabled
    {
        return;
    }
    let baseline = world
        .resources
        .renderer_state
        .auto_frame_rate_limit_baseline;
    if baseline.is_none() {
        return;
    }
    if world
        .expect_resource_mut::<crate::render::config::RenderSettings>()
        .frame_rate_limit
        != baseline
    {
        return;
    }
    let Some(target) = current_monitor_frame_rate_target(window_handle) else {
        return;
    };
    world
        .expect_resource_mut::<crate::render::config::RenderSettings>()
        .frame_rate_limit = Some(target);
    world
        .resources
        .renderer_state
        .auto_frame_rate_limit_baseline = Some(target);
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn refresh_macos_frame_rate_limit(
    _world: &mut World,
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
pub(crate) fn wasm_capped_dpr(css_width: f64, css_height: f64, raw_dpr: f64) -> f64 {
    let smallest_css = css_width.min(css_height);
    let max_dpr = if smallest_css < 700.0 { 1.5 } else { 2.0 };
    raw_dpr.clamp(1.0, max_dpr)
}
