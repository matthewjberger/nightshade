use nightshade_platform::contract::window::{Monitor, Monitors};

pub fn refresh_monitors_from_event_loop(
    monitors: &mut Monitors,
    event_loop: &winit::event_loop::ActiveEventLoop,
) {
    monitors.monitors.clear();
    let primary_handle = event_loop.primary_monitor();
    let primary_handle_ref = primary_handle.as_ref();
    let mut primary_index = None;
    for (index, handle) in event_loop.available_monitors().enumerate() {
        let position = handle.position();
        let size = handle.size();
        let is_primary = primary_handle_ref.is_some_and(|primary| primary == &handle);
        if is_primary {
            primary_index = Some(index);
        }
        monitors.monitors.push(Monitor {
            index,
            name: handle.name(),
            position: (position.x, position.y),
            size: (size.width, size.height),
            scale_factor: handle.scale_factor() as f32,
            refresh_rate_millihertz: handle.refresh_rate_millihertz(),
            is_primary,
        });
    }
    monitors.primary_index = primary_index;
}
