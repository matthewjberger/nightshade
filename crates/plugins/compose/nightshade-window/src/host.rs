use nightshade::World;
use nightshade::ecs::{ResourceMap, Storage};
use nightshade_platform::winit::event::WindowEvent;

pub trait Host {
    fn initialize(&mut self, _world: &mut World) {}
    fn run_systems(&mut self, _world: &mut World) {}
    fn window_event(
        &mut self,
        _storage: &mut Storage,
        _resources: &mut ResourceMap,
        _event: &WindowEvent,
    ) {
    }
}

pub type NextHostBuilder =
    Box<dyn FnOnce(&mut Storage, &mut ResourceMap) -> Box<dyn Host> + Send + Sync>;
