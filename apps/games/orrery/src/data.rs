pub struct BodyDefinition {
    pub name: &'static str,
    pub parent: Option<usize>,
    pub orbit_radius: f32,
    pub orbit_period: f32,
    pub orbit_phase: f32,
    pub orbit_tilt: f32,
    pub orbit_stretch: f32,
    pub radius: f32,
    pub inner_radius: f32,
    pub aperture: f32,
    pub gravity: f32,
    pub drag: f32,
    pub drawn: bool,
    pub color: [f32; 4],
    pub emissive: [f32; 3],
    pub roughness: f32,
}

pub const STAR: usize = 0;
pub const CINDER: usize = 1;
pub const PLANET: usize = 2;
pub const MOON: usize = 3;
pub const HOLLOW: usize = 4;
pub const ANCHOR: usize = 5;
pub const BELL: usize = 6;
pub const CLASP: usize = 7;
pub const LANTERN: usize = 9;

pub const BODIES: &[BodyDefinition] = &[
    BodyDefinition {
        name: "Ember",
        parent: None,
        orbit_radius: 0.0,
        orbit_period: 1.0,
        orbit_phase: 0.0,
        orbit_tilt: 0.0,
        orbit_stretch: 1.0,
        radius: 8.0,
        inner_radius: 0.0,
        aperture: 1.0,
        gravity: 1880.0,
        drag: 0.0,
        drawn: true,
        color: [1.0, 0.86, 0.55, 1.0],
        emissive: [6.0, 4.4, 2.0],
        roughness: 1.0,
    },
    BodyDefinition {
        name: "Cinder",
        parent: Some(STAR),
        orbit_radius: 55.0,
        orbit_period: 172.0,
        orbit_phase: 1.4,
        orbit_tilt: 0.1,
        orbit_stretch: 1.0,
        radius: 3.0,
        inner_radius: 0.0,
        aperture: 1.0,
        gravity: 54.0,
        drag: 0.0,
        drawn: true,
        color: [0.42, 0.22, 0.16, 1.0],
        emissive: [0.12, 0.03, 0.0],
        roughness: 0.95,
    },
    BodyDefinition {
        name: "Kettle",
        parent: Some(STAR),
        orbit_radius: 140.0,
        orbit_period: 240.0,
        orbit_phase: 0.6,
        orbit_tilt: 0.0,
        orbit_stretch: 1.0,
        radius: 5.0,
        inner_radius: 0.0,
        aperture: 1.0,
        gravity: 250.0,
        drag: 0.0,
        drawn: true,
        color: [0.24, 0.46, 0.38, 1.0],
        emissive: [0.0, 0.0, 0.0],
        roughness: 0.92,
    },
    BodyDefinition {
        name: "Pebble",
        parent: Some(PLANET),
        orbit_radius: 30.0,
        orbit_period: 65.0,
        orbit_phase: 2.1,
        orbit_tilt: 0.32,
        orbit_stretch: 1.0,
        radius: 2.0,
        inner_radius: 0.0,
        aperture: 1.0,
        gravity: 14.0,
        drag: 0.0,
        drawn: true,
        color: [0.58, 0.55, 0.5, 1.0],
        emissive: [0.0, 0.0, 0.0],
        roughness: 0.95,
    },
    BodyDefinition {
        name: "Hollow",
        parent: Some(STAR),
        orbit_radius: 215.0,
        orbit_period: 420.0,
        orbit_phase: 3.4,
        orbit_tilt: 0.12,
        orbit_stretch: 1.0,
        radius: 12.0,
        inner_radius: 9.4,
        aperture: 0.86,
        gravity: 1152.0,
        drag: 0.0,
        drawn: false,
        color: [0.3, 0.31, 0.42, 1.0],
        emissive: [0.02, 0.02, 0.05],
        roughness: 0.85,
    },
    BodyDefinition {
        name: "Anchor",
        parent: Some(STAR),
        orbit_radius: 300.0,
        orbit_period: 640.0,
        orbit_phase: 5.0,
        orbit_tilt: 0.06,
        orbit_stretch: 1.0,
        radius: 0.0,
        inner_radius: 0.0,
        aperture: 1.0,
        gravity: 0.0,
        drag: 0.0,
        drawn: false,
        color: [0.0, 0.0, 0.0, 1.0],
        emissive: [0.0, 0.0, 0.0],
        roughness: 1.0,
    },
    BodyDefinition {
        name: "Bell",
        parent: Some(ANCHOR),
        orbit_radius: 7.0,
        orbit_period: 22.0,
        orbit_phase: 0.0,
        orbit_tilt: 0.0,
        orbit_stretch: 1.0,
        radius: 2.2,
        inner_radius: 0.0,
        aperture: 1.0,
        gravity: 14.5,
        drag: 0.0,
        drawn: true,
        color: [0.72, 0.66, 0.48, 1.0],
        emissive: [0.035, 0.03, 0.022],
        roughness: 0.9,
    },
    BodyDefinition {
        name: "Clasp",
        parent: Some(ANCHOR),
        orbit_radius: 7.0,
        orbit_period: 22.0,
        orbit_phase: std::f32::consts::PI,
        orbit_tilt: 0.0,
        orbit_stretch: 1.0,
        radius: 2.2,
        inner_radius: 0.0,
        aperture: 1.0,
        gravity: 14.5,
        drag: 0.0,
        drawn: true,
        color: [0.5, 0.54, 0.62, 1.0],
        emissive: [0.024, 0.026, 0.03],
        roughness: 0.9,
    },
    BodyDefinition {
        name: "Drift",
        parent: Some(STAR),
        orbit_radius: 400.0,
        orbit_period: 900.0,
        orbit_phase: 2.6,
        orbit_tilt: 0.04,
        orbit_stretch: 1.0,
        radius: 16.0,
        inner_radius: 0.0,
        aperture: 1.0,
        gravity: 420.0,
        drag: 1.3,
        drawn: true,
        color: [0.36, 0.3, 0.5, 0.55],
        emissive: [0.05, 0.03, 0.12],
        roughness: 1.0,
    },
    BodyDefinition {
        name: "Lantern",
        parent: Some(STAR),
        orbit_radius: 260.0,
        orbit_period: 520.0,
        orbit_phase: 0.2,
        orbit_tilt: 0.5,
        orbit_stretch: 1.9,
        radius: 1.6,
        inner_radius: 0.0,
        aperture: 1.0,
        gravity: 3.8,
        drag: 0.0,
        drawn: true,
        color: [0.16, 0.18, 0.22, 1.0],
        emissive: [0.02, 0.06, 0.08],
        roughness: 0.7,
    },
];

