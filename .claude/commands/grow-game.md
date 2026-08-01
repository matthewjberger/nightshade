---
description: Grow a nightshade game past its vertical slice. Screens and states, an overworld of levels, progression and story, a second mode, then prove it with captures and ship it.
---

# Grow Game

The slice runs. This is the horizontal pass. Work through the steps in order.

## Step 1: read what is already there

Run `/intro-nightshade` if this session has not. Then read the game's own
`src/` in band order: `data`, `components`, `resources`, `queries`,
`commands`, `plugin`, then every file in `systems/`. Read them yourself. The
pass you are about to make moves most of these files, and a summary is not
enough to move them safely.

`apps/games/tower_defense/src/` is the reference for screens driven by app
states. Read it before you write your first one.

## Step 2: ask what this pass is for

One `AskUserQuestion` call, multi-select, before any code:

- `Scope` - What goes in this pass? Offer "screens and an overworld", "more
  levels", "progression and story", "a second mode", "feel and polish".

Take the answers as the whole of the pass. Do not widen it. The standing brief
from `/new-game` still holds: per-entity data in components, shared data in
resources, behavior in systems, the `./template` bands, and no comments
anywhere.

## Step 3: screens are app states

`Screen` is a `Copy + PartialEq` enum in `resources.rs`. The plugin inserts it
once with `app.insert_state(Screen::Title)`, builds a screen with
`app.on_enter(Screen::X, system)`, and runs a screen with
`app.add_system(Stage::Update, while_in(Screen::X, (systems, ...)))`.

- A tuple holds at most 16 systems. Split into two `while_in` calls and keep
  the order you want, because a stage runs systems in registration order.
- A parameter system asks for the next screen with
  `ResMut<nightshade::ecs::state::NextState<Screen>>` and sets `pending`.
- A transition into the screen you are already in fires no event, so
  `on_enter` does not run. When a screen must rebuild itself in place, gate the
  build system on a flag in your own resource and let it run under `while_in`.
- Gate every level system behind the level screen. A look system that keeps
  running turns the map camera with the mouse, and waves keep spawning behind
  a menu.

## Step 4: one tag tears a screen down

Put a marker component on everything a screen spawns: the geometry, the
lights, the crates, the pickups, the enemies, the projectiles, and the
particle bursts. Then one query despawns the screen and nothing leaks into the
next one.

```rust
pieces.for_each(|entity, _| commands.despawn_recursive(entity));
```

Tag at the spawn site, not afterwards, or the thing you forget is the thing
that survives.

## Step 5: a second camera

`spawn_camera` gives the entity. Set `Camera { projection, smoothing: None }`
and a `LocalTransform` whose rotation is a pure X rotation for the pitch, then
point `ActiveCamera` at it. Put it back on the player's eyes when the level
starts.

Do not compose `CameraControllerPlugin` in a game that drives its own camera.
Its frame systems move whichever camera is active: WASD flies it and a right
drag turns it, and there is no marker component to opt out of. Call
`nightshade_camera_controller::systems::controllers::first_person_camera_look_system`
from the game's own look system instead, and leave the plugin uncomposed.

## Step 6: levels are data

A level plan is a struct of tables: the start point, the start facing, the
rooms, the doorways, the platforms, the pillars, the crates, the lamps, the
gate, the secret, the spawn points, the loot, the objective, and the twist.

- The table of plans is a `static`, not a `const`. Taking a `&'static` out of
  a `const` with a runtime index borrows a temporary and will not compile.
- Write a room builder in `systems/level.rs`: for each room a floor slab and
  four walls, with each wall cut where a doorway crosses it and a lintel over
  the gap. A level is then a short block of data instead of a wall of blocks.
  Keep the hand cut tables for the one level that earned them.
- Dedupe the walls by a rounded key. Two rooms that share an edge each emit
  the same wall, and the pair z-fights.
- Keep the start point clear: nothing solid on it, no pickup within three
  metres, no enemy within eight, and something worth looking at in front of
  it. Store a facing on the plan and set the eye rotation from it, or half
  your levels open against a wall.

## Step 7: progression, story, and a second mode

- An overworld map is a plate, roads between dots, and one dot per level, seen
  from a camera above it. A dot is open, cleared, or shut, and the colour says
  which. A token stands on the dot the player has chosen and slides along the
  road when they move.
- Progression is data: a cleared list, a link list per dot, and a rule that
  opens a dot when a dot beside it is cleared.
- Story is two lines per level, one before and one after, plus a line for each
  district. Show the line on the map panel and again as a banner when the
  level opens. The story lives in the level table, next to the level it tells.
- Vary the play with objectives and twists rather than new systems. Five
  objectives and six twists cover more ground than five more levels.
- A second mode is the same levels in a different order under a different
  rule. A run mode wants a seed, a stop counter, a purse, and a shop between
  stops that sells changes to the numbers the systems already read.

## Step 8: screen interface

A second `UiTreeBuilder::new(build.context())` builds a second tree, so the
head-up display and the screen board hide apart. `ui_set_visible` takes
`RetainedUiDirty` and a node, not a tree root, so keep the handle to the panel
you built rather than to the tree.

Rebuild the panel text only when it changes. A short string that describes the
screen state makes a cheap key.

Never list the controls on any screen. Not as text, not as key caps.

## Step 9: prove it by picture

In order:

1. `just format`
2. `cargo clippy -p <name> --all-targets`, and fix every warning. Dead code is
   removed, never suppressed.
3. `just lint-dag`
4. Add a clap flag to `Arguments` that opens a screen directly, for example
   `--map` and `--level <n>`. A capture cannot press a key, and without a flag
   you can only ever see the first screen.
5. Start the game once per screen and once per level with
   `just run <name> --level <n> --capture <path>`, and look at every picture.

A capture catches what compiling cannot: a start point inside a pillar, a
medkit under the player's feet, an enemy in reach at frame one, a level that
opens facing a wall, and a panel that covers the thing it describes. Fix what
you see and capture again.

## Step 10: ship it

Commit with a Conventional Commits subject that says what changed and why,
then push the branch. Tell the user what is new, how to reach it, and which
flows you read but could not play.
