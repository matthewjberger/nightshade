use nightshade::prelude::Vec3;

pub const MODEL_SCALE: f32 = 1.6;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ModelKind {
    BoxClosed,
    BoxOpen,
    Trashcan,
    Chair,
    Table,
    Desk,
    Bookcase,
    Books,
    Radio,
    SideTable,
    LoungeChair,
    WallLamp,
}

pub struct Model {
    pub kind: ModelKind,
    pub bytes: &'static [u8],
    pub low: Vec3,
    pub high: Vec3,
    pub mass: f32,
}

pub struct Placement {
    pub model: ModelKind,
    pub at: Vec3,
    pub yaw: f32,
    pub loose: bool,
}

pub struct LampMount {
    pub at: Vec3,
    pub yaw: f32,
}

pub const MODELS: &[Model] = &[
    Model {
        kind: ModelKind::BoxClosed,
        bytes: include_bytes!("../../assets/props/cardboardBoxClosed.glb"),
        low: Vec3::new(0.0, 0.0, -0.21),
        high: Vec3::new(0.21, 0.28, 0.0),
        mass: 5.0,
    },
    Model {
        kind: ModelKind::BoxOpen,
        bytes: include_bytes!("../../assets/props/cardboardBoxOpen.glb"),
        low: Vec3::new(-0.08, 0.0, -0.21),
        high: Vec3::new(0.29, 0.28, 0.0),
        mass: 4.0,
    },
    Model {
        kind: ModelKind::Trashcan,
        bytes: include_bytes!("../../assets/props/trashcan.glb"),
        low: Vec3::new(-0.26, 0.0, -0.22),
        high: Vec3::new(0.24, 0.91, 0.22),
        mass: 16.0,
    },
    Model {
        kind: ModelKind::Chair,
        bytes: include_bytes!("../../assets/props/chair.glb"),
        low: Vec3::new(0.0, 0.0, -0.2),
        high: Vec3::new(0.2, 0.47, 0.0),
        mass: 7.0,
    },
    Model {
        kind: ModelKind::Table,
        bytes: include_bytes!("../../assets/props/table.glb"),
        low: Vec3::new(0.0, 0.0, -0.45),
        high: Vec3::new(0.84, 0.33, 0.0),
        mass: 22.0,
    },
    Model {
        kind: ModelKind::Desk,
        bytes: include_bytes!("../../assets/props/desk.glb"),
        low: Vec3::new(-0.01, 0.0, -0.38),
        high: Vec3::new(0.72, 0.38, 0.18),
        mass: 30.0,
    },
    Model {
        kind: ModelKind::Bookcase,
        bytes: include_bytes!("../../assets/props/bookcaseClosed.glb"),
        low: Vec3::new(0.0, 0.0, -0.25),
        high: Vec3::new(0.4, 0.85, 0.0),
        mass: 45.0,
    },
    Model {
        kind: ModelKind::Books,
        bytes: include_bytes!("../../assets/props/books.glb"),
        low: Vec3::new(0.0, 0.0, -0.09),
        high: Vec3::new(0.15, 0.1, 0.0),
        mass: 1.4,
    },
    Model {
        kind: ModelKind::Radio,
        bytes: include_bytes!("../../assets/props/radio.glb"),
        low: Vec3::new(0.0, 0.0, -0.1),
        high: Vec3::new(0.32, 0.23, 0.0),
        mass: 2.6,
    },
    Model {
        kind: ModelKind::SideTable,
        bytes: include_bytes!("../../assets/props/sideTableDrawers.glb"),
        low: Vec3::new(-0.01, 0.0, -0.21),
        high: Vec3::new(0.52, 0.38, 0.18),
        mass: 18.0,
    },
    Model {
        kind: ModelKind::LoungeChair,
        bytes: include_bytes!("../../assets/props/loungeChair.glb"),
        low: Vec3::new(0.0, 0.0, -0.41),
        high: Vec3::new(0.49, 0.46, 0.0),
        mass: 14.0,
    },
    Model {
        kind: ModelKind::WallLamp,
        bytes: include_bytes!("../../assets/props/lampWall.glb"),
        low: Vec3::new(-0.2, 0.0, 0.0),
        high: Vec3::new(0.2, 0.16, 0.26),
        mass: 1.0,
    },
];

