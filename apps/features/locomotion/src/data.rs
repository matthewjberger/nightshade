pub const CHARACTERS: [(&str, &[u8]); 4] = [
    (
        "Y Bot",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/gltf/y_bot_locomotion.glb"
        )),
    ),
    (
        "X Bot",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/gltf/x_bot.glb"
        )),
    ),
    (
        "Ninja",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/gltf/ninja.glb"
        )),
    ),
    (
        "Swat",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/gltf/swat.glb"
        )),
    ),
];

pub const IDLE: &str = "idle";
pub const WALKING: &str = "walking";
pub const RUNNING: &str = "running";
pub const LEFT_STRAFE_WALKING: &str = "left_strafe_walking";
pub const RIGHT_STRAFE_WALKING: &str = "right_strafe_walking";
pub const LEFT_STRAFE: &str = "left_strafe";
pub const RIGHT_STRAFE: &str = "right_strafe";

pub const FORWARD: &str = "forward";
pub const LATERAL: &str = "lateral";

pub const WALK_SPEED: f32 = 1.67;
pub const RUN_SPEED: f32 = 4.31;

pub const ACCELERATION: f32 = 12.0;
pub const DECELERATION: f32 = 16.0;
pub const TURN_SHARPNESS: f32 = 10.0;
pub const STRAFE_BIAS: f32 = 1.2;
pub const PARAMETER_SHARPNESS: f32 = 9.0;

pub const CAMERA_DISTANCE: f32 = 5.5;
pub const CAMERA_PITCH: f32 = 0.28;
pub const CAMERA_HEIGHT: f32 = 1.1;

pub const BLOCKS: [([f32; 3], [f32; 3]); 23] = [
    ([8.24, 0.00, 2.10], [0.80, 0.64, 0.80]),
    ([4.34, 0.00, 7.31], [1.25, 1.00, 1.25]),
    ([-2.10, 0.00, 8.24], [1.70, 1.36, 1.70]),
    ([-7.31, 0.00, 4.34], [0.80, 0.64, 0.80]),
    ([-8.24, 0.00, -2.10], [1.25, 1.00, 1.25]),
    ([-4.34, 0.00, -7.31], [1.70, 1.36, 1.70]),
    ([2.10, 0.00, -8.24], [0.80, 0.64, 0.80]),
    ([7.31, 0.00, -4.34], [1.25, 1.00, 1.25]),
    ([-4.00, 0.00, -16.00], [6.00, 0.90, 0.60]),
    ([4.00, 0.00, -22.00], [6.00, 0.90, 0.60]),
    ([-16.00, 0.00, 4.00], [0.60, 1.10, 6.00]),
    ([-22.00, 0.00, -4.00], [0.60, 1.10, 6.00]),
    ([18.00, 0.00, 10.00], [4.00, 1.40, 0.60]),
    ([12.00, 0.00, 16.00], [0.60, 1.40, 4.00]),
    ([-9.16, 0.00, -18.16], [1.29, 0.97, 1.29]),
    ([-22.23, 0.00, 1.87], [1.03, 0.77, 1.03]),
    ([-22.98, 0.00, 0.39], [0.73, 0.55, 0.73]),
    ([-3.45, 0.00, -22.37], [0.78, 0.58, 0.78]),
    ([-3.93, 0.00, 17.00], [0.81, 0.61, 0.81]),
    ([-14.39, 0.00, 6.63], [1.55, 1.16, 1.55]),
    ([24.77, 0.00, -23.58], [1.47, 1.10, 1.47]),
    ([-10.94, 0.00, -18.50], [0.81, 0.61, 0.81]),
    ([-9.96, 0.00, 16.44], [0.86, 0.65, 0.86]),
];

pub const PICKUPS: [[f32; 2]; 18] = [
    [5.00, 0.00],
    [8.83, 3.21],
    [10.57, 8.87],
    [9.10, 15.76],
    [3.92, 22.26],
    [-0.87, 4.92],
    [-4.70, 8.14],
    [-10.57, 8.87],
    [-17.10, 6.22],
    [-22.60, 0.00],
    [-4.70, -1.71],
    [-7.20, -6.04],
    [-6.90, -11.95],
    [-3.16, -17.92],
    [3.92, -22.26],
    [2.50, -4.33],
    [7.20, -6.04],
    [12.97, -4.72],
];

