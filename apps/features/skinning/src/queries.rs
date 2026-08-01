use crate::data::ROOT_BONES;
use nightshade_animation::contract::{AnimationClip, AnimationProperty};

pub fn strip_root_motion(clips: &[AnimationClip]) -> Vec<AnimationClip> {
    clips
        .iter()
        .map(|clip| AnimationClip {
            name: clip.name.clone(),
            duration: clip.duration,
            channels: clip
                .channels
                .iter()
                .filter(|channel| {
                    if channel.target_property == AnimationProperty::Translation {
                        return false;
                    }
                    !(ROOT_BONES.contains(&channel.target_node)
                        && channel.target_property == AnimationProperty::Rotation)
                })
                .cloned()
                .collect(),
            events: clip.events.clone(),
            source: 0,
        })
        .collect()
}
