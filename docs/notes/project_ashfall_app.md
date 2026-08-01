---
name: project_ashfall_app
description: apps/games/ashfall is the PS1-style survival horror app, whose two-layer town map holds to four reachability rules
type: project
---

`apps/games/ashfall` is a PS1-style psychological survival horror game drawing on Silent Hill 1, Resident Evil, Penumbra and SOMA.

The town is a 64x48 two-layer tile map in `data/level.rs`, with rooms in `data/rooms.rs`, notes in `data/notes.rs`, props and examine sites in `data/props.rs`. `MAP` and `UPPER` are the two layers, each `HEIGHT` rows of `WIDTH` characters. The town holds to four rules, and a hand edit that breaks one makes the run unplayable without any error: each key is reachable only behind the door its predecessor opens, the evidence room is reachable only over the catwalk, no husk stands within reach of the waking place, and nothing sits on top of anything else. Walk the whole run after you change a layer.

Text placement is the exception: notes and examine texts are placed by hand against the room they describe, each with an anchor prop on the same tile, because generating those positions randomly puts "your car" on the clinic's upper floor.

`--capture <path> --capture-frame <n>` is the bounded launch, and it auto-starts a run from the title screen, so it exercises the whole level build. The `just run` and `just run-app` recipes already pass `--` to cargo, so write the flags straight after the app name and add no separator of your own.

It composes the six horror crates for its era, its beam, its lamps, its noises, its faces and its screen washes, so `PlayerState` and `RunState` hold only what is the game's own. Refer to [the horror crates](project_horror_crates.md).

It holds to the template's bands, with no comments anywhere.
