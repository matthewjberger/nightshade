---
name: project_kit_crate
description: nightshade-kit holds the models, sounds and icons a game builds its world from, the catalog is written from the folders under assets/kits/models, and each kit is a cargo feature
type: project
---

`crates/framework/base/nightshade-kit` holds the catalog that a game builds its
world from: twenty-six Kenney kits, 109 sound clips, the interface icons and
the world map marks. `MODELS` gives each model its name, its `Kit` and its
bytes, `kit_textures` gives the pictures that kit paints with, `CLIPS`, `ICONS`
and `MAP_ICONS` give the rest, and `model(name)` gives the place a name holds
in the catalog.

The bytes live at `assets/kits/` at the root of the repository and reach a
binary through `include_bytes!`, so a game that runs in a browser carries its
world with it. The catalog is not written by hand: `build.rs` reads the folders
under `assets/kits/models/`, makes one `Kit` variant for each folder, one
`ModelFile` for each glTF file, and one texture list for each kit from every
picture under the folder, keyed as `Textures/<file>` because that is how a
Kenney model names its color map. A new kit is a new folder.

Each kit is a cargo feature named after its folder. The default set is the
eighteen kits the fantasy games draw from, so those games carry 19 MB and
nothing they do not name. A game that wants the city kits, the cars or another
kit outside the default set names the crate by path with `default-features =
false` and lists its kits, the way `apps/games/gridlock` does. The lookup is
by name alone and takes the first match, so a kit whose files share names with
another is copied under a prefix.

A game that wants a Kenney kit takes the crate. It does not copy the file.
