# nightshade-gamepad

A gamepad reaches a program as one `Gamepad` resource holding the connection flag, the axis values, the buttons held and the buttons pressed this frame. On the desktop the values come from gilrs, and on android they come from the input queue through `nightshade-platform`. The button and axis names are the crate's own, so a system reads `GamepadButton::South` rather than a backend's spelling.

## Composing it

```rust
app.add_plugin(nightshade_gamepad::GamepadPlugin)
    .add_plugin(nightshade_physics::PhysicsPlugin)
    .run()
```

`GamepadPlugin` inserts `GamepadBackend` and `Gamepad`, and registers three systems on `Stage::First`: the poll that fills the pad, the input mode detection, and the axis application.

## Using it

```rust
use nightshade_gamepad::contract::{Gamepad, GamepadAxis, deadzoned};
use nightshade_gamepad::queries::query_active_gamepad;

pub fn look(pad: Res<Gamepad>, time: Res<Time>) {
    let stick = query_active_gamepad(&pad, |pad| {
        Vec2::new(
            deadzoned(pad.axis(GamepadAxis::RightStickX)),
            deadzoned(pad.axis(GamepadAxis::RightStickY)),
        )
    })
    .unwrap_or_else(Vec2::zeros);
}
```

`query_active_gamepad` runs the closure only while a pad is connected and reports `None` otherwise, so a system that also reads the mouse falls through to its other input. `Gamepad::axis` reports zero for an axis the pad does not carry, and `deadzoned` drops values below `STICK_DEADZONE` and rescales the rest so the stick starts from zero. `Gamepad::pressed` reads the held set and `Gamepad::just_pressed` reads what arrived this frame.

## Taking it from another crate

The crate splits at a `contract` feature.

```toml
nightshade-gamepad = { path = "../nightshade-gamepad", default-features = false, features = ["contract"] }
```

The contract carries `Gamepad`, `GamepadButton`, `GamepadAxis`, `STICK_DEADZONE` and `deadzoned`, and it takes no dependency at all, so naming the pad never links gilrs.
