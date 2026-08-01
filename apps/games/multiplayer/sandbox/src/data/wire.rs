use serde::{Deserialize, Serialize};

pub type PeerId = u32;

pub const WANT_JUMP: u8 = 1;
pub const WANT_CROUCH: u8 = 2;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Request {
    Hello {
        name: String,
    },
    Input {
        sequence: u32,
        walk: [f32; 2],
        wants: u8,
    },
    Chat {
        text: String,
    },
    Tag {
        tick: u32,
        origin: [f32; 3],
        direction: [f32; 3],
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Update {
    Welcome {
        you: PeerId,
        character: u8,
        roster: Vec<Seat>,
        things: Vec<Thing>,
    },
    Joined {
        seat: Seat,
    },
    Left {
        peer: PeerId,
    },
    Snapshot {
        tick: u32,
        bodies: Vec<Body>,
        things: Vec<Thing>,
    },
    Spawned {
        thing: Thing,
    },
    Despawned {
        id: u32,
    },
    Chat {
        peer: PeerId,
        text: String,
    },
    Tagged {
        by: PeerId,
        peer: PeerId,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Seat {
    pub peer: PeerId,
    pub name: String,
    pub character: u8,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Thing {
    pub id: u32,
    pub kind: u8,
    pub position: [f32; 3],
    pub facing: f32,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Body {
    pub peer: PeerId,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub facing: f32,
    pub crouch: f32,
    pub grounded: bool,
    pub acknowledged: u32,
}
