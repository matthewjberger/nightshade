use nightshade::prelude::*;
use nightshade_animation::contract::AnimationPlayer;
use nightshade_platform::prelude::*;

use crate::data;
use crate::resources::LocomotionResources;

pub fn swap_the_character(
    mut locomotion_resources: ResMut<LocomotionResources>,
    mut loading: ResMut<nightshade_gltf::Loading>,
    input: Res<Input>,
    mut caches: nightshade_graphics::commands::builtins::EntityCaches,
    storage: &mut nightshade::ecs::Storage,
) {
    let step = asked_for_a_step(&input);
    if step == 0 {
        return;
    }
    let character = &locomotion_resources.character;
    if character.swapping || character.asked.is_some() {
        return;
    }
    let Some(model) = character.model else {
        return;
    };
    let count = data::CHARACTERS.len() as i32;
    let next = (character.index as i32 + step).rem_euclid(count) as usize;

    let held = storage
        .get::<AnimationPlayer>(model)
        .filter(|player| !player.clips.is_empty())
        .map(|player| nightshade_animation::clip_set_from_player(player, "locomotion"));

    let subtree = {
        let hierarchy = &*caches.hierarchy;
        let mut all = hierarchy.descendants(model);
        all.insert(0, model);
        all
    };
    nightshade_graphics::commands::builtins::despawn_entities_now(storage, &mut caches, &subtree);
    caches.hierarchy.despawn_recursive(storage, model);

    let handle = nightshade_load::load(
        &mut *loading,
        nightshade_load::Source::Bytes(data::CHARACTERS[next].1.to_vec()),
    );

    let character = &mut locomotion_resources.character;
    character.model = None;
    character.index = next;
    if held.is_some() {
        character.held_clips = held;
    }
    character.swapping = true;
    character.asked = Some(handle);
}

fn asked_for_a_step(input: &Input) -> i32 {
    let keyboard = &input.keyboard;
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        return 1;
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        return -1;
    }
    0
}
