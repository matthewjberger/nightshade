# nightshade-animation

nightshade-animation plays keyframe clips over the entities a model spawned. It holds `AnimationPlayer`, the clips, the layers, the blend trees and the state graphs, and it advances them each frame and writes the result onto the transforms the renderer reads. It also carries clip retargeting, clip compression and the inverse kinematics chains a player can apply.

## Composing it

```rust
App::new()
    .add_plugins(DefaultPlugins)
    .add_plugin(nightshade_gltf::GltfPlugin)
    .add_plugin(nightshade_animation::AnimationPlugin)
    .run()
```

`AnimationPlugin` installs no resource. It registers `evaluate_animation_graphs`, `evaluate_blend_trees`, `update_animation_players`, `apply_animations` and `apply_root_motion` on `Stage::FrameUpdate`, then `render_sync_animation_system` on `Stage::Render`.

## Using it

```rust
use nightshade_animation::contract::AnimationPlayer;

if let Some(player) = spawn.storage.get_mut::<AnimationPlayer>(fox) {
    player.play(walk);
    player.looping = true;
    player.speed = 1.0;
}

fn change_gait(mut players: Query<(&mut AnimationPlayer,)>, fox: Entity, run: usize) {
    players.for_entity(fox, |(player,)| {
        player.blend_to(run, 0.3);
    });
}
```

`AnimationPlayer` lands on the model root when a glTF file carrying clips is spawned, and its `clips` field holds them in the order the file listed. `play` selects a clip by index, `blend_to` crossfades to another over a number of seconds, and `blend_to_in_phase` does the same while keeping the current clip's phase. `looping`, `speed` and `playing` are read every frame, and `layers`, `blend_tree` and `graph` drive the same player from a layered mask, a parameter space or a state machine instead.

## Taking it from another crate

The crate splits at a `contract` feature, so a crate that only names the types writes `nightshade-animation = { path = "../nightshade-animation", default-features = false, features = ["contract"] }`. The contract carries `AnimationPlayer`, `AnimationClip`, `AnimationChannel`, the layer and loop types, the blend tree, the animation graph and the IK chains, which is what `nightshade-gltf` reads to build clips from a file.
