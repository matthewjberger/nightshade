use crate::contract::{AnimationPlayer, ClipSet, ClipSetReport};

pub fn clip_set_take_from_player(player: &mut AnimationPlayer, name: impl Into<String>) -> ClipSet {
    let set = ClipSet {
        name: name.into(),
        clips: std::mem::take(&mut player.clips),
    };
    player.current_clip = None;
    player.playing = false;
    player.layers.clear();
    player.active_clips.clear();
    set
}

pub fn clip_set_apply(
    player: &mut AnimationPlayer,
    skeleton_bones: &[String],
    set: &ClipSet,
) -> ClipSetReport {
    let map = crate::queries::retarget::retarget_map_by_humanoid(&set.bones(), skeleton_bones);
    clip_set_apply_with_map(player, &map, set)
}

pub fn clip_set_apply_with_map(
    player: &mut AnimationPlayer,
    map: &crate::queries::retarget::RetargetMap,
    set: &ClipSet,
) -> ClipSetReport {
    let mut report = ClipSetReport::default();
    let mut clips = Vec::with_capacity(set.clips.len());

    for source in &set.clips {
        let mut clip = source.clone();
        clip.channels.retain_mut(|channel| {
            let Some(bone) = channel.target_bone_name.as_ref() else {
                report.channels_dropped += 1;
                return false;
            };
            let Some(target) = map.bones.get(bone) else {
                if !report.unresolved_bones.iter().any(|held| held == bone) {
                    report.unresolved_bones.push(bone.clone());
                }
                report.channels_dropped += 1;
                return false;
            };
            channel.target_bone_name = Some(target.clone());
            report.channels_resolved += 1;
            true
        });
        clips.push(clip);
    }

    report.unresolved_bones.sort();
    report.clips = clips.len();

    player.clips = clips;
    player.current_clip = None;
    player.playing = false;
    player.time = 0.0;
    player.layers.clear();
    player.active_clips.clear();
    player.blend_from_clip = None;
    player.blend_factor = 1.0;
    player.resolve_by_bone_name = true;

    report
}
