use crate::data::CLIP_NAMES;
use nightshade_animation::contract::{AnimationClip, find_clip};

const LOOSE_CLIPS: [&str; 8] = ["walk", "walk", "run", "walk", "down", "down", "up", "up"];

pub fn pick_clips(available: &[AnimationClip], loose: bool) -> Vec<AnimationClip> {
    let picked: Vec<AnimationClip> = CLIP_NAMES
        .iter()
        .filter_map(|wanted| find_clip(available, wanted))
        .collect();
    if picked.len() == CLIP_NAMES.len() {
        return picked;
    }
    if !loose {
        return Vec::new();
    }
    let loose: Vec<AnimationClip> = LOOSE_CLIPS
        .iter()
        .filter_map(|wanted| find_clip(available, wanted))
        .collect();
    if loose.len() == LOOSE_CLIPS.len() {
        loose
    } else {
        Vec::new()
    }
}
