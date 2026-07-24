//! Physics-driven ragdoll: proxy capsule bodies linked by spherical
//! joints drive a skinned skeleton while it is ragdolled.

use crate::ecs::animation::ragdoll::RagdollLink;
use crate::ecs::animation::systems::resolve_bone_world_transform;
use crate::ecs::world::{Entity, World};
use crate::plugins::physics::components::{ColliderComponent, RigidBodyComponent};
use crate::plugins::physics::joints::{SphericalJoint, create_spherical_joint};
use nalgebra_glm::{Quat, Vec3};

fn rotation_between(from: &Vec3, to: &Vec3) -> Quat {
    let from = from.normalize();
    let to = to.normalize();
    let dot = from.dot(&to).clamp(-1.0, 1.0);
    if dot > 0.999_999 {
        return Quat::identity();
    }
    if dot < -0.999_999 {
        let mut axis = Vec3::x().cross(&from);
        if axis.magnitude() < 1.0e-4 {
            axis = Vec3::y().cross(&from);
        }
        return nalgebra_glm::quat_angle_axis(std::f32::consts::PI, &axis.normalize());
    }
    let axis = from.cross(&to).normalize();
    nalgebra_glm::quat_angle_axis(dot.acos(), &axis)
}

fn bone_world(world: &World, player: Entity, bone: &str) -> Option<(Vec3, Quat)> {
    let matrix = resolve_bone_world_transform(world, player, bone)?;
    let position = Vec3::new(matrix[(0, 3)], matrix[(1, 3)], matrix[(2, 3)]);
    let basis = nalgebra_glm::mat4_to_mat3(&matrix);
    let x = basis.column(0).normalize();
    let y = basis.column(1).normalize();
    let z = basis.column(2).normalize();
    let orthonormal = nalgebra_glm::Mat3::from_columns(&[x, y, z]);
    Some((
        position,
        nalgebra_glm::mat3_to_quat(&orthonormal).normalize(),
    ))
}

/// Builds the proxy bodies and joints from the current animated pose and
/// pauses the player so its bones follow the physics sync.
pub fn ragdoll_build(world: &mut World, player_entity: Entity) {
    let ragdoll =
        match world.get::<crate::ecs::animation::components::AnimationPlayer>(player_entity) {
            Some(player) => match &player.ragdoll {
                Some(ragdoll) if !ragdoll.active && !ragdoll.bones.is_empty() => ragdoll.clone(),
                _ => return,
            },
            None => return,
        };

    let bone_map: std::collections::HashMap<String, Entity> = {
        let Some(player) =
            world.get::<crate::ecs::animation::components::AnimationPlayer>(player_entity)
        else {
            return;
        };
        ragdoll
            .bones
            .iter()
            .filter_map(|bone| {
                player
                    .bone_name_to_entity
                    .get(bone)
                    .copied()
                    .map(|entity| (bone.clone(), entity))
            })
            .collect()
    };

    let mut links: Vec<RagdollLink> = Vec::new();
    let mut body_by_bone: std::collections::HashMap<usize, Entity> =
        std::collections::HashMap::new();
    for index in 1..ragdoll.bones.len() {
        let Some((parent_position, _)) =
            bone_world(world, player_entity, &ragdoll.bones[index - 1])
        else {
            continue;
        };
        let Some((position, rotation)) = bone_world(world, player_entity, &ragdoll.bones[index])
        else {
            continue;
        };
        let segment = position - parent_position;
        let length = segment.magnitude();
        if length < 0.02 {
            continue;
        }
        let Some(&bone_entity) = bone_map.get(&ragdoll.bones[index]) else {
            continue;
        };
        let direction = segment / length;
        let center = parent_position + segment * 0.5;
        let body_rotation = rotation_between(&Vec3::y(), &direction);
        let half_height = ((length * 0.7) * 0.5 - ragdoll.radius).max(0.01);
        let body = world.spawn_with((
            crate::ecs::transform::components::LocalTransform {
                translation: center,
                rotation: body_rotation,
                ..Default::default()
            },
            crate::ecs::transform::components::GlobalTransform::default(),
            RigidBodyComponent::new_dynamic()
                .with_translation(center.x, center.y, center.z)
                .with_rotation(
                    body_rotation.i,
                    body_rotation.j,
                    body_rotation.k,
                    body_rotation.w,
                ),
            ColliderComponent::new_capsule(half_height, ragdoll.radius)
                .with_friction(0.8)
                .with_restitution(0.05),
        ));
        body_by_bone.insert(index, body);
        links.push(RagdollLink {
            body,
            bone: bone_entity,
            rotation_offset: nalgebra_glm::quat_inverse(&body_rotation) * rotation,
        });

        if let Some(&parent_body) = body_by_bone.get(&(index - 1)) {
            create_spherical_joint(world, parent_body, body, SphericalJoint::new());
        }
    }

    if let Some(player) =
        world.get_mut::<crate::ecs::animation::components::AnimationPlayer>(player_entity)
    {
        player.playing = false;
        if let Some(ragdoll) = player.ragdoll.as_mut() {
            ragdoll.active = true;
            ragdoll.links = links;
        }
    }
}

