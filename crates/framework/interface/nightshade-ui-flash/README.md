# nightshade-ui-flash

`nightshade-ui-flash` is the wash of color a screen takes after something happens, and the fade that takes it away. A game pushes a color and a fade rate, the crate sums every wash it is holding and paints the total over one full-bleed node. A red hit, a white pickup and a slow black-out can all be on screen at once.

## Composing it

```rust
app.add_plugin(nightshade_ui_flash::FlashPlugin);
```

The plugin inserts `Flash`, `FlashHandles` and the color it last painted, and registers `fade_the_flash` on `Stage::Update`. It builds no node of its own, because the game owns its interface tree, so nothing is painted until `add_flash` has put the node in and handed back the handle. The system fades every wash at its own rate, drops the ones that reach nothing, and writes the color only when the total has moved.

## Using it

```rust
use nightshade_ui_flash::commands::{add_flash, clear, flash};
use nightshade_ui_flash::queries::{fading, strength, wash};

*handles = add_flash(&mut tree);
flash(&mut screen, Vec4::new(1.0, 0.0, 0.0, 0.4), 2.0);
```

`add_flash` builds the full-bleed node, which takes no pointer events so it never eats a click, and returns the `FlashHandles` the game keeps in its own resource. `flash` pushes a color and the rate it fades at, and `clear` takes every wash away at once for a cut. `wash` sums what is showing and clamps each channel, `strength` gives the strongest single wash for a game that wants to size an effect from it, and `fading` reports whether anything is left.

## Driving it

```
just example ui_flash
```

The digit keys throw three different colors and fade rates over the screen, and holding them down stacks the washes so the total climbs. R clears the screen at once. The readout gives the color that is showing and whether anything is still fading.
