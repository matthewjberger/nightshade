use crate::components::Post;
use crate::resources::Hammer;
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::ecs::Storage;
use nightshade::prelude::*;
use nightshade_platform::contract::input::Input;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_scene::contract::LocalTransform;
use nightshade_shake::commands::{settle, shake};
use nightshade_shake::queries::{shaking, weight};
use nightshade_shake::{ShakeSettings, Trauma};

pub fn take_the_orders(
    input: Res<Input>,
    settings: Res<ShakeSettings>,
    mut hammer: ResMut<Hammer>,
    storage: &mut Storage,
) {
    if input.keyboard.just_pressed(KeyCode::ArrowLeft) && hammer.over > 0 {
        hammer.over -= 1;
    }
    if input.keyboard.just_pressed(KeyCode::ArrowRight) && hammer.over + 1 < theme::POSTS {
        hammer.over += 1;
    }

    let posts: Vec<_> = storage
        .query_ref::<(&Post, &Trauma)>()
        .iter()
        .map(|(entity, (post, trauma))| (entity, post.0, *trauma))
        .collect();

    if input.keyboard.just_pressed(KeyCode::Space) {
        hammer.struck += 1;
        for (entity, index, held) in &posts {
            if *index != hammer.over {
                continue;
            }
            let mut trauma = *held;
            shake(&mut trauma, &settings, theme::BLOW);
            storage.set(*entity, trauma);
        }
    }

    if input.keyboard.just_pressed(KeyCode::KeyR) {
        for (entity, _index, held) in &posts {
            let mut trauma = *held;
            settle(&mut trauma);
            storage.set(*entity, trauma);
        }
    }
}

pub fn move_the_hammer(hammer: Res<Hammer>, storage: &mut Storage) {
    let Some(entity) = hammer.entity else {
        return;
    };
    let start = -(theme::POSTS as f32 - 1.0) * theme::SPACING * 0.5;
    if let Some(transform) = storage.get_mut::<LocalTransform>(entity) {
        transform.translation = vec3(
            start + hammer.over as f32 * theme::SPACING,
            theme::HAMMER_HEIGHT,
            0.0,
        );
    }
}

pub fn show_the_shake(hammer: Res<Hammer>, mut readout: ResMut<Readout>, posts: Query<(&Trauma,)>) {
    let mut worst = 0.0_f32;
    let mut moving = 0;
    posts.for_each(|_entity, (trauma,)| {
        worst = worst.max(weight(trauma));
        if shaking(trauma) {
            moving += 1;
        }
    });
    readout.notes.clear();
    readout.notes.push(format!("{moving} still shaking"));
    readout.notes.push(format!("worst weight {worst:.2}"));
    readout.notes.push(format!("{} blows", hammer.struck));
}
