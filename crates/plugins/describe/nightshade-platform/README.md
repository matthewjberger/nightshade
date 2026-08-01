# nightshade-platform

`nightshade-platform` holds the window, the input and the clock. It declares the `Window` a program sets a title and a cursor on, the `Input` a system reads the keyboard, the mouse, the touches and the axes from, the `Time` that carries the delta and the frame count, and the `InputCapture` that says whether the UI has taken the pointer or the keys this frame. It owns the frame contract around those values, which is the resets and the cursor work that has to happen at the end of every frame, while the winit event loop that fills them lives in `nightshade-window`.

## Composing it

```rust
App::new()
    .add_plugin(nightshade_platform::PlatformPlugin)
    .add_plugin(nightshade_scene::ScenePlugin)
    .add_plugin(nightshade_graphics::GraphicsPlugin)
    .run()
```

`PlatformPlugin` inserts `Input`, `InputCapture`, `Time`, `Window` and the monitor and android input resources. It recenters a locked cursor on `Stage::First`, and on `Stage::Last` it applies the requested cursor and the IME state and clears the keyboard, mouse, touch, axis and event state for the next frame. `WindowPlugin` composes it, so a program that opens a window already has all of this and adds it directly only when it runs headless.

## Using it

```rust
pub fn update(input: Res<Input>, mut window: ResMut<Window>) {
    if input.keyboard.just_pressed(KeyCode::KeyP) {
        window.title = "paused".to_string();
    }
}
```

`input.keyboard` answers `is_key_pressed`, `just_pressed` and `just_released`, and `input.mouse` and `input.touch` carry the rest of the frame's input beside `input.input_mode` and the `AppEvent` list. A game does not read those by name: it declares its actions in `nightshade-input` and reads `Actions<A>`. `Window` is written rather than called, so a system sets `title`, `show_cursor`, `use_fullscreen` or `should_exit` and the runner applies it on the thread that owns the event loop. `set_cursor_locked`, `set_cursor_visible` and `open_url` are the commands over that, `pause`, `unpause` and `set_time_scale` drive the clock, and `keyboard_captured` and `pointer_captured` tell a game whether the UI has claimed the input this frame.

## Taking it from another crate

The crate splits at a `contract` feature, and almost every plugin crate takes it that way for the window and input vocabulary.

```toml
nightshade-platform = { path = "../nightshade-platform", default-features = false, features = ["contract"] }
```

The contract carries `Window`, `Input`, `Time`, `InputCapture`, the monitors and `AppEvent`, with none of the frame systems behind them.
