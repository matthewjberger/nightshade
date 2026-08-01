# nightshade-input

A game names the actions a person can ask for as an enum that implements `Action`, and the crate turns the keys, the pointer buttons, the pad buttons, the sticks and the triggers into those actions each frame. A system reads `Actions<Ask>` and asks whether an action is held, was pressed this frame, was released this frame, or how far an axis is pushed. Only the bindings table names a key or a pad button, so `just lint-dag` fails on a game that reads one anywhere else.

## Composing it

```rust
app.add_plugin(nightshade_input::InputPlugin::<Ask>::new())
    .add_plugin(nightshade_gamepad::GamepadPlugin)
    .run()
```

`InputPlugin::<A>` inserts `Bindings<A>` and `Actions<A>`, and registers the system that reads the actions on `Stage::First`, after the gamepad poll. `profile(name)` opens a named profile that the builder calls that follow fill, `bind` adds a binding to an action in that profile, `only` replaces the bindings of an action, and `wearing(name)` selects the profile the frame reads. `persist(folder, name)` reads the bindings from a file at startup through `nightshade-store`, writes the defaults when the file is missing, and writes the file again when a binding changes, after a short settle or when the window closes.

## Using it

```rust
use nightshade_input::{Action, Actions, Binding, Pointer, Stick};
use nightshade_platform::KeyCode;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Move,
    Jump,
    Mine,
}

impl Action for Ask {
    fn all() -> &'static [Self] {
        &[Self::Move, Self::Jump, Self::Mine]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Move => "move",
            Self::Jump => "jump",
            Self::Mine => "mine",
        }
    }

    fn default_bindings(&self) -> &'static [Binding] {
        match self {
            Self::Move => &[
                Binding::Keys {
                    up: KeyCode::KeyW,
                    down: KeyCode::KeyS,
                    left: KeyCode::KeyA,
                    right: KeyCode::KeyD,
                },
                Binding::Stick(Stick::Left),
            ],
            Self::Jump => &[Binding::Key(KeyCode::Space)],
            Self::Mine => &[Binding::Pointer(Pointer::Left)],
        }
    }
}

pub fn dig(actions: Res<Actions<Ask>>, mut player: ResMut<Player>) {
    let walk = actions.axis2(Ask::Move);
    if actions.just_pressed(Ask::Jump) {
        player.jumping = true;
    }
    player.mining = actions.pressed(Ask::Mine);
}
```

A `Binding` is one key, one pointer button, one pad button, one stick, one trigger, four keys that make a two axis pair, or a positive and a negative press that make one axis. `Actions::pressed` reads the held set, `just_pressed` and `just_released` read what changed this frame, and `axis` and `axis2` read one axis or two. An action is not read while another crate holds the keyboard or the pointer through `InputCapture`, so a text field takes the keys away from the game for as long as it has them.

`Bindings<A>` holds the named profiles and the one the frame wears. The commands `open_profile`, `bind`, `bind_only`, `unbind`, `wear` and `forget` change it, each one raising its revision so the file is written again, and the queries `profiles`, `worn` and `bindings_of` read it back for a controls screen. The default profile cannot be forgotten.

`EngineAction` is the engine's own set, which `WindowPlugin` composes, so the camera controllers, the physics character and the interface navigation read `Actions<EngineAction>`. A game reads its own enum beside it.

## Taking it from another crate

The crate splits at a `contract` feature.

```toml
nightshade-input = { path = "../nightshade-input", default-features = false, features = ["contract"] }
```

The contract carries `Action`, `Actions`, `Binding`, `Bindings`, `Press`, `Pointer`, `Stick`, `EngineAction` and `DEFAULT_PROFILE`, and it takes only the gamepad and platform contracts for the button and key names, so naming an action never links the frame system or the store.
