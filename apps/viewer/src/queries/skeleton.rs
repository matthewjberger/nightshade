use std::collections::HashMap;

use nightshade::prelude::*;
use nightshade_render::components::skin::Skin;
use nightshade_scene::contract::GlobalTransform;

pub fn bone_segments(storage: &nightshade::ecs::Storage) -> Vec<(Vec3, Vec3)> {
    let mut segments = Vec::new();
    let skins: Vec<(Entity, Vec<Entity>, Vec<Mat4>)> = storage
        .query_ref::<(&Skin,)>()
        .iter()
        .map(|(owner, (skin,))| {
            (
                owner,
                skin.joints.clone(),
                skin.inverse_bind_matrices.as_ref().clone(),
            )
        })
        .collect();

    for (owner, joints, inverse_bind) in skins {
        let owner_matrix = storage
            .get::<GlobalTransform>(owner)
            .map(|global| global.0)
            .unwrap_or_else(Mat4::identity);
        let index_of: HashMap<Entity, usize> = joints
            .iter()
            .enumerate()
            .map(|(index, entity)| (*entity, index))
            .collect();

        let places: Vec<Option<Vec3>> = joints
            .iter()
            .enumerate()
            .map(|(index, _)| {
                let bind = inverse_bind.get(index)?.try_inverse()?;
                let point = owner_matrix * bind * nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0);
                Some(nalgebra_glm::vec3(point.x, point.y, point.z))
            })
            .collect();

        for (index, joint) in joints.iter().enumerate() {
            let Some(parent) = storage
                .get::<nightshade::ecs::ChildOf>(*joint)
                .map(|link| link.0)
            else {
                continue;
            };
            let Some(parent_index) = index_of.get(&parent).copied() else {
                continue;
            };
            let (Some(start), Some(end)) = (
                places.get(parent_index).copied().flatten(),
                places.get(index).copied().flatten(),
            ) else {
                continue;
            };
            segments.push((start, end));
        }
    }
    segments
}
