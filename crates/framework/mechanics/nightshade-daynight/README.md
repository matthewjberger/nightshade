# nightshade-daynight

The hour a world keeps and how much of the night is in it. The crate turns the hour on `Clock` forward at the rate the settings name, wrapping at twenty four, and reports how far into the night that hour sits and where the sun stands. A game reads those to drive its sun, its grading, its fog, and whatever only comes out after dark.

## Composing it

```rust
use nightshade_daynight::{DayNightPlugin, DaySettings};

app.add_plugin(DayNightPlugin {
    settings: DaySettings {
        hours_a_second: 0.05,
        dawn: 6.0,
        dusk: 19.0,
    },
    hour: 8.0,
});
```

The plugin inserts `DaySettings` and the `Clock` from `nightshade-clock` at the starting hour, and registers `turn_the_hour` on `Stage::Update`. The hour on `Clock` is what the mechanic announces.

## Using it

```rust
use nightshade_daynight::commands::set_hour;
use nightshade_daynight::queries::{is_night, night_share, sun_direction};
use nightshade_daynight::{Clock, DaySettings};

set_hour(&mut clock, 21.5);

let share = night_share(&clock, &settings);
let dark = is_night(&clock, &settings);
let sun = sun_direction(&clock);
```

`night_share` is zero between dawn and dusk and climbs to one over the two hours past either edge, which is what a lantern or a fog color reads to fade in. `is_night` is that share past a half. `sun_direction` turns the hour into a direction for a directional light, and `set_hour` moves the clock straight to an hour when a game skips ahead.

## Driving it

```
just example daynight
```

The left and right arrows run the hour back and forward through `set_hour`, 1 jumps to dawn, 2 to midday, 3 to dusk and 4 to midnight, and Space stops and restarts the hour the frame system turns. The sun swings to the direction `sun_direction` gives and the sky color darkens with `night_share`, both read straight back out of the crate.
