# nightshade-platformer

`nightshade-platformer` is the short mercy a jump keeps after the ledge and before the ground. It holds a coyote window that refills while a body stands and drains while it falls, and a buffer window that remembers a jump asked for slightly too early. A game reads the two windows together to decide whether a jump is allowed this frame.

## Composing it

```rust
app.add_plugin(nightshade_platformer::PlatformerPlugin {
    settings: nightshade_platformer::PlatformerSettings {
        coyote: 0.12,
        buffer: 0.12,
    },
});
```

The plugin inserts `PlatformerSettings` and registers `spend_the_mercy` on `Stage::First`, which is where input becomes intent. The system writes `Coyote` and `JumpAsked` on every body that carries `Airborne`, and announces nothing.

## Using it

```rust
use nightshade_platformer::commands::{ask_for_a_jump, spend_the_jump};
use nightshade_platformer::queries::may_jump;
use nightshade_platformer::{Airborne, Coyote, JumpAsked, PlatformerSettings};

if pressed {
    ask_for_a_jump(&mut asked, &settings);
}
if may_jump(&coyote, &asked) {
    spend_the_jump(&mut asked, &mut coyote);
    body.rise = JUMP_SPEED;
}
storage.set(walker, Airborne(off_the_ground));
```

`Airborne` is the one value the game owns, written from whatever tells it the body left the ground. `ask_for_a_jump` opens the buffer window from the settings and `spend_the_jump` closes both windows once the jump is taken. `may_jump` answers whether the coyote and the buffer windows are open at the same moment. The settings live in the resource map, so a system reads them as `Res<PlatformerSettings>`.

## Driving it

```
just example platformer
```

Left and right run the body along the floor and over the ledge, and Space asks for a jump. A jump asked for just after the body leaves an edge still lifts it, and so does one asked for just before it lands, with the readout counting both windows down and the jumps taken.
