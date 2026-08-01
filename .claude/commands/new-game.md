---
description: Start a new game in this engine. Asks what to build, opens a worktree, scaffolds the app on the template bands, ships a vertical slice, then commits and pushes.
---

# New Game

You are starting a new game in the nightshade engine. Work through the steps in order. Do not skip the questions and do not start writing code before you have the answers.

## Step 1: load the map

Run `/intro-nightshade` if this session has not already. Then read, in this order:

- `template/README.md` and every file under `template/src/`. This is the architecture the new app must match.
- `apps/games/coin_rush/src/` for the same architecture at playable size.
- `apps/games/tower_defense/src/` only if the answers call for screens, states, or a retained-UI front end.

Read them yourself. Do not delegate this to an agent and do not work from a summary.

## Step 2: get the prompt, then the composition

The prompt is the design. Everything downstream follows from it, so write it first and write it properly. Ask with `AskUserQuestion`. Every question offers "Other", so the user can type an answer none of the options cover.

**Round zero, where the idea comes from.** One question on its own, before anything else:

0. `Idea` - Where does this game come from? Offer:
   - "You come up with it (Recommended)" - you invent the game.
   - "I'll type a rough idea" - the user puts a sentence or two in Other and you build the prompt out of it.
   - "I'll write the whole prompt" - the user supplies the finished text.

**Then write the prompt.** Whatever the source, you write it out in full and post it. A sentence is not a prompt. Turn whatever you were given into one, and invent the whole thing if that is what the answer said. Pick something you would want to play, not a demo. Keep it to these parts and make every one of them concrete:

- **The line it sits in.** One sentence, naming two games it stands beside.
- **Feel.** What the player's hands do, second to second. This is the part a prompt usually skips and the part that decides whether the game is any good.
- **Core loop.** What the player does, and what the game does back.
- **World.** What the space is like, and what makes it worth crossing.
- **Vertical slice.** The one thin path all the way through. Name what is in it, and say plainly that nothing else gets built until it opens and runs.
- **Then, horizontally.** What you expand into once the slice runs.

Post it, then ask one question: `Prompt` - "Build this?" Offer "Build it" and "Change something", and take the change out of Other. Rewrite and post again until the answer is "Build it". This is the only confirmation in the whole run. The prompt earns it. Nothing else does.

**Then the composition.** One question:

1. `Decide` - Who picks the camera, the assets, the crates, and the name? Offer "You pick it all (Recommended)" and "I'll pick them".

Take the first answer and choose them yourself, state them in one short block, and start building. Do not ask for confirmation. If a choice turns out to be wrong later, change it and say so.

Take the second answer and ask these four in one call. Each capability is a crate under `crates/plugins/`, and composing its plugin is what turns it on. Leave audio out; the standing brief below says it is not wanted yet.

2. `World` (multi-select) - `nightshade-physics`, `nightshade-navmesh`, `nightshade-picking`, `nightshade-particles`.
3. `Visuals` (multi-select) - `nightshade-sprite`, `nightshade-animation`, `nightshade-decals`, `nightshade-prefab`.
4. `Interface` (multi-select) - `nightshade-ui`, `nightshade-text`, `nightshade-gamepad`, `nightshade-camera-controller`.
5. `Name` - Offer three names you derive from the prompt. The answer becomes the crate name in snake_case, the app directory, the worktree, and the window title.

Whoever picks, take only what the vertical slice needs and say which crates you are holding back for the horizontal pass. `nightshade-camera-controller` carries a fly camera, an orbit camera, and a first person look. Take the crate for the functions, but do not compose `CameraControllerPlugin` unless the fly camera is what the game wants: its frame systems move whichever camera is active, so WASD flies it and a right drag turns it, and no marker component opts out. A game with its own controller calls `first_person_camera_look_system` from a system in `systems/` and leaves the plugin uncomposed.

`nightshade-window`, `nightshade-cli`, `nightshade-log`, `nightshade-wgpu`, `nightshade-camera`, `nightshade-scene`, `nightshade-platform`, `nightshade-graphics`, and `nightshade-render` are not questions. Every app takes them.

## Step 3: open the worktree

The game is built in a worktree, never on `main`.

- If the session is already in a worktree, stay in it.
- Otherwise call `EnterWorktree` with the game's snake_case name.

## Step 4: copy the template, then edit it

`template/` is meant to be copied and then changed. It is a starting point, not a picture to work from. Do not hand-write these files and do not reimplement them from memory: copy them and edit what you copied. A copy that still runs is worth more than a rewrite that nearly does.

Copy `template/src/`, `template/Cargo.toml`, `template/Trunk.toml`, `template/index.html`, `template/X.toml`, and `template/manifest.yaml` into `apps/games/<name>/`. Copy the tracked files only. `git ls-files template` is the list, and `template/target/` is not in it. The four beside `Cargo.toml` are what makes the app run on the web and on android the way every other game does: `X.toml` and `manifest.yaml` name the android package, `Trunk.toml` and `index.html` name the web one, and `index.html` carries `data-bin="<name>-run"`. Rename the package inside each.

