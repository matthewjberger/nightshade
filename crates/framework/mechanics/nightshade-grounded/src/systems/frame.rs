use crate::components::Grounded;
use nalgebra_glm::vec2;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::Res;
use nightshade_ground::queries::height_at;
use nightshade_scene::contract::LocalTransform;

pub fn stand_on_the_ground(ground: Res<nightshade_ground::Ground>, storage: &mut Storage) {
    let mut settled = Vec::new();
    for (entity, (transform, grounded)) in
        storage.query_ref::<(&LocalTransform, &Grounded)>().iter()
    {
        let spot = vec2(transform.translation.x, transform.translation.z);
        let Some(height) = height_at(&ground, storage, spot) else {
            continue;
        };
        settled.push((entity, height + grounded.0));
    }
    for (entity, height) in settled {
        if let Some(transform) = storage.get_mut::<LocalTransform>(entity) {
            transform.translation.y = height;
        }
    }
}
