# nightshade-shake

`nightshade-shake` is the trauma a camera carries after a blow and the offset it asks for while that trauma holds. Trauma builds when something hits, decays on its own, and drives a random offset and roll whose size is the square of what is left. A camera, a held weapon and a struck post all shake off the same three components.

## Composing it

```rust
app.add_plugin(nightshade_shake::ShakePlugin {
    settings: nightshade_shake::ShakeSettings {
        decay: 1.6,
        reach: 0.35,
        turn: 0.08,
        ceiling: 1.0,
    },
});
```

The plugin inserts `ShakeSettings`, inserts the `Roll` the random offsets come from when the game has not already, and registers `settle_the_shake` on `Stage::LateUpdate`. `decay` is how fast trauma drains a second, `reach` and `turn` bound the offset and the roll, and `ceiling` caps how much trauma can stack. The system announces nothing.

## Using it

```rust
use nightshade_shake::commands::{settle, shake};
use nightshade_shake::queries::{shaking, weight};
use nightshade_shake::{ShakeOffset, ShakeTurn, Trauma};

shake(&mut trauma, &settings, 0.4);
storage.set(camera, trauma);

if let Some(offset) = storage.get::<ShakeOffset>(camera) {
    transform.translation = wanted + offset.0;
}
```

`shake` adds trauma and clamps it at the ceiling, and `settle` drops it to zero for a cut or a respawn. The system writes `ShakeOffset` and `ShakeTurn` each frame, and the game applies them on top of wherever it wanted the camera, so the shake never fights the camera's own movement. `weight` gives the squared falloff the offsets are scaled by, which is also what a game uses to size an effect of its own, and `shaking` reports whether any trauma is left.

## Driving it

```
just example shake
```

The left and right arrows slide the red hammer along the row of posts and Space strikes the one underneath it, which jumps and rolls by the offset the crate writes and settles back as the trauma decays. R settles every post at once, and the readout gives how many are still shaking, the worst weight and the count of blows.
