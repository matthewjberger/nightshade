# nightshade-paint

`nightshade-paint` is the two dimensional draw list the retained UI and the backends share. It holds the rectangle, the image and the text mesh a frame wants on the screen, the instance layouts the GPU reads them as, and the slot bookkeeping that lets an entity keep the same instance between frames. It draws nothing itself, so whoever has something to put over the scene fills the list and whichever backend is composed empties it.

## Composing it

`PaintPlugin` installs `PaintFrame`, `PaintDrawList`, `RenderSlotAllocator` and `PaintTextCache`, and registers no system. It holds each one only if it is not there, so two composers do not fight over it. `nightshade-wgpu-paint` composes it for the wgpu backend and `nightshade-vulkan` composes it for its own, so a program that draws over the scene gets the resources from whichever backend it picked. `nightshade_ui` writes the frame through `ResMut<PaintFrame>` and composes nothing.

## Using it

```rust
fn paint_cursor(mut paint: ResMut<nightshade_paint::PaintFrame>, position: Vec2) {
    paint.rects.push(DrawRect {
        position,
        size: Vec2::new(18.0, 18.0),
        color: Vec4::new(1.0, 1.0, 1.0, 0.95),
        corner_radius: 9.0,
        border_width: 2.0,
        layer: u8::MAX,
        ..Default::default()
    });
    paint.rect_entities.push(None);
}
```

`PaintFrame` holds `rects`, `images`, `text_meshes` and the `image_uploads` waiting for a texture layer, with `rect_entities` running alongside the rectangles so a rectangle that belongs to an entity keeps its render slot. Every entry names a `layer` and a `z_index`, and `draw_order` and `layer_depth` turn that pair into the sort and the depth the backend writes. `DrawRect` carries a corner radius, a border, a rotation, an optional shadow and an optional clip rectangle, and a rectangle with `quad_corners` set is drawn as an arbitrary quadrilateral. `PaintDrawList` is the retained form the UI keeps between frames, with `RenderSlotAllocator` handing out and recycling the slots.

## Taking it from another crate

The crate does not split, because the whole of it is the contract. A crate that draws over the scene or reads the draw list declares it plainly.

```toml
nightshade-paint = { path = "../nightshade-paint" }
```
