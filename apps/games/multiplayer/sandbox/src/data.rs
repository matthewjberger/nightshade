pub mod assets;
pub mod wire;

use nalgebra_glm::Vec3;

pub const CLIP_NAMES: [&str; 5] = ["idle", "walk", "sprint", "sit", "static"];
pub const CLIP_IDLE: usize = 0;
pub const CLIP_WALK: usize = 1;
pub const CLIP_SPRINT: usize = 2;
pub const CLIP_CROUCH: usize = 3;
pub const CLIP_STATIC: usize = 4;

pub const ANIMAL_CLIP_NAMES: [&str; 2] = ["walk", "run"];

pub const CHARACTERS: [&str; 18] = [
    "blocky-character-a",
    "blocky-character-b",
    "blocky-character-c",
    "blocky-character-d",
    "blocky-character-e",
    "blocky-character-f",
    "blocky-character-g",
    "blocky-character-h",
    "blocky-character-i",
    "blocky-character-j",
    "blocky-character-k",
    "blocky-character-l",
    "blocky-character-m",
    "blocky-character-n",
    "blocky-character-o",
    "blocky-character-p",
    "blocky-character-q",
    "blocky-character-r",
];

pub struct Prop {
    pub model: &'static str,
    pub position: Vec3,
    pub yaw: f32,
    pub scale: f32,
    pub block: Option<Block>,
}

pub struct Block {
    pub half: Vec3,
    pub lift: f32,
    pub pitch: f32,
}

const UNIT: f32 = 2.0;
const QUARTER: f32 = std::f32::consts::FRAC_PI_2;
const HALF_TURN: f32 = std::f32::consts::PI;
const RAMP_PITCH: f32 = -std::f32::consts::FRAC_PI_4;

const fn block(half: Vec3, lift: f32) -> Option<Block> {
    Some(Block {
        half,
        lift,
        pitch: 0.0,
    })
}

const fn ramp(half: Vec3, lift: f32) -> Option<Block> {
    Some(Block {
        half,
        lift,
        pitch: RAMP_PITCH,
    })
}

fn slope(model: &'static str, x: f32, z: f32, yaw: f32) -> Prop {
    Prop {
        model,
        position: Vec3::new(x, 0.0, z),
        yaw,
        scale: UNIT,
        block: ramp(Vec3::new(UNIT * 0.5, 0.12, UNIT * 0.72), UNIT * 0.5),
    }
}

fn cube(model: &'static str, x: f32, y: f32, z: f32, height: f32) -> Prop {
    Prop {
        model,
        position: Vec3::new(x, y, z),
        yaw: 0.0,
        scale: UNIT,
        block: block(
            Vec3::new(UNIT * 0.5, height * 0.5, UNIT * 0.5),
            height * 0.5,
        ),
    }
}

fn wall(model: &'static str, x: f32, z: f32, yaw: f32) -> Prop {
    let half = if yaw.abs() < 0.01 || (yaw - HALF_TURN).abs() < 0.01 {
        Vec3::new(UNIT * 0.15, UNIT * 0.5, UNIT * 0.5)
    } else {
        Vec3::new(UNIT * 0.5, UNIT * 0.5, UNIT * 0.15)
    };
    Prop {
        model,
        position: Vec3::new(x, 0.0, z),
        yaw,
        scale: UNIT,
        block: block(half, UNIT * 0.5),
    }
}

fn scenery(model: &'static str, x: f32, z: f32, yaw: f32, scale: f32) -> Prop {
    Prop {
        model,
        position: Vec3::new(x, 0.0, z),
        yaw,
        scale,
        block: None,
    }
}