pub struct BeaconDefinition {
    pub body: usize,
    pub anchor: [f32; 3],
    pub inside: bool,
}

pub const BEACONS: &[BeaconDefinition] = &[
    BeaconDefinition {
        body: PLANET,
        anchor: [0.45, 0.35, -0.82],
        inside: false,
    },
    BeaconDefinition {
        body: CINDER,
        anchor: [0.5, 0.8, 0.33],
        inside: false,
    },
    BeaconDefinition {
        body: MOON,
        anchor: [-0.62, 0.24, 0.75],
        inside: false,
    },
    BeaconDefinition {
        body: HOLLOW,
        anchor: [0.2, -0.86, 0.47],
        inside: true,
    },
    BeaconDefinition {
        body: BELL,
        anchor: [0.8, 0.5, 0.33],
        inside: false,
    },
    BeaconDefinition {
        body: CLASP,
        anchor: [-0.3, 0.62, -0.72],
        inside: false,
    },
    BeaconDefinition {
        body: LANTERN,
        anchor: [0.1, 0.95, 0.3],
        inside: false,
    },
];

pub const BEACON_HEIGHT: f32 = 1.6;
pub const BEACON_REACH: f32 = 3.4;
pub const BEACON_COLOR: [f32; 4] = [0.25, 0.95, 0.9, 1.0];
pub const BEACON_EMISSIVE: [f32; 3] = [0.4, 3.0, 2.8];
pub const BEACON_FOUND_EMISSIVE: [f32; 3] = [3.0, 2.4, 0.5];

pub const TOWER_HEIGHT: f32 = 4.6;
pub const TOWER_WIDTH: f32 = 0.9;
pub const TOWER_COLOR: [f32; 4] = [0.2, 0.19, 0.22, 1.0];

pub const SHELL_POINTS: usize = 260;
pub const SHELL_STONE: f32 = 1.7;

pub const RUBBLE_COUNT: usize = 110;
pub const RUBBLE_INNER: f32 = 21.0;
pub const RUBBLE_OUTER: f32 = 31.0;
pub const RUBBLE_SIZE: f32 = 0.9;
pub const RUBBLE_COLOR: [f32; 4] = [0.42, 0.4, 0.46, 1.0];

