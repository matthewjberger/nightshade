# nightshade-state

An app state is one value a program switches on, and a game's screens are the usual one. The crate holds `State<S>` and `NextState<S>`, applies a requested transition at the start of the next frame, and announces each one as a `StateTransition<S>` on the event bus. The gates `while_in`, `while_in_any`, `on_enter` and `on_exit` wrap systems so they run only in the state they belong to.

## Composing it

```rust
let opening = if app.world.res::<Arguments>().play {
    Screen::Play
} else {
    Screen::Title
};
app.add_plugin(StatePlugin::new(opening));
```

`StatePlugin::new` inserts `State<S>` and `NextState<S>` with the state a program opens in, and registers `apply_state_transition::<S, World>` on `Stage::First`. A program composes one plugin for each state type it carries.

## Using it

```rust
use nightshade_state::prelude::*;

app.on_enter(Screen::Title, systems::screens::open_title);
app.on_exit(Screen::Play, systems::screens::close_play);
app.add_system(
    Stage::Update,
    while_in(Screen::Play, systems::player::ask),
);

if in_state(world, Screen::Title) {
    next_state(world, Screen::Play);
}
```

`on_enter` and `on_exit` come from `StateAppExt` and take a system or a tuple of them, and both register on `Stage::First` so a transition is served before the frame's work. `while_in` gates systems on the current state and `while_in_any` on a set of them, both returning a plain system a stage takes. `current_state` reads the value and `in_state` compares it, while `next_state` records a request that the transition system applies at the start of the next frame. Both readers panic when no state of that type was inserted, so a missing plugin fails loudly. A system that wants to see the change itself reads `StateTransition<S>` with an `EventReader`, which carries the state before and the state after.

## Taking it from another crate

The crate splits at a `contract` feature.

```toml
nightshade-state = { path = "../../../plugins/describe/nightshade-state", default-features = false, features = ["contract"] }
```

The contract carries `State<S>`, `NextState<S>` and `StateTransition<S>`, which is what a crate needs to speak about a program's screens without linking the transition system. `nightshade-ui-menu` and `nightshade-screens` take it that way.
