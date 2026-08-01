# nightshade-cli

nightshade-cli is the command line every program shares. It parses the flags each app carries in common, the scheduled capture and the window size, alongside the app's own argument struct, and it inserts that struct as a resource for the app's systems to read. On the web the same fields come from the URL query string, and on android an argument list clap rejects falls back to the defaults.

## Composing it

```rust
App::new()
    .add_plugin(nightshade_window::WindowPlugin::default())
    .add_plugin(nightshade_cli::CliPlugin::<Arguments>::default())
    .run()
```

`CliPlugin` installs no system. In `build` it parses once, writes the `ScheduledCapture` resource and the size on the `Window` resource, and inserts the app's argument struct. It asserts that `WindowPlugin` was composed first, because those two resources have to exist before it writes them.

## Using it

```rust
use nightshade_cli::clap;

#[derive(clap::Args, Default)]
pub struct SdfArguments {
    #[arg(
        long,
        help = "Open on one of the cache pictures: 1 bricks, 2 clipmap levels, 3 march steps"
    )]
    pub view: Option<u32>,
}

pub fn open_on_a_view(arguments: Res<SdfArguments>, mut view: ResMut<SdfDebugView>) {
    *view = match arguments.view.unwrap_or(0) {
        1 => SdfDebugView::Bricks,
        2 => SdfDebugView::Levels,
        3 => SdfDebugView::Steps,
        _ => SdfDebugView::Lit,
    };
}
```

An app declares its own flags as a `#[derive(clap::Args, Default)]` struct in `resources.rs`, with `use nightshade_cli::clap;` for the derive and `#[arg(help = "...")]` for the help text. A system reads it back as `Res<Arguments>`, and a plugin composed after `CliPlugin` may read it in `build`. `NoArguments` is what an app with no flags of its own names. The shared flags are `--capture`, `--capture-next`, `--capture-frame`, `--capture-settle` and `--size WIDTHxHEIGHT`, and `capture_frame` on the plugin is the frame the picture is taken on when the flag does not say.

## Taking it from another crate

The crate does not split at a `contract` feature. The whole crate is the contract, and `nightshade-app` is what names it, taking `CliPlugin`, `NoArguments` and the re-exported `clap` to compose the command line into every program.
