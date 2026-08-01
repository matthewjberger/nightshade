use crate::contract::{Cue, Dimension, Glyph};
use nightshade::prelude::Vec3;

pub const TITLE: &str = "Fourfold";
pub const MENU_WIDTH: f32 = 470.0;
pub const TITLE_HEIGHT: f32 = 286.0;
pub const PAUSE_HEIGHT: f32 = 220.0;

pub const STONE: &str = "fourfold_stone";
pub const WALL: &str = "fourfold_wall";
pub const PILLAR: &str = "fourfold_pillar";
pub const SEAL: &str = "fourfold_seal";
pub const SHUT_ARCH: &str = "fourfold_shut_arch";
pub const PLATE: &str = "fourfold_plate";
pub const PIP_LIT: &str = "fourfold_pip_lit";
pub const GLASS_GROUND: &str = "fourfold_glass_ground";
pub const GLASS_LEDGE: &str = "fourfold_glass_ledge";
pub const GLASS_WALL: &str = "fourfold_glass_wall";
pub const RUNNER: &str = "fourfold_runner";
pub const BONE: &str = "fourfold_bone";
pub const AMBER: &str = "fourfold_amber";
pub const AZURE: &str = "fourfold_azure";
pub const MOSS: &str = "fourfold_moss";
pub const PORTAL_MARK: &str = "fourfold_portal_mark";

pub const STONE_COLOR: [f32; 4] = [0.19, 0.19, 0.23, 1.0];
pub const WALL_COLOR: [f32; 4] = [0.32, 0.31, 0.36, 1.0];
pub const PILLAR_COLOR: [f32; 4] = [0.34, 0.33, 0.38, 1.0];
pub const SEAL_COLOR: [f32; 4] = [0.78, 0.20, 0.22, 1.0];
pub const SHUT_COLOR: [f32; 4] = [0.92, 0.62, 0.22, 1.0];
pub const PLATE_COLOR: [f32; 4] = [0.10, 0.10, 0.13, 1.0];
pub const PIP_COLOR: [f32; 4] = [0.96, 0.92, 0.80, 1.0];
pub const GLASS_GROUND_COLOR: [f32; 4] = [0.14, 0.17, 0.24, 1.0];
pub const GLASS_LEDGE_COLOR: [f32; 4] = [0.30, 0.52, 0.66, 1.0];
pub const GLASS_WALL_COLOR: [f32; 4] = [0.07, 0.09, 0.15, 1.0];
pub const RUNNER_COLOR: [f32; 4] = [0.98, 0.86, 0.52, 1.0];
pub const RUNNER_SIZE: Vec3 = Vec3::new(0.68, 1.8, 0.68);

pub const BONE_COLOR: [f32; 4] = [0.93, 0.91, 0.85, 1.0];
pub const AMBER_COLOR: [f32; 4] = [1.00, 0.60, 0.16, 1.0];
pub const AZURE_COLOR: [f32; 4] = [0.30, 0.66, 1.00, 1.0];
pub const MOSS_COLOR: [f32; 4] = [0.44, 0.78, 0.36, 1.0];
pub const PORTAL_RIM: [f32; 4] = [0.60, 0.82, 0.92, 1.0];

pub const HUB_FOG: [f32; 3] = [0.05, 0.05, 0.07];
pub const GLASS_FOG: [f32; 3] = [0.06, 0.08, 0.13];
pub const LOOM_FOG: [f32; 3] = [0.10, 0.09, 0.07];
pub const FOG_START: f32 = 14.0;
pub const FOG_END: f32 = 46.0;
pub const LOOM_FOG_START: f32 = 20.0;
pub const LOOM_FOG_END: f32 = 90.0;
pub const CAIRN_FOG_START: f32 = 18.0;
pub const CAIRN_FOG_END: f32 = 70.0;
pub const HEARTH_FOG_START: f32 = 16.0;
pub const HEARTH_FOG_END: f32 = 60.0;

