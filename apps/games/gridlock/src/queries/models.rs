use crate::data::districts::CLIP_NAMES;
use nightshade_animation::contract::{AnimationClip, find_clip};

pub fn pick_clips(available: &[AnimationClip]) -> Vec<AnimationClip> {
    let picked: Vec<AnimationClip> = CLIP_NAMES
        .iter()
        .filter_map(|wanted| find_clip(available, wanted))
        .collect();
    if picked.len() == CLIP_NAMES.len() {
        picked
    } else {
        Vec::new()
    }
}