pub const STAR_LIGHT_COLOR: [f32; 3] = [1.0, 0.94, 0.82];
pub const STAR_LIGHT_INTENSITY: f32 = 22000.0;
pub const STAR_LIGHT_RANGE: f32 = 1400.0;

pub const SHIP_HULL: [f32; 3] = [0.9, 0.62, 2.0];
pub const SHIP_NOSE: [f32; 3] = [1.1, 1.3, 1.1];
pub const SHIP_WING: [f32; 3] = [1.1, 0.16, 0.72];
pub const SHIP_RADIUS: f32 = 1.5;
pub const SHIP_MAIN_THRUST: f32 = 18.0;
pub const SHIP_SIDE_THRUST: f32 = 10.0;
pub const SHIP_ROLL_RATE: f32 = 1.9;
pub const SHIP_LOOK_RATE: f32 = 0.0032;
pub const SHIP_START_OFFSET: [f32; 3] = [18.0, 5.0, -20.0];
pub const LANDING_SPEED: f32 = 9.0;

pub const WALKER_HEIGHT: f32 = 1.8;
pub const WALKER_SCALE: [f32; 3] = [0.7, 1.8, 0.7];
pub const WALK_SPEED: f32 = 6.0;
pub const JUMP_SPEED: f32 = 6.5;
pub const BOARD_REACH: f32 = 4.5;
pub const FOOT_LOOK_RATE: f32 = 0.0032;
pub const PITCH_LIMIT: f32 = 1.4;

pub const CHASE_DISTANCE: f32 = 11.0;
pub const CHASE_HEIGHT: f32 = 3.2;
pub const COCKPIT_FORWARD: f32 = 2.7;
pub const COCKPIT_HEIGHT: f32 = 0.35;
pub const FOOT_DISTANCE: f32 = 5.0;
pub const FOOT_HEIGHT: f32 = 2.0;
pub const CAMERA_LAG: f32 = 12.0;

pub const CAPTURE_FRAME: u64 = 60;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Walk,
    Jump,
    Board,
    Disembark,
    Pause,
    NextTarget,
    Autopilot,
    MatchSpeed,
    SwitchView,
    Thrust,
    Lift,
    Roll,
}

impl nightshade_input::Action for Ask {
    fn all() -> &'static [Self] {
        &[
            Self::Walk,
            Self::Jump,
            Self::Board,
            Self::Disembark,
            Self::Pause,
            Self::NextTarget,
            Self::Autopilot,
            Self::MatchSpeed,
            Self::SwitchView,
            Self::Thrust,
            Self::Lift,
            Self::Roll,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Walk => "walk",
            Self::Jump => "jump",
            Self::Board => "board",
            Self::Disembark => "disembark",
            Self::Pause => "pause",
            Self::NextTarget => "next target",
            Self::Autopilot => "autopilot",
            Self::MatchSpeed => "match speed",
            Self::SwitchView => "switch view",
            Self::Thrust => "thrust",
            Self::Lift => "lift",
            Self::Roll => "roll",
        }
    }

    fn default_bindings(&self) -> &'static [nightshade_input::Binding] {
        use nightshade_input::{Binding, Press};
        use nightshade_platform::KeyCode;
        match self {
            Self::Walk | Self::Thrust => &[Binding::Keys {
                up: KeyCode::KeyW,
                down: KeyCode::KeyS,
                left: KeyCode::KeyA,
                right: KeyCode::KeyD,
            }],
            Self::Jump => &[Binding::Key(KeyCode::Space)],
            Self::Board | Self::Disembark => &[Binding::Key(KeyCode::KeyF)],
            Self::Pause => &[Binding::Key(KeyCode::KeyP)],
            Self::NextTarget => &[Binding::Key(KeyCode::KeyT)],
            Self::Autopilot => &[Binding::Key(KeyCode::KeyV)],
            Self::MatchSpeed => &[Binding::Key(KeyCode::KeyX)],
            Self::SwitchView => &[Binding::Key(KeyCode::KeyC)],
            Self::Lift => &[Binding::Pair {
                positive: Press::Key(KeyCode::Space),
                negative: Press::Key(KeyCode::ControlLeft),
            }],
            Self::Roll => &[Binding::Pair {
                positive: Press::Key(KeyCode::KeyQ),
                negative: Press::Key(KeyCode::KeyE),
            }],
        }
    }
}
