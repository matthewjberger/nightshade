pub mod arsenal;
pub mod bestiary;
pub mod levels;
pub mod loot;
pub mod shape;

use nightshade::prelude::Vec3;

pub const EYE_HEIGHT: f32 = 1.16;
pub const FIELD_OF_VIEW_DEGREES: f32 = 72.0;
pub const CAPSULE_HALF_HEIGHT: f32 = 0.93;
pub const CAPSULE_RADIUS: f32 = 0.50;
pub const STANCE_HEIGHT: f32 = CAPSULE_HALF_HEIGHT + CAPSULE_RADIUS + 0.07;

pub const GRAVITY: f32 = -28.0;
pub const PHYSICS_STEP_SHORTEST: f32 = 1.0 / 360.0;
pub const PHYSICS_STEP_LONGEST: f32 = 1.0 / 60.0;
pub const PHYSICS_SUBSTEPS: u32 = 8;
pub const RUN_SPEED: f32 = 16.0;
pub const ACCELERATION: f32 = 160.0;
pub const JUMP_IMPULSE: f32 = 13.5;
pub const TERMINAL_FALL: f32 = 60.0;

pub const BOB_DISTANCE_PER_CYCLE: f32 = 10.0;
pub const BOB_VERTICAL: f32 = 0.05;
pub const BOB_LATERAL: f32 = 0.028;
pub const BOB_SETTLE_RATE: f32 = 10.0;
pub const LANDING_DIP: f32 = 0.30;
pub const DIP_SPEED: f32 = 9.0;

pub const SWAY_LAG_RATE: f32 = 16.0;
pub const SWAY_TRAVEL: f32 = 0.08;

pub const KICK_RECOVER_RATE: f32 = 8.0;
pub const HURT_KICK: f32 = 0.14;

pub const VIEWMODEL_AT: Vec3 = Vec3::new(0.34, -0.30, -0.98);
pub const VIEWMODEL_SCALE: f32 = 0.80;
pub const VIEWMODEL_SWAY: f32 = 1.5;
pub const VIEWMODEL_RECOVER_RATE: f32 = 10.0;
pub const VIEWMODEL_GLOW: f32 = 0.22;

pub const START_HEALTH: f32 = 100.0;
pub const MAX_HEALTH: f32 = 100.0;
pub const HEALTH_CEILING: f32 = 200.0;
pub const MAX_ARMOR: f32 = 150.0;
pub const ARMOR_SHARE: f32 = 0.55;

pub const START_SHELLS: u32 = 20;
pub const MAX_SHELLS: u32 = 60;
pub const MAX_BULLETS: u32 = 250;
pub const MAX_ROCKETS: u32 = 40;
pub const MAX_CELLS: u32 = 220;

pub const MEDKIT: f32 = 25.0;
pub const STIMPACK: f32 = 60.0;
pub const PLATE: f32 = 50.0;
pub const SHELL_BOX: u32 = 10;
pub const BULLET_BOX: u32 = 45;
pub const ROCKET_BOX: u32 = 5;
pub const CELL_BOX: u32 = 45;

pub const HITSCAN_RANGE: f32 = 120.0;
pub const SHOTGUN_FALLOFF_START: f32 = 14.0;
pub const SHOTGUN_FALLOFF_END: f32 = 38.0;
pub const SHOTGUN_FAR_SHARE: f32 = 0.55;

pub const ROCKET_SPEED: f32 = 34.0;
pub const ROCKET_LIFE: f32 = 6.0;
pub const ROCKET_DIRECT: f32 = 45.0;
pub const ROCKET_BLAST: f32 = 95.0;
pub const ROCKET_RADIUS: f32 = 6.5;
pub const BLAST_SELF_SHARE: f32 = 0.5;
pub const BLAST_PUSH: f32 = 21.0;

pub const PLASMA_SPEED: f32 = 62.0;
pub const PLASMA_LIFE: f32 = 3.0;
pub const PLASMA_DAMAGE: f32 = 20.0;

pub const SHOT_RADIUS: f32 = 0.58;

pub const MODEL_HEIGHT: f32 = 1.9;
pub const HIT_CENTER_SHARE: f32 = 0.55;
pub const HIT_RADIUS_SHARE: f32 = 0.34;
pub const HIT_RADIUS_FLOOR: f32 = 0.78;
pub const HIT_OVERHANG: f32 = 0.15;

pub const TURN_RATE: f32 = 8.0;
pub const STEP_AHEAD: f32 = 0.9;
pub const CHEST_HEIGHT: f32 = 1.3;
pub const GROUND_PROBE_ABOVE: f32 = 3.2;
pub const GROUND_PROBE_DEPTH: f32 = 12.0;
pub const DROP_LIMIT: f32 = 2.0;
pub const SLIDE_SHARE: f32 = 0.9;
pub const DRIFT_RATE: f32 = 0.45;
pub const REEL_TIME: f32 = 0.24;
pub const REEL_DAMAGE: f32 = 30.0;
pub const CORPSE_TIME: f32 = 3.0;