/// Drives each ragdolled bone from its proxy body every frame.
pub fn ragdoll_sync_system(world: &mut World) {
    let mut writes: Vec<(Entity, Vec3, Quat)> = Vec::new();
    let mut players: Vec<Entity> = Vec::new();
    for (entity, player) in world
        .query_ref::<&crate::ecs::animation::components::AnimationPlayer>()
        .iter()
    {
        if player
            .ragdoll
            .as_ref()
            .is_some_and(|ragdoll| ragdoll.active)
        {
            players.push(entity);
        }
    }
    for player_entity in players {
        let links =
            match world.get::<crate::ecs::animation::components::AnimationPlayer>(player_entity) {
                Some(player) => match &player.ragdoll {
                    Some(ragdoll) => ragdoll.links.clone(),
                    None => continue,
                },
                None => continue,
            };
        for link in &links {
            let Some(body_transform) =
                world.get::<crate::ecs::transform::components::GlobalTransform>(link.body)
            else {
                continue;
            };
            let basis = nalgebra_glm::mat4_to_mat3(&body_transform.0);
            let x = basis.column(0).normalize();
            let y = basis.column(1).normalize();
            let z = basis.column(2).normalize();
            let orthonormal = nalgebra_glm::Mat3::from_columns(&[x, y, z]);
            let body_rotation = nalgebra_glm::mat3_to_quat(&orthonormal).normalize();
            let bone_world_rotation = body_rotation * link.rotation_offset;

            let parent_world_rotation = world
                .get::<nightshade_ecs::dynamic::ChildOf>(link.bone)
                .map(|child_of| child_of.0)
                .and_then(|parent| {
                    world.get::<crate::ecs::transform::components::GlobalTransform>(parent)
                })
                .map(|transform| {
                    let basis = nalgebra_glm::mat4_to_mat3(&transform.0);
                    let x = basis.column(0).normalize();
                    let y = basis.column(1).normalize();
                    let z = basis.column(2).normalize();
                    nalgebra_glm::mat3_to_quat(&nalgebra_glm::Mat3::from_columns(&[x, y, z]))
                        .normalize()
                })
                .unwrap_or_else(Quat::identity);
            let local_rotation =
                nalgebra_glm::quat_inverse(&parent_world_rotation) * bone_world_rotation;
            let translation = world
                .get::<crate::ecs::transform::components::LocalTransform>(link.bone)
                .map(|transform| transform.translation)
                .unwrap_or_else(Vec3::zeros);
            writes.push((link.bone, translation, local_rotation.normalize()));
        }
    }
    for (bone, translation, rotation) in writes {
        if let Some(transform) =
            world.get_mut::<crate::ecs::transform::components::LocalTransform>(bone)
        {
            transform.translation = translation;
            transform.rotation = rotation;
        }
    }
}

/// Tears down the proxy bodies and resumes the player.
pub fn ragdoll_recover(world: &mut World, player_entity: Entity) {
    let bodies: Vec<Entity> =
        match world.get::<crate::ecs::animation::components::AnimationPlayer>(player_entity) {
            Some(player) => match &player.ragdoll {
                Some(ragdoll) => ragdoll.links.iter().map(|link| link.body).collect(),
                None => return,
            },
            None => return,
        };
    for body in bodies {
        world.despawn_recursive(body);
    }
    if let Some(player) =
        world.get_mut::<crate::ecs::animation::components::AnimationPlayer>(player_entity)
    {
        player.playing = true;
        if let Some(ragdoll) = player.ragdoll.as_mut() {
            ragdoll.active = false;
            ragdoll.links.clear();
        }
    }
}
