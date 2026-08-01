use crate::components::Needs;
use crate::data::{self, Need};
use crate::resources::{Aim, Grid, Lot};
use nightshade::prelude::Vec3;
use petgraph::graphmap::UnGraphMap;
use petgraph::visit::EdgeRef;

pub fn cell_index(column: usize, row: usize) -> usize {
    row * data::COLUMNS + column
}

pub fn cell_column_row(index: usize) -> (usize, usize) {
    (index % data::COLUMNS, index / data::COLUMNS)
}

pub fn cell_center(index: usize) -> Vec3 {
    let (column, row) = cell_column_row(index);
    Vec3::new(
        (column as f32 + 0.5) * data::GRID - data::LOT_HALF_WIDTH,
        0.0,
        (row as f32 + 0.5) * data::GRID - data::LOT_HALF_DEPTH,
    )
}

pub fn north_center(index: usize) -> Vec3 {
    let column = index % data::COLUMNS;
    let row = index / data::COLUMNS;
    Vec3::new(
        (column as f32 + 0.5) * data::GRID - data::LOT_HALF_WIDTH,
        0.0,
        row as f32 * data::GRID - data::LOT_HALF_DEPTH,
    )
}

pub fn west_center(index: usize) -> Vec3 {
    let column = index % (data::COLUMNS + 1);
    let row = index / (data::COLUMNS + 1);
    Vec3::new(
        column as f32 * data::GRID - data::LOT_HALF_WIDTH,
        0.0,
        (row as f32 + 0.5) * data::GRID - data::LOT_HALF_DEPTH,
    )
}

pub fn aim_position(aim: Aim) -> Option<Vec3> {
    match aim {
        Aim::Nowhere => None,
        Aim::Cell(index) => Some(cell_center(index)),
        Aim::North(index) => Some(north_center(index)),
        Aim::West(index) => Some(west_center(index)),
    }
}

pub fn piece_yaw(aim: Aim, placement: data::Placement, steps: u8) -> f32 {
    match placement {
        data::Placement::Edge => match aim {
            Aim::North(_) => std::f32::consts::FRAC_PI_2,
            _ => 0.0,
        },
        _ => steps as f32 * std::f32::consts::FRAC_PI_2,
    }
}

pub fn point_to_cell(point: Vec3) -> Option<usize> {
    let column = (point.x + data::LOT_HALF_WIDTH) / data::GRID;
    let row = (point.z + data::LOT_HALF_DEPTH) / data::GRID;
    if column < 0.0 || row < 0.0 {
        return None;
    }
    let column = column.floor() as usize;
    let row = row.floor() as usize;
    if column >= data::COLUMNS || row >= data::ROWS {
        return None;
    }
    Some(cell_index(column, row))
}

pub fn point_to_edge(point: Vec3) -> Option<Aim> {
    let column_exact = (point.x + data::LOT_HALF_WIDTH) / data::GRID;
    let row_exact = (point.z + data::LOT_HALF_DEPTH) / data::GRID;
    if column_exact < 0.0 || row_exact < 0.0 {
        return None;
    }
    let column = column_exact.floor() as usize;
    let row = row_exact.floor() as usize;
    if column >= data::COLUMNS || row >= data::ROWS {
        return None;
    }

    let inside_x = column_exact - column as f32;
    let inside_z = row_exact - row as f32;
    let to_west = inside_x;
    let to_east = 1.0 - inside_x;
    let to_north = inside_z;
    let to_south = 1.0 - inside_z;
    let nearest = to_west.min(to_east).min(to_north).min(to_south);

    if nearest == to_north {
        Some(Aim::North(row * data::COLUMNS + column))
    } else if nearest == to_south {
        Some(Aim::North((row + 1) * data::COLUMNS + column))
    } else if nearest == to_west {
        Some(Aim::West(row * (data::COLUMNS + 1) + column))
    } else {
        Some(Aim::West(row * (data::COLUMNS + 1) + column + 1))
    }
}

pub fn cell_is_free(grid: &Grid, index: usize) -> bool {
    grid.cells[index].is_none()
}

pub fn cell_walkable(grid: &Grid, index: usize) -> bool {
    grid.cells[index].is_none_or(|piece| !data::CATALOG[piece].blocks)
}

pub fn edge_is_free(grid: &Grid, aim: Aim) -> bool {
    match aim {
        Aim::North(index) => grid.north[index].is_none(),
        Aim::West(index) => grid.west[index].is_none(),
        _ => false,
    }
}

fn edge_blocks(slot: Option<usize>) -> bool {
    slot.is_some_and(|piece| data::CATALOG[piece].blocks)
}

