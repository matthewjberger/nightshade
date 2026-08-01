use crate::resources::StageState;
use nightshade::World;
use nightshade::ecs::{ResourceMap, Storage};

impl nightshade_window::Host for StageState {
    fn initialize(&mut self, world: &mut World) {
        self.startup.run(world);
    }

    fn run_systems(&mut self, world: &mut World) {
        self.stages.run(world);
    }

    fn window_event(
        &mut self,
        storage: &mut Storage,
        resources: &mut ResourceMap,
        event: &nightshade_platform::winit::event::WindowEvent,
    ) {
        for hook in &mut self.window_event {
            hook(storage, resources, event);
        }
    }
}
