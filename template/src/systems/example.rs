use crate::ecs::TemplateResources;
use nightshade::prelude::*;

/// Example system. Each system is a free function that takes
/// `&mut TemplateResources` for app-wide state and `&mut World` for the
/// engine world. Game components live in the app member world on the
/// same entities that carry render components: set them with
/// `world.set(entity, component)` and read them with
/// `world.get::<Component>(entity)`, both routed to the owning member
/// world. Add more files in `src/systems/` and register them in
/// `src/systems.rs` to grow your game.
pub fn tick(template_resources: &mut TemplateResources, world: &mut World) {
    template_resources.example.frame_count =
        template_resources.example.frame_count.saturating_add(1);
    template_resources.example.elapsed_seconds += world.res::<Time>().delta_time;
}
