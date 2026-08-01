# nightshade-senses

`nightshade-senses` is what a watcher hears and sees, and how exposed whoever hides from it is. A noise goes onto a bus with a place and a loudness, a hider holds how much of itself it shows, and a watcher holds how far it hears, how far it sees, how alarmed it is and where it last had something. Each frame the crate raises the alert on what a watcher noticed and lets it fall when the watcher noticed nothing.

## Composing it

```rust
app.add_plugin(nightshade_senses::SensesPlugin {
    settings: nightshade_senses::SensesSettings {
        plane: nightshade_plane::Plane::Ground,
        crouch_scale: 0.45,
        moving_scale: 1.4,
        half_angle: std::f32::consts::FRAC_PI_3,
        noise_gain: 0.6,
        calm: 0.35,
        alarmed_above: 0.5,
    },
});
```

The plugin inserts `SensesSettings` and `Noises`, registers `notice_the_hiders` on `Stage::LateUpdate` and `clear_the_noises` on `Stage::Last`. `plane` is the plane the distances are measured across, `crouch_scale` and `moving_scale` are what a stance does to exposure, `half_angle` is half the cone a watcher sees through, `noise_gain` is how much a noise raises the alert, `calm` is how fast it falls a second, and `alarmed_above` is the mark that counts as alarmed. The bus is cleared every frame, so a noise must be pushed the frame it is made.

## Using it

```rust
use nightshade_senses::commands::{alarm, hide, make_a_noise, settle};
use nightshade_senses::queries::{alarmed, alertness, last_seen, loudest, visible};
use nightshade_senses::{Alert, Exposure, Hearing, LastSeen, Sight};

make_a_noise(&mut noises, at, 1.0);
hide(&mut exposure, &settings, crouching, moving);
```

`make_a_noise` pushes onto the bus and `forget` empties it early. `hide` works the exposure out of a stance, and `alarm` and `settle` move an alert by hand. Sight beats sound: a watcher that sees a hider goes straight to a full alert and writes `LastSeen`, one that only hears the loudest noise in range walks its alert up toward full, and one that has neither calms down. A game with its own line of sight test can take the bus and the exposure and leave the rest, because `visible` and `loudest` are public.

## Driving it

```
just example senses
```

A field of watchers turns while you move a hider through it. The arrows walk the hider, left shift crouches it to cut the exposure, and Space shouts, which puts a noise on the bus that the watchers turn toward. The watchers repaint themselves by how alarmed they are, and the readout gives the exposure, the stance, the worst alert on the field and a count of the shouts.