pub const PLACEMENTS: &[Placement] = &[
    fixed(ModelKind::Desk, 0.72, 3.6, -std::f32::consts::FRAC_PI_2),
    loose(ModelKind::Chair, 1.75, 3.6, std::f32::consts::FRAC_PI_2),
    loose_at(ModelKind::Books, 0.62, DESK_TOP, 3.12, 0.3),
    loose_at(
        ModelKind::Radio,
        0.6,
        DESK_TOP,
        4.02,
        -std::f32::consts::FRAC_PI_2,
    ),
    fixed(ModelKind::Table, 2.4, 0.62, 0.0),
    loose(ModelKind::BoxClosed, 5.4, 0.75, 0.4),
    loose(ModelKind::BoxClosed, 6.4, 0.6, 0.1),
    loose(ModelKind::BoxOpen, 6.5, 1.3, -0.8),
    fixed(ModelKind::Trashcan, 7.3, 0.9, 0.0),
    loose(ModelKind::BoxOpen, 6.9, 6.0, 1.1),
    loose(ModelKind::BoxClosed, 6.6, 8.6, 0.2),
    fixed(ModelKind::Trashcan, 14.6, 8.9, 0.0),
    loose(ModelKind::BoxOpen, 10.2, 8.4, 1.1),
    fixed(ModelKind::Table, 1.4, 12.2, 0.0),
    loose(ModelKind::Radio, 1.7, 12.0, 0.6),
    loose(ModelKind::Chair, 2.9, 13.4, 1.4),
    loose(ModelKind::BoxOpen, 5.7, 17.4, -0.4),
    fixed(ModelKind::Trashcan, 6.1, 11.2, 0.0),
    loose(ModelKind::BoxClosed, 5.9, 12.2, 0.9),
    fixed(
        ModelKind::Bookcase,
        15.0,
        11.4,
        -std::f32::consts::FRAC_PI_2,
    ),
    fixed(
        ModelKind::Bookcase,
        15.0,
        13.2,
        -std::f32::consts::FRAC_PI_2,
    ),
    loose(ModelKind::BoxClosed, 10.3, 13.8, 0.3),
    loose(ModelKind::BoxClosed, 10.7, 14.3, -0.6),
    fixed(ModelKind::SideTable, 9.7, 11.1, std::f32::consts::FRAC_PI_2),
    fixed(ModelKind::Desk, 12.2, 18.6, 0.0),
    loose(ModelKind::Books, 12.5, 18.4, 0.3),
    loose(ModelKind::Chair, 12.4, 17.4, 0.0),
    fixed(
        ModelKind::Bookcase,
        15.0,
        19.6,
        -std::f32::consts::FRAC_PI_2,
    ),
    fixed(ModelKind::LoungeChair, 10.1, 20.1, 0.7),
    loose(ModelKind::Radio, 11.6, 18.5, -0.4),
    loose(ModelKind::BoxOpen, 1.2, 21.0, 0.2),
    fixed(ModelKind::Trashcan, 0.9, 25.0, 0.0),
    loose(ModelKind::BoxClosed, 5.8, 21.4, -0.3),
];

pub const LAMPS: &[LampMount] = &[
    lamp(2.0, 0.18, 0.0),
    lamp(6.0, 6.82, std::f32::consts::PI),
    lamp(4.0, 7.18, 0.0),
    lamp(13.0, 9.82, std::f32::consts::PI),
    lamp(0.18, 13.0, std::f32::consts::FRAC_PI_2),
    lamp(6.82, 16.5, -std::f32::consts::FRAC_PI_2),
    lamp(9.18, 12.5, std::f32::consts::FRAC_PI_2),
    lamp(15.82, 18.0, -std::f32::consts::FRAC_PI_2),
    lamp(3.5, 25.82, std::f32::consts::PI),
];

pub const LAMP_HEIGHT: f32 = 2.2;
pub const DESK_TOP: f32 = 0.61;

pub const HANGING_LAMPS: &[Vec3] = &[Vec3::new(4.0, 3.06, 3.6)];
pub const HANGING_DROP: f32 = 0.9;
pub const HANGING_NUDGE: f32 = 0.28;

pub const NOTE_AT: Vec3 = Vec3::new(0.78, 0.63, 3.42);
pub const NOTE_SIZE: Vec3 = Vec3::new(0.21, 0.012, 0.29);
pub const GLOWSTICK_AT: Vec3 = Vec3::new(7.7, 1.02, 2.38);
pub const GLOWSTICK_HALF_LENGTH: f32 = 0.11;
pub const GLOWSTICK_RADIUS: f32 = 0.014;

pub const CABINET_AT: Vec3 = Vec3::new(13.2, 0.0, 13.6);
pub const CABINET_WIDTH: f32 = 1.0;
pub const CABINET_HEIGHT: f32 = 0.9;
pub const CABINET_DEPTH: f32 = 0.7;
pub const CABINET_SHELL: f32 = 0.1;
pub const TRAY_HEIGHT: f32 = 0.46;
pub const KEY_SIZE: Vec3 = Vec3::new(0.09, 0.03, 0.2);
pub const CELL_AT: Vec3 = Vec3::new(1.7, 0.66, 12.1);

pub fn model(kind: ModelKind) -> &'static Model {
    MODELS
        .iter()
        .find(|entry| entry.kind == kind)
        .unwrap_or(&MODELS[0])
}

pub fn extents(entry: &Model) -> Vec3 {
    (entry.high - entry.low) * MODEL_SCALE
}

pub fn centering(entry: &Model) -> Vec3 {
    -(entry.low + entry.high) * 0.5 * MODEL_SCALE
}

const fn loose(model: ModelKind, x: f32, z: f32, yaw: f32) -> Placement {
    Placement {
        model,
        at: Vec3::new(x, 0.0, z),
        yaw,
        loose: true,
    }
}

const fn loose_at(model: ModelKind, x: f32, y: f32, z: f32, yaw: f32) -> Placement {
    Placement {
        model,
        at: Vec3::new(x, y, z),
        yaw,
        loose: true,
    }
}

const fn fixed(model: ModelKind, x: f32, z: f32, yaw: f32) -> Placement {
    Placement {
        model,
        at: Vec3::new(x, 0.0, z),
        yaw,
        loose: false,
    }
}

const fn lamp(x: f32, z: f32, yaw: f32) -> LampMount {
    LampMount {
        at: Vec3::new(x, LAMP_HEIGHT, z),
        yaw,
    }
}
