# Plugins

A plugin installs resources and registers systems. That is all it does. It has
no lifetime of its own, no update method, and no state the engine keeps.

## Writing one

```rust
pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Score::default());
        app.add_system(Stage::Update, tally_score);
        app.add_system(Stage::Update, show_score);
    }
}
```

Compose it like any other:

```rust
App::new()
    .add_plugin(nightshade_window::WindowPlugin)
    .add_plugin(nightshade_wgpu::RenderPlugin)
    .add_plugin(ScorePlugin)
    .run();
```

`add_plugin` builds a type at most once and drops a duplicate rather than
panicking, so a plugin that composes its own dependencies is safe to name
twice.

## One crate, one domain

A capability is a crate. Twenty-six of them ship with the engine, and each
holds one domain: animation, audio, camera, decals, gamepad, navigation,
particles, physics, picking, text, the interface, and the rest.

The module layout is the same in every one:

- `components` for per-entity data
- `resources` for shared data
- `data` for plain values and lookup tables
- `systems` for frame behavior
- `commands` for functions that change a resource
- `queries` for functions that only report
- `plugin.rs` for registration and nothing else

A cycle between those bands fails the lint that checks them. What goes in
`systems` is covered in [Systems and stages](systems-and-stages.md).

## Plugins do not reference each other

Two capabilities that must speak do it through data. Each crate that has
something to say publishes types in its own `contract` feature, which pulls in
no `nightshade` at all:

```toml
nightshade-picking = { path = "...", default-features = false, features = ["contract"] }
```

The asker gets the types and never the other capability's behavior. Naming the
ask never links the answer, and the program composes both plugins when it
wants both to run.

An ask about an entity goes on that entity as a component the owner picks up
by archetype and takes off again:

```rust
world.set(
    camera,
    PendingCameraMove {
        kind: CameraMove::Orbit { yaw: 0.4, pitch: 0.2 },
    },
);
```

A component works because a frame runs between the ask and the answer.

## Extending the renderer

Four extension points live on `RenderAppExt` rather than on `App`:

1. `add_render_graph_config` to build passes and name targets
2. `on_update_render_graph` to change one each frame
3. `on_pre_render` to hand the renderer data before it draws
4. `on_window_event` to read raw platform events

A domain that draws places its own pass with `install_scene_pass`,
`install_lit_pass`, or `add_overlay_sub_pass`, then reaches it each frame by
name.

> A pre-render hook is a system, not a closure. It names the renderer first,
> then the resources and the one query it reads, and the runner takes one
> disjoint borrow per resource the signature names.

## Composing behavior

A plugin is a set of systems put into the frame together. Nothing about a
plugin knows what else was composed beside it, and what a program does is
whatever was composed into it.

The ring below carries three behaviors. Each writes its own part of the state
and none of them names the others, so switching one off leaves the rest as
they were.

```demo
id: plugin_stack
title: Three behaviors over one ring
caption: Turning, bobbing and tilting, each on its own switch. The transform is made from all three, and any of them can be absent.
```
