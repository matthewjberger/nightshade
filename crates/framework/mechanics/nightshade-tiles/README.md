# nightshade-tiles

A grid of bytes read from a picture of a level, and the walk across it. A level written as rows of text becomes a `TileMap` of one byte per cell, and the queries answer where a byte sits, whether a cell is on the board, and how far a flood reaches from a starting cell. The map also answers `nightshade-cells` on behalf of the game, so a mechanic that asks whether a cell is walkable gets the byte grid without naming it.

## Composing it

```rust
use nightshade_tiles::queries::read;
use nightshade_tiles::{OpenTile, TileMap};

commands.spawn((read(&["#####", "#   #", "#####"]), OpenTile(b' ')));
```

The crate carries no plugin and no system of its own, so a game spawns one entity carrying the `TileMap` it read and the `OpenTile` byte that counts as floor, then reads the grid from whichever stage asks. The map is a component rather than a resource, which is what lets the `nightshade-cells` answers read it from the rows. Nothing is announced, because the map is state a game reads rather than a thing that happens.

## Using it

```rust
use nalgebra_glm::IVec2;
use nightshade_tiles::commands::answer_the_cells;
use nightshade_tiles::queries::{at, cells_of, flood, inside, read};
use nightshade_tiles::TileMap;

let map: TileMap = read(&rows);
let byte = at(&map, IVec2::new(3, 2));
let on_the_board = inside(&map, IVec2::new(3, 2));
let doors = cells_of(&map, b'D');
let reached = flood(&map, doors[0], b' ');

answer_the_cells(&mut cells);
```

`read` pads every row out to the widest one, so a ragged level still gives a rectangular grid. `at` returns a space for a cell off the board, which keeps a walk from having to test `inside` first. `cells_of` gathers every cell carrying one byte, which is how a game finds its spawns and its doors, and `flood` walks the four neighbors from a cell and returns everything it could reach over one open byte. `answer_the_cells` fills the `walkable` and `in_bounds` pointers on a `Cells` table so a mechanic asks the grid through `nightshade-cells`.

## Driving it

```
just example tiles
```

The arrow keys walk the yellow cursor one cell at a time, and it refuses a step the grid answers as a wall through `nightshade-cells`. Space floods from the cell it stands on, which raises every tile the flood reaches, and R lowers them again. The readout gives the cell, the byte under it, and how many cells the last flood covered.
