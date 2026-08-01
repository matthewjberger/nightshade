use crate::data::{Ask, CropKind};
use crate::resources::Player;
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_camera::prelude::Camera;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;
use nightshade_sprite::prelude::Sprite;

pub fn ride_the_subject(
    mut player: ResMut<Player>,
    actions: Res<Actions<Ask>>,
    window: Res<Window>,
    mut subjects: Query<(&Sprite, &LocalTransform), Without<CropKind>>,
    mut cameras: Query<(&Camera, &mut LocalTransform)>,
) {
    let Some(camera) = player.camera else {
        return;
    };
    if !player.watching {
        player.watched = None;
        return;
    }
    if player.watched.is_none() && window.cursor_locked && actions.just_pressed(Ask::Pick) {
        let found = nearest_subject(player.eye, player.forward, &mut subjects);
        player.watched = found;
    }
    let Some(subject) = player.watched else {
        return;
    };
    let mut eye = None;
    subjects.for_entity(subject, |(sprite, transform)| {
        let lift = sprite.size.y * theme::WATCH_EYE_LIFT;
        eye = Some(transform.translation + Vec3::new(0.0, lift, 0.0));
    });
    let Some(eye) = eye else {
        player.watched = None;
        return;
    };
    player.position = eye - Vec3::new(0.0, theme::EYE_HEIGHT, 0.0);
    player.eye = eye;
    cameras.for_entity(camera, |(_, transform)| {
        transform.translation = eye;
    });
}

fn nearest_subject(
    eye: Vec3,
    forward: Vec3,
    subjects: &mut Query<(&Sprite, &LocalTransform), Without<CropKind>>,
) -> Option<Entity> {
    let mut best: Option<(Entity, f32)> = None;
    subjects.for_each_mut(|entity, (sprite, transform)| {
        let middle = transform.translation + Vec3::new(0.0, sprite.size.y * 0.5, 0.0);
        let to_subject = middle - eye;
        let gap = to_subject.magnitude();
        if gap > theme::WATCH_PICK_REACH || gap < 0.001 {
            return;
        }
        if to_subject.dot(&forward) / gap < theme::WATCH_PICK_COSINE {
            return;
        }
        if best.is_none_or(|(_, nearest)| gap < nearest) {
            best = Some((entity, gap));
        }
    });
    best.map(|(entity, _)| entity)
}
