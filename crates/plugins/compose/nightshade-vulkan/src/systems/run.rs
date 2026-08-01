use crate::resources::RenderHooks;
use crate::systems::frame::VulkanFrameRenderer;
use nightshade::app::App;

pub fn renderer_factory(
    hooks: RenderHooks,
    vsync: bool,
) -> nightshade_window::run::RendererFactory<VulkanFrameRenderer> {
    Box::new(move |window, width, height| {
        Box::pin(async move {
            let renderer = crate::renderer::create_renderer(window.as_ref(), width, height, vsync)?;
            Ok(VulkanFrameRenderer {
                renderer,
                hooks,
                frames: 0,
                window,
                vsync,
            })
        })
    })
}

pub fn run_windowed(app: App) -> Result<(), Box<dyn std::error::Error>> {
    let mut app = app;
    let kept_alive = app.take_kept_alive();
    let vsync = app
        .world
        .res::<nightshade_render::data::config::FramePacing>()
        .vsync_enabled;
    let (world, host, hooks) = crate::commands::into_parts(app);
    let result = nightshade_window::run::launch_with_world(
        Box::new(host),
        renderer_factory(hooks, vsync),
        world,
    );
    drop(kept_alive);
    result
}