pub const HUB_START: Vec3 = Vec3::new(0.0, 0.0, 6.0);
pub const HUB_HALF: f32 = 12.0;
pub const HUB_FLOOR: f32 = -10.0;
pub const WALL_HEIGHT: f32 = 6.0;
pub const ARCH_WIDTH: f32 = 3.0;
pub const ARCH_HEIGHT: f32 = 4.0;
pub const PILLAR_COUNT: usize = 8;
pub const PILLAR_RING: f32 = 8.0;
pub const PORTAL_LIFT: f32 = 2.0;
pub const SHAFT_WIDTH: f32 = 4.0;
pub const SHAFT_MOUTH: f32 = -2.0;
pub const ENDING_HOLD: f32 = 11.0;
pub const PORTAL_INSET: f32 = 0.45;
pub const PORTAL_TARGET_MAX: u32 = 1600;
pub const PORTAL_PLANE_BIAS: f32 = 0.05;
pub const GLASS_WINDOW: Vec3 = Vec3::new(515.0, 2.0, 3.0);
pub const LOOM_WINDOW: Vec3 = Vec3::new(0.0, 2.0, 523.0);
pub const CAIRN_WINDOW: Vec3 = Vec3::new(0.0, 2.6, 1036.0);
pub const HEARTH_WINDOW: Vec3 = Vec3::new(1024.0, 2.6, 13.0);

pub const WALK_SPEED: f32 = 5.4;
pub const EYE_HEIGHT: f32 = 1.62;
pub const BODY_RADIUS: f32 = 0.34;
pub const BODY_HEIGHT: f32 = 1.8;
pub const MOUSE_SENSITIVITY: f32 = 0.0022;
pub const PITCH_LIMIT: f32 = 1.5;
pub const HUB_FIELD_OF_VIEW: f32 = 1.16;
pub const NEAR_PLANE: f32 = 0.05;
pub const FAR_PLANE: f32 = 220.0;

pub const GLASS_START: Vec3 = Vec3::new(515.0, 0.0, 0.0);
pub const GLASS_ARMED_AT: f32 = 520.0;
pub const GLASS_GATE: Vec3 = Vec3::new(512.2, 2.0, 0.0);
pub const GLASS_GATE_SIZE: Vec3 = Vec3::new(2.0, 4.0, 4.0);
pub const RUN_SPEED: f32 = 7.2;
pub const RUN_JUMP: f32 = 11.2;
pub const RUN_GRAVITY: f32 = 26.0;
pub const AIR_CONTROL: f32 = 0.68;
pub const GLASS_Y_MAG: f32 = 8.0;
pub const GLASS_CAMERA_Z: f32 = 18.0;
pub const GLASS_CAMERA_LIFT: f32 = 2.4;
pub const GLASS_CATCH: f32 = 7.0;
pub const GLASS_FLOOR: f32 = -8.0;

pub const GRAVITY: f32 = 26.0;
pub const JUMP_SPEED: f32 = 8.8;
pub const TERMINAL_FALL: f32 = 44.0;
pub const STEP_LIMIT: f32 = 0.16;
pub const MAX_SLICES: u32 = 16;
pub const COYOTE_TIME: f32 = 0.12;

pub const LOOM_ORIGIN: Vec3 = Vec3::new(0.0, 0.0, 512.0);
pub const LOOM_SPAN: i32 = 3;
pub const LOOM_TILE: f32 = 2.0;
pub const LOOM_STEP_SECONDS: f32 = 0.13;
pub const LOOM_CAMERA_LIFT: f32 = 20.0;
pub const LOOM_Y_MAG: f32 = 9.5;
pub const LOOM_START: [i32; 2] = [0, 2];
pub const LOOM_BLOCKS: [[i32; 2]; 3] = [[-1, 0], [1, 0], [0, -1]];
pub const LOOM_PLATES: [[i32; 2]; 3] = [[-3, -2], [3, -2], [0, -3]];
pub const LOOM_GATE: [i32; 2] = [0, 3];

