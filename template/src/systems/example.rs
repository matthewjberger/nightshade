use crate::ecs::TemplateResources;
use nightshade::prelude::*;

/// Example system. Each system is a free function that takes
/// `&mut TemplateResources` for app-wide state and `&mut World` for the
/// engine world. Game components live in `world.ecs.worlds[GAME]` on the
/// same entities that carry render components, so set them with
/// `world.ecs.worlds[GAME].set(entity, component)` and read them back
/// with typed queries. Add more files in `src/systems/` and register
/// them in `src/systems.rs` to grow your game.
pub fn tick(template_resources: &mut TemplateResources, world: &mut World) {
    template_resources.example.frame_count =
        template_resources.example.frame_count.saturating_add(1);
    template_resources.example.elapsed_seconds += world.resources.window.timing.delta_time;
}
