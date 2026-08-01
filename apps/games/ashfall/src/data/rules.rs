use crate::components::HuskKind;

pub const PLAYER_SPEED: f32 = 3.0;
pub const PLAYER_SPRINT: f32 = 1.8;
pub const EYE_HEIGHT: f32 = 1.55;
pub const FIELD_OF_VIEW: f32 = 68.0;
pub const MAX_HEALTH: f32 = 100.0;
pub const STARTING_AMMO: u32 = 4;
pub const RADIO_RANGE: f32 = 12.0;

pub struct HuskStats {
    pub health: f32,
    pub speed: f32,
    pub damage: f32,
    pub reach: f32,
    pub interval: f32,
    pub sight: f32,
    pub rise: f32,
    pub height: f32,
    pub width: f32,
    pub screams: bool,
}

pub const fn stats(kind: HuskKind) -> HuskStats {
    match kind {
        HuskKind::Wanderer => HuskStats {
            health: 3.0,
            speed: 2.3,
            damage: 16.0,
            reach: 1.7,
            interval: 1.3,
            sight: 1.0,
            rise: 2.6,
            height: 1.0,
            width: 1.0,
            screams: false,
        },
        HuskKind::Crawler => HuskStats {
            health: 2.0,
            speed: 3.2,
            damage: 10.0,
            reach: 1.5,
            interval: 0.85,
            sight: 0.7,
            rise: 1.5,
            height: 0.45,
            width: 1.15,
            screams: false,
        },
        HuskKind::Wretch => HuskStats {
            health: 7.0,
            speed: 1.5,
            damage: 32.0,
            reach: 2.1,
            interval: 2.0,
            sight: 0.85,
            rise: 4.0,
            height: 1.35,
            width: 1.3,
            screams: false,
        },
        HuskKind::Screamer => HuskStats {
            health: 2.0,
            speed: 2.0,
            damage: 6.0,
            reach: 1.6,
            interval: 1.6,
            sight: 1.35,
            rise: 2.0,
            height: 1.15,
            width: 0.8,
            screams: true,
        },
    }
}

pub const HUSK_STAGGER: f32 = 0.55;
pub const HUSK_REPATH_INTERVAL: f32 = 0.4;
pub const HUSK_SIGHT_DARK: f32 = 7.0;
pub const HUSK_SIGHT_LIT: f32 = 16.0;
pub const HUSK_ROUSE_RANGE: f32 = 3.4;
pub const HUSK_ROUSE_NOISE: f32 = 13.0;
pub const HUSK_DOWN_SECONDS: f32 = 7.0;
pub const HUSK_DOWN_HEALTH: f32 = 0.5;
pub const HUSK_ROT_SECONDS: f32 = 12.0;
pub const SCREAM_LOUDNESS: f32 = 60.0;
pub const SCREAM_INTERVAL: f32 = 6.0;

pub const PIPE_REACH: f32 = 2.3;
pub const PIPE_COOLDOWN: f32 = 0.8;
pub const PIPE_DAMAGE: f32 = 1.0;
pub const PISTOL_RANGE: f32 = 30.0;
pub const PISTOL_COOLDOWN: f32 = 0.5;
pub const PISTOL_DAMAGE: f32 = 2.0;
pub const PISTOL_SPREAD: f32 = 0.5;

pub const HEALTH_DRINK: f32 = 40.0;
pub const AMMO_BOX: u32 = 5;
pub const BATTERY_CHARGE: f32 = 55.0;
pub const MAX_BATTERY: f32 = 100.0;
pub const BATTERY_DRAIN: f32 = 0.62;
pub const BATTERY_LOW: f32 = 22.0;
pub const PICKUP_REACH: f32 = 2.1;
pub const DOOR_REACH: f32 = 2.7;
pub const EXAMINE_REACH: f32 = 2.6;
pub const MESSAGE_SECONDS: f32 = 3.5;
pub const HURT_FADE: f32 = 2.2;
pub const SHIFT_FADE: f32 = 0.6;

pub const NOISE_FOOTFALL: f32 = 7.0;
pub const NOISE_SWING: f32 = 9.0;
pub const NOISE_IMPACT: f32 = 14.0;
pub const NOISE_GLASS: f32 = 20.0;
pub const NOISE_SHOT: f32 = 34.0;
pub const NOISE_BLAST: f32 = 46.0;
pub const FOOTFALL_INTERVAL: f32 = 0.42;
pub const INVESTIGATE_SECONDS: f32 = 5.0;
pub const INVESTIGATE_ARRIVAL: f32 = 1.4;
pub const CROUCH_SIGHT_SCALE: f32 = 0.55;
pub const CROUCH_SPEED_SCALE: f32 = 0.5;
pub const MOVING_SIGHT_SCALE: f32 = 1.25;