pub const AUTO_ARCH: Vec3 = Vec3::new(0.0, 0.0, -12.4);
pub const AUTO_LOOM: Vec3 = Vec3::new(12.4, 0.0, 0.0);
pub const AUTO_HEARTH: Vec3 = Vec3::new(-12.4, 0.0, 0.0);
pub const AUTO_CAIRN: Vec3 = Vec3::new(0.0, 0.0, 12.4);
pub const LOOM_SOLUTION: [[i32; 2]; 31] = [
    [0, -1],
    [0, -1],
    [0, -1],
    [0, -1],
    [0, 1],
    [0, 1],
    [-1, 0],
    [-1, 0],
    [0, 1],
    [-1, 0],
    [0, -1],
    [0, -1],
    [0, 1],
    [0, 1],
    [1, 0],
    [1, 0],
    [1, 0],
    [0, -1],
    [1, 0],
    [1, 0],
    [0, 1],
    [1, 0],
    [0, -1],
    [0, -1],
    [0, 1],
    [0, 1],
    [0, 1],
    [0, 1],
    [-1, 0],
    [-1, 0],
    [-1, 0],
];
pub const AUTO_DIALS: Vec3 = Vec3::new(4.8, 0.0, 8.4);
pub const AUTO_JUMPS: [f32; 4] = [517.3, 522.0, 528.0, 534.5];
pub const AUTO_GLASS_TOP: f32 = 540.0;
pub const AUTO_LOOK_SECONDS: f32 = 3.0;
pub const AUTO_TURN_SECONDS: f32 = 0.3;
pub const AUTO_TURN_RATE: f32 = 2.6;
pub const AUTO_FACING: f32 = 0.35;

pub const FADE_SECONDS: f32 = 0.46;
pub const FADE_COLOR: [f32; 3] = [0.72, 0.86, 0.94];

pub const DIAL_COUNT: usize = 4;
pub const DIAL_REACH: f32 = 4.2;
pub const DIAL_CENTER: Vec3 = Vec3::new(3.2, 1.8, 10.6);
pub const DIAL_SPACING: f32 = 1.6;
pub const DIAL_PLATE: Vec3 = Vec3::new(1.3, 1.3, 0.14);
pub const PIP_SPACING: f32 = 0.34;
pub const PIP_SIZE: Vec3 = Vec3::new(0.24, 0.24, 0.1);
pub const PIP_STANDOFF: f32 = 0.1;

pub const LAMP_LIFT: f32 = 2.4;
pub const LAMP_SIZE: Vec3 = Vec3::new(0.46, 0.46, 0.46);
pub const MARK_SIZE: Vec3 = Vec3::new(0.34, 0.14, 0.34);
pub const MARK_BASE: f32 = 1.05;
pub const MARK_STEP: f32 = 0.34;
pub const LAMP_INSET: f32 = 0.62;

pub const WALL_PLATE: Vec3 = Vec3::new(1.9, 1.9, 0.16);
pub const WALL_FRAME: Vec3 = Vec3::new(2.4, 2.4, 0.1);
pub const WALL_PIP_SPACING: f32 = 0.5;
pub const WALL_PIP_SIZE: Vec3 = Vec3::new(0.36, 0.36, 0.12);

pub struct Tone {
    pub material: &'static str,
    pub glyph: Glyph,
    pub marks: usize,
    pub pillar: usize,
    pub post: Vec3,
    pub home: Dimension,
}

pub const CAIRN_ORIGIN: Vec3 = Vec3::new(0.0, 0.0, 1024.0);
pub const CAIRN_SPAN: i32 = 9;
pub const CAIRN_START: Vec3 = Vec3::new(0.0, 2.0, 1032.0);
pub const CAIRN_PEAK: Vec3 = Vec3::new(-3.0, 0.0, 1018.0);
pub const CAIRN_REACH: f32 = 2.4;
pub const CAIRN_GATE: Vec3 = Vec3::new(0.0, 0.0, 1033.0);
pub const CAIRN_GATE_REACH: f32 = 2.2;
pub const CAIRN_FOG: [f32; 3] = [0.08, 0.10, 0.12];

pub const HEARTH_ORIGIN: Vec3 = Vec3::new(1024.0, 0.0, 0.0);
pub const HEARTH_START: Vec3 = Vec3::new(1024.0, 0.0, 9.0);
pub const HEARTH_WELL: Vec3 = Vec3::new(1024.0, 0.0, -5.0);
pub const HEARTH_REACH: f32 = 2.6;
pub const HEARTH_GATE: Vec3 = Vec3::new(1024.0, 0.0, 11.0);
pub const HEARTH_GATE_REACH: f32 = 2.0;
pub const HEARTH_FOG: [f32; 3] = [0.16, 0.13, 0.18];
pub const HEARTH_TILT: f32 = -0.62;
pub const HEARTH_LIFT: f32 = 9.0;
pub const HEARTH_BACK: f32 = 12.0;
pub const HEARTH_FIELD_OF_VIEW: f32 = 0.7;
pub const HEARTH_SPEED: f32 = 5.0;

