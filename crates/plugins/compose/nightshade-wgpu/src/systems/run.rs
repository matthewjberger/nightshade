use crate::systems::frame::WgpuFrameRenderer;
use nightshade::app::App;

pub fn renderer_factory(
    hooks: crate::resources::RenderHooks,
) -> nightshade_window::run::RendererFactory<WgpuFrameRenderer> {
    Box::new(move |window, width, height| {
        Box::pin(async move {
            let renderer =
                crate::renderer::wgpu::create::create_renderer(window, width, height).await?;
            Ok(WgpuFrameRenderer::new(renderer, hooks))
        })
    })
}

pub fn run_windowed(app: App) -> Result<(), Box<dyn std::error::Error>> {
    let mut app = app;
    let kept_alive = app.take_kept_alive();
    let (world, host, hooks) = crate::commands::into_parts(app);
    let result =
        nightshade_window::run::launch_with_world(Box::new(host), renderer_factory(hooks), world);
    drop(kept_alive);
    result
}
