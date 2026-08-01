---
name: project_config_crates
description: nightshade-config is the settings an app declares as one enum, and nightshade-ui-config is the panel that edits them
type: project
---

An app's tuning used to be a wall of constants. `apps/games/farm` keeps 520 of
them in one `theme.rs`, and `apps/games/hollowmere` keeps 731 across thirty
files. A constant is right for what the program is made of: an asset, a sprite
index, a table of places. It is wrong for a value a person is meant to change,
because nothing can read the set of them, nothing can show them, and nothing
can keep them between runs.

`nightshade-config` takes that second group. An app declares them once:

```rust
nightshade_config::config! {
    pub enum Settings {
        "Motion" {
            SpinSpeed: number(0.0, 4.0, 1.0),
            Bob: toggle(true),
        }
        "Scene" {
            Cubes: whole(1, 8, 5),
            Palette: choice(0, ["Ember", "Meadow", "Ice"]),
        }
    }
}
```

The macro writes the enum and a `ConfigSchema` for it. The schema is a table of
`ConfigEntry`, one for each variant, holding the key, the section, and a
`ConfigKind` that carries the default and the limits. `Config<Settings>` is the
resource that holds the values, and a system reads one by kind:

```rust
pub fn arrange(config: Res<Config<Settings>>) {
    let speed = config.number(Settings::SpinSpeed);
}
```

An accessor of the wrong kind panics with the key in the message, because a
setting read as the wrong type is a fault in the program rather than in the
file.

## What the parts are for

- The kinds are `toggle`, `number`, `whole`, `choice`, `text`, and `color`. A
  number and a whole carry a minimum and a maximum, a choice carries the labels
  it offers, a color is four channels, and `Config::set_at` holds every write
  inside those limits.
- `Config` counts a revision, and a write that does not change the value does
  not count. Persistence and any other watcher compare that count.
- The whole of `Config`, the schema and the entries sit behind the `contract`
  feature, so a crate that reads settings never links the file or the engine.

## Keeping them between runs

Persistence is what the app asks for, not what the crate does:

```rust
app.add_plugin(ConfigPlugin::<Settings>::new().persist("nightshade", "settings.toml"));
```

Without `persist` the plugin inserts the resource and registers nothing. With
it, `build` reads the file through `nightshade-store` and applies what it
carries, and writes the defaults when the store holds no file. A system on
`Stage::Last` writes the file back when the revision moves. The file is TOML,
one key to a line, a choice is written as its label rather than as an index,
and a color as four numbers, so the file stays readable. A key the schema does
not carry is ignored, and so is a value of the wrong type, so an old file never
stops the program.

The write waits for the value to settle for `SETTLE_SECONDS`, because a slider
that is being dragged moves the revision on every frame and each write is a
whole file. `Window::close_requested` skips that wait, so the last change
before the window closes reaches the store in the one frame the runner gives
it.

## The panel

`nightshade-ui-config` takes the schema and the retained UI and nothing else.
`build_config_rows(tree, config)` adds a heading for each section and one
widget for each entry, picked from the kind: a checkbox, a slider, a drag
value, a dropdown, a text input, a color picker. It hands back a `ConfigPanel<S>` of row
handles, which the app puts in the resource the plugin inserted.
`ConfigUiPlugin::<S>` then reads those rows every frame and writes what changed
into `Config<S>`, which is what makes the file follow the panel.

`build_config_rows` puts a "Restore Defaults" button below the rows.
`follow_the_reset` watches for that click, resets the values, and writes each
one back into its widget through the `ui_*_set_value` commands, which refresh
the fill, the text slot and the swatch as well as the data. Two of those
commands are new in `nightshade-ui`: `ui_checkbox_set_value` and
`ui_color_picker_set_value`.

Outside that button the rows are read and not written, so a value set from code
does not move the widget that shows it.

`apps/features/settings` is the whole of it on one screen: a row of cubes whose
count, spacing, spin, rise and color come from the settings, and a panel on the
left that the file remembers.
