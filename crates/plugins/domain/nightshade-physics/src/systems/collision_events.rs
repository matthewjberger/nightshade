pub fn emit_collision_events_system(
    physics: Option<nightshade::ecs::system_param::Res<crate::resources::PhysicsWorld>>,
    mut writer: nightshade::ecs::system_param::EventWriter<
        crate::resources::events::CollisionEvent,
    >,
) {
    let Some(physics) = physics else {
        return;
    };
    let collisions =
        crate::queries::physics_world::physics_world_collision_events(&physics).to_vec();
    for collision in collisions {
        writer.send(collision);
    }
}