pub const ART_ROWS: usize = 16;

pub const TONES: [Tone; DIAL_COUNT] = [
    Tone {
        material: AMBER,
        glyph: Glyph::Wedge,
        marks: 2,
        pillar: 1,
        post: Vec3::new(533.5, 7.4, -1.9),
        home: Dimension::Glass,
    },
    Tone {
        material: AZURE,
        glyph: Glyph::Comb,
        marks: 3,
        pillar: 4,
        post: Vec3::new(0.0, 3.4, 502.8),
        home: Dimension::Loom,
    },
    Tone {
        material: BONE,
        glyph: Glyph::Ring,
        marks: 1,
        pillar: 6,
        post: Vec3::new(-3.0, 6.4, 1018.0),
        home: Dimension::Cairn,
    },
    Tone {
        material: MOSS,
        glyph: Glyph::Fork,
        marks: 4,
        pillar: 3,
        post: Vec3::new(1024.0, 3.4, -7.6),
        home: Dimension::Hearth,
    },
];

pub const GLYPH_ROWS: usize = 3;

pub fn glyph_pattern(glyph: Glyph) -> [&'static str; GLYPH_ROWS] {
    match glyph {
        Glyph::Bar => [".#.", ".#.", ".#."],
        Glyph::Fork => ["#.#", "#.#", ".#."],
        Glyph::Ring => ["###", "#.#", "###"],
        Glyph::Wedge => ["#..", "##.", "###"],
        Glyph::Cross => ["#.#", ".#.", "#.#"],
        Glyph::Comb => ["###", ".#.", ".#."],
    }
}

pub fn pip_lit(glyph: Glyph, index: usize) -> bool {
    let rows = glyph_pattern(glyph);
    let row = index / GLYPH_ROWS;
    let column = index % GLYPH_ROWS;
    rows[row].as_bytes()[column] == b'#'
}

pub fn pip_offset(index: usize, spacing: f32) -> (f32, f32) {
    let row = index / GLYPH_ROWS;
    let column = index % GLYPH_ROWS;
    (
        (column as f32 - 1.0) * spacing,
        (1.0 - row as f32) * spacing,
    )
}

pub fn dial_center(dial: usize) -> Vec3 {
    DIAL_CENTER + Vec3::new(dial as f32 * DIAL_SPACING, 0.0, 0.0)
}

pub fn pillar_at(index: usize) -> Vec3 {
    let angle = (index as f32 + 0.5) / PILLAR_COUNT as f32 * std::f32::consts::TAU;
    Vec3::new(
        angle.sin() * PILLAR_RING,
        WALL_HEIGHT * 0.5,
        angle.cos() * PILLAR_RING,
    )
}

pub struct Line {
    pub cue: Cue,
    pub speaker: &'static str,
    pub text: &'static str,
    pub hold: f32,
}

pub const KEEPER: &str = "KEEPER";
pub const SURVEYOR: &str = "SURVEYOR";
pub const SCRIPT_COUNT: usize = 15;

