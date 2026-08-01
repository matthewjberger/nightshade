# nightshade-ui-test

`nightshade-ui-test` is the retained interface test driver. A developer schedules clicks, drags, typing and assertions against frame numbers, and the driver feeds them into the platform input each time it is stepped, so the interface sees them as a person's input. It collects the interface events that come back and records a pass or a fail for every assertion.

## Composing it

The crate carries no plugin. It is a driver a program holds and steps, so a program composes nothing and names the crate where it scripts.

```toml
nightshade-ui-test = { workspace = true }
```

A caller keeps the `UiTestDriver` in a resource of its own, usually as an `Option<UiTestDriver>` field, and calls `step` once a frame while it is active.

## Using it

```rust
let mut driver = UiTestDriver::new("book appearance");
driver.set_exit_on_complete(false);
driver.at_frame(OPEN).click_entity("appearance");
driver.wait(SETTLE).assert_visible("appearance-panel");
driver.at_frame(TAKE_DRACULA).click_entity(DRACULA);
driver.start();
driver.step(&mut ecs.storage, &mut ecs.resources);
```

`at_frame` moves the cursor to a frame number and `wait` advances it by a count, and each returns an `ActionBuilder` that chains actions onto that frame. The builder carries `move_mouse_to`, `click_entity`, `set_slider_value`, `select_dropdown_option`, `select_tab`, `type_text`, `scroll` and `log`, all addressing a widget by the test id `ui_set_test_id` gave it. Assertions sit on the same chain as `assert_visible`, `assert_not_visible`, `assert_value_f32`, `assert_value_bool` and the `assert_event_fired_` calls for each widget event. `start` sorts the scheduled actions and arms the driver, `step` runs one frame of them, and `all_passed` reports the outcome, with `set_json_output_path` writing the whole report to a file.

## Running it

```
just example ui_events
```

The example needs no input, because the driver is the one supplying it. It builds a small panel, clicks its button through the driver, and prints the events a scripted click raises against the silence a code-driven slider change leaves.