The rest of the tracked files at the top of `template/` are there so the template also works as a standalone project outside the repo, and the workspace supplies them here, so leave behind `justfile`, `rust-toolchain`, `rustfmt.toml`, `.gitattributes`, `.gitignore`, `.github/`, `CROSS.toml`, `README.md`, the licenses, and the `about` and `deny` configs.

The workspace globs `apps/games/*`, so the root `Cargo.toml` needs no edit.

Then work through the copy:

1. `Cargo.toml` - set `name`, rename the `[[bin]]` to `<name>-run`, and turn every `path = "../crates/..."` dependency into `{ workspace = true, optional = true }`. The path is two directories too shallow from `apps/games/`, and the workspace already declares each one. Add the crates the answers picked and put them in the `plugin` feature list.
2. Rename `TemplatePlugin`, `TemplateResources`, and `TemplateCommands` after the game. Set the window title in `plugin.rs` and the `LogPlugin` title in `lib.rs`.
3. Leave every comment out. The repository carries none and `just lint-dag` fails on one.
4. Keep the `contract` and `plugin` feature split the template gives. It costs nothing and it is what `just lint-apps-isolated` builds against.
5. Replace `systems/example.rs` and `systems/ping.rs` with the game's own systems, one file per concern. Keep `systems/setup.rs` and `systems/hud.rs` and grow them.

Read `apps/games/coin_rush/src/` for the same structure at playable size, and follow it where the template is silent.

What the copy already gets right, and what you must not undo: `src/main.rs` is three lines and calls `<name>::launch()`. `launch()` lives in the library, not in `main.rs`, because android loads the library and `main` never runs there. `launch()` composes `WindowPlugin`, then `CliPlugin`, then `LogPlugin`, then `RenderPlugin`, then the capability plugins, then the game's own plugin, then `run()`. Composing `CliPlugin` before `WindowPlugin` trips an assertion that says so. `nightshade_platform::android_main!(launch);` sits at the bottom of `lib.rs`. The template registers no escape key handler, so decide what escape does: a game with a pause screen reads it in its own system, and a game without one registers `nightshade_platform::systems::escape_key_exit_system` on `Stage::FrameUpdate`. See `docs/notes/reference_template_escape_exit_trap.md`.

The game's own plugin registers and nothing else. Behavior goes in `src/systems/<concern>.rs`, one file per concern, and the systems name the `Res`, `ResMut`, `Commands`, and `Query` parameters they touch instead of taking the world. Queries go last in a system signature.

An app's own command-line flags are a `#[derive(clap::Args, Default)]` struct in `resources.rs`, composed as `CliPlugin::<Arguments>`. Never read `std::env::args`.

## Step 5: find the assets before you make any

Look first. Do not hand-build art the repo already carries. In order:

1. **`assets/` at the repo root.** Shared and already tracked: `gltf/`, `textures/`, `sky/`, `particles/`. Every app can read these.
2. **Another app's `assets/`.** `apps/games/srpg/assets/tiny_battle` and `tiny_town`, `apps/features/sprites/assets/tiny_farm`, `apps/games/voxels/assets/blocks_atlas.png`, `apps/games/ashfall/assets/textures`. The `tiny_*` sets are Kenney packs. Copy the file you want into the new app rather than reaching across into another app's directory.
3. **`../glTF-Sample-Assets`.** The Khronos models, a sibling of this repo and not part of it. `apps/rendering/` reads them where they sit. A game that wants one copies it in.
4. **Kenney, at kenney.nl.** CC0, and where the `tiny_*` sets came from. Tell the user which pack and why before you fetch anything.

Assets go under source control at `apps/games/<name>/assets/`, and the app reaches them with `include_bytes!("../assets/...")`, the way `apps/games/srpg/src/theme.rs` does. Keep them small: a packed atlas is a file, a whole downloaded pack is not.

If nothing that exists fits, build primitives in code for the slice and come back to art in the horizontal pass. Never stall the slice on art.

## Step 6: the standing brief

This is the user's brief. It holds for the whole build, not just the first pass.

> ensure all per-entity data is in components, all data shared across entities is in resources, and all procedural logic and behavior is in systems
> compliance with the architecture as dictated in ./template is crucial
> and the entire project should have no comments
>
> commit and push to the worktree when done
>
> run it when done
>
> don't need audio yet
>
> run the vertical slice when you can and then we can expand horizontally

"No comments" means none: no `//`, no `///`, no `//!`, anywhere in the app.

"Vertical slice first" means one thin path all the way through before any breadth. A camera, a controllable player, one thing to interact with, and one readout that proves the loop closed. No menus, no screens, no second level, no polish until that opens and runs.

## Step 7: prove it, then ship it

In order:

1. `just format`
2. `cargo clippy -p <name> --all-targets` and fix every warning. Dead code is removed, never suppressed. No `#[allow(...)]`.
3. Launch it once: `just run <name> --capture <path>`. This is the bounded launch, and it catches a first-frame panic that compiling cannot. Run this app and no other.
4. Commit with a Conventional Commits subject, then push the worktree branch.

Do not run `just lint` or `just test` across the workspace. CI runs those.

Then tell the user what the slice does, and ask which direction to expand.
