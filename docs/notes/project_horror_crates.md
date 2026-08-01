---
name: project_horror_crates
description: Six framework crates carry what makes a horror game feel like one, and a game composes only the ones it wants
type: project
---

There is no horror crate. The feel is six crates, and a game composes the ones it wants.

`nightshade-lofi` at `interface/` writes one named look over `RenderSettings`: the render scale, the vertex snap, affine texture mapping, the anisotropy, the anti-aliasing, the bloom, the ambient occlusion, the saturation, the contrast, the vignette and the tonemap. `playstation()`, `handheld()` and `clean()` are the looks it ships. `wear` asks for a new one and the plugin applies it on the next frame, so a game can change era at run time.

`nightshade-flashlight` at `mechanics/` puts `Battery`, `Drain`, `Lit` and `Beam` on the entity that carries the light. It drains the cell, browns the beam out below the low mark, sets `Light.intensity` from the beam and the brownout, and sends `CellLow` and `CellDied`. `brightness` reports the same number a system wrote, so a readout never keeps a copy.

`nightshade-flicker` at `mechanics/` wanders a light around `FlickerBase` and drops it out at random, and writes the multiplier into `Flickering`. It draws from `nightshade-random`, so one seed gives one street.

`nightshade-senses` at `mechanics/` holds the `Noises` bus, the `Exposure` of whoever hides, and the `Hearing`, `Sight`, `Alert` and `LastSeen` of whoever listens. `make_a_noise` pushes, a system clears the bus in `Stage::Last`, and `hide` works the exposure out of a stance. A game that has its own line of sight test can take the bus and the exposure and leave the rest.

`nightshade-phase` at `mechanics/` turns one world between faces. A `PhaseLook` carries the fog, the grading and the light color of one face, the system writes them and repaints every `PhaseTinted` light, and `PhaseTurned` tells the game to swap what only it knows about. The phase never writes an intensity, so a lamp can flicker and change face at the same time.

`nightshade-ui-flash` at `interface/` sums the washes a screen carries and fades each one at its own rate. `add_flash` builds the full-screen node and `flash` pushes a color.

`apps/games/ashfall` composes all six and is the live example of them together.
