use crate::contract::AnimationPlayer;
use crate::data::{effective_loop_mode, wrap_time};

pub fn advance_player(player: &mut AnimationPlayer, delta_time: f32) {
    if !player.playing {
        return;
    }

    let clip_duration = if let Some(index) = player.current_clip {
        if let Some(clip) = player.clips.get(index) {
            clip.duration
        } else {
            return;
        }
    } else {
        return;
    };

    player.time += delta_time * player.speed;

    let mode = effective_loop_mode(player.loop_mode, player.looping);
    let (wrapped, still_playing) = wrap_time(player.time, clip_duration, mode);
    player.time = wrapped;
    if !still_playing {
        player.playing = false;
    }

    if player.blend_from_clip.is_some() {
        player.blend_factor += delta_time / player.blend_duration.max(0.001);
        if player.blend_factor >= 1.0 {
            player.blend_factor = 1.0;
            player.blend_from_clip = None;
            player.blend_from_time = 0.0;
        } else if let Some(from_index) = player.blend_from_clip
            && let Some(from_clip) = player.clips.get(from_index)
        {
            player.blend_from_time += delta_time * player.speed;
            if player.blend_from_time >= from_clip.duration && player.looping {
                player.blend_from_time %= from_clip.duration;
            }
        }
    }

    for layer_index in 0..player.layers.len() {
        let layer = &player.layers[layer_index];
        if !layer.playing {
            continue;
        }
        let Some(duration) = player
            .clips
            .get(layer.clip_index)
            .map(|clip| clip.duration)
            .filter(|duration| *duration > 0.0)
        else {
            continue;
        };
        let layer = &mut player.layers[layer_index];
        layer.time += delta_time * layer.speed;
        if layer.time >= duration {
            if layer.looping {
                layer.time %= duration;
            } else {
                layer.time = duration;
                layer.playing = false;
            }
        }
    }
}
