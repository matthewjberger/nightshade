pub const EYE_HEIGHT: f32 = 1.66;
pub const FIELD_OF_VIEW: f32 = 68.0;
pub const WALK_SPEED: f32 = 2.5;
pub const SPRINT_SCALE: f32 = 1.85;
pub const CROUCH_SCALE: f32 = 0.42;

pub const REACH: f32 = 2.2;
pub const HOLD_NEAR: f32 = 0.9;
pub const HOLD_FAR: f32 = 2.4;
pub const HOLD_STEP: f32 = 0.35;
pub const PULL_STIFFNESS: f32 = 26.0;
pub const PULL_DAMPING: f32 = 5.5;
pub const PULL_CEILING: f32 = 34.0;
pub const FIXTURE_SCALE: f32 = 2.4;
pub const THROW_SPEED: f32 = 8.5;
pub const THROW_LIFT: f32 = 1.4;
pub const LOOSE_SPIN_DAMPING: f32 = 0.35;
pub const HELD_SPIN_DAMPING: f32 = 9.0;

pub const CELL_FULL: f32 = 260.0;
pub const CELL_DRAIN: f32 = 1.55;
pub const CELL_LOW: f32 = 55.0;
pub const CELL_BROWNOUT: f32 = 0.34;
pub const CELL_REFILL: f32 = 150.0;

pub const FOOTFALL_INTERVAL: f32 = 0.52;
pub const NOISE_FOOTFALL: f32 = 0.5;
pub const NOISE_SPRINT: f32 = 1.0;
pub const NOISE_CROUCH: f32 = 0.16;
pub const NOISE_THROW: f32 = 0.7;
pub const NOISE_DOOR: f32 = 0.9;

pub const WOLF_SIGHT: f32 = 15.0;
pub const WOLF_HEARING: f32 = 22.0;
pub const WOLF_HALF_ANGLE: f32 = 1.0;
pub const WOLF_PROWL: f32 = 1.1;
pub const WOLF_CHARGE: f32 = 4.6;
pub const WOLF_TURN: f32 = 4.0;
pub const WOLF_REACH: f32 = 1.25;
pub const WOLF_BITE: f32 = 34.0;
pub const WOLF_BITE_INTERVAL: f32 = 1.1;
pub const WOLF_PATIENCE: f32 = 9.0;
pub const WOLF_ARRIVAL: f32 = 0.8;
pub const WOLF_SETTLE: f32 = 3.0;
pub const WOLF_SCALE: f32 = 0.34;
pub const WOLF_RADIUS: f32 = 0.42;

pub const MAX_HEALTH: f32 = 100.0;
pub const NOCLIP_SPEED: f32 = 6.0;
pub const MESSAGE_SECONDS: f32 = 4.5;
pub const EXIT_RADIUS: f32 = 1.6;

pub const DRAWER_TRAVEL: f32 = 0.52;
pub const DOOR_SWING: f32 = 1.85;

pub fn flashlight_settings() -> nightshade_flashlight::FlashlightSettings {
    nightshade_flashlight::FlashlightSettings {
        full: CELL_FULL,
        low: CELL_LOW,
        brownout: CELL_BROWNOUT,
    }
}

pub fn senses_settings() -> nightshade_senses::SensesSettings {
    nightshade_senses::SensesSettings {
        half_angle: WOLF_HALF_ANGLE,
        ..Default::default()
    }
}