pub const GRAB_REACH: f32 = 3.0;
pub const GRAB_NEAR: f32 = 1.1;
pub const GRAB_FAR: f32 = 2.8;
pub const GRAB_STEP: f32 = 0.25;
pub const GRAB_STIFFNESS: f32 = 11.0;
pub const GRAB_MAX_SPEED: f32 = 9.0;
pub const GRAB_SPIN_DAMPING: f32 = 8.0;
pub const LOOSE_SPIN_DAMPING: f32 = 0.7;
pub const THROW_SPEED: f32 = 13.0;
pub const THROW_LIFT: f32 = 1.6;
pub const THROW_ARMING: f32 = 0.18;
pub const THROW_LIFE: f32 = 6.0;
pub const LANDING_PACE: f32 = 1.1;
pub const THROWN_DAMAGE: f32 = 0.09;
pub const THROWN_REACH: f32 = 1.3;
pub const THROWN_MINIMUM: f32 = 6.0;
pub const THROWN_CAP: f32 = 4.0;
pub const SWING_IMPACT: f32 = 2.0;
pub const GLASS_MINIMUM: f32 = 1.5;
pub const BARREL_TRIGGER: f32 = 2.5;
pub const SHOT_IMPACT: f32 = 3.0;
pub const PERSON_HEALTH: f32 = 4.0;
pub const PERSON_DEATH_NOISE: f32 = 24.0;
pub const PERSON_FALLEN_HEIGHT: f32 = 0.45;
pub const LANTERN_DROP: f32 = 0.85;
pub const LANTERN_NUDGE: f32 = 0.9;
pub const GLASS_HEALTH: f32 = 1.0;
pub const BARREL_FUSE: f32 = 0.75;
pub const BLAST_RADIUS: f32 = 5.5;
pub const BLAST_DAMAGE: f32 = 6.0;
pub const BLAST_PLAYER_DAMAGE: f32 = 34.0;

pub const HAND_LAG: f32 = 7.0;
pub const HAND_SWAY: f32 = 0.5;
pub const HAND_BOB: f32 = 0.016;
pub const HAND_BOB_RATE: f32 = 6.2;
pub const HAND_SETTLED: f32 = 0.99995;
pub const SATCHEL_SLOTS: usize = 8;
pub const STACK_LIMIT: u32 = 4;
pub const SPIN_RATE: f32 = 2.6;
pub const CATCH_FACING: f32 = 0.86;
pub const CASE_DIGITS: usize = 3;
pub const CASE_CODE: [u32; CASE_DIGITS] = [1, 1, 4];
pub const BOARD_HEALTH: f32 = 2.0;

pub const STAIR_STEPS: usize = 12;
pub const MAP_REVEAL_RADIUS: i32 = 7;
pub const UPPER_FLOOR_LEVEL: f32 = 1.6;

pub const FLICKER_DEPTH: f32 = 0.3;
pub const FLICKER_DROPOUT_CHANCE: f32 = 0.03;
pub const FLICKER_DROPOUT_SCALE: f32 = 0.15;
pub const FLICKER_RESPONSE: f32 = 18.0;
pub const FLICKER_DARK_BELOW: f32 = 0.4;
pub const BEAM_BROWNOUT: f32 = 0.55;
pub const MUZZLE_FADE: f32 = 9.0;
pub const BLAST_FADE: f32 = 2.6;

pub fn flashlight_settings() -> nightshade_flashlight::FlashlightSettings {
    nightshade_flashlight::FlashlightSettings {
        full: MAX_BATTERY,
        low: BATTERY_LOW,
        brownout: BEAM_BROWNOUT,
    }
}

pub fn flicker_settings() -> nightshade_flicker::FlickerSettings {
    nightshade_flicker::FlickerSettings {
        depth: FLICKER_DEPTH,
        dropout_chance: FLICKER_DROPOUT_CHANCE,
        dropout_scale: FLICKER_DROPOUT_SCALE,
        response: FLICKER_RESPONSE,
        dark_below: FLICKER_DARK_BELOW,
    }
}

pub fn senses_settings() -> nightshade_senses::SensesSettings {
    nightshade_senses::SensesSettings {
        plane: nightshade_plane::Plane::Ground,
        crouch_scale: CROUCH_SIGHT_SCALE,
        moving_scale: MOVING_SIGHT_SCALE,
        ..Default::default()
    }
}
