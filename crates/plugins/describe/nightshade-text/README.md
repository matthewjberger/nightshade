# nightshade-text

Text is glyphs turned into meshes. The crate keeps the strings in one cache, shapes them with a font engine, and puts a `Text` component on an entity so a string draws where that entity stands. A `Text` row with a transform draws in the world, and the same component set to billboard turns to face the camera.

## Composing it

```rust
app.add_plugin(nightshade_text::TextPlugin)
    .add_plugin(nightshade_ui::UiPlugin::new())
    .add_plugin(nightshade_camera_controller::CameraControllerPlugin)
    .add_plugin(TextStylePlugin)
    .run()
```

`TextPlugin` inserts `TextSyncState` and registers `systems::sync::mark_text_sync_dirty_system` and `systems::sync::render_sync_text_system` on `Stage::Render`.

## Using it

```rust
let properties = TextProperties {
    font_size: 96.0,
    color: theme::TEXT_COLOR,
    alignment: TextAlignment::Center,
    vertical_alignment: VerticalAlignment::Middle,
    outline_width: style.outline_width,
    outline_color: style.outline_color,
    smoothing: style.smoothing,
    ..Default::default()
};

let title = spawn_3d_billboard_text_with_properties(
    &mut text_state,
    &mut commands,
    "Nightshade",
    Vec3::new(0.0, 2.2, 0.0),
    properties,
);
```

A system takes `ResMut<TextState>` and `Commands`, and the spawn command puts the string in `text_state.cache` and spawns a row carrying `Text`, a `LocalTransform`, a `GlobalTransform`, a `Name` and a `Visibility`. `spawn_3d_text_with_properties` is the same call for text that stays where it is placed, and `spawn_ui_text` and `spawn_ui_text_with_properties` spawn a bare `Text` row for a screen-space string. `TextProperties` carries the font size, the color, the two alignments, the outline width and color and the edge smoothing. `TextCache::set_text` replaces the string in a slot an entity already points at, so the mesh rebuilds without a new spawn.

## Taking it from another crate

The crate splits at a `contract` feature.

```toml
nightshade-text = { path = "../nightshade-text", default-features = false, features = ["contract"] }
```

The contract carries `Text`, `TextProperties`, `TextCharacterColors`, `TextCharacterBackgroundColors`, `TextState`, `TextCache`, `FontEngine`, `TextMesh`, `TextVertex` and `RenderText`, which is what a backend or a user interface needs to read the glyph meshes and the string cache without linking the sync systems.
