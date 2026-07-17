use crate::ecs::TemplateResources;
use nightshade::prelude::*;

/// Example per-frame system: advances the demo counters, then exits on Q.
/// Game components live on engine entities in the app member world; set them
/// with `world.set(entity, component)` and read them with
/// `world.get::<Component>(entity)`, both routed to the owning member world.
/// Add more files in `src/systems/` and register them in `src/systems.rs` to
/// grow your game.
pub fn tick(template_resources: &mut TemplateResources, world: &mut World) {
    template_resources.example.frame_count =
        template_resources.example.frame_count.saturating_add(1);
    template_resources.example.elapsed_seconds += world.res::<Time>().delta_time;

    let events = std::mem::take(&mut world.res_mut::<Input>().events);
    for event in events {
        if let AppEvent::Keyboard { key, state } = event
            && matches!((key, state), (KeyCode::KeyQ, KeyState::Pressed))
        {
            world.res_mut::<Window>().should_exit = true;
        }
    }
}
