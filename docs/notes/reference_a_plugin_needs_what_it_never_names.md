---
name: reference_a_plugin_needs_what_it_never_names
description: A plugin reads resources another plugin installs, and no manifest line records that, so a plugin declares its needs and the App checks them before the first frame
type: reference
---

The crate graph is acyclic. One hundred and thirteen crates, no cycle, and
`just lint-dag` holds the levels, the bands and the contract edges. The plugin
graph is a different graph, and Cargo cannot see it.

`world.res::<T>()` panics when the resource is absent. A plugin that reads a
resource another plugin installs therefore carries a dependency that no manifest
line and no ledger records. Composing that plugin alone builds, links, and then
panics inside the frame.

## What a plugin declares

`Plugin::needs` returns the resources the plugin reads and never installs, each
with the plugin that installs it, written with the `needs!` macro:

```rust
fn needs(&self) -> Vec<nightshade::plugin::Need> {
    nightshade::needs![
        nightshade_platform::contract::time::Time => "PlatformPlugin",
    ]
}
```

`add_plugin` and `add_plugins` hold the list, and `App::run` checks the whole
list against the resource map before the first frame. What comes back names
every missing resource, the plugin that asked for it, and the plugin that
installs it, so a program that composed a capability alone reads the answer at
the composition rather than in a frame.

Fifty-eight plugins declare their needs today. Two hubs hold most of them:
`PlatformPlugin` for `Time`, `Input`, `Window` and `InputCapture`, and
`GraphicsPlugin` for the render description.

## What the check made possible

A resource has one owner, and no crate installs another crate's resource to stop
its own systems from panicking. A backend reads a capability's frame optionally
and draws nothing when the capability is absent, so the dependency points from
the capability to the backend and never back.

A composition root that drives another capability composes its plugin and
records the edge in `tools/dag-check/plugin-edges.txt`. The retained interface
composes the text plugin that way, because it meshes strings.

## Add order, which a graph cannot express

The runner is a claimed slot: a backend claims it with `set_runner` under its
own name, a second claim panics with both names, and `take_over_runner` is the
deliberate replacement `nightshade-openxr` and the headless server call. A
shared default that several plugins want is installed with
`resource_or_insert_default`, which holds what the game inserted whichever way
the composition ran.