pub fn course() -> Vec<Prop> {
    let mut props = vec![cube("shape-cube", 8.0, 0.0, -6.0, UNIT)];
    props.push(cube("shape-cube", 8.0, UNIT, -6.0, UNIT));
    props.push(slope("shape-slope", 8.0, -6.0 + UNIT, 0.0));

    props.push(cube("shape-cube-half", -8.0, 0.0, -6.0, UNIT * 0.5));
    props.push(slope("stairs", -8.0, -6.0 + UNIT, 0.0));

    props.push(cube("shape-cube", 0.0, 0.0, -14.0, UNIT));
    props.push(cube("shape-cube", UNIT, 0.0, -14.0, UNIT));
    props.push(cube("shape-cube", -UNIT, 0.0, -14.0, UNIT));
    props.push(slope("stairs", 0.0, -14.0 + UNIT, 0.0));
    props.push(slope("shape-slope", UNIT, -14.0 + UNIT, 0.0));

    props.push(wall("wall", -6.0, 10.0, QUARTER));
    props.push(wall("wall-doorway", -4.0, 10.0, QUARTER));
    props.push(wall("wall-window-large", -2.0, 10.0, QUARTER));
    props.push(wall("wall", 0.0, 10.0, QUARTER));
    props.push(wall("wall-corner", 2.0, 10.0, QUARTER));
    props.push(wall("wall", 2.0, 12.0, 0.0));
    props.push(wall("wall-window-large", 2.0, 14.0, 0.0));

    for step in 0..4 {
        let x = -9.0 + step as f32 * 3.0;
        props.push(Prop {
            model: "column",
            position: Vec3::new(x, 0.0, 4.0),
            yaw: 0.0,
            scale: UNIT,
            block: block(Vec3::new(0.4, UNIT * 0.5, 0.4), UNIT * 0.5),
        });
    }

    props.push(cube("crate", 12.0, 0.0, 2.0, UNIT * 0.5));
    props.push(cube("crate-color", 12.0, UNIT * 0.5, 2.0, UNIT * 0.5));
    props.push(cube("crate", 13.5, 0.0, 3.2, UNIT * 0.5));
    props.push(cube("shape-cylinder", -12.0, 0.0, -2.0, UNIT));
    props.push(cube("shape-hexagon", -12.0, 0.0, 2.0, UNIT));

    props.push(scenery("flag", 0.0, -18.0, 0.0, UNIT));
    props.push(scenery("flag", 8.0, -9.0, HALF_TURN, UNIT));
    props.push(scenery("ladder", 9.1, -6.0, QUARTER, UNIT));
    props.push(scenery("target-a-round", -6.0, -10.0, 0.0, UNIT));
    props.push(scenery("pipe", 14.0, -4.0, 0.0, UNIT));
    props.push(scenery("pipe-corner", 14.0, -2.0, 0.0, UNIT));
    props.push(scenery("button-floor-round", 0.0, 6.0, 0.0, UNIT));

    props
}

pub const KINDS: [&str; 1] = ["target-a-round"];

pub const KIND_MARK: u8 = 0;

pub const HERD: [(&str, f32, f32); 3] = [
    ("animal-horse", 20.0, 1.9),
    ("animal-bison", 24.0, 1.5),
    ("animal-dog", 17.0, 2.6),
];

pub const HERD_MEMBERS: usize = 6;
pub const HERD_SPEED: f32 = 2.2;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Walk,
    Jump,
    Crouch,
    Fire,
    Pause,
    Quit,
}

impl nightshade_input::Action for Ask {
    fn all() -> &'static [Self] {
        &[
            Self::Walk,
            Self::Jump,
            Self::Crouch,
            Self::Fire,
            Self::Pause,
            Self::Quit,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Walk => "walk",
            Self::Jump => "jump",
            Self::Crouch => "crouch",
            Self::Fire => "fire",
            Self::Pause => "pause",
            Self::Quit => "quit",
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
                Binding::Stick(Stick::Left),
            ],
            Self::Jump => &[
                Binding::Key(KeyCode::Space),
                Binding::Pad(GamepadButton::South),
            ],
            Self::Crouch => &[
                Binding::Key(KeyCode::ControlLeft),
                Binding::Key(KeyCode::KeyC),
                Binding::Pad(GamepadButton::East),
            ],
            Self::Fire => &[Binding::Pointer(Pointer::Left)],
            Self::Pause => &[
                Binding::Key(KeyCode::Escape),
                Binding::Pad(GamepadButton::Start),
            ],
            Self::Quit => &[Binding::Key(KeyCode::KeyQ)],
        }
    }
}