pub const SCRIPT: [Line; SCRIPT_COUNT] = [
    Line {
        cue: Cue::Opening,
        speaker: KEEPER,
        text: "You came in by the one door that still opens. Nobody has, in a long while.",
        hold: 5.0,
    },
    Line {
        cue: Cue::Opening,
        speaker: SURVEYOR,
        text: "I am here to record the place. Not to keep it.",
        hold: 4.2,
    },
    Line {
        cue: Cue::Opening,
        speaker: KEEPER,
        text: "The lattice is closing. What you do not record, it loses.",
        hold: 4.6,
    },
    Line {
        cue: Cue::Entered,
        speaker: KEEPER,
        text: "From your side it looked like a room. It is not a room.",
        hold: 4.4,
    },
    Line {
        cue: Cue::Entered,
        speaker: KEEPER,
        text: "Four shapes stand along the way. Each one holds its own colour.",
        hold: 5.0,
    },
    Line {
        cue: Cue::Solved,
        speaker: KEEPER,
        text: "The colour holds now. Carry it back and do not write it down.",
        hold: 4.6,
    },
    Line {
        cue: Cue::Returned,
        speaker: KEEPER,
        text: "The shapes are not the lock. Their order is, and the shapes do not carry it.",
        hold: 5.4,
    },
    Line {
        cue: Cue::NearDials,
        speaker: KEEPER,
        text: "Four pillars hold a light. Count the marks cut below each one.",
        hold: 5.0,
    },
    Line {
        cue: Cue::NearDials,
        speaker: SURVEYOR,
        text: "One mark, one dial. The colour says which shape belongs to it.",
        hold: 5.0,
    },
    Line {
        cue: Cue::Turned,
        speaker: KEEPER,
        text: "Set all four. The arch reads the dials. It does not read you.",
        hold: 4.4,
    },
    Line {
        cue: Cue::Opened,
        speaker: KEEPER,
        text: "There. The floor. You can carry the map out through it.",
        hold: 5.0,
    },
    Line {
        cue: Cue::Opened,
        speaker: SURVEYOR,
        text: "Come out with me.",
        hold: 3.4,
    },
    Line {
        cue: Cue::Opened,
        speaker: KEEPER,
        text: "I am the part of it that holds the door. Walk, surveyor.",
        hold: 5.6,
    },
    Line {
        cue: Cue::Ended,
        speaker: SURVEYOR,
        text: "Four worlds, and one lattice under all of them. The map holds.",
        hold: 5.0,
    },
    Line {
        cue: Cue::Ended,
        speaker: KEEPER,
        text: "Then the place is recorded. Go, and let the door close behind you.",
        hold: 5.4,
    },
];

pub fn speaker_color(speaker: &str) -> [f32; 4] {
    if speaker == KEEPER {
        [0.66, 0.84, 1.0, 1.0]
    } else {
        [1.0, 0.86, 0.62, 1.0]
    }
}

pub struct Slab {
    pub center: Vec3,
    pub size: Vec3,
    pub material: &'static str,
    pub dimension: Dimension,
}

pub struct Gate {
    pub dimension: Dimension,
    pub center: Vec3,
    pub normal: Vec3,
    pub window: Vec3,
    pub window_normal: Vec3,
}

pub fn gates() -> [Gate; 4] {
    [
        Gate {
            dimension: Dimension::Glass,
            center: Vec3::new(0.0, PORTAL_LIFT, -HUB_HALF + PORTAL_INSET),
            normal: Vec3::new(0.0, 0.0, 1.0),
            window: GLASS_WINDOW,
            window_normal: Vec3::new(0.0, 0.0, -1.0),
        },
        Gate {
            dimension: Dimension::Loom,
            center: Vec3::new(HUB_HALF - PORTAL_INSET, PORTAL_LIFT, 0.0),
            normal: Vec3::new(-1.0, 0.0, 0.0),
            window: LOOM_WINDOW,
            window_normal: Vec3::new(0.0, 0.0, -1.0),
        },
        Gate {
            dimension: Dimension::Hearth,
            center: Vec3::new(-HUB_HALF + PORTAL_INSET, PORTAL_LIFT, 0.0),
            normal: Vec3::new(1.0, 0.0, 0.0),
            window: HEARTH_WINDOW,
            window_normal: Vec3::new(0.0, 0.0, 1.0),
        },
        Gate {
            dimension: Dimension::Cairn,
            center: Vec3::new(0.0, PORTAL_LIFT, HUB_HALF - PORTAL_INSET),
            normal: Vec3::new(0.0, 0.0, -1.0),
            window: CAIRN_WINDOW,
            window_normal: Vec3::new(0.0, 0.0, 1.0),
        },
    ]
}

