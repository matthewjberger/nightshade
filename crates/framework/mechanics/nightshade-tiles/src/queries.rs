use crate::components::TileMap;
use nalgebra_glm::IVec2;
use std::collections::VecDeque;

pub fn at(map: &TileMap, cell: IVec2) -> u8 {
    if !inside(map, cell) {
        return b' ';
    }
    map.bytes[cell.y as usize * map.width + cell.x as usize]
}

pub fn inside(map: &TileMap, cell: IVec2) -> bool {
    cell.x >= 0 && cell.y >= 0 && (cell.x as usize) < map.width && (cell.y as usize) < map.height
}

pub fn cells_of(map: &TileMap, byte: u8) -> Vec<IVec2> {
    let mut found = Vec::new();
    for down in 0..map.height {
        for across in 0..map.width {
            if map.bytes[down * map.width + across] == byte {
                found.push(IVec2::new(across as i32, down as i32));
            }
        }
    }
    found
}

pub fn flood(map: &TileMap, from: IVec2, open: u8) -> Vec<IVec2> {
    let mut seen = vec![false; map.width * map.height];
    let mut queue = VecDeque::from([from]);
    let mut reached = Vec::new();
    while let Some(cell) = queue.pop_front() {
        if !inside(map, cell) {
            continue;
        }
        let index = cell.y as usize * map.width + cell.x as usize;
        if seen[index] || map.bytes[index] != open {
            continue;
        }
        seen[index] = true;
        reached.push(cell);
        queue.push_back(IVec2::new(cell.x + 1, cell.y));
        queue.push_back(IVec2::new(cell.x - 1, cell.y));
        queue.push_back(IVec2::new(cell.x, cell.y + 1));
        queue.push_back(IVec2::new(cell.x, cell.y - 1));
    }
    reached
}

pub fn read(rows: &[&str]) -> TileMap {
    let width = rows.iter().map(|row| row.len()).max().unwrap_or(0);
    let height = rows.len();
    let mut bytes = vec![b' '; width * height];
    for (down, row) in rows.iter().enumerate() {
        for (across, byte) in row.bytes().enumerate() {
            bytes[down * width + across] = byte;
        }
    }
    TileMap {
        width,
        height,
        bytes,
    }
}
