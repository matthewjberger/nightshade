# nightshade-flashlight

`nightshade-flashlight` is the cell a torch runs on and what the beam does as that cell drains. An entity holds how much charge is left, how fast it drains, whether it is switched on and how strong its beam is, and the crate turns those four into the intensity of the light it is standing on. A hand torch, a helmet lamp and a dying lantern all run off the same four components.

## Composing it

```rust
app.add_plugin(nightshade_flashlight::FlashlightPlugin {
    settings: nightshade_flashlight::FlashlightSettings {
        full: 100.0,
        low: 22.0,
        brownout: 0.45,
    },
});
```

The plugin inserts `FlashlightSettings` and registers `drain_the_cell` on `Stage::LateUpdate`. `full` is a full cell, `low` is the mark the beam starts browning out below, and `brownout` is the share of the beam that is left when the cell is empty. The system announces `CellLow` when the charge crosses the low mark and `CellDied` when it reaches zero, and it switches `Lit` off itself when the cell dies.

## Using it

```rust
use nightshade_flashlight::commands::{douse, recharge, switch};
use nightshade_flashlight::queries::{brightness, flat, low, remaining, shining};
use nightshade_flashlight::{Battery, Beam, Drain, Lit};

torches.for_entity(torch, |(battery, lit)| switch(lit, battery));
torches.for_entity(torch, |(battery, _lit)| recharge(battery, &settings, 55.0));
```

`switch` turns the torch on and off and refuses a flat cell, `douse` turns it off, `recharge` adds charge and clamps it at `full`, and `empty` takes it all. The system writes `Light.intensity` from `Beam` and the brownout each frame, so a game never writes that intensity itself. `remaining` gives the share left for a gauge, `brightness` gives the same number the system wrote so a readout keeps no copy, and `flat`, `low` and `shining` report the three states worth naming.

## Driving it

```
just example flashlight
```

A torch sweeps its cone around a ring of crates in the dark while its cell drains. F switches the torch on and off, and R puts charge back into it. The readout gives the charge left, the beam strength, whether the torch is on, flat or running low, and a count of the brownouts and the deaths the events announced.
