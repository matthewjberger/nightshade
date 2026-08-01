# nightshade-clock

The hour a world keeps, and the reading of it. `Clock` holds one float, the hour of day, wrapped into the twenty four hour range every time it is written. A game and its mechanics both read it, so a villager schedule, a lighting pass, and a shop that closes at dusk all agree on what time it is without any of them owning the number.

## Composing it

This crate carries no plugin. A game names it in its own `Cargo.toml` and inserts the resource, or composes a plugin that inserts it already.

```rust
app.insert_resource(Clock::default());
```

A caller holds nothing. The hour lives in the resource and something in the game has to advance it each frame.

## Using it

```rust
use nightshade_clock::Clock;
use nightshade_clock::commands::{pass, set_hour};
use nightshade_clock::queries::{HOURS_A_DAY, between, hour, share_of_day};

let clock = resources.res_mut::<Clock>();
pass(clock, delta * hours_a_second);
set_hour(clock, 6.0);

let now = hour(clock);
let share = share_of_day(clock);
let daylight = between(clock, 6.0, 20.0);
let whole_day = HOURS_A_DAY;
```

`pass` advances the hour by an amount and `set_hour` puts it at one, and both wrap, so an hour of twenty six reads back as two. `hour` reports the current hour and `share_of_day` reports it as a fraction of the day, which is the form a sun angle or a grading curve wants. `between` answers whether the hour falls in a window and handles a window that crosses midnight, so a night that runs from twenty to six is one call rather than two. `HOURS_A_DAY` is the length of a day the whole crate wraps against.

## Driving it

```
just example clock
```

Hold the right arrow to run the hours forward and the left arrow to run them back. Press 1, 2, 3 or 4 to put the hour at dawn, noon, dusk or midnight. The hand swings around the dial to the hour it is on and the readout gives that hour and whether it counts as daylight.
