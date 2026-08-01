# nightshade-shell

`nightshade-shell` is a developer console that drops over a running program. It is generic over a context type the program owns, so a command reads and writes the program's own state alongside the resource map and the command queue. The crate carries the input handling, the command registry, the scroll and history behavior, and the retained interface overlay that draws it.

## Composing it

```rust
app.insert_resource(new_shell());
app.add_plugin(nightshade_ui::UiPlugin::new());
app.add_plugin(ShellOverlayPlugin::<Console>::default());
app.add_system(Stage::Update, systems::play::read_the_keys);
```

The program inserts `ShellState<C>` itself, because the registry and the starting height are its own. `ShellOverlayPlugin<C>` inserts `ShellOverlayState`, registers `shell_input_system` and `shell_command_system` on `Stage::Update`, and registers `shell_retained_ui_system` on `Stage::Render`.

## Using it

```rust
let mut shell = ShellState::new(Console::default());
shell.registry.register(Command {
    name: "spin",
    description: "Set how fast the body turns",
    usage: "spin <rate>",
    execute: set_the_spin,
});
for event in reader.iter() {
    if let AppEvent::Keyboard { key, state } = event {
        nightshade_shell::shell_key(&mut shell, resources.res::<Input>(), *key, *state);
    }
}
```

A `Command<C>` carries a name, a description, a usage line and an `execute` function taking the arguments, the `ResourceMap`, the `Commands` queue and the context, and returning the line the console prints. `registry.register` adds a command the program owns and `register_builtin` adds one listed above it under help, and `clear` and `help` are answered by the state itself. `shell_key` routes a keyboard event, toggles the console on the backquote key or on alt with C, and feeds the rest to `handle_key` while the console is open. `shell_is_capturing_input` says whether the console is taking the keys, which is what a game reads before moving a player. `ShellState` also carries `insert_entity`, which puts an entity into the input buffer in the text `parse_entity` reads back.

## Driving it

```
just example shell
```

The backquote key opens and closes the console, and typing `spin 2` sets how fast the body turns while `stop` holds it still. The body answers at once and the readout gives the current rate and whether the console stands open.
