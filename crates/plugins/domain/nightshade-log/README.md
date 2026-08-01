# nightshade-log

This crate installs the tracing subscriber a program logs through. On the desktop it writes to the console, and with the `tracing` feature it also writes a file under a directory the program names. On android it goes to the platform log and in a browser it goes to the console, with the panic hook set. `DefaultPlugins` already composes it, so a program names it only when it wants a title or a configuration of its own.

## Composing it

```rust
App::new()
    .add_plugin(nightshade_log::LogPlugin {
        title: Some(theme::TITLE.to_string()),
        ..Default::default()
    })
    .add_plugin(nightshade_platform::PlatformPlugin)
    .add_plugin(nightshade_scene::ScenePlugin)
    .run()
```

`LogPlugin` registers no system and inserts no resource. Its `build` initializes the subscriber once and parks the guards it returns on `app.log_guards`, so the file writer lives as long as the app does.

## Using it

```rust
let plugin = nightshade_log::LogPlugin {
    title: None,
    config: nightshade_log::LogConfig {
        directory: "logs".to_string(),
        rotation: nightshade_log::LogRotation::Daily,
        default_filter: "info".to_string(),
        timestamp_format: "%Y-%m-%d_%H-%M-%S".to_string(),
    },
};
```

A `title` of `None` reads the window's title off the `Window` resource, so the log file is named after the program without repeating the name. `directory` is where the file lands and `rotation` decides the file name: `PerSession` stamps the time in the format `timestamp_format` gives, while `Daily` and `Never` keep one name. `default_filter` is used when the environment sets no filter of its own, and `log_file_name` is the function that spells the name out.

## Taking it from another crate

The crate carries no `contract` feature, because nothing asks it for types. A program that logs depends on `tracing` directly and composes this plugin to decide where the records go.
