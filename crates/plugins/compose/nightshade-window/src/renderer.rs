use nightshade::World;
use nightshade::ecs::{ResourceMap, Storage};

pub struct RenderError(pub String);

impl std::fmt::Display for RenderError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

pub trait Renderer {
    fn configure(&mut self) -> Result<(), RenderError>;
    fn resize(&mut self, width: u32, height: u32) -> Result<(), RenderError>;
    fn release_surface(&mut self);
    fn recreate_surface(
        &mut self,
        window: std::sync::Arc<winit::window::Window>,
        width: u32,
        height: u32,
    ) -> Result<(), RenderError>;
    fn update(&mut self, storage: &Storage, resources: &mut ResourceMap)
    -> Result<(), RenderError>;
    fn prepare(&mut self, storage: &mut Storage, resources: &mut ResourceMap);
    fn render(&mut self, world: &mut World) -> Result<(), RenderError>;
}
