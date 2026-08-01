# nightshade-sprite

Sprites are pictures packed into a named atlas and drawn on ordinary transform entities. A `Sprite`, a `SpriteAnimator` or a `Tilemap` beside a transform is gathered every frame into `SpriteFrame`, which both backends draw after the lit meshes, sorted by layer, then depth, then Y, and tested against the scene depth without writing it. The crate packs the atlases itself and names no GPU API.

## Composing it

```rust
app.add_plugin(nightshade_sprite::SpritePlugin)
    .add_plugin(SpritesPlugin)
    .run()
```

`SpritePlugin` inserts `SpriteFrame` and `SpriteAtlases`, registers `systems::animate::advance_sprite_animators` on `Stage::FrameUpdate`, and registers `systems::gather::gather_sprites_system` on `Stage::Render`.

## Using it

```rust
use nightshade_sprite::prelude::*;

sprite_atlas_create(atlases, theme::ATLAS, theme::ATLAS_SIZE, SpriteFilter::Nearest);

let tiles = sprite_atlas_insert(
    atlases,
    theme::ATLAS,
    theme::SHEET,
    &decoded.rgba,
    width,
    height,
)
.expect("the sheet fits the atlas");

let sprite = Sprite::new(theme::ATLAS, tiles.cell(columns, rows, index)).with_size(1.0, 1.0);
let plant = spawn_sprite(commands, sprite, position);
```

`sprite_atlas_create` names an atlas and its square size and picks the filter, and `sprite_atlas_insert` shelf-packs one RGBA picture into it and hands back the `SpriteRegion` it landed at. `SpriteRegion::cell` and `cells` cut a packed sheet into grid tiles, and `flipped` swaps the bounds. `Sprite` carries the atlas name, the region, the size, the anchor, the color, the flips, the layer, the billboard flag and the lean, each with a builder call. `SpriteAnimator::from_regions` or `from_grid` turns a run of regions into frames the animate system advances under `SpriteLoop::Once`, `Loop` or `PingPong`. `tile_trim` and `standing_tile` in `queries::sheet` cut a grid cell down to the paint inside it, which keeps an upright sprite on its feet.

## Taking it from another crate

The crate splits at a `contract` feature.

```toml
nightshade-sprite = { path = "../nightshade-sprite", default-features = false, features = ["contract"] }
```

The contract carries the components and the render data both backends read: `Sprite`, `SpriteAnimator`, `SpriteRegion`, `SpriteFilter`, `Tilemap`, `TileData`, `TileTrim`, and `SpriteFrame` with `SpriteInstance`, `SpriteAtlasSpec` and `SpriteAtlasUpload`. Both backends take it that way.
