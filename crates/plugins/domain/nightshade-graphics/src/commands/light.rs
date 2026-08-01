use nightshade_ecs::Entity;
use nightshade_ecs::system_param::Commands;
use nightshade_scene::contract::primitives::Name;
use nightshade_scene::contract::{GlobalTransform, LocalTransform};

pub fn spawn_light_entity(
    commands: &mut Commands,
    position: nalgebra_glm::Vec3,
    name: &str,
) -> Entity {
    let local_transform = LocalTransform {
        translation: position,
        rotation: nalgebra_glm::quat_identity(),
        scale: nalgebra_glm::Vec3::new(1.0, 1.0, 1.0),
    };
    commands.spawn((
        Name(name.to_string()),
        local_transform,
        GlobalTransform(local_transform.as_matrix()),
        nightshade_render::contract::Light::default(),
    ))
}
