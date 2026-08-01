pub const FLOOR_SPAN: i32 = 31;

pub const BARROW_SINK: i32 = 4;

pub const HOARD_LAYER: i32 = 2;

pub const WIGHT_CELL: [i32; 2] = [15, 4];

pub const KEPT_CELL: [i32; 2] = [9, 11];

pub const HOARD_CELL: [i32; 2] = [15, 2];

pub const MOUTH_CELL: [i32; 2] = [15, 30];

pub const WAITING_CELL: [i32; 2] = [10, 33];

pub const CAMP_HAUNTS: [[i32; 2]; 4] = [[6, 10], [14, 10], [10, 6], [10, 14]];

pub const CAMP_WIGHT: [i32; 2] = [10, 12];

pub const WATCHMAN_CELL: [i32; 2] = [17, 13];

pub const CAMP_SPAN: i32 = 21;

pub const HAUNT_CELLS: [[i32; 2]; 9] = [
    [3, 18],
    [4, 17],
    [26, 18],
    [27, 17],
    [12, 10],
    [18, 11],
    [20, 9],
    [16, 17],
    [13, 20],
];

pub const COUNTING_LINES: [&str; 5] = [
    "Somewhere ahead in the dark, something counts under its breath. One. Two.",
    "The counting stops. Then it starts again from one.",
    "Four. Five. A pause, and then six, softer, as if it was not sure.",
    "Stone grinds on stone further in. The counting does not stop for it.",
    "Your own footsteps come back to you a beat late, and there is one more than there should be.",
];

const fn cell_is_hollow(cell: [i32; 2]) -> bool {
    cell[0] > 0 && cell[0] < FLOOR_SPAN - 1 && cell[1] > 0 && cell[1] < FLOOR_SPAN
}

const fn every_cell_is_hollow() -> bool {
    if !cell_is_hollow(WIGHT_CELL) || !cell_is_hollow(KEPT_CELL) || !cell_is_hollow(HOARD_CELL) {
        return false;
    }
    let mut index = 0;
    while index < HAUNT_CELLS.len() {
        if !cell_is_hollow(HAUNT_CELLS[index]) {
            return false;
        }
        index += 1;
    }
    true
}

const _: () = assert!(every_cell_is_hollow());
