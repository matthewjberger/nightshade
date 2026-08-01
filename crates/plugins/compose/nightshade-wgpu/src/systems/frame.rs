use crate::renderer::wgpu::WgpuRenderer;
use crate::resources::RenderHooks;
use nightshade::World;
use nightshade::ecs::{ResourceMap, Storage};
use nightshade_window::renderer::{RenderError, Renderer};

pub struct WgpuFrameRenderer {
    pub renderer: WgpuRenderer,
    pub hooks: RenderHooks,
}

impl WgpuFrameRenderer {
    pub fn new(renderer: WgpuRenderer, hooks: RenderHooks) -> Self {
        Self { renderer, hooks }
    }
}

impl Renderer for WgpuFrameRenderer {
    fn configure(&mut self) -> Result<(), RenderError> {
        crate::systems::render_driver::configure_graph(&mut self.renderer, &mut self.hooks)
            .map_err(|error| RenderError(error.to_string()))
    }

    fn resize(&mut self, width: u32, height: u32) -> Result<(), RenderError> {
        self.renderer
            .resize_surface(width, height)
            .map_err(|error| RenderError(error.to_string()))
    }

    fn release_surface(&mut self) {
        self.renderer.release_surface();
    }

    fn recreate_surface(
        &mut self,
        window: std::sync::Arc<nightshade_platform::winit::window::Window>,
        width: u32,
        height: u32,
    ) -> Result<(), RenderError> {
        self.renderer
            .recreate_surface(window, width, height)
            .map_err(|error| RenderError(error.to_string()))
    }

    fn update(
        &mut self,
        storage: &Storage,
        resources: &mut ResourceMap,
    ) -> Result<(), RenderError> {
        crate::systems::render_driver::update_graph(
            &mut self.renderer,
            &mut self.hooks,
            storage,
            resources,
        )
        .map_err(|error| RenderError(error.to_string()))
    }

    fn prepare(&mut self, storage: &mut Storage, resources: &mut ResourceMap) {
        self.hooks
            .pre_render(&mut self.renderer, storage, resources);
    }

    fn render(&mut self, world: &mut World) -> Result<(), RenderError> {
        crate::systems::render_driver::render_frame(&mut self.renderer, &mut self.hooks, world)
            .map_err(|error| RenderError(error.to_string()))
    }
}
