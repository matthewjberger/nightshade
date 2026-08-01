# nightshade-cells

The question a mechanic asks a discrete world about a cell. `Cells` is a table of function pointers the game fills, so a pathfinder or a build mode can ask whether a cell is walkable, what it costs to step onto, and whether it exists at all without knowing what kind of world answers. `CellSize` carries the width of one cell, which is what turns a world position into a cell and back.

## Composing it

This crate carries no plugin. A game names it in its own `Cargo.toml` and inserts both resources before anything asks them a question.

```rust
app.insert_resource(Cells::default());
app.insert_resource(CellSize(1.0));
```

A caller holds nothing. The answers live in the `Cells` resource and every query reads them out of the resource map it is handed.

## Using it

```rust
use nalgebra_glm::{IVec2, vec2};
use nightshade_cells::{CellSize, Cells};
use nightshade_cells::commands::{answer_in_bounds_with, answer_step_cost_with, answer_walkable_with};
use nightshade_cells::queries::{cell_of, center_of, in_bounds, manhattan, neighbours, step_cost, walkable};

answer_walkable_with(resources.res_mut::<Cells>(), walkable_here);
answer_step_cost_with(resources.res_mut::<Cells>(), cost_here);
answer_in_bounds_with(resources.res_mut::<Cells>(), inside_the_map);

let size = CellSize(1.0);
let here = cell_of(&size, vec2(translation.x, translation.z));
let middle = center_of(&size, here);
let open: Vec<IVec2> = neighbours(here).into_iter().filter(|cell| walkable(&cells, storage, *cell)).collect();
let cost = step_cost(&cells, storage, here);
let reach = manhattan(here, goal);
```

Each answer is a plain `fn(&Storage, IVec2)`, so the game's own function reads the rows that hold its map, and a caller names `Res<Cells>` rather than the whole resource map. An answer the game never sets has a default the queries fall back to: `walkable` and `in_bounds` answer true and `step_cost` answers one, which lets a mechanic run over an open field before the game fills anything in. `cell_of` and `center_of` convert between a flat world position and a cell using the size handed to them. `neighbours` returns the four cells sharing an edge, and `manhattan` is the step count between two cells with no diagonals.

## Driving it

```
just example cells
```

Each arrow key asks for one step onto the neighboring cell, and the walker moves only when the game's own answers call that cell in bounds and walkable, so it slides along the open lanes and stops dead against a block. Space plants the mark on the cell the walker stands in, and the readout gives the cell, how many of its four neighbors are open, what the cell costs to stand on and the manhattan distance back to the mark.
