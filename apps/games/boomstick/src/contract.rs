#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shot {
    pub gun: usize,
    pub origin: [f32; 3],
    pub direction: [f32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lob {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
    pub speed: f32,
    pub damage: f32,
    pub splash: f32,
    pub fall: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Blast {
    pub at: [f32; 3],
    pub radius: f32,
    pub damage: f32,
    pub from_player: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BoomstickEvent {
    Missed,
    EnemyHit { damage: f32 },
    EnemyKilled,
    SecretFound,
    SwitchThrown,
    DoorOpened,
    PlayerHurt { damage: f32 },
    PickedUp { label: String },
}

#[derive(Default)]
pub struct FireQueue {
    pub shots: Vec<Shot>,
    pub lobs: Vec<Lob>,
    pub blasts: Vec<Blast>,
}
