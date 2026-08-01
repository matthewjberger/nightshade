pub mod autopilot;
pub mod footing;
pub mod mode;

use nightshade::prelude::*;
use nightshade_gamepad::contract::{GamepadAxis, GamepadButton};
use nightshade_input::{Action, Binding, Pointer, Press, Stick};
use nightshade_platform::KeyCode;

nightshade_config::config! {
    pub enum Settings {
        "Flight" {
            Thrust: number(4.0, 200.0, 60.0),
            Assist: number(0.0, 2.0, 0.6),
            Autopilot: number(0.2, 3.0, 1.0),
            Roll: number(0.5, 4.0, 1.6),
            Look: number(0.2, 3.0, 1.0),
        }
        "On foot" {
            Walk: number(2.0, 14.0, 6.0),
            Jump: number(2.0, 15.0, 6.0),
            Jetpack: number(0.0, 60.0, 24.0),
            Fuel: number(1.0, 15.0, 4.0),
        }
        "System" {
            Gravity: number(0.0, 3.0, 1.0),
            Clock: number(0.0, 4.0, 1.0),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Move,
    Look,
    Jump,
    Lift,
    Roll,
    Match,
    Interact,
    Lock,
    Engage,
    Lights,
    Pause,
}

impl Action for Ask {
    fn all() -> &'static [Self] {
        &[
            Self::Move,
            Self::Look,
            Self::Jump,
            Self::Lift,
            Self::Roll,
            Self::Match,
            Self::Interact,
            Self::Lock,
            Self::Engage,
            Self::Lights,
            Self::Pause,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Move => "move",
            Self::Look => "look",
            Self::Jump => "jump",
            Self::Lift => "thrust",
            Self::Roll => "roll mode",
            Self::Match => "match velocity",
            Self::Interact => "interact",
            Self::Lock => "lock on",
            Self::Engage => "autopilot",
            Self::Lights => "lights",
            Self::Pause => "pause",
        }
    }

    fn default_bindings(&self) -> &'static [Binding] {
        match self {
            Self::Move => &[
                Binding::Keys {
                    up: KeyCode::KeyW,
                    down: KeyCode::KeyS,
                    left: KeyCode::KeyA,
                    right: KeyCode::KeyD,
                },
                Binding::Stick(Stick::Left),
            ],
            Self::Look => &[Binding::Stick(Stick::Right)],
            Self::Jump => &[
                Binding::Key(KeyCode::Space),
                Binding::Pad(GamepadButton::South),
            ],
            Self::Lift => &[
                Binding::Pair {
                    positive: Press::Key(KeyCode::ShiftLeft),
                    negative: Press::Key(KeyCode::ControlLeft),
                },
                Binding::Pair {
                    positive: Press::Trigger(GamepadAxis::RightTrigger),
                    negative: Press::Trigger(GamepadAxis::LeftTrigger),
                },
            ],
            Self::Roll => &[
                Binding::Key(KeyCode::KeyR),
                Binding::Pad(GamepadButton::LeftBumper),
            ],
            Self::Match => &[
                Binding::Key(KeyCode::Space),
                Binding::Pad(GamepadButton::South),
            ],
            Self::Interact => &[
                Binding::Key(KeyCode::KeyE),
                Binding::Pad(GamepadButton::West),
            ],
            Self::Lock => &[
                Binding::Pointer(Pointer::Left),
                Binding::Pad(GamepadButton::LeftThumb),
            ],
            Self::Engage => &[
                Binding::Key(KeyCode::KeyX),
                Binding::Pad(GamepadButton::DPadUp),
            ],
            Self::Lights => &[
                Binding::Key(KeyCode::KeyF),
                Binding::Pad(GamepadButton::RightThumb),
            ],
            Self::Pause => &[
                Binding::Key(KeyCode::Escape),
                Binding::Pad(GamepadButton::Start),
            ],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Scorched,
    Home,
    Moon,
    Shell,
    Giant,
    Ice,
}

pub struct BodyPlan {
    pub name: &'static str,
    pub kind: Kind,
    pub grip: f32,
    pub radius: f32,
    pub orbit: f32,
    pub phase: f32,
    pub day: f32,
    pub tilt: f32,
    pub gravity: f32,
    pub color: [f32; 4],
    pub roughness: f32,
    pub metallic: f32,
    pub around: Option<usize>,
    pub ringed: bool,
}

pub const HOME: usize = 1;

pub const BODIES: [BodyPlan; 6] = [
    BodyPlan {
        name: "Cinder",
        kind: Kind::Scorched,
        grip: 1.0,
        radius: 250.0,
        orbit: 3000.0,
        phase: 2.1,
        day: 60.0,
        tilt: 0.05,
        gravity: 8.0,
        color: [0.50, 0.30, 0.22, 1.0],
        roughness: 0.95,
        metallic: 0.0,
        around: None,
        ringed: false,
    },
    BodyPlan {
        name: "Verdant",
        kind: Kind::Home,
        grip: 1.0,
        radius: 625.0,
        orbit: 6400.0,
        phase: 0.0,
        day: 240.0,
        tilt: 0.4,
        gravity: 12.0,
        color: [0.16, 0.50, 0.30, 1.0],
        roughness: 0.65,
        metallic: 0.0,
        around: None,
        ringed: false,
    },
    BodyPlan {
        name: "Pale",
        kind: Kind::Moon,
        grip: 1.0,
        radius: 150.0,
        orbit: 1600.0,
        phase: 1.0,
        day: 120.0,
        tilt: 0.0,
        gravity: 4.0,
        color: [0.70, 0.70, 0.68, 1.0],
        roughness: 0.9,
        metallic: 0.0,
        around: Some(HOME),
        ringed: false,
    },
    BodyPlan {
        name: "Rust",
        kind: Kind::Shell,
        grip: 1.0,
        radius: 450.0,
        orbit: 10400.0,
        phase: 3.9,
        day: 200.0,
        tilt: 0.45,
        gravity: 10.0,
        color: [0.70, 0.30, 0.16, 1.0],
        roughness: 0.85,
        metallic: 0.0,
        around: None,
        ringed: false,
    },
    BodyPlan {
        name: "Halo",
        kind: Kind::Giant,
        grip: 1.0,
        radius: 1125.0,
        orbit: 15200.0,
        phase: 5.2,
        day: 160.0,
        tilt: 0.5,
        gravity: 20.0,
        color: [0.78, 0.64, 0.42, 1.0],
        roughness: 0.6,
        metallic: 0.0,
        around: None,
        ringed: true,
    },
    BodyPlan {
        name: "Rime",
        kind: Kind::Ice,
        grip: 0.12,
        radius: 350.0,
        orbit: 20800.0,
        phase: 1.6,
        day: 300.0,
        tilt: 0.2,
        gravity: 8.0,
        color: [0.70, 0.84, 0.95, 1.0],
        roughness: 0.25,
        metallic: 0.1,
        around: None,
        ringed: false,
    },
];

pub struct BodyView {
    pub entity: Entity,
    pub name: String,
    pub center: Vec3,
    pub rotation: Quat,
    pub radius: f32,
    pub pull: f32,
    pub reach: f32,
    pub spin: f32,
    pub motion: Vec3,
    pub grip: f32,
}

pub struct Updraft {
    pub body: Entity,
    pub center: Vec3,
    pub radius: f32,
    pub lift: Vec3,
}

pub struct SpeakerSpot {
    pub entity: Entity,
    pub index: usize,
    pub center: Vec3,
}

pub struct RelicSpot {
    pub entity: Entity,
    pub index: usize,
    pub body: Entity,
    pub center: Vec3,
}

pub struct Globe {
    pub center: Vec2,
    pub radius: f32,
    pub lines: Vec<(Vec2, Vec2, bool)>,
    pub ship: Vec2,
    pub path: Vec<Vec2>,
}

pub struct Drift {
    pub direction: Vec2,
    pub planar: f32,
    pub forward: f32,
}

pub struct Marker {
    pub screen: Vec2,
    pub radius: f32,
    pub name: String,
    pub in_view: bool,
}

pub struct Collider {
    pub body: Entity,
    pub center: Vec3,
    pub radius: f32,
}

pub struct Piece {
    pub mesh: &'static str,
    pub offset: Vec3,
    pub scale: Vec3,
    pub tilt: f32,
    pub glowing: bool,
}

pub const HULL: [Piece; 11] = [
    Piece {
        mesh: "Cube",
        offset: Vec3::new(0.0, 0.0, 0.0),
        scale: Vec3::new(1.6, 1.0, 4.2),
        tilt: 0.0,
        glowing: false,
    },
    Piece {
        mesh: "Cone",
        offset: Vec3::new(0.0, 0.0, -2.9),
        scale: Vec3::new(1.5, 1.6, 1.0),
        tilt: -std::f32::consts::FRAC_PI_2,
        glowing: false,
    },
    Piece {
        mesh: "Cube",
        offset: Vec3::new(0.0, -0.3, 0.5),
        scale: Vec3::new(5.4, 0.14, 1.6),
        tilt: 0.0,
        glowing: false,
    },
    Piece {
        mesh: "Cube",
        offset: Vec3::new(0.0, 1.0, 1.6),
        scale: Vec3::new(0.14, 1.2, 1.2),
        tilt: 0.0,
        glowing: false,
    },
    Piece {
        mesh: "Cylinder",
        offset: Vec3::new(0.6, -0.1, 2.3),
        scale: Vec3::new(0.9, 0.9, 0.9),
        tilt: std::f32::consts::FRAC_PI_2,
        glowing: false,
    },
    Piece {
        mesh: "Cylinder",
        offset: Vec3::new(-0.6, -0.1, 2.3),
        scale: Vec3::new(0.9, 0.9, 0.9),
        tilt: std::f32::consts::FRAC_PI_2,
        glowing: false,
    },
    Piece {
        mesh: "Sphere",
        offset: Vec3::new(0.6, -0.1, 2.8),
        scale: Vec3::new(0.32, 0.32, 0.32),
        tilt: 0.0,
        glowing: true,
    },
    Piece {
        mesh: "Sphere",
        offset: Vec3::new(-0.6, -0.1, 2.8),
        scale: Vec3::new(0.32, 0.32, 0.32),
        tilt: 0.0,
        glowing: true,
    },
    Piece {
        mesh: "Cylinder",
        offset: Vec3::new(1.0, -0.95, 1.0),
        scale: Vec3::new(0.24, 0.9, 0.24),
        tilt: 0.0,
        glowing: false,
    },
    Piece {
        mesh: "Cylinder",
        offset: Vec3::new(-1.0, -0.95, 1.0),
        scale: Vec3::new(0.24, 0.9, 0.24),
        tilt: 0.0,
        glowing: false,
    },
    Piece {
        mesh: "Cylinder",
        offset: Vec3::new(0.0, -0.95, -1.5),
        scale: Vec3::new(0.24, 0.9, 0.24),
        tilt: 0.0,
        glowing: false,
    },
];

pub const HULL_COLOR: [f32; 4] = [0.62, 0.64, 0.68, 1.0];
pub const HULL_GLOW: [f32; 3] = [1.0, 0.45, 0.12];
pub const HULL_SHEEN: [f32; 3] = [0.10, 0.11, 0.13];
pub const HULL_GLOW_STRENGTH: f32 = 5.0;
pub const COCKPIT: Vec3 = Vec3::new(0.0, 1.1, -1.6);
pub const HEADLIGHTS: [Vec3; 2] = [Vec3::new(-0.7, -0.1, -2.2), Vec3::new(0.7, -0.1, -2.2)];
pub const HEADLIGHT_COLOR: Vec3 = Vec3::new(1.0, 0.96, 0.88);
pub const HEADLIGHT: f32 = 160.0;
pub const HEADLIGHT_RANGE: f32 = 180.0;
pub const HEADLIGHT_INNER: f32 = 0.30;
pub const HEADLIGHT_OUTER: f32 = 0.45;
pub const SHIP_RADIUS: f32 = 1.6;
pub const EXIT_OFFSET: f32 = 4.0;
pub const BOARD_REACH: f32 = 7.0;

pub const WALKER_RADIUS: f32 = 0.8;
pub const WALK_PITCH_LIMIT: f32 = 1.45;
pub const SLOPE_STAND: f32 = 0.55;

pub const SCATTER_SEED: u64 = 0x71DE_FA11;
pub const BOULDERS_PER_AREA: f32 = 0.0025;
pub const SPIRES_PER_AREA: f32 = 0.0003;
pub const BOULDER_SIZE: (f32, f32) = (1.0, 5.0);
pub const SPIRE_HEIGHT: (f32, f32) = (6.0, 20.0);
pub const SPIRE_WIDTH: (f32, f32) = (1.6, 4.5);
pub const BEACON_HEIGHT: f32 = 30.0;
pub const BEACON_WIDTH: f32 = 2.5;
pub const BEACON_GLOW: [f32; 3] = [0.5, 0.9, 1.0];
pub const BEACON_GLOW_STRENGTH: f32 = 6.0;
pub const BOULDER_SHADE: f32 = 0.55;
pub const VENTS_PER_AREA: f32 = 0.0004;
pub const VENT_HEIGHT: f32 = 3.0;
pub const VENT_WIDTH: f32 = 5.0;
pub const VENT_GLOW: [f32; 3] = [1.0, 0.42, 0.10];
pub const VENT_GLOW_STRENGTH: f32 = 5.0;
pub const VENT_SHADE: f32 = 0.35;
pub const DRAFT_RADIUS: f32 = 22.0;
pub const DRAFT_LIFT: f32 = 44.0;
pub const HUTS: usize = 7;
pub const HUT_SPREAD: f32 = 0.08;
pub const HUT_KEEP_OUT: f32 = 0.9994;
pub const HUT_WALL: [f32; 4] = [0.80, 0.70, 0.52, 1.0];
pub const HUT_ROOF: [f32; 4] = [0.55, 0.20, 0.16, 1.0];
pub const HUT_SIZE: f32 = 4.0;
pub const HUT_ROOF_HEIGHT: f32 = 3.0;
pub const CRATERS_PER_AREA: f32 = 0.0003;
pub const CRATER_RIM: usize = 12;
pub const CRATER_ANGLE: f32 = 0.12;
pub const CRATER_STONE: (f32, f32) = (2.0, 3.5);
pub const SHELL_ROCKS_PER_AREA: f32 = 0.0007;
pub const SHELL_HEIGHT: (f32, f32) = (14.0, 40.0);
pub const SHELL_SIZE: (f32, f32) = (3.0, 8.0);
pub const RING_ISLANDS: usize = 60;
pub const RING_ISLAND_SIZE: (f32, f32) = (12.0, 22.0);
pub const ICE_SPIRE_FACTOR: f32 = 3.0;
pub const AIR_CONTROL: f32 = 9.0;
pub const REFUEL_SECONDS: f32 = 2.0;
pub const GRIP_RATE: f32 = 14.0;
pub const SPIRE_SHADE: f32 = 1.35;

pub const BALL_MESH: &str = "tidefall_ball";
pub const BALL_SEGMENTS: u32 = 96;

pub const SUN_RADIUS: f32 = 1200.0;
pub const SUN_GRAVITY: f32 = 6.5;
pub const SUN_DAY: f32 = 200.0;
pub const REACH_FACTOR: f32 = 8.0;
pub const REACH_FADE: f32 = 0.15;
pub const MAX_STEP: f32 = 0.05;
pub const SUN_COLOR: [f32; 4] = [1.0, 0.86, 0.55, 1.0];
pub const SUN_GLOW: [f32; 3] = [1.0, 0.72, 0.36];
pub const SUN_GLOW_STRENGTH: f32 = 7.0;
pub const SUN_LIGHT_COLOR: Vec3 = Vec3::new(1.0, 0.94, 0.82);
pub const SUN_LIGHT: f32 = 2.6;

pub const TREES_PER_AREA: f32 = 0.006;
pub const TRUNK_HEIGHT: (f32, f32) = (3.0, 6.0);
pub const TRUNK_WIDTH: f32 = 0.6;
pub const CANOPY_HEIGHT: (f32, f32) = (5.0, 9.0);
pub const CANOPY_WIDTH: (f32, f32) = (3.0, 5.5);
pub const TRUNK_COLOR: [f32; 4] = [0.36, 0.23, 0.12, 1.0];
pub const CANOPY_COLOR: [f32; 4] = [0.08, 0.36, 0.14, 1.0];
pub const PAD_DIRECTION: Vec3 = Vec3::new(-0.55, 0.8, -0.25);
pub const STARLIGHT_COLOR: Vec3 = Vec3::new(0.55, 0.65, 0.9);
pub const STARLIGHT: f32 = 0.35;
pub const PAD_RADIUS: f32 = 14.0;
pub const PAD_HEIGHT: f32 = 0.4;
pub const PAD_CLEARANCE: f32 = 0.995;
pub const PAD_COLOR: [f32; 4] = [0.25, 0.26, 0.28, 1.0];
pub const RING_SPREAD: f32 = 1.9;
pub const RING_THICKNESS: f32 = 0.4;
pub const RING_COLOR: [f32; 4] = [0.82, 0.74, 0.58, 1.0];

pub const FIELD_OF_VIEW_DEGREES: f32 = 68.0;
pub const LOOK_RADIANS_PER_PIXEL: f32 = 0.0009;
pub const GAMEPAD_LOOK: f32 = 2.2;
pub const LIFT_OFF: f32 = 4.0;
pub const LEAN_RATE: f32 = 1.5;
pub const FRICTION: f32 = 6.0;
pub const REST_SPEED: f32 = 0.8;
pub const ARRIVAL_FACTOR: f32 = 1.6;
pub const BRAKE_MARGIN: f32 = 0.9;
pub const ALIGNED_ANGLE: f32 = 0.035;
pub const ARRIVE_SPEED: f32 = 2.0;
pub const AUTOPILOT_TURN: f32 = 1.2;
pub const DESCENT_GAIN: f32 = 0.15;
pub const DESCENT_MIN: f32 = 2.0;
pub const DESCENT_MAX: f32 = 80.0;
pub const DESCENT_RESPONSE: f32 = 1.2;

pub const HILLS_PER_AREA: f32 = 0.00025;
pub const HILL_SIZE: (f32, f32) = (18.0, 70.0);
pub const HILL_FLATTEN: f32 = 0.4;
pub const HILL_SHADE: f32 = 0.9;
pub const MOUNTAINS_PER_AREA: f32 = 0.0001;
pub const MOUNTAIN_HEIGHT: (f32, f32) = (30.0, 110.0);
pub const LAKES_PER_AREA: f32 = 0.00006;
pub const LAKE_SIZE: (f32, f32) = (20.0, 60.0);
pub const LAKE_COLOR: [f32; 4] = [0.12, 0.35, 0.55, 1.0];
pub const POOLS_PER_AREA: f32 = 0.0004;
pub const LAVA_SIZE: (f32, f32) = (6.0, 16.0);
pub const LAVA_COLOR: [f32; 4] = [1.0, 0.5, 0.2, 1.0];
pub const LAVA_GLOW: [f32; 3] = [1.0, 0.35, 0.08];
pub const LAVA_GLOW_STRENGTH: f32 = 4.0;
pub const MARIA_PER_AREA: f32 = 0.0002;
pub const MARIA_SIZE: (f32, f32) = (40.0, 110.0);
pub const MARIA_SHADE: f32 = 0.6;
pub const PILLARS_PER_AREA: f32 = 0.0002;
pub const PILLAR_HEIGHT: (f32, f32) = (20.0, 60.0);
pub const DOMES_PER_AREA: f32 = 0.00012;
pub const DOME_SIZE: (f32, f32) = (60.0, 200.0);
pub const DOME_SHADE: f32 = 1.15;
pub const FROZEN_PER_AREA: f32 = 0.0001;
pub const FROZEN_COLOR: [f32; 4] = [0.82, 0.92, 1.0, 1.0];
pub const VILLAGER_STANDOFF: f32 = 1.3;
pub const VILLAGER_HEIGHT: f32 = 1.5;
pub const VILLAGER_WIDTH: f32 = 0.9;
pub const VILLAGER_HEAD: f32 = 0.42;
pub const VILLAGER_SKIN: [f32; 4] = [0.85, 0.66, 0.50, 1.0];
pub const VILLAGER_COATS: [[f32; 4]; 4] = [
    [0.55, 0.20, 0.16, 1.0],
    [0.20, 0.35, 0.55, 1.0],
    [0.62, 0.55, 0.22, 1.0],
    [0.30, 0.45, 0.28, 1.0],
];
pub const TALK_REACH: f32 = 5.0;
pub const RELIC_HUM: f32 = 260.0;
pub const RELIC_SIZE: f32 = 2.2;
pub const RELICS_PER_WORLD: usize = 3;
pub const SHARD_COLOR: [f32; 4] = [0.7, 1.0, 0.9, 1.0];
pub const SHARD_GLOW: [f32; 3] = [0.3, 1.0, 0.75];
pub const SHARD_GLOW_STRENGTH: f32 = 5.0;
pub const MARKER_SPEECH: usize = 5;
pub const GLOBE_RADIUS: f32 = 58.0;
pub const GLOBE_TILT: f32 = 1.05;
pub const GLOBE_MERIDIANS: usize = 6;
pub const GLOBE_PARALLELS: [f32; 5] = [-1.05, -0.52, 0.0, 0.52, 1.05];
pub const GLOBE_STEPS: usize = 36;
pub const PATH_STEPS: usize = 60;
pub const PATH_STEP: f32 = 0.5;
pub const DRIFT_RING: f32 = 44.0;
pub const DRIFT_FULL: f32 = 60.0;

pub struct Speech {
    pub name: &'static str,
    pub lines: &'static [&'static str],
    pub after: &'static [&'static str],
}

