use crate::contract::{AnimationChannel, AnimationClip};
use crate::data::humanoid::{HumanoidSide, humanoid_joint};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RetargetMap {
    pub bones: HashMap<String, String>,
}

fn bone_suffix(name: &str) -> &str {
    match name.rsplit_once(':') {
        Some((_, suffix)) => suffix,
        None => name,
    }
}

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

pub fn retarget_map_by_humanoid(source_bones: &[String], target_bones: &[String]) -> RetargetMap {
    let mut map = retarget_map_by_suffix(source_bones, target_bones);
    let mut target_by_joint: HashMap<(HumanoidSide, &'static str), &String> = HashMap::new();
    for name in target_bones {
        if let Some(joint) = humanoid_joint(name) {
            target_by_joint.entry(joint).or_insert(name);
        }
    }
    for source in source_bones {
        if map.bones.contains_key(source) {
            continue;
        }
        let Some(joint) = humanoid_joint(source) else {
            continue;
        };
        let Some(target) = target_by_joint.get(&joint) else {
            continue;
        };
        map.bones.insert(source.clone(), (*target).clone());
    }
    map
}

fn retarget_channel(channel: &AnimationChannel, map: &RetargetMap) -> Option<AnimationChannel> {
    let source_name = channel.target_bone_name.as_ref()?;
    let target_name = map.bones.get(source_name)?;
    let mut retargeted = channel.clone();
    retargeted.target_bone_name = Some(target_name.clone());
    Some(retargeted)
}

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
        source: 0,
    }
}
