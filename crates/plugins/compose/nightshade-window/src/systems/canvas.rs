use nightshade_platform::contract::window::Window;

pub fn sync_wasm_canvas_size(mut window: nightshade::ecs::system_param::ResMut<Window>) {
    use winit::platform::web::WindowExtWebSys;
    if let Some(canvas) = window
        .handle
        .as_ref()
        .and_then(|window_handle| window_handle.canvas())
    {
        window.cached_viewport_size = Some((canvas.width(), canvas.height()));
    }
}
