# The retained interface

The interface is retained rather than immediate. A node is an entity, its
layout is a component, and the layout runs only when something says it is
dirty. This book is drawn by it.

## Building a tree

```rust
fn build_panel(mut build: UiBuild) {
    let mut tree = UiTreeBuilder::new(build.context());
    let panel = tree
        .add_node()
        .window(Ab(vec2(24.0, 24.0)), Ab(vec2(280.0, 160.0)), Anchor::TopLeft)
        .rect(8.0)
        .fg(ThemeColor::Panel)
        .flow(FlowDirection::Vertical, 12.0, 8.0)
        .entity();

    tree.in_parent(panel, |tree| {
        tree.add_node()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 28.0)))
            .with_text("Settings", 16.0)
            .text_left();
        tree.add_slider(0.0, 1.0, 0.5);
    });
}
```

`Ab` is absolute pixels and `Rl` is a percentage of the parent. The two add,
so `Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 28.0))` is the full width of the
parent at a fixed height.

## Positioning

Two ways to place a node, and they do not mix on the same node:

- `window(position, size, anchor)` puts it where you say
- `flow_child(size)` hands it to the parent's flow

A parent that flows lays its children out in order, with a padding and a gap
you give it. A child that wants to size itself to its own text asks for
`auto_size(AutoSizeMode::Width)`.

## Reading a widget

The widget components carry their own state, and a system reads it:

```rust
fn read_controls(
    buttons: Query<(&UiButtonData,)>,
    sliders: Query<(&UiSliderData,)>,
) {
    // ...
}
```

For a plain node with `with_interaction`, `UiNodeInteraction.clicked` is the
whole story. There is no callback, no event handler, and no closure held
across frames.

## Theming

Colours are roles rather than values. A node asks for `ThemeColor::Panel` and
gets whatever the current theme says panel is, so switching themes needs no
rebuild:

```rust
tree.add_node()
    .with_theme_color::<UiBase>(ThemeColor::Panel)
    .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
    .with_theme_border_color(ThemeColor::Border);
```

A theme is built from a palette, and a program installs its own by putting it
in `ThemeState.presets`. This book installs two, and the control in the top
bar switches between them.

## The dirty flag

The layout runs when `RetainedUiDirty.layout_dirty` is set. The builder sets
it for you. Writing a `UiLayoutNode` by hand does not:

```rust
if let Some(node) = storage.get_mut::<UiLayoutNode>(entity) {
    node.base_layout = new_layout;
}
dirty.layout_dirty = true;
```

Forget the second line and the node keeps the position it had, on every frame,
forever. That is the single easiest mistake to make in the whole interface,
and it fails silently.

## Where the world shows through

A node that paints nothing is a hole. The book points a camera at the hole and
lets the scene behind the interface reach the reader through it:

1. A stage node paints transparent and reports its rectangle
2. A system turns that rectangle into a camera tile
3. The scene renders to the tile, under the interface

That is how every figure in this book works, and it is why the figures are
scenes rather than images.
