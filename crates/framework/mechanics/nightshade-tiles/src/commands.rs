use crate::components::{OpenTile, TileMap};
use crate::queries::{at, inside};
use nalgebra_glm::IVec2;
use nightshade_cells::Cells;
use nightshade_ecs::Storage;

pub fn answer_the_cells(cells: &mut Cells) {
    cells.walkable = Some(walkable);
    cells.in_bounds = Some(in_bounds);
}

fn walkable(storage: &Storage, cell: IVec2) -> bool {
    let Some((_entity, (map, open))) = storage.query_ref::<(&TileMap, &OpenTile)>().iter().next()
    else {
        return true;
    };
    at(map, cell) == open.0
}

fn in_bounds(storage: &Storage, cell: IVec2) -> bool {
    let Some((_entity, (map,))) = storage.query_ref::<(&TileMap,)>().iter().next() else {
        return true;
    };
    inside(map, cell)
}
