pub struct Beacon;

pub struct Rope;

pub struct Runner;

pub struct Slab {
    pub center: [f32; 3],
    pub size: [f32; 3],
    pub texture: &'static str,
}

pub const LEDGE_TEXTURE: &str = "tether_ledge";
pub const STONE_TEXTURE: &str = "tether_stone";

pub const START: [f32; 3] = [0.0, 1.7, 0.0];

pub const GRAVITY: f32 = -26.0;
pub const FALL_LEVEL: f32 = -25.0;

pub const PLAYER_RADIUS: f32 = 0.5;
pub const PLAYER_MASS: f32 = 70.0;
pub const EYE_OFFSET: f32 = 0.35;

pub const REACH: f32 = 110.0;
pub const PULL_ACCELERATION: f32 = 52.0;
pub const REEL_RATE: f32 = 10.0;
pub const SHORTEST_LINE: f32 = 3.0;

pub const AIR_ACCELERATION: f32 = 48.0;
pub const AIR_SPEED_CAP: f32 = 11.0;
pub const GROUND_ACCELERATION: f32 = 90.0;
pub const GROUND_SPEED_CAP: f32 = 13.0;
pub const GROUND_FRICTION: f32 = 9.0;
pub const JUMP_SPEED: f32 = 11.0;
pub const DRAG: f32 = 0.0016;
pub const GROUND_PROBE: f32 = 0.14;
pub const LONGEST_STEP: f32 = 0.05;

pub const GAMEPAD_LOOK_RATE: f32 = 2.4;

pub const ROPE_THICKNESS: f32 = 0.09;
pub const HAND_ACROSS: f32 = 0.26;
pub const HAND_DOWN: f32 = 0.2;

pub const BEACON_RADIUS: f32 = 3.0;
pub const BEACON_BOB: f32 = 2.6;
pub const BEACON_RATE: f32 = 0.7;

pub const BASE_VIEW_DEGREES: f32 = 72.0;
pub const WIDE_VIEW_DEGREES: f32 = 94.0;
pub const VIEW_SPEED_RANGE: f32 = 62.0;
pub const VIEW_EASE_RATE: f32 = 4.0;
pub const VIEW_STEP_DEGREES: f32 = 0.04;

pub const SLAB_TILE_SIZE: f32 = 4.0;
pub const METER_SPEED_RANGE: f32 = 80.0;

pub const SLABS: [Slab; 16] = [
    Slab {
        center: [0.0, 0.0, 0.0],
        size: [24.0, 2.0, 24.0],
        texture: LEDGE_TEXTURE,
    },
    Slab {
        center: [-18.0, 4.0, -46.0],
        size: [14.0, 2.0, 16.0],
        texture: LEDGE_TEXTURE,
    },
    Slab {
        center: [20.0, 9.0, -88.0],
        size: [14.0, 2.0, 16.0],
        texture: LEDGE_TEXTURE,
    },
    Slab {
        center: [-24.0, 15.0, -132.0],
        size: [14.0, 2.0, 18.0],
        texture: LEDGE_TEXTURE,
    },
    Slab {
        center: [16.0, 22.0, -176.0],
        size: [16.0, 2.0, 16.0],
        texture: LEDGE_TEXTURE,
    },
    Slab {
        center: [0.0, 30.0, -224.0],
        size: [28.0, 2.0, 28.0],
        texture: LEDGE_TEXTURE,
    },
    Slab {
        center: [0.0, 20.0, -24.0],
        size: [5.0, 40.0, 5.0],
        texture: STONE_TEXTURE,
    },
    Slab {
        center: [-6.0, 26.0, -66.0],
        size: [5.0, 52.0, 5.0],
        texture: STONE_TEXTURE,
    },
    Slab {
        center: [10.0, 32.0, -110.0],
        size: [5.0, 64.0, 5.0],
        texture: STONE_TEXTURE,
    },
    Slab {
        center: [-12.0, 40.0, -154.0],
        size: [5.0, 80.0, 5.0],
        texture: STONE_TEXTURE,
    },
    Slab {
        center: [6.0, 46.0, -200.0],
        size: [5.0, 92.0, 5.0],
        texture: STONE_TEXTURE,
    },
    Slab {
        center: [-10.0, 32.0, -36.0],
        size: [18.0, 2.0, 10.0],
        texture: STONE_TEXTURE,
    },
    Slab {
        center: [14.0, 40.0, -76.0],
        size: [18.0, 2.0, 10.0],
        texture: STONE_TEXTURE,
    },
    Slab {
        center: [-16.0, 48.0, -120.0],
        size: [18.0, 2.0, 10.0],
        texture: STONE_TEXTURE,
    },
    Slab {
        center: [12.0, 56.0, -164.0],
        size: [18.0, 2.0, 10.0],
        texture: STONE_TEXTURE,
    },
    Slab {
        center: [-8.0, 62.0, -208.0],
        size: [18.0, 2.0, 10.0],
        texture: STONE_TEXTURE,
    },
];

pub const BEACONS: [[f32; 3]; 5] = [
    [0.0, 14.0, -21.4],
    [-6.0, 26.0, -63.4],
    [10.0, 36.0, -107.4],
    [-12.0, 50.0, -151.4],
    [6.0, 62.0, -197.4],
];

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Walk,
    Jump,
    Throw,
    Pause,
    Back,
}

impl nightshade_input::Action for Ask {
    fn all() -> &'static [Self] {
        &[Self::Walk, Self::Jump, Self::Throw, Self::Pause, Self::Back]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Walk => "walk",
            Self::Jump => "jump",
            Self::Throw => "throw",
            Self::Pause => "pause",
            Self::Back => "back",
        }
    }

    fn default_bindings(&self) -> &'static [nightshade_input::Binding] {
        use nightshade_gamepad::contract::{GamepadAxis, GamepadButton};
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
            Self::Throw => &[
                Binding::Pointer(Pointer::Left),
                Binding::Pad(GamepadButton::RightTrigger),
                Binding::Pad(GamepadButton::RightBumper),
                Binding::Trigger(GamepadAxis::RightTrigger),
            ],
            Self::Pause => &[
                Binding::Key(KeyCode::Escape),
                Binding::Pad(GamepadButton::Start),
            ],
            Self::Back => &[Binding::Pad(GamepadButton::East)],
        }
    }
}
