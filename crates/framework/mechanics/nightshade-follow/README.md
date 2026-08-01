# nightshade-follow

The lag a camera keeps behind the thing it watches. An entity names the entity it follows and the offset it holds from it, and the crate eases its transform toward that place every frame at the rate the settings name. The same pair works for a chase camera, a trailing companion, or a line of carriages behind an engine.

## Composing it

```rust
use nightshade_follow::{FollowPlugin, FollowSettings};

app.add_plugin(FollowPlugin {
    settings: FollowSettings {
        rate: 6.0,
        snap: None,
        aim: true,
    },
});
```

The plugin inserts `FollowSettings` and registers `follow_the_target` on `Stage::FrameUpdate`, so the follower reacts to where its target ended the frame. The `LocalTransform` it writes is what the mechanic announces.

## Using it

```rust
use nalgebra_glm::vec3;
use nightshade_follow::queries::snapped;
use nightshade_follow::systems::frame::aimed_at;
use nightshade_follow::{FollowOffset, Following};

storage.set(camera, Following(player));
storage.set(camera, FollowOffset(vec3(0.0, 4.0, 8.0)));

let step = snapped(place.x, 0.25);
let facing = aimed_at(here, there);
```

A follower needs `Following`, `FollowOffset`, and a `LocalTransform`, and it is skipped while its target has no transform of its own. `rate` is how fast the follower closes the gap, and a higher rate leaves less lag. `snap` rounds the resting place onto a grid of that size when it is `Some`, which is how a pixel camera keeps its rows aligned, and `aim` turns the follower to face its target. `snapped` and `aimed_at` are the same calls the system makes, available for a game that wants to place something itself.

## Driving it

```
just example follow
```

The arrow keys walk the gold lead across the ground and the tail streams after it, Q and E lower and raise the rate the settings hold, Space turns the aim on and off so the tail swings to face what it chases, and Z puts the resting places on a grid. The readout gives the rate, the aim, the snap and how far the first follower is from where the crate is taking it.
