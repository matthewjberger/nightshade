# nightshade-countdown

The time a round has left, and the word it sends when the time is gone. The crate holds one remainder and whether it is running, spends the remainder down each frame, and sends `RanOut` the moment it reaches zero. A game starts a round with a number of seconds and reacts to the event when the round ends.

## Composing it

```rust
use nightshade_countdown::CountdownPlugin;

app.add_plugin(CountdownPlugin);
```

The plugin inserts `Remaining` and `Running` and registers `spend_the_time` on `Stage::LateUpdate`. When the remainder reaches zero the system stops the clock and sends `RanOut`, which is the whole of what the mechanic announces.

## Using it

```rust
use nightshade::ecs::system_param::EventReader;
use nightshade::prelude::*;
use nightshade_countdown::commands::{spend, start, stop};
use nightshade_countdown::queries::{gone, seconds};
use nightshade_countdown::{RanOut, Remaining, Running};

fn play(
    mut remaining: ResMut<Remaining>,
    mut running: ResMut<Running>,
    ran_out: EventReader<RanOut>,
) {
    let left = seconds(&remaining);
    if left > 60.0 {
        spend(&mut remaining, 5.0);
    }
    if gone(&remaining) {
        stop(&mut running);
    }
    if ran_out.iter().count() > 0 {
        start(&mut remaining, &mut running, 90.0);
    }
}
```

`start` sets the remainder and turns the clock on, and a start of zero seconds leaves it off. `spend` takes seconds off outside the frame tick, which is how a penalty lands, and `stop` pauses the clock without clearing the remainder. `seconds` is what a timer draws and `gone` answers whether the remainder has reached zero. The crate splits at the `contract` feature, so a crate that only needs to name `RanOut` takes it with `default-features = false`.

## Driving it

```
just example countdown
```

Space starts a fresh round, Q stops the clock where it stands and F takes a second off as a penalty. The bar shrinks with the remainder the crate spends down, the readout says whether the clock is running, stopped or gone, and the round tally comes from the `RanOut` the crate sends.
