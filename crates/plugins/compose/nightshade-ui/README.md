# nightshade-ui

The retained user interface. A program builds a tree of nodes once, keeps the handles it cares about, and reads the widget state back off those entities every frame. Layout is computed with taffy, the theme colors and transitions blend over time, and the finished tree is published as a `PaintFrame` that a backend draws.

## Composing it

```rust
app.add_plugin(nightshade_text::TextPlugin)
    .add_plugin(nightshade_ui::UiPlugin::new())
    .add_plugin(nightshade_camera_controller::CameraControllerPlugin)
    .add_plugin(TextStylePlugin)
    .run()
```

`UiPlugin` inserts the retained interface resources, `RetainedUiRuntime`, `ThemeState`, `RetainedUiInteraction`, `RetainedUiFrame`, `UiViewport` and the rest, along with the `RetainedUiSchedule` that holds the widget systems. It registers `run_retained_ui_schedule` on `Stage::FramePostUpdate`, ordered after the scene's transform propagation, and `systems::render_sync::publish_frame` on `Stage::Last`. `UiPlugin::with_system` adds a system of the program's own at the `Interaction`, `PreLayout` or `PreDraw` phase.

## Using it

```rust
let mut tree = UiTreeBuilder::new(context);
let panel = tree.add_floating_panel("text_style", "Text style", theme::PANEL_RECT);
let content = tree.panel_content(panel).expect("panel content");
tree.in_parent(content, |tree| {
    tree.add_label("Outline width, in pixels");
    handles.width_slider = tree.add_slider(0.0, theme::WIDEST_OUTLINE, style.outline_width);
    handles.billboard_toggle = tree.add_checkbox("Title faces the camera", style.billboard);
});
tree.finish();
```

A build system takes a `UiBuild` parameter and hands its `context()` to `UiTreeBuilder::new`. Each `add_` call returns the `Entity` of the node, which the program stores in a plain resources struct so a later system can read the widget back. That reading system takes an ordinary query, for example `Query<(Option<&UiSliderData>, Option<&UiCheckboxData>)>`, and looks up the stored handle. `ui_wants_pointer` and `ui_wants_keyboard` report whether the tree has claimed the input this frame, and the plugin already forwards both claims to the platform's `InputCapture`.

## Taking it from another crate

The crate does not split at a `contract` feature. A crate that draws a panel takes it whole and composes `UiPlugin` alongside, which is what the interface crates under `crates/framework/interface/` do.