pub const BLOCK_COLOR: [f32; 4] = [0.46, 0.42, 0.36, 1.0];
pub const PICKUP_COLOR: [f32; 4] = [1.0, 0.82, 0.25, 1.0];
pub const PICKUP_GLOW: [f32; 3] = [0.65, 0.45, 0.05];
pub const PICKUP_HEIGHT: f32 = 0.95;
pub const PICKUP_SIZE: f32 = 0.34;
pub const PICKUP_SPIN: f32 = 1.4;
pub const PICKUP_BOB: f32 = 0.16;
pub const PICKUP_REACH: f32 = 0.95;
pub const BODY_RADIUS: f32 = 0.38;
pub const GROUND_EXTENT: f32 = 60.0;
pub const GROUND_TEXTURE: &str = "locomotion_ground";
pub const GROUND_TEXTURE_SIDE: u32 = 512;
pub const GROUND_TILES: f32 = 16.0;
pub const GROUND_SOIL: [f32; 3] = [0.21, 0.2, 0.18];
pub const GROUND_MOSS: [f32; 3] = [0.19, 0.24, 0.17];
pub const GROUND_STONE: [f32; 3] = [0.33, 0.33, 0.34];
pub const GROUND_PEBBLE: [f32; 3] = [0.44, 0.43, 0.4];
pub const PILLAR_RING_RADIUS: f32 = 14.0;
pub const PILLAR_COUNT: usize = 10;

pub const LEFT_TURN: &str = "left_turn";
pub const RIGHT_TURN: &str = "right_turn";
pub const LEFT_TURN_WIDE: &str = "left_turn_90";
pub const RIGHT_TURN_WIDE: &str = "right_turn_90";

pub const TURN_STILL_SPEED: f32 = 1.2;
pub const TURN_ENTER: f32 = 1.05;
pub const TURN_WIDE: f32 = 1.75;
pub const TURN_FADE: f32 = 10.0;

pub const JUMP: &str = "jump";

pub const JUMP_START: f32 = 0.30;
pub const JUMP_TAKEOFF: f32 = 0.75;
pub const JUMP_LAND: f32 = 1.30;
pub const JUMP_RECOVERY_END: f32 = 1.46;
pub const JUMP_MOVING_SPEED: f32 = 0.6;
pub const JUMP_ANTICIPATION_SPEED: f32 = 1.6;
pub const JUMP_AIR_SPEED: f32 = 0.85;
pub const JUMP_RECOVERY_SPEED: f32 = 1.4;

pub const JUMP_HEIGHT_SCALE: f32 = 1.2;

pub const JUMP_FADE_IN: f32 = 12.0;
pub const JUMP_FADE_OUT: f32 = 16.0;
pub const AIR_CONTROL: f32 = 0.45;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Move {
    Walk,
    Slow,
    LockFacing,
}

impl nightshade_input::Action for Move {
    fn all() -> &'static [Self] {
        &[Self::Walk, Self::Slow, Self::LockFacing]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Walk => "walk",
            Self::Slow => "slow",
            Self::LockFacing => "lock facing",
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
                Binding::Stick(Stick::Left),
            ],
            Self::Slow => &[
                Binding::Key(KeyCode::ShiftLeft),
                Binding::Key(KeyCode::ShiftRight),
            ],
            Self::LockFacing => &[
                Binding::Key(KeyCode::AltLeft),
                Binding::Key(KeyCode::AltRight),
                Binding::Key(KeyCode::ControlLeft),
                Binding::Key(KeyCode::ControlRight),
                Binding::Pointer(Pointer::Right),
                Binding::Pad(GamepadButton::LeftBumper),
                Binding::Pad(GamepadButton::RightBumper),
                Binding::Trigger(GamepadAxis::LeftTrigger),
                Binding::Trigger(GamepadAxis::RightTrigger),
            ],
        }
    }
}
