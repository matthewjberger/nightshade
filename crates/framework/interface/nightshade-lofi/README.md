# nightshade-lofi

`nightshade-lofi` is one named look written over the render settings. A look gathers the render scale, the vertex snap, the affine texture mapping, the anisotropy, the anti-aliasing, the bloom, the ambient occlusion, the saturation, the contrast, the vignette and the tonemap into one value a game can name and swap. `playstation`, `handheld` and `clean` are the three it ships.

## Composing it

```rust
app.add_plugin(nightshade_lofi::LofiPlugin {
    look: nightshade_lofi::playstation(),
});
```

The plugin inserts `LofiSettings` holding that look, marks it waiting, and registers `apply_the_look` on `Stage::Update`. The system writes the whole look over `RenderSettings` on the first frame and then does nothing until another look is asked for, so a game is free to change a render setting by hand in between. It announces nothing.

## Using it

```rust
use nightshade_lofi::commands::{restore, wear};
use nightshade_lofi::queries::{scale, snapping, waiting};
use nightshade_lofi::{clean, handheld, playstation};

wear(&mut settings, playstation());
```

`wear` asks for a look and the plugin applies it on the next frame, so a game can change era while it runs. `restore` is `wear` with the clean look. A game that wants a look of its own builds a `LofiLook` and hands that to `wear` rather than picking one of the three. `waiting` reports that a look has not landed yet, and `scale` and `snapping` report the two fields a readout usually wants.

## Driving it

```
just example lofi
```

A scene stands under one of the three looks. The digit keys wear the clean, the PlayStation and the handheld look in turn, and the render scale, the vertex snap, the grading and the vignette all change together. The readout gives which look is being worn and whether it snaps its vertices.
