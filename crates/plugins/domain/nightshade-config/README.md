# nightshade-config

nightshade-config is the settings a program declares rather than the constants it scatters. The `config!` macro writes an enum whose variants name the settings and a table of defaults, ranges and choices beside it. `Config<S>` holds the values a system reads, and the plugin reads the file at startup and writes it back when a value changes.

## Composing it

```rust
App::new()
    .add_plugins(DefaultPlugins)
    .add_plugin(ConfigPlugin::<Settings>::new().persist("nightshade", "settings.toml"))
    .run()
```

`ConfigPlugin` inserts `Config<S>`. When `persist` names a folder and a file it also inserts `ConfigFile<S>` and registers `keep_the_settings` on `Stage::Last`, which writes the file back through `nightshade-store` once a change has settled.

## Using it

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
            Tint: color([1.0, 1.0, 1.0, 1.0]),
        }
    }
}

pub fn keep_the_count(config: Res<Config<Settings>>, mut resources: ResMut<SettingsResources>) {
    let wanted = config.whole(Settings::Cubes).max(0) as usize;
    let speed = config.number(Settings::SpinSpeed);
}
```

A group name in quotes heads each block, and each variant declares its kind and its default, with the range for a number or a whole and the options for a choice. A system takes `Res<Config<Settings>>` and reads a value with the accessor for its kind: `toggle`, `number`, `whole`, `choice`, `text`, `color` and `option`. Writing takes `ResMut` and the matching `set_toggle`, `set_number` or `set_choice`, and `reset` puts a key back to its default. `revision` reports how many changes have landed, which is what the persisting system watches.

## Taking it from another crate

The crate splits at a `contract` feature, so a crate that only names the types writes `nightshade-config = { path = "../nightshade-config", default-features = false, features = ["contract"] }`. The contract carries `Config`, `ConfigEntry`, `ConfigKind`, `ConfigSchema` and `ConfigValue` with no dependency at all, which is what `nightshade-ui-config` reads to build a settings panel from the same declaration.