pub const SPEECHES: [Speech; 11] = [
    Speech {
        name: "Marrow",
        lines: &[
            "You are the one who flies the old ship. Good. It still has a little fire in it.",
            "The survey left shards on every world when it went dark. Eighteen, by the count.",
            "Bring them back here and I will tell you where the survey went.",
        ],
        after: &[
            "Every shard, home. The survey did not die. It went past Rime, into the dark.",
            "It is still calling. When you are ready, point the ship past Rime and keep going.",
        ],
    },
    Speech {
        name: "Fen",
        lines: &[
            "Did you see the vents on Cinder? They throw rocks clear into the sky.",
            "I bet a person could ride one.",
        ],
        after: &["You found all of them? Take me next time."],
    },
    Speech {
        name: "Orrin",
        lines: &[
            "Keep the nose up when you set her down. The legs will take the rest.",
            "Rime is all ice. Nothing holds there, so come in slow and kill your drift first.",
        ],
        after: &["She flew well for you. Better than for the survey, at the end."],
    },
    Speech {
        name: "Tansy",
        lines: &[
            "Halo is the big one. Its ring is not dust. There are islands out there.",
            "Pale keeps craters like a bowl keeps rain. Look in the bowls.",
        ],
        after: &["Past Rime the sky is empty on every chart. That is where I would look."],
    },
    Speech {
        name: "Bell",
        lines: &[
            "The waystones tell you where a world hides its shards. Look for the glow.",
            "Rust is hollow, they say. Stones float over it like they forgot how to fall.",
        ],
        after: &["The beacon has burned for the survey since before I was born. Tonight it rests."],
    },
    Speech {
        name: "Cinder waystone",
        lines: &[
            "CINDER SURVEY POST. Three shards lie beside the vents. Stand in a draft to climb.",
        ],
        after: &[],
    },
    Speech {
        name: "Verdant waystone",
        lines: &[
            "VERDANT SURVEY POST. One shard rests at this stone, one in the hamlet,",
            "and one below the tallest peak.",
        ],
        after: &[],
    },
    Speech {
        name: "Pale waystone",
        lines: &["PALE SURVEY POST. Three shards lie at the centers of the craters."],
        after: &[],
    },
    Speech {
        name: "Rust waystone",
        lines: &["RUST SURVEY POST. Three shards lie on the ground below the floating stones."],
        after: &[],
    },
    Speech {
        name: "Halo waystone",
        lines: &[
            "HALO SURVEY POST. Three shards ride the islands of the ring. Land on the islands.",
        ],
        after: &[],
    },
    Speech {
        name: "Rime waystone",
        lines: &[
            "RIME SURVEY POST. Three shards lie at the feet of the tall spires. The ice holds nothing.",
        ],
        after: &[],
    },
];

