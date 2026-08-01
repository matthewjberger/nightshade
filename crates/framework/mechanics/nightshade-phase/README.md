# nightshade-phase

`nightshade-phase` turns one world between faces. A face is a look, the fog, the grading and the light color that belong to it, and the game names the whole list up front. When the phase moves, the crate writes that look over the render settings, repaints every light that asked to be tinted, and announces the turn so the game can swap what only it knows about.

## Composing it

```rust
app.add_plugin(nightshade_phase::PhasePlugin {
    looks: vec![
        nightshade_phase::PhaseLook {
            fog_color: [0.5, 0.5, 0.55],
            fog_start: 10.0,
            fog_end: 80.0,
            saturation: 1.0,
            contrast: 1.0,
            light_color: Vec3::new(1.0, 1.0, 1.0),
        },
    ],
    start: 0,
});
```

The plugin inserts `PhaseSettings` with the looks and `Phase` at the face it starts on, and registers `settle_the_phase` on `Stage::LateUpdate`. The system does nothing on a frame where the face has not moved, so the render settings are written once for each turn rather than every frame. It announces `PhaseTurned` with the face it arrived at.

## Using it

```rust
use nightshade_phase::commands::{apply_again, turn_over, turn_to};
use nightshade_phase::queries::{at, faces, look, turning};
use nightshade_phase::{PhaseTinted, PhaseTurned};

turn_over(&mut phase, &settings);
storage.set(lamp, PhaseTinted(1.0));
```

`turn_to` moves to a named face, `turn_over` steps to the next and wraps, and `apply_again` makes the system write the look once more after something else changed the render settings. A light that holds `PhaseTinted` is repainted to the face's color scaled by what the component holds, and the phase never writes an intensity, so a lamp can flicker and change face at the same time. `at` and `faces` report where the phase is and how many there are, `look` hands back a face for a readout, and `turning` reports that the next frame will write.

## Driving it

```
just example phase
```

A town stands under one of three faces. Space turns the town to the next face, and the digit keys go straight to one. The fog, the grading and the lamps all change together, and the readout gives the face the town is on, where its fog ends, the color its lamps took and a count of the turns.
