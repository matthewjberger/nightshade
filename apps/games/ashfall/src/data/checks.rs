use crate::data::level::{
    BOLTED, BONE_DOOR, BONE_HAFT, BONE_WARD, BRASS_DOOR, CASE, CRAWLER, EXIT, FACADE, HEIGHT,
    LOCKBOX, MAP, NOTHING, PARTITION, PLAYER, SCREAMER, STAIR, UPPER, WANDERER, WARD_DOOR, WIDTH,
    WRETCH,
};
use crate::data::notes::NOTE_SITES;
use crate::data::people::PEOPLE;
use crate::data::props::{EXAMINE_SITES, LANTERNS};

const FLOOR: usize = WIDTH * HEIGHT;
const CELLS: usize = FLOOR * 2;
const BRASS: u8 = 1;
const WARD: u8 = 2;
const BONE: u8 = 4;
const STEPS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
const QUIET_STEPS: u16 = 24;
const UNREACHED: u16 = u16::MAX;

const fn cell(col: i32, row: i32, upper: bool) -> usize {
    let flat = row as usize * WIDTH + col as usize;
    if upper { FLOOR + flat } else { flat }
}

const fn inside(col: i32, row: i32) -> bool {
    col >= 0 && row >= 0 && col < WIDTH as i32 && row < HEIGHT as i32
}

const fn glyph(col: i32, row: i32, upper: bool) -> u8 {
    if !inside(col, row) {
        return NOTHING;
    }
    let line = if upper {
        UPPER[row as usize]
    } else {
        MAP[row as usize]
    };
    line.as_bytes()[col as usize]
}

const fn passable(here: u8, keys: u8) -> bool {
    match here {
        FACADE | PARTITION | NOTHING | BOLTED => false,
        BRASS_DOOR => keys & BRASS != 0,
        WARD_DOOR => keys & WARD != 0,
        BONE_DOOR => keys & BONE != 0,
        _ => true,
    }
}

const fn stair_beyond(col: i32, row: i32, step: (i32, i32)) -> bool {
    glyph(col + step.0, row + step.1, false) == STAIR
        && glyph(col + step.0 * 2, row + step.1 * 2, false) == STAIR
}

const fn waking_place() -> (i32, i32) {
    let mut row = 0;
    while row < HEIGHT as i32 {
        let mut col = 0;
        while col < WIDTH as i32 {
            if glyph(col, row, false) == PLAYER {
                return (col, row);
            }
            col += 1;
        }
        row += 1;
    }
    panic!("the town has nowhere to wake up");
}

const fn walk(keys: u8) -> [u16; CELLS] {
    let mut steps = [UNREACHED; CELLS];
    let mut queue = [0usize; CELLS];
    let (start_col, start_row) = waking_place();
    let start = cell(start_col, start_row, false);
    steps[start] = 0;
    queue[0] = start;
    let mut head = 0;
    let mut tail = 1;

    while head < tail {
        let at = queue[head];
        head += 1;
        let upper = at >= FLOOR;
        let flat = at % FLOOR;
        let col = (flat % WIDTH) as i32;
        let row = (flat / WIDTH) as i32;
        let here = steps[at];

        let mut index = 0;
        while index < STEPS.len() {
            let step = STEPS[index];
            index += 1;
            let (next_col, next_row) = (col + step.0, row + step.1);
            if !inside(next_col, next_row) {
                continue;
            }
            if passable(glyph(next_col, next_row, upper), keys) {
                let next = cell(next_col, next_row, upper);
                if steps[next] == UNREACHED {
                    steps[next] = here + 1;
                    queue[tail] = next;
                    tail += 1;
                }
            }
            if !stair_beyond(col, row, step) {
                continue;
            }
            let (top_col, top_row) = (col + step.0 * 3, row + step.1 * 3);
            if !inside(top_col, top_row) {
                continue;
            }
            if passable(glyph(top_col, top_row, !upper), keys) {
                let next = cell(top_col, top_row, !upper);
                if steps[next] == UNREACHED {
                    steps[next] = here + 1;
                    queue[tail] = next;
                    tail += 1;
                }
            }
        }
    }
    steps
}

const fn found(steps: &[u16; CELLS], wanted: u8, upper: bool) -> bool {
    let mut row = 0;
    while row < HEIGHT as i32 {
        let mut col = 0;
        while col < WIDTH as i32 {
            if glyph(col, row, upper) == wanted && steps[cell(col, row, upper)] != UNREACHED {
                return true;
            }
            col += 1;
        }
        row += 1;
    }
    false
}

const fn nearest_husk(steps: &[u16; CELLS]) -> u16 {
    let mut nearest = UNREACHED;
    let mut index = 0;
    while index < CELLS {
        let upper = index >= FLOOR;
        let flat = index % FLOOR;
        let col = (flat % WIDTH) as i32;
        let row = (flat / WIDTH) as i32;
        let here = glyph(col, row, upper);
        let is_a_husk = here == WANDERER || here == CRAWLER || here == WRETCH || here == SCREAMER;
        if is_a_husk && steps[index] < nearest {
            nearest = steps[index];
        }
        index += 1;
    }
    nearest
}

const _: () = {
    let nothing = walk(0);
    assert!(
        found(&nothing, CASE, false),
        "the evidence case cannot be reached with empty pockets"
    );
    assert!(
        !found(&nothing, EXIT, false),
        "the gate stands open to anybody who walks in"
    );
    assert!(
        !found(&nothing, BONE_HAFT, true),
        "the bone haft lies where no key is needed"
    );
    let brass = walk(BRASS);
    assert!(
        found(&brass, LOCKBOX, false),
        "the dispensary lockbox cannot be reached with the brass key"
    );
    let ward = walk(BRASS | WARD);
    assert!(
        found(&ward, BONE_HAFT, true),
        "the bone haft cannot be reached with the ward key"
    );
    assert!(
        found(&ward, BONE_WARD, false),
        "the bone ward cannot be reached with the ward key"
    );
    let bone = walk(BRASS | WARD | BONE);
    assert!(
        found(&bone, EXIT, false),
        "the gate cannot be reached with every key"
    );
    assert!(
        nearest_husk(&bone) >= QUIET_STEPS,
        "a husk stands too near the waking place"
    );

    let mut index = 0;
    while index < NOTE_SITES.len() {
        let site = &NOTE_SITES[index];
        assert!(
            bone[cell(site.col, site.row, site.upper)] != UNREACHED,
            "a note lies where nobody can reach it"
        );
        index += 1;
    }
    let mut index = 0;
    while index < EXAMINE_SITES.len() {
        let site = &EXAMINE_SITES[index];
        assert!(
            bone[cell(site.col, site.row, site.upper)] != UNREACHED,
            "something to look at stands where nobody can reach it"
        );
        index += 1;
    }
    let mut index = 0;
    while index < PEOPLE.len() {
        let who = &PEOPLE[index];
        assert!(
            bone[cell(who.col, who.row, who.upper)] != UNREACHED,
            "somebody is standing where nobody can reach them"
        );
        index += 1;
    }
    let mut index = 0;
    while index < LANTERNS.len() {
        let (col, row) = LANTERNS[index];
        assert!(
            bone[cell(col, row, false)] != UNREACHED,
            "a lantern hangs where nobody can reach it"
        );
        index += 1;
    }
};
