# nightshade-flicker

`nightshade-flicker` is the unsteadiness a light keeps. Each frame it wanders a multiplier around the intensity a lamp was built at, drops the lamp out at random, and eases toward that target at its own rate. A failing street lamp, a guttering candle and a bad fluorescent tube all run off the same two components.

## Composing it

```rust
app.add_plugin(nightshade_flicker::FlickerPlugin {
    settings: nightshade_flicker::FlickerSettings {
        depth: 0.3,
        dropout_chance: 0.03,
        dropout_scale: 0.15,
        response: 18.0,
        dark_below: 0.4,
    },
});
```

The plugin inserts `FlickerSettings`, inserts the `Random` it draws from when the game has not already, and registers `flicker_the_lights` on `Stage::LateUpdate`. `depth` is how far the multiplier can fall, `dropout_chance` and `dropout_scale` are how often a lamp cuts out and how far down it goes, `response` is how fast it eases toward the target, and `dark_below` is the level a lamp counts as dark under. The system announces nothing. It draws from `nightshade-random`, so one seed gives one street.

## Using it

```rust
use nightshade_flicker::commands::{douse, set_base, steady};
use nightshade_flicker::queries::{dark, intensity, level};
use nightshade_flicker::{FlickerBase, Flickering};

lamps.for_entity(lamp, |(flickering,)| douse(flickering));
```

`FlickerBase` is the intensity the lamp was built at and `Flickering` is the multiplier the system writes over it. Where the entity also holds a `Light`, the system writes the product into `Light.intensity`, so a game never writes that intensity itself. `steady` puts a lamp back to full, `douse` takes it to nothing, and `set_base` moves the intensity it wanders around. `level` reports the multiplier, `intensity` reports the product for an entity that has no `Light` of its own, and `dark` reports whether the lamp is under the mark.

## Driving it

```
just example flicker
```

A row of street lamps flickers over a dark street. The up and down arrows widen and narrow the depth while the lamps are running, Space blacks the whole street out at once, and R settles every lamp back to full. The readout gives how many lamps are dark, the brightest and the lowest level, the depth in force and a count of the blackouts.
