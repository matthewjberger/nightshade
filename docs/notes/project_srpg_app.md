---
name: project_srpg_app
description: apps/games/srpg is the tactics campaign (FFT plus Fire Emblem Three Houses) on two Kenney sheets packed into one atlas
type: project
---

`apps/games/srpg` is a Final Fantasy Tactics style battle with a Fire Emblem Three Houses style
camp between missions. The art is two Kenney sheets, `assets/tiny_battle/tilemap_packed.png` for
the field and `assets/tiny_town/tilemap_packed.png` for the camp, packed into the one atlas by
`pack_the_sheet` in `commands/textures.rs`.

The campaign is the six `CHAPTERS` in `data/story.rs`, each with its own enemy squad, hour of day,
deployment limit, and objective (`Objective::Rout`, `Seize`, or `Survive`). The four `PROLOGUE`
pages open over the camp at dusk. The camp is the south end of the same hill map, so no terrain is
rebuilt: camp props spawn on arrival and despawn on the march. Each companion is a persistent
`Fighter` entity that carries the campaign state and its camp avatar sprite, and a battle spawns
separate `Unit` entities linked back through `Unit.fighter`. The camp day has two kinds of
action. A companion takes one of the six `ACTIVITIES`, talk, train, advance, gift, spar, or muster,
and each spends one of `theme::ACTIONS_PER_DAY` except muster and advance, which spend none. A room
takes its own `RoomKind::call`, so the mess calls the squad to supper, the yard runs a drill, the
workshop patches the wounded, and the gate marches out. Talk and gifts raise the companion's
`rapport` with the commander, which opens its arc lines and its growth. Supper and sparring raise
the pairwise `bonds`, and `support_rank` reads the best of them among friends one tile away, worth
`SUPPORT_HIT` and `SUPPORT_POWER` on the attack. Damage and kills become merit, which becomes
experience and levels at the debrief. A fallen companion is wounded for `WOUND_DAYS` days rather
than dead.

**How to apply:** the switches come from `nightshade-cli` as `SrpgArguments` in `resources.rs`, so
run it as `just run-app srpg --camp --chapter 3 --capture <png> --capture-frame 220` with NO `--`
separator before the flags. `--prologue`, `--camp`, `--battle`, `--auto`, `--hour`, and `--chapter`
are the app's own. The app carries a `vulkan` cargo feature, so `just run-vulkan srpg <flags>` opens the
same game on the Vulkan backend, which is where the sprite pass of that backend was proved. A full automatic first mission needs about 9000 frames to reach the debrief.
Never nest one query inside another query's closure: see [[reference_no_nested_queries]].