pub fn step_is_open(grid: &Grid, from: usize, to: usize) -> bool {
    let (from_column, from_row) = cell_column_row(from);
    let (to_column, to_row) = cell_column_row(to);
    if from_column == to_column && to_row == from_row + 1 {
        return !edge_blocks(grid.north[to_row * data::COLUMNS + to_column]);
    }
    if from_column == to_column && from_row == to_row + 1 {
        return !edge_blocks(grid.north[from_row * data::COLUMNS + from_column]);
    }
    if from_row == to_row && to_column == from_column + 1 {
        return !edge_blocks(grid.west[to_row * (data::COLUMNS + 1) + to_column]);
    }
    if from_row == to_row && from_column == to_column + 1 {
        return !edge_blocks(grid.west[from_row * (data::COLUMNS + 1) + from_column]);
    }
    false
}

fn neighbours(index: usize) -> Vec<usize> {
    let (column, row) = cell_column_row(index);
    let mut found = Vec::with_capacity(4);
    if column + 1 < data::COLUMNS {
        found.push(cell_index(column + 1, row));
    }
    if column > 0 {
        found.push(cell_index(column - 1, row));
    }
    if row + 1 < data::ROWS {
        found.push(cell_index(column, row + 1));
    }
    if row > 0 {
        found.push(cell_index(column, row - 1));
    }
    found
}

fn manhattan(from: usize, to: usize) -> u32 {
    let (from_column, from_row) = cell_column_row(from);
    let (to_column, to_row) = cell_column_row(to);
    from_column.abs_diff(to_column) as u32 + from_row.abs_diff(to_row) as u32
}

pub fn find_route(grid: &Grid, start: usize, goal: usize) -> Option<Vec<usize>> {
    if start == goal {
        return Some(vec![start]);
    }
    let open = |index: usize| index == goal || index == start || cell_walkable(grid, index);

    let mut graph: UnGraphMap<usize, u32> = UnGraphMap::new();
    for index in 0..data::COLUMNS * data::ROWS {
        if !open(index) {
            continue;
        }
        for neighbour in neighbours(index) {
            if !open(neighbour) || !step_is_open(grid, index, neighbour) {
                continue;
            }
            graph.add_edge(index, neighbour, 1);
        }
    }
    if !graph.contains_node(start) || !graph.contains_node(goal) {
        return None;
    }

    petgraph::algo::astar(
        &graph,
        start,
        |index| index == goal,
        |edge| *edge.weight(),
        |index| manhattan(index, goal),
    )
    .map(|(_, route)| route)
}

pub fn needs_by_urgency(needs: &Needs) -> [Need; 4] {
    let mut order = data::NEEDS;
    order.sort_by(|first, second| {
        let left = needs.0[data::need_index(*first)];
        let right = needs.0[data::need_index(*second)];
        left.total_cmp(&right)
    });
    order
}

pub fn lowest_need(needs: &Needs) -> Need {
    needs_by_urgency(needs)[0]
}

pub fn content(needs: &Needs) -> bool {
    needs.0.iter().all(|level| *level >= data::CONTENT_LINE)
}

pub fn wage_rate(needs: &Needs, indoors: bool) -> f32 {
    let base = if content(needs) {
        data::WAGE_CONTENT
    } else {
        data::WAGE_STRAINED
    };
    if indoors {
        base * (1.0 + data::INDOOR_BONUS)
    } else {
        base
    }
}

pub fn slots(lot: &Lot) -> Vec<usize> {
    data::CATALOG
        .iter()
        .enumerate()
        .filter(|(_, piece)| piece.category == lot.category)
        .map(|(index, _)| index)
        .collect()
}

pub fn palette_columns() -> usize {
    data::CATEGORIES
        .iter()
        .map(|category| {
            data::CATALOG
                .iter()
                .filter(|piece| piece.category == *category)
                .count()
        })
        .max()
        .unwrap_or(1)
}

pub fn palette_width() -> f32 {
    let columns = palette_columns() as f32;
    columns * data::PALETTE_SLOT_WIDTH + (columns + 1.0) * data::PALETTE_GAP
}

pub fn affordable(lot: &Lot, index: usize) -> bool {
    data::CATALOG
        .get(index)
        .is_some_and(|piece| lot.money >= piece.price as f32)
}

pub fn facing_toward(from: Vec3, to: Vec3) -> f32 {
    let mut apart = to - from;
    apart.y = 0.0;
    if apart.magnitude() < 1.0e-4 {
        return 0.0;
    }
    apart.x.atan2(apart.z)
}
