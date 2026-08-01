use crate::components::Cooldowns;
use crate::queries::raycast;
use crate::resources::{Feel, Hero, Player, VoxelViewer, VoxelWorld};
use crate::theme;
use nightshade::ecs::HierarchyIndex;
use nightshade::prelude::*;
use nightshade_camera::prelude::{ActiveCamera, Camera};
use nightshade_camera_controller::prelude::ThirdPersonCamera;
use nightshade_scene::prelude::LocalTransform;
use nightshade_scene::prelude::Visibility;

pub fn keep_out_of_the_blocks(
    world: Res<VoxelWorld>,
    hero: Res<Hero>,
    mut players: Query<(&LocalTransform,), With<Cooldowns>>,
    mut cameras: Query<(&mut LocalTransform,), With<ThirdPersonCamera>>,
) {
    let (Some(camera), Some(player)) = (hero.camera, hero.player) else {
        return;
    };
    let mut head = None;
    players.for_entity(player, |(transform,)| {
        head = Some(transform.translation + Vec3::new(0.0, theme::CAMERA_HEAD, 0.0));
    });
    let Some(head) = head else {
        return;
    };
    cameras.for_entity(camera, |(transform,)| {
        let offset = transform.translation - head;
        let distance = offset.magnitude();
        if distance < theme::FIRST_PERSON_DISTANCE {
            return;
        }
        let direction = offset / distance;
        let Some(hit) = raycast(&world, head, direction, distance) else {
            return;
        };
        let axis = if hit.normal.x != 0 {
            0
        } else if hit.normal.y != 0 {
            1
        } else {
            2
        };
        let face = hit.block[axis] as f32 + if hit.normal[axis] > 0 { 1.0 } else { 0.0 };
        let along = direction[axis];
        if along.abs() < 1e-4 {
            return;
        }
        let reach = ((face - head[axis]) / along - theme::CAMERA_SKIN).max(0.6);
        if reach < distance {
            transform.translation = head + direction * reach;
        }
    });
}

pub fn aim_the_viewer(
    active: Res<ActiveCamera>,
    mut viewer: ResMut<VoxelViewer>,
    mut player: ResMut<Player>,
    mut cameras: Query<(&LocalTransform,), With<Camera>>,
) {
    let Some(camera) = active.0 else {
        return;
    };
    cameras.for_entity(camera, |(transform,)| {
        let forward =
            nalgebra_glm::quat_rotate_vec3(&transform.rotation, &Vec3::new(0.0, 0.0, -1.0));
        viewer.eye = transform.translation;
        viewer.forward = forward;
        viewer.whole = false;
        player.eye = transform.translation;
        player.forward = forward;
    });
}

pub fn first_person(
    hero: Res<Hero>,
    hierarchy: Res<HierarchyIndex>,
    mut shown: ResMut<Feel>,
    mut commands: Commands,
    mut cameras: Query<(&ThirdPersonCamera,)>,
) {
    let (Some(camera), Some(player)) = (hero.camera, hero.player) else {
        return;
    };
    let mut close = false;
    cameras.for_entity(camera, |(third_person,)| {
        close = third_person.distance < theme::FIRST_PERSON_DISTANCE;
    });
    if shown.first_person == close {
        return;
    }
    shown.first_person = close;
    let mut pending = vec![player];
    while let Some(entity) = pending.pop() {
        commands.set(entity, Visibility { visible: !close });
        if let Some(children) = hierarchy.children.get(&entity) {
            pending.extend(children.iter().copied());
        }
    }
}
