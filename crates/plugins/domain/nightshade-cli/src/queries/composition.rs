use nightshade::World;
use nightshade_platform::contract::window::Window;
use nightshade_render::resources::capture::ScheduledCapture;

pub fn window_plugin_composed(world: &World) -> bool {
    world.resources.get::<ScheduledCapture>().is_some() && world.resources.get::<Window>().is_some()
}