pub const PICKUP_RADIUS: f32 = 2.3;
pub const PICKUP_SPIN: f32 = 1.5;
pub const PICKUP_BOB: f32 = 0.2;
pub const PICKUP_LIFT: f32 = 0.85;
pub const LOOT_SCALE: f32 = 1.0;
pub const GUN_PICKUP_SCALE: f32 = 2.2;
pub const CRATE_SIZE: Vec3 = Vec3::new(1.4, 1.4, 1.4);

pub const DOOR_REACH: f32 = 5.0;
pub const DOOR_RATE: f32 = 1.4;
pub const LEVER_HEALTH: f32 = 1.0;
pub const SECRET_HEALTH: f32 = 1.0;
pub const EXIT_REACH: f32 = 3.2;
pub const EXIT_BEACON: f32 = 7.0;

pub const FLASH_FADE: f32 = 3.4;
pub const FLASH_ALPHA: f32 = 0.22;
pub const HIT_MARK_TIME: f32 = 0.18;
pub const LOW_HEALTH: f32 = 30.0;
pub const HAZARD_TICK: f32 = 0.28;

pub const ANISOTROPY: u16 = 8;
pub const LAMP_SHARE: f32 = 0.45;
pub const FILL_BASE: f32 = 20.0;
pub const FILL_PER_AREA: f32 = 0.06;
pub const LIGHT_SHARE: f32 = 0.5;
pub const FOG_COLOR: [f32; 3] = [0.10, 0.09, 0.12];
pub const FOG_START: f32 = 26.0;
pub const FOG_END: f32 = 110.0;

pub const MUZZLE_SLOT: u32 = 1;
pub const SPARK_SLOT: u32 = 2;
pub const SMOKE_SLOT: u32 = 3;
pub const SCORCH_SLOT: u32 = 4;

pub const BRIEF_TIME: f32 = 4.0;
pub const MESSAGE_TIME: f32 = 2.4;
pub const CLEAR_DELAY: f32 = 2.6;
pub const DEATH_DELAY: f32 = 3.0;

pub const LOBBY_AT: Vec3 = Vec3::new(0.0, 200.0, 0.0);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Fire,
    GunOne,
    GunTwo,
    GunThree,
    GunFour,
    PreviousGun,
    NextGun,
    Pause,
    Resume,
    Confirm,
}

impl nightshade_input::Action for Ask {
    fn all() -> &'static [Self] {
        &[
            Self::Fire,
            Self::GunOne,
            Self::GunTwo,
            Self::GunThree,
            Self::GunFour,
            Self::PreviousGun,
            Self::NextGun,
            Self::Pause,
            Self::Resume,
            Self::Confirm,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Fire => "fire",
            Self::GunOne => "gun one",
            Self::GunTwo => "gun two",
            Self::GunThree => "gun three",
            Self::GunFour => "gun four",
            Self::PreviousGun => "previous gun",
            Self::NextGun => "next gun",
            Self::Pause => "pause",
            Self::Resume => "resume",
            Self::Confirm => "confirm",
        }
    }

    fn default_bindings(&self) -> &'static [nightshade_input::Binding] {
        use nightshade_gamepad::contract::{GamepadAxis, GamepadButton};
        use nightshade_input::{Binding, Pointer};
        use nightshade_platform::KeyCode;
        match self {
            Self::Fire => &[
                Binding::Pointer(Pointer::Left),
                Binding::Trigger(GamepadAxis::RightTrigger),
                Binding::Pad(GamepadButton::RightTrigger),
            ],
            Self::GunOne => &[Binding::Key(KeyCode::Digit1)],
            Self::GunTwo => &[Binding::Key(KeyCode::Digit2)],
            Self::GunThree => &[Binding::Key(KeyCode::Digit3)],
            Self::GunFour => &[Binding::Key(KeyCode::Digit4)],
            Self::PreviousGun => &[
                Binding::Key(KeyCode::KeyQ),
                Binding::Pad(GamepadButton::LeftBumper),
            ],
            Self::NextGun => &[
                Binding::Key(KeyCode::KeyE),
                Binding::Pad(GamepadButton::RightBumper),
            ],
            Self::Pause => &[
                Binding::Key(KeyCode::Escape),
                Binding::Pad(GamepadButton::Start),
            ],
            Self::Resume => &[
                Binding::Key(KeyCode::Escape),
                Binding::Pad(GamepadButton::Start),
                Binding::Pad(GamepadButton::East),
            ],
            Self::Confirm => &[
                Binding::Key(KeyCode::Enter),
                Binding::Key(KeyCode::Space),
                Binding::Pad(GamepadButton::South),
            ],
        }
    }
}
