use crate::contract::{ItemKind, Surface};

pub const ROOM_HEIGHT: f32 = 3.2;
pub const WALL_THICKNESS: f32 = 0.3;
pub const SLAB_THICKNESS: f32 = 0.4;
pub const DOOR_WIDTH: f32 = 1.3;
pub const DOOR_HEIGHT: f32 = 2.3;
pub const DOOR_THICKNESS: f32 = 0.09;
pub const DOOR_CLEARANCE: f32 = 0.04;

pub struct Room {
    pub name: &'static str,
    pub x0: f32,
    pub z0: f32,
    pub x1: f32,
    pub z1: f32,
    pub floor: Surface,
    pub ceiling: Surface,
}

pub struct Wall {
    pub fixed: f32,
    pub from: f32,
    pub to: f32,
    pub along_x: bool,
    pub face: Surface,
}

pub struct Doorway {
    pub name: &'static str,
    pub fixed: f32,
    pub at: f32,
    pub along_x: bool,
    pub hinge_low: bool,
    pub lock: Option<ItemKind>,
}

pub const ROOMS: &[Room] = &[
    Room {
        name: "the entry hall",
        x0: 0.0,
        z0: 0.0,
        x1: 8.0,
        z1: 7.0,
        floor: Surface::Floor,
        ceiling: Surface::Ceiling,
    },
    Room {
        name: "the spine",
        x0: 0.0,
        z0: 7.0,
        x1: 16.0,
        z1: 10.0,
        floor: Surface::Rock,
        ceiling: Surface::Ceiling,
    },
    Room {
        name: "the pump room",
        x0: 0.0,
        z0: 10.0,
        x1: 7.0,
        z1: 19.0,
        floor: Surface::Plate,
        ceiling: Surface::Ceiling,
    },
    Room {
        name: "the store",
        x0: 9.0,
        z0: 10.0,
        x1: 16.0,
        z1: 15.0,
        floor: Surface::Floor,
        ceiling: Surface::Ceiling,
    },
    Room {
        name: "the office",
        x0: 9.0,
        z0: 15.0,
        x1: 16.0,
        z1: 21.0,
        floor: Surface::Floor,
        ceiling: Surface::Ceiling,
    },
    Room {
        name: "the shaft head",
        x0: 0.0,
        z0: 19.0,
        x1: 7.0,
        z1: 26.0,
        floor: Surface::Rock,
        ceiling: Surface::Rock,
    },
];

pub const WALLS: &[Wall] = &[
    wall_x(0.0, 0.0, 8.0, Surface::Wall),
    wall_x(7.0, 0.0, 8.0, Surface::Wall),
    wall_x(7.0, 8.0, 16.0, Surface::Rock),
    wall_x(10.0, 0.0, 7.0, Surface::Wall),
    wall_x(10.0, 7.0, 9.0, Surface::Rock),
    wall_x(10.0, 9.0, 16.0, Surface::Wall),
    wall_x(15.0, 9.0, 16.0, Surface::Wall),
    wall_x(19.0, 0.0, 7.0, Surface::Metal),
    wall_x(21.0, 9.0, 16.0, Surface::Wall),
    wall_x(26.0, 0.0, 7.0, Surface::Rock),
    wall_z(0.0, 0.0, 7.0, Surface::Wall),
    wall_z(8.0, 0.0, 7.0, Surface::Wall),
    wall_z(0.0, 7.0, 10.0, Surface::Rock),
    wall_z(16.0, 7.0, 10.0, Surface::Rock),
    wall_z(0.0, 10.0, 19.0, Surface::Rock),
    wall_z(7.0, 10.0, 19.0, Surface::Wall),
    wall_z(9.0, 10.0, 15.0, Surface::Wall),
    wall_z(16.0, 10.0, 21.0, Surface::Wall),
    wall_z(9.0, 15.0, 21.0, Surface::Wall),
    wall_z(0.0, 19.0, 26.0, Surface::Rock),
    wall_z(7.0, 19.0, 26.0, Surface::Rock),
];

pub const DOORWAYS: &[Doorway] = &[
    Doorway {
        name: "the hall door",
        fixed: 7.0,
        at: 4.0,
        along_x: true,
        hinge_low: true,
        lock: None,
    },
    Doorway {
        name: "the pump room door",
        fixed: 10.0,
        at: 3.0,
        along_x: true,
        hinge_low: false,
        lock: None,
    },
    Doorway {
        name: "the store door",
        fixed: 10.0,
        at: 12.5,
        along_x: true,
        hinge_low: true,
        lock: None,
    },
    Doorway {
        name: "the office door",
        fixed: 15.0,
        at: 12.5,
        along_x: true,
        hinge_low: false,
        lock: None,
    },
    Doorway {
        name: "the shaft door",
        fixed: 19.0,
        at: 3.0,
        along_x: true,
        hinge_low: true,
        lock: Some(ItemKind::ShaftKey),
    },
];

const fn wall_x(fixed: f32, from: f32, to: f32, face: Surface) -> Wall {
    Wall {
        fixed,
        from,
        to,
        along_x: true,
        face,
    }
}

const fn wall_z(fixed: f32, from: f32, to: f32, face: Surface) -> Wall {
    Wall {
        fixed,
        from,
        to,
        along_x: false,
        face,
    }
}