pub fn hub_solids() -> Vec<Slab> {
    let span = HUB_HALF * 2.0 + 2.0;
    let flank = (span - ARCH_WIDTH) * 0.5;
    let offset = (ARCH_WIDTH + flank) * 0.5;
    let lintel = ARCH_HEIGHT + (WALL_HEIGHT - ARCH_HEIGHT) * 0.5;
    let mut slabs = Vec::new();
    let quadrant = (span - SHAFT_WIDTH) * 0.5;
    let reach = (SHAFT_WIDTH + quadrant) * 0.5;
    for side in [-1.0_f32, 1.0] {
        slabs.push(slab(
            Vec3::new(side * reach, -0.5, 0.0),
            Vec3::new(quadrant, 1.0, span),
            STONE,
            Dimension::Hub,
        ));
        slabs.push(slab(
            Vec3::new(0.0, -0.5, side * reach),
            Vec3::new(SHAFT_WIDTH, 1.0, quadrant),
            STONE,
            Dimension::Hub,
        ));
    }

    for depth in [-HUB_HALF, HUB_HALF] {
        for side in [-1.0_f32, 1.0] {
            slabs.push(slab(
                Vec3::new(side * offset, WALL_HEIGHT * 0.5, depth),
                Vec3::new(flank, WALL_HEIGHT, 1.0),
                WALL,
                Dimension::Hub,
            ));
        }
        slabs.push(slab(
            Vec3::new(0.0, lintel, depth),
            Vec3::new(ARCH_WIDTH, WALL_HEIGHT - ARCH_HEIGHT, 1.0),
            WALL,
            Dimension::Hub,
        ));
    }

    for across in [-HUB_HALF, HUB_HALF] {
        for side in [-1.0_f32, 1.0] {
            slabs.push(slab(
                Vec3::new(across, WALL_HEIGHT * 0.5, side * offset),
                Vec3::new(1.0, WALL_HEIGHT, flank),
                WALL,
                Dimension::Hub,
            ));
        }
        slabs.push(slab(
            Vec3::new(across, lintel, 0.0),
            Vec3::new(1.0, WALL_HEIGHT - ARCH_HEIGHT, ARCH_WIDTH),
            WALL,
            Dimension::Hub,
        ));
    }

    for index in 0..PILLAR_COUNT {
        slabs.push(slab(
            pillar_at(index),
            Vec3::new(0.9, WALL_HEIGHT, 0.9),
            PILLAR,
            Dimension::Hub,
        ));
    }

    slabs
}

pub fn glass_solids() -> Vec<Slab> {
    vec![
        slab(
            Vec3::new(514.0, -0.5, 0.0),
            Vec3::new(8.0, 1.0, 4.0),
            GLASS_GROUND,
            Dimension::Glass,
        ),
        slab(
            Vec3::new(521.0, 1.2, 0.0),
            Vec3::new(3.4, 0.6, 4.0),
            GLASS_LEDGE,
            Dimension::Glass,
        ),
        slab(
            Vec3::new(527.0, 2.8, 0.0),
            Vec3::new(3.4, 0.6, 4.0),
            GLASS_LEDGE,
            Dimension::Glass,
        ),
        slab(
            Vec3::new(533.5, 4.4, 0.0),
            Vec3::new(3.4, 0.6, 4.0),
            GLASS_LEDGE,
            Dimension::Glass,
        ),
        slab(
            Vec3::new(539.5, 5.8, 0.0),
            Vec3::new(5.0, 0.6, 4.0),
            GLASS_LEDGE,
            Dimension::Glass,
        ),
        slab(
            Vec3::new(509.4, 2.5, 0.0),
            Vec3::new(1.0, 7.0, 4.0),
            GLASS_WALL,
            Dimension::Glass,
        ),
        slab(
            Vec3::new(542.8, 6.0, 0.0),
            Vec3::new(1.0, 9.0, 4.0),
            GLASS_WALL,
            Dimension::Glass,
        ),
        slab(
            Vec3::new(526.0, 7.0, -2.8),
            Vec3::new(40.0, 24.0, 0.8),
            GLASS_WALL,
            Dimension::Glass,
        ),
        slab(
            Vec3::new(533.5, 4.5, -2.2),
            Vec3::new(0.45, 5.4, 0.45),
            GLASS_WALL,
            Dimension::Glass,
        ),
    ]
}

pub fn loom_cell(cell: [i32; 2]) -> Vec3 {
    LOOM_ORIGIN + Vec3::new(cell[0] as f32 * LOOM_TILE, 0.0, cell[1] as f32 * LOOM_TILE)
}

pub fn loom_inside(cell: [i32; 2]) -> bool {
    cell[0].abs() <= LOOM_SPAN && cell[1].abs() <= LOOM_SPAN
}

