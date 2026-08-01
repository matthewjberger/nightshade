# nightshade-topdown

The eight ways a body walks when the camera looks down on it. The crate turns the keyboard and the left stick into one wish vector, then moves every walking entity across the ground plane at its own share of the settings speed. The direction it last moved in stays on the entity, so whatever draws the body can turn it to face the way it went.

## Composing it

```rust
use nightshade::app::App;
use nightshade_topdown::{TopDownPlugin, TopDownSettings};

fn build(app: &mut App) {
    app.add_plugin(TopDownPlugin {
        settings: TopDownSettings { speed: 4.0 },
    });
}
```

The plugin inserts `TopDownSettings` and registers `walk` on `Stage::Update`, which reads the input and writes the transform and the heading of every walker. Nothing is announced, because the heading on the entity is what a game reads back.

## Using it

```rust
use nalgebra_glm::vec2;
use nightshade_topdown::queries::{stick, wish, wish_with_pad};
use nightshade_topdown::{Heading, Walking};

storage.set(walker, Walking(1.0));
storage.set(walker, Heading(vec2(0.0, 0.0)));

let wanted = wish(&input);
let padded = wish_with_pad(&input, &pad);
let left = stick(&pad);
```

`Walking` is the share of the settings speed this body moves at, so a zero holds it still and a larger number runs it. `Heading` is the last direction it moved, written every frame the wish is not empty and left alone when the input is quiet. `wish` reads WASD and the arrow keys into a normalized vector, `stick` reads the left stick and the direction pad through the gamepad contract, and `wish_with_pad` takes the keys when they say anything and falls through to the stick otherwise. The x of the wish moves the entity along world x and the y moves it along world z.

## Driving it

```
just example topdown
```

WASD or the arrow keys walk the block around a ring of posts, and holding Space raises its share of the speed so it runs. The readout gives the heading it last moved in and the share it is walking at.
