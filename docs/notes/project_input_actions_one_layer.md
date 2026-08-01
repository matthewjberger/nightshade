# nightshade-input is the only input layer, and a game never reads a key by name

`crates/plugins/domain/nightshade-input` turns keys, pointer buttons, pad
buttons, sticks and triggers into the actions a program named. There is no
other layer: the fixed `InputAxes` and `InputAction` set is gone from
`nightshade-platform`, and `nightshade-gamepad` only polls the pad.

Two action enums live in a program on purpose. `EngineAction` is the engine's
vocabulary, composed by `WindowPlugin` so every program has it, and it is what
the camera controllers, the physics character, the retained interface
navigation and `nightshade-screens` read. A game declares its own enum, named
`Ask` by convention, in its `data` band, composes `InputPlugin::<Ask>`, and its
systems take `Res<Actions<Ask>>`. The two are separate resources because an
engine crate cannot name a game type, and a game that wants a different key
for the engine's move or menu rebinds `EngineAction` through the same commands.

`Menu` binds the escape key and the pad start button. No camera controller
closes the program on it, because `nightshade-app` registers the escape exit
and a camera controller is not the place for that.

`just lint-dag` fails on a file under `apps/games` or `template/src` that
calls `is_key_pressed`, `keyboard.just_pressed`, `keyboard.just_released`,
`pad.pressed`, `pad.just_pressed` or `pad.axis`, or names `Res<Gamepad>`,
apart from the apps `tools/dag-check/raw-input.txt` records. That list only
shrinks. Mouse position, mouse deltas, the wheel, drags, touch and text entry
stay raw, because the layer has no shape for them.

`Actions<A>` and `Bindings<A>` sit in the crate's contract with the reads as
methods, so a crate that takes only the contract can read them. The gamepad
poll runs in `Stage::First` beside the action read, and the input plugin
orders its read after the poll by the system name the gamepad contract
publishes as `POLL_SYSTEM`, which is inert when no gamepad plugin is composed.
Rename that system and the order goes quiet rather than loud, so change the
constant with it.

`persist(folder, name)` keeps the bindings through `nightshade-store` as TOML
keyed by profile and action name. An action the file does not name keeps its
default and a binding the build cannot read is dropped.