pub fn loom_solids() -> Vec<Slab> {
    let width = (LOOM_SPAN * 2 + 1) as f32 * LOOM_TILE;
    vec![slab(
        LOOM_ORIGIN + Vec3::new(0.0, -0.7, 0.0),
        Vec3::new(width + 1.0, 1.0, width + 1.0),
        WALL,
        Dimension::Loom,
    )]
}

pub fn loom_tiles() -> Vec<Slab> {
    let mut tiles = Vec::new();
    for row in -LOOM_SPAN..=LOOM_SPAN {
        for column in -LOOM_SPAN..=LOOM_SPAN {
            let pale = (row + column).rem_euclid(2) == 0;
            tiles.push(slab(
                loom_cell([column, row]) + Vec3::new(0.0, -0.1, 0.0),
                Vec3::new(LOOM_TILE * 0.92, 0.2, LOOM_TILE * 0.92),
                if pale { STONE } else { PILLAR },
                Dimension::Loom,
            ));
        }
    }
    tiles
}

pub fn cairn_height(x: i32, z: i32) -> f32 {
    let fx = x as f32;
    let fz = z as f32;
    let roll = (fx * 0.31).sin() * 1.6 + (fz * 0.27).cos() * 1.4;
    let ridge = ((fx + fz) * 0.19).sin() * 1.5;
    (roll + ridge + 2.0).round().clamp(0.0, 7.0)
}

pub fn cairn_solids() -> Vec<Slab> {
    let mut blocks = Vec::new();
    for z in -CAIRN_SPAN..=CAIRN_SPAN {
        for x in -CAIRN_SPAN..=CAIRN_SPAN {
            let height = cairn_height(x, z);
            let center = CAIRN_ORIGIN + Vec3::new(x as f32, height - 0.5, z as f32);
            let stone = if height > 4.0 { STONE } else { PILLAR };
            blocks.push(slab(
                center,
                Vec3::new(1.0, 1.0, 1.0),
                stone,
                Dimension::Cairn,
            ));
        }
    }
    blocks
}

pub fn hearth_solids() -> Vec<Slab> {
    let mut blocks = vec![slab(
        HEARTH_ORIGIN + Vec3::new(0.0, -0.5, 0.0),
        Vec3::new(26.0, 1.0, 26.0),
        PILLAR,
        Dimension::Hearth,
    )];
    for (across, along) in [(-6.0_f32, -2.0_f32), (6.0, -1.0), (-5.0, 5.0), (5.0, 6.0)] {
        blocks.push(slab(
            HEARTH_ORIGIN + Vec3::new(across, 1.2, along),
            Vec3::new(4.0, 2.4, 3.4),
            STONE,
            Dimension::Hearth,
        ));
        blocks.push(slab(
            HEARTH_ORIGIN + Vec3::new(across, 2.7, along),
            Vec3::new(4.6, 0.6, 4.0),
            SEAL,
            Dimension::Hearth,
        ));
    }
    blocks.push(slab(
        HEARTH_WELL + Vec3::new(0.0, 0.4, 0.0),
        Vec3::new(2.2, 0.8, 2.2),
        WALL,
        Dimension::Hearth,
    ));
    blocks
}

pub const TREE_ART: [&str; ART_ROWS] = [
    "......####......",
    "....########....",
    "...##########...",
    "..####gggg####..",
    "..###gggggg###..",
    ".###gggggggg###.",
    ".##gggggggggg##.",
    ".##gggggggggg##.",
    "..##gggggggg##..",
    "...##########...",
    ".....######.....",
    "......wwww......",
    "......wwww......",
    "......wwww......",
    ".....wwwwww.....",
    "....wwwwwwww....",
];

pub const FOLK_ART: [&str; ART_ROWS] = [
    ".....hhhhhh.....",
    "....hhhhhhhh....",
    "....hssssssh....",
    "....skssssks....",
    "....hssssssh....",
    ".....ssssss.....",
    "....bbbbbbbb....",
    "...bbbbbbbbbb...",
    "...bsbbbbbbsb...",
    "...bsbbbbbbsb...",
    "...bbbbbbbbbb...",
    "....bbbbbbbb....",
    "....bbbbbbbb....",
    "....ww....ww....",
    "....ww....ww....",
    "...kkk....kkk...",
];

pub const ART_MARKS: [char; 7] = ['k', 'g', 'w', 's', 'h', 'b', '#'];
pub const ART_CELL: f32 = 0.11;

