# nightshade-kit

The models, sounds and icons that a game builds its world from. The crate holds
one catalog of Kenney kits, 109 sound clips, and the icon sheets a head-up
display and a world map draw from. A game takes the crate and names what it
wants. The bytes sit in `assets/kits/` at the root of the repository, and each
one reaches the binary through `include_bytes!`, so a game that runs in a
browser carries its world with it.

## The catalog

Each folder under `assets/kits/models/` is a kit, and the build script writes
the catalog from the folders: the `Kit` enum has one variant for each folder,
`MODELS` has one `ModelFile` for each glTF file in the kits the build enables,
and `KIT_TEXTURES` has the pictures each kit paints with, keyed the way a
model file names them. A new kit is a new folder and nothing else.

Each kit is a cargo feature named after its folder, and the default set is the
eighteen kits the fantasy games draw from. A game that wants a kit outside that
set names the crate by path with `default-features = false` and lists the kits
it wants, so a binary carries only the kits it names.

```toml
nightshade-kit = { path = "../../../crates/framework/base/nightshade-kit", default-features = false, features = ["car", "roads"] }
```

A model's name is its file name, and the lookup is by name alone, so a kit
whose files share names with another kit is copied under a prefix, the way the
city kits carry `commercial-`, `industrial-`, `suburban-` and `car-`.

## Composing it

The crate holds no plugin and no system. A game reads the tables where it wants
them.

```rust
use nightshade_kit::prelude::*;

let file = &MODELS[model("tree-pine")];
let textures = kit_textures(file.kit);
```

## Using it

`MODELS` is the catalog. Each `ModelFile` carries the name a game asks for, the
`Kit` it belongs to, and the bytes of the glTF file. `kit_textures` gives the
color map that kit paints with, which the model names but does not carry.

```rust
use nightshade_kit::prelude::*;
use nightshade_load::load_model_with_files;

let file = &MODELS[model("barrel")];
let handle = load_model_with_files(
    &mut loading,
    file.bytes.to_vec(),
    kit_textures(file.kit)
        .iter()
        .map(|(path, bytes)| ((*path).to_string(), bytes.to_vec())),
);
```

`CLIPS` gives each sound by name and bytes, `ICONS` gives the interface pictures,
and `MAP_ICONS` gives the marks a world map puts on its chart.

## The example

`just example kit` lays a village out of the town kit and stands it up: a paved
street with a market square, a fountain and stalls at the middle of it, houses
of four walls and a roof with their doors to the street, fenced gardens,
lanterns along the curb, a windmill and a watermill at the ends, and trees and
rocks beyond the last plot.

The example holds no tuning constant. What a person turns is declared through
`nightshade-config`, so the panel carries the seed, the number of houses, the
reach of the street, the size of the square, the spacing of the plots, the
number of trees and rocks, and the color of the grass. Every change lays the
village again. The pieces come out of `MODELS` by the shape of their names, so
the doors, the windows, the stalls, the trees and the rocks are whatever the
kit carries.
