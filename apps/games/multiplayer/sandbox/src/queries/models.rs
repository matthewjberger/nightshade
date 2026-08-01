use crate::data::{ANIMAL_CLIP_NAMES, CLIP_NAMES};
use nightshade_animation::contract::{AnimationClip, find_clip};

pub fn pick_clips(available: &[AnimationClip]) -> Vec<AnimationClip> {
    let picked: Vec<AnimationClip> = CLIP_NAMES
        .iter()
        .filter_map(|wanted| find_clip(available, wanted))
        .collect();
    if picked.len() == CLIP_NAMES.len() {
        return picked;
    }
    let loose: Vec<AnimationClip> = ANIMAL_CLIP_NAMES
        .iter()
        .filter_map(|wanted| find_clip(available, wanted))
        .collect();
    if loose.len() == ANIMAL_CLIP_NAMES.len() {
        loose
    } else {
        Vec::new()
    }
}
