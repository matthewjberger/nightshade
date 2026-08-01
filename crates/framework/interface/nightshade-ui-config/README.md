# nightshade-ui-config

A settings panel built from a `nightshade-config` schema rather than laid out by hand. Each entry in the schema becomes the widget its kind calls for, a heading opens every new section, and a restore button closes the list. What a person changes in the panel is read back into `Config<S>` every frame. `apps/features/settings` is the live example.

## Composing it

```rust
app.add_plugin(nightshade_ui::UiPlugin::new());
app.add_plugin(nightshade_config::ConfigPlugin::<Panel>::new());
app.add_plugin(nightshade_ui_config::ConfigUiPlugin::<Panel>::new());
```

`ConfigUiPlugin` is generic over the schema and inserts the `ConfigPanel<S>` that holds the row entities. Its two systems, the one that reads the rows and the one that follows the restore button, run on `Stage::Update`.

## Using it

```rust
fn build_the_panel(
    config: Res<Config<Panel>>,
    mut rows: ResMut<ConfigPanel<Panel>>,
    mut build: UiBuild,
) {
    let mut tree = UiTreeBuilder::new(build.context());
    let panel = tree.add_docked_panel_left("settings", "Settings", 300.0);
    let content = tree.panel_content(panel).expect("panel content");
    tree.in_parent(content, |tree| {
        *rows = build_config_rows(tree, &config);
    });
    tree.finish();
}
```

`build_config_rows` walks `S::entries()` and adds a checkbox for a toggle, a slider for a number, a drag value for a whole number, a dropdown for a choice, a text input for text and a color picker for a color, then appends the button named by `RESET_LABEL`. It returns the `ConfigPanel<S>` a game stores in the resource. From then on the plugin reads each row with `row_value` and writes the result into `Config<S>`, so a system that wants a setting reads the config and never the widget.

## Driving it

```
just example ui_config
```

Drag the spin and lift sliders in the panel on the left, click the turning checkbox to stop and start the rotation, and pick a palette from the dropdown to retint the body. The reset button at the foot of the panel puts every value back to its default, and the readout gives what the config holds.
