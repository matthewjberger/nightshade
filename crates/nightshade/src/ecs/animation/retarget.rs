//! Animation retargeting: rebinding a clip authored for one skeleton onto
//! another by mapping bone names. Same-topology rigs that differ only in naming
//! (a `mixamorig:` prefix, say) retarget exactly; differently proportioned rigs
//! reuse the rotation channels, which read naturally for humanoid motion.

use crate::ecs::animation::components::{AnimationChannel, AnimationClip};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A source-bone to target-bone name mapping used to rebind a clip.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RetargetMap {
    /// Source bone name to target bone name.
    pub bones: HashMap<String, String>,
}

fn bone_suffix(name: &str) -> &str {
    match name.rsplit_once(':') {
        Some((_, suffix)) => suffix,
        None => name,
    }
}

/// Builds a retarget map by matching bone-name suffixes (the part after the last
/// `:`), so `mixamorig:LeftArm` maps to `LeftArm`. Case-sensitive.
pub fn retarget_map_by_suffix(source_bones: &[String], target_bones: &[String]) -> RetargetMap {
    let target_by_suffix: HashMap<&str, &String> = target_bones
        .iter()
        .map(|name| (bone_suffix(name), name))
        .collect();
    let mut bones = HashMap::new();
    for source in source_bones {
        if let Some(target) = target_by_suffix.get(bone_suffix(source)) {
            bones.insert(source.clone(), (*target).clone());
        }
    }
    RetargetMap { bones }
}

fn retarget_channel(channel: &AnimationChannel, map: &RetargetMap) -> Option<AnimationChannel> {
    let source_name = channel.target_bone_name.as_ref()?;
    let target_name = map.bones.get(source_name)?;
    let mut retargeted = channel.clone();
    retargeted.target_bone_name = Some(target_name.clone());
    Some(retargeted)
}

/// Rebinds a clip onto a target skeleton by renaming its channels' target bones
/// through `map`, dropping channels with no mapping.
pub fn retarget_clip(clip: &AnimationClip, map: &RetargetMap) -> AnimationClip {
    let channels = clip
        .channels
        .iter()
        .filter_map(|channel| retarget_channel(channel, map))
        .collect();
    AnimationClip {
        name: clip.name.clone(),
        duration: clip.duration,
        channels,
        events: clip.events.clone(),
    }
}
