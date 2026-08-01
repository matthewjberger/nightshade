# nightshade-toast

The one short line a game says to itself and then forgets. A game writes a line into the `Toast` resource, the crate counts it down each frame, and the line clears itself when the time runs out. Anything that draws reads the line and its opacity, so the message and the place it appears stay apart.

## Composing it

```rust
use nightshade::app::App;
use nightshade_toast::{ToastPlugin, ToastSettings};

fn build(app: &mut App) {
    app.add_plugin(ToastPlugin {
        settings: ToastSettings {
            showing: 2.5,
            fading: 0.5,
        },
    });
}
```

The plugin inserts `ToastSettings` and an empty `Toast`, then registers `fade_the_toast` on `Stage::LateUpdate`, where the one writer of the remaining time lives. Nothing is announced, because the line itself is the state a game reads.

## Using it

```rust
use nightshade_toast::commands::{hush, say};
use nightshade_toast::queries::{opacity, showing};
use nightshade_toast::{Toast, ToastSettings};

say(&mut toast, &settings, "the gate is open");

if showing(&toast) {
    let share = opacity(&toast, &settings);
}

hush(&mut toast);
```

`say` takes anything that turns into a `String` and starts the countdown at the `showing` seconds the settings carry. `showing` reports whether a line is still up, and `opacity` returns the share of the `fading` window that is left, clamped between zero and one, so a panel can fade the line out over its last moments. `hush` clears the line at once for a game that wants the screen quiet.

## Driving it

```
just example toast
```

Press 1, 2, 3 or 4 to say one of four lines, and Space to hush whatever is up. The card in the scene grows and rises with the opacity and settles back as the line runs out, and the readout gives the line and the share left.
