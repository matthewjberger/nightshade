use crate::ecs::TemplateWorld;
use nightshade::prelude::*;

/// Example system. Each system is a free function that takes
/// `&mut TemplateWorld` for app-specific state and `&mut World` for the
/// engine's renderer-visible world. Add more files in `src/systems/` and
/// register them in `src/systems.rs` to grow your game.
pub fn tick(template_world: &mut TemplateWorld, _world: &mut World) {
    template_world.resources.example.frame_count = template_world
        .resources
        .example
        .frame_count
        .saturating_add(1);
}
