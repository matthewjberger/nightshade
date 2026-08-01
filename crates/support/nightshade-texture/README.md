# nightshade-texture

Painted textures generated on the processor. Each function takes a side length in texels and a handful of colors, and returns the RGBA8 bytes of a square picture a program can hand to the texture loader. The crate holds no state and names no GPU API, so a game builds the surfaces of its world without shipping image files.

## Composing it

The crate carries no plugin. A program calls its functions directly from a startup system or a helper in its own `commands` band.

## Using it

```rust
use nightshade_texture::prelude::*;

let pixels = speckled(
    theme::BLOCK_TEXELS,
    theme::GRASS_COLOR,
    theme::GRASS_DARK,
    theme::GRASS_LIGHT,
    11,
    22,
    12,
);
```

`Rgba` is the color type, a `[u8; 4]`. `speckled` scatters a dark share and a light share of texels over a base color from a seed, and `cobbles` and `stone_wall` lay stones with mortar between them. `planks`, `furrows` and `door` paint wood and worked ground, `shingles` lays a roof in courses, `window` and `pane` draw framed glass, and `radial_blob` fades a color out to transparent from the middle. `paint` is the loop underneath all of them, taking a side length and a closure that answers with the color at a texel, so a program writes its own pattern the same way.

## Taking it from another crate

The crate does not split at a `contract` feature, because the whole crate is plain values and functions with no world, no system and no engine dependency. A caller takes it whole.
