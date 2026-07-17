use crate::components::Spin;
use crate::resources::TemplateResources;
use nightshade::prelude::*;

/// Example per-frame system: advances the demo counters, spins every entity
/// carrying a [`Spin`] about the Y axis, then exits on Q. The resources it
/// reads and writes come in as `Res`/`ResMut` params; the world is for entity
/// and component access, here `query_ref` to find the spinners and `get_mut`
/// to turn them. Add more files in `src/systems/` and register them in
/// `src/systems.rs` to grow your game.
pub fn tick(
    mut template_resources: ResMut<TemplateResources>,
    time: Res<Time>,
    input: Res<Input>,
    mut window: ResMut<Window>,
    world: &mut World,
) {
    let example = &mut template_resources.example;
    example.frame_count = example.frame_count.saturating_add(1);
    example.elapsed_seconds += time.delta_time;

    let spinners: Vec<(Entity, f32)> = world
        .query_ref::<&Spin>()
        .iter()
        .map(|(entity, spin)| (entity, spin.speed))
        .collect();
    for (entity, speed) in spinners {
        if let Some(transform) = world.get_mut::<LocalTransform>(entity) {
            let rotation = nalgebra_glm::quat_angle_axis(speed * time.delta_time, &Vec3::y());
            transform.rotation = rotation * transform.rotation;
        }
    }

    if input.keyboard.just_pressed(KeyCode::KeyQ) {
        window.should_exit = true;
    }
}