pub fn art_material(mark: char) -> &'static str {
    match mark {
        'k' => "fourfold_art_k",
        'g' => "fourfold_art_g",
        'w' => "fourfold_art_w",
        's' => "fourfold_art_s",
        'h' => "fourfold_art_h",
        'b' => "fourfold_art_b",
        _ => "fourfold_art_leaf",
    }
}

pub fn art_color(mark: char) -> Option<[f32; 4]> {
    match mark {
        'k' => Some([0.12, 0.10, 0.14, 1.0]),
        'g' => Some([0.24, 0.58, 0.28, 1.0]),
        'w' => Some([0.45, 0.31, 0.19, 1.0]),
        's' => Some([0.95, 0.78, 0.62, 1.0]),
        'h' => Some([0.32, 0.20, 0.12, 1.0]),
        'b' => Some([0.24, 0.38, 0.72, 1.0]),
        '#' => Some([0.31, 0.69, 0.33, 1.0]),
        _ => None,
    }
}

pub fn hearth_folk() -> Vec<Vec3> {
    vec![
        HEARTH_ORIGIN + Vec3::new(-2.5, 0.0, 1.5),
        HEARTH_ORIGIN + Vec3::new(3.0, 0.0, 2.5),
        HEARTH_ORIGIN + Vec3::new(0.5, 0.0, -1.5),
    ]
}

pub fn hearth_trees() -> Vec<Vec3> {
    vec![
        HEARTH_ORIGIN + Vec3::new(-9.0, 0.0, -6.0),
        HEARTH_ORIGIN + Vec3::new(9.0, 0.0, -6.0),
        HEARTH_ORIGIN + Vec3::new(-9.0, 0.0, 8.0),
        HEARTH_ORIGIN + Vec3::new(9.0, 0.0, 8.0),
        HEARTH_ORIGIN + Vec3::new(0.0, 0.0, -10.0),
    ]
}

fn slab(center: Vec3, size: Vec3, material: &'static str, dimension: Dimension) -> Slab {
    Slab {
        center,
        size,
        material,
        dimension,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Walk,
    Jump,
    DialOne,
    DialTwo,
    DialThree,
    Grab,
    Pause,
    Autoplay,
}

impl nightshade_input::Action for Ask {
    fn all() -> &'static [Self] {
        &[
            Self::Walk,
            Self::Jump,
            Self::DialOne,
            Self::DialTwo,
            Self::DialThree,
            Self::Grab,
            Self::Pause,
            Self::Autoplay,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Walk => "walk",
            Self::Jump => "jump",
            Self::DialOne => "dial one",
            Self::DialTwo => "dial two",
            Self::DialThree => "dial three",
            Self::Grab => "grab",
            Self::Pause => "pause",
            Self::Autoplay => "autoplay",
        }
    }

    fn default_bindings(&self) -> &'static [nightshade_input::Binding] {
        use nightshade_gamepad::contract::GamepadButton;
        use nightshade_input::{Binding, Pointer, Stick};
        use nightshade_platform::KeyCode;
        match self {
            Self::Walk => &[
                Binding::Keys {
                    up: KeyCode::KeyW,
                    down: KeyCode::KeyS,
                    left: KeyCode::KeyA,
                    right: KeyCode::KeyD,
                },
                Binding::Keys {
                    up: KeyCode::ArrowUp,
                    down: KeyCode::ArrowDown,
                    left: KeyCode::ArrowLeft,
                    right: KeyCode::ArrowRight,
                },
                Binding::Stick(Stick::Left),
            ],
            Self::Jump => &[
                Binding::Key(KeyCode::Space),
                Binding::Pad(GamepadButton::South),
            ],
            Self::DialOne => &[Binding::Key(KeyCode::Digit1)],
            Self::DialTwo => &[Binding::Key(KeyCode::Digit2)],
            Self::DialThree => &[Binding::Key(KeyCode::Digit3)],
            Self::Grab => &[Binding::Pointer(Pointer::Left)],
            Self::Pause => &[
                Binding::Key(KeyCode::Escape),
                Binding::Pad(GamepadButton::Start),
            ],
            Self::Autoplay => &[Binding::Key(KeyCode::F1)],
        }
    }
}
