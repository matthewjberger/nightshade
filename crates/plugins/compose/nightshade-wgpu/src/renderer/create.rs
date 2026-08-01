pub async fn create_renderer<W>(
    window_handle: W,
    initial_width: u32,
    initial_height: u32,
) -> std::result::Result<crate::renderer::wgpu::WgpuRenderer, Box<dyn std::error::Error>>
where
    W: raw_window_handle::HasDisplayHandle
        + raw_window_handle::HasWindowHandle
        + Send
        + Sync
        + 'static,
{
    crate::renderer::wgpu::create_wgpu_renderer(window_handle, initial_width, initial_height).await
}