pub struct Find {
    pub text: &'static str,
}

pub const RELICS: [Find; 18] = [
    Find {
        text: "Survey log 1. We came down on Cinder first. The ground breathes here.",
    },
    Find {
        text: "Survey log 2. The vents lifted our lander like a leaf. We rode them for the joy of it.",
    },
    Find {
        text: "Survey log 3. Something answered our signal from beyond Rime. We turned outward.",
    },
    Find {
        text: "Survey log 4. Verdant, home. The hamlet gave us bread and asked us to stay.",
    },
    Find {
        text: "Survey log 5. We buried a shard at every post so the way could be followed.",
    },
    Find {
        text: "Survey log 6. The peak here is the highest ground in the system. You can see the ring.",
    },
    Find {
        text: "Survey log 7. Pale is quiet. The craters hold the only shadows.",
    },
    Find {
        text: "Survey log 8. We taught the young ones to jump here. The moon barely pulls.",
    },
    Find {
        text: "Survey log 9. The signal was clearer here, out from under Verdant.",
    },
    Find {
        text: "Survey log 10. Rust is a shell. Knock and it rings.",
    },
    Find {
        text: "Survey log 11. The floating stones fall inward if you push them. We did not push twice.",
    },
    Find {
        text: "Survey log 12. The signal repeats a count. Eighteen, then silence, then eighteen again.",
    },
    Find {
        text: "Survey log 13. Halo would crush a ship. We stayed on the ring.",
    },
    Find {
        text: "Survey log 14. The islands turn with the ring. Sleep on one and wake somewhere else.",
    },
    Find {
        text: "Survey log 15. Fuel is low. We will make Rime and no further, unless the dark is kind.",
    },
    Find {
        text: "Survey log 16. Rime. We slid a full turn of the world before the ship caught a spire.",
    },
    Find {
        text: "Survey log 17. The signal is loud here. It comes from past the last light.",
    },
    Find {
        text: "Survey log 18. We are going out. If you read this, follow the count.",
    },
];
