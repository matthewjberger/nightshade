pub mod ask;
pub mod rows;
pub mod save;
pub mod story;

use crate::theme;
pub use ask::*;
use nalgebra_glm::Vec4;
pub use rows::*;
pub use save::*;
pub use story::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Team {
    #[default]
    Player,
    Enemy,
}

impl Team {
    pub fn name(self) -> &'static str {
        match self {
            Team::Player => "Blue",
            Team::Enemy => "Red",
        }
    }

    pub fn color(self) -> Vec4 {
        match self {
            Team::Player => theme::PLAYER_COLOR,
            Team::Enemy => theme::ENEMY_COLOR,
        }
    }

    pub fn shadow_tint(self) -> [f32; 4] {
        match self {
            Team::Player => theme::PLAYER_SHADOW,
            Team::Enemy => theme::ENEMY_SHADOW,
        }
    }

    pub fn sheet_offset(self) -> u32 {
        match self {
            Team::Player => theme::BLUE_OFFSET,
            Team::Enemy => theme::RED_OFFSET,
        }
    }
}

pub fn team_tile(gray: u32, owner: Option<Team>) -> u32 {
    gray + owner.map_or(0, Team::sheet_offset)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Mobility {
    #[default]
    Foot,
    Wheels,
    Tracks,
    Walker,
    Air,
    Naval,
}

impl Mobility {
    pub fn name(self) -> &'static str {
        match self {
            Mobility::Foot => "Foot",
            Mobility::Wheels => "Wheels",
            Mobility::Tracks => "Tracks",
            Mobility::Walker => "Walker",
            Mobility::Air => "Air",
            Mobility::Naval => "Naval",
        }
    }

    pub fn hover(self) -> f32 {
        match self {
            Mobility::Air => theme::AIR_HOVER,
            _ => 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CursorPartKind {
    #[default]
    Bracket,
    Arrow,
    Ring,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Status {
    #[default]
    Stunned,
    Suppressed,
}

impl Status {
    pub fn name(self) -> &'static str {
        match self {
            Status::Stunned => "Stunned",
            Status::Suppressed => "Suppressed",
        }
    }

    pub fn icon(self) -> u32 {
        match self {
            Status::Stunned => theme::LOCK_ICON,
            Status::Suppressed => theme::DOTTED_ICON,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Feature {
    #[default]
    Open,
    Tree,
    Bunker,
    Structure,
    Obstacle,
}

impl Feature {
    pub fn name(self) -> &'static str {
        match self {
            Feature::Open => "Open",
            Feature::Tree => "Cover",
            Feature::Bunker => "Bunker",
            Feature::Structure => "Building",
            Feature::Obstacle => "Blocked",
        }
    }

    pub fn defense(self) -> i32 {
        match self {
            Feature::Tree => 1,
            Feature::Bunker => 3,
            Feature::Structure => 2,
            _ => 0,
        }
    }

    pub fn evasion(self) -> i32 {
        match self {
            Feature::Tree => 10,
            Feature::Bunker => 5,
            _ => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StructureKind {
    #[default]
    Depot,
    Headquarters,
    Tower,
    Factory,
}

impl StructureKind {
    pub fn name(self) -> &'static str {
        match self {
            StructureKind::Depot => "Depot",
            StructureKind::Headquarters => "Headquarters",
            StructureKind::Tower => "Radio Tower",
            StructureKind::Factory => "Factory",
        }
    }

    pub fn effect(self) -> &'static str {
        match self {
            StructureKind::Depot => "heals and resupplies the owner's units on it",
            StructureKind::Headquarters => "its loss ends the battle",
            StructureKind::Tower => "the owner's units near it hit more often",
            StructureKind::Factory => "worth holding",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Class {
    #[default]
    Trooper,
    Grenadier,
    Tank,
    Supply,
    Recon,
    Artillery,
    Helicopter,
    Gunboat,
    Walker,
    Bomber,
}

pub struct ClassSheet {
    pub name: &'static str,
    pub skill_set: &'static str,
    pub mobility: Mobility,
    pub gray_tile: u32,
    pub hp: i32,
    pub mp: i32,
    pub attack: i32,
    pub defense: i32,
    pub magic: i32,
    pub resist: i32,
    pub speed: i32,
    pub move_range: u32,
    pub jump: u32,
    pub evasion: i32,
    pub can_capture: bool,
    pub attack_range: (u32, u32, u32),
    pub attack_air: f32,
    pub attack_stationary: bool,
    pub abilities: &'static [Ability],
}

impl Class {
    pub fn sheet(self) -> ClassSheet {
        match self {
            Class::Trooper => ClassSheet {
                name: "Trooper",
                skill_set: "Assault",
                mobility: Mobility::Foot,
                gray_tile: theme::TROOPER_TILE,
                hp: 90,
                mp: 12,
                attack: 14,
                defense: 8,
                magic: 4,
                resist: 5,
                speed: 7,
                move_range: 3,
                jump: 2,
                evasion: 6,
                can_capture: true,
                attack_range: (1, 2, 2),
                attack_air: 0.6,
                attack_stationary: false,
                abilities: &[Ability::Burst, Ability::Suppress],
            },
            Class::Grenadier => ClassSheet {
                name: "Grenadier",
                skill_set: "Ordnance",
                mobility: Mobility::Foot,
                gray_tile: theme::GRENADIER_TILE,
                hp: 72,
                mp: 14,
                attack: 12,
                defense: 6,
                magic: 6,
                resist: 6,
                speed: 9,
                move_range: 4,
                jump: 3,
                evasion: 12,
                can_capture: true,
                attack_range: (2, 3, 3),
                attack_air: 0.8,
                attack_stationary: false,
                abilities: &[Ability::Rocket, Ability::Frag],
            },
            Class::Tank => ClassSheet {
                name: "Tank",
                skill_set: "Heavy Guns",
                mobility: Mobility::Tracks,
                gray_tile: theme::TANK_TILE,
                hp: 110,
                mp: 30,
                attack: 10,
                defense: 12,
                magic: 15,
                resist: 8,
                speed: 6,
                move_range: 4,
                jump: 1,
                evasion: 2,
                can_capture: false,
                attack_range: (1, 3, 3),
                attack_air: 0.0,
                attack_stationary: false,
                abilities: &[Ability::Shell, Ability::RailShot],
            },
            Class::Supply => ClassSheet {
                name: "Supply Jeep",
                skill_set: "Logistics",
                mobility: Mobility::Wheels,
                gray_tile: theme::SUPPLY_TILE,
                hp: 70,
                mp: 36,
                attack: 6,
                defense: 6,
                magic: 12,
                resist: 8,
                speed: 8,
                move_range: 5,
                jump: 1,
                evasion: 6,
                can_capture: false,
                attack_range: (1, 1, 2),
                attack_air: 0.0,
                attack_stationary: false,
                abilities: &[Ability::Resupply, Ability::FieldRepair],
            },
            Class::Recon => ClassSheet {
                name: "Recon Jeep",
                skill_set: "Harassment",
                mobility: Mobility::Wheels,
                gray_tile: theme::RECON_TILE,
                hp: 64,
                mp: 16,
                attack: 11,
                defense: 5,
                magic: 5,
                resist: 5,
                speed: 11,
                move_range: 6,
                jump: 1,
                evasion: 16,
                can_capture: false,
                attack_range: (1, 1, 2),
                attack_air: 0.6,
                attack_stationary: false,
                abilities: &[Ability::Ram, Ability::Harass],
            },
            Class::Artillery => ClassSheet {
                name: "Artillery",
                skill_set: "Bombardment",
                mobility: Mobility::Tracks,
                gray_tile: theme::ARTILLERY_TILE,
                hp: 80,
                mp: 28,
                attack: 8,
                defense: 7,
                magic: 16,
                resist: 6,
                speed: 5,
                move_range: 3,
                jump: 1,
                evasion: 2,
                can_capture: false,
                attack_range: (2, 5, 6),
                attack_air: 2.0,
                attack_stationary: true,
                abilities: &[Ability::Barrage, Ability::Flak],
            },
            Class::Helicopter => ClassSheet {
                name: "Helicopter",
                skill_set: "Air Support",
                mobility: Mobility::Air,
                gray_tile: theme::HELICOPTER_TILE,
                hp: 76,
                mp: 24,
                attack: 13,
                defense: 6,
                magic: 9,
                resist: 5,
                speed: 10,
                move_range: 6,
                jump: 9,
                evasion: 10,
                can_capture: false,
                attack_range: (1, 2, 9),
                attack_air: 1.0,
                attack_stationary: false,
                abilities: &[Ability::Strafe, Ability::Rockets],
            },
            Class::Gunboat => ClassSheet {
                name: "Gunboat",
                skill_set: "Naval Guns",
                mobility: Mobility::Naval,
                gray_tile: theme::GUNBOAT_TILE,
                hp: 100,
                mp: 26,
                attack: 12,
                defense: 10,
                magic: 13,
                resist: 7,
                speed: 6,
                move_range: 5,
                jump: 0,
                evasion: 3,
                can_capture: false,
                attack_range: (1, 3, 4),
                attack_air: 0.8,
                attack_stationary: false,
                abilities: &[Ability::Broadside, Ability::DepthCharge],
            },
            Class::Walker => ClassSheet {
                name: "Walker",
                skill_set: "Crush",
                mobility: Mobility::Walker,
                gray_tile: theme::WALKER_TILE,
                hp: 96,
                mp: 20,
                attack: 15,
                defense: 10,
                magic: 6,
                resist: 7,
                speed: 6,
                move_range: 4,
                jump: 3,
                evasion: 4,
                can_capture: false,
                attack_range: (1, 1, 3),
                attack_air: 0.0,
                attack_stationary: false,
                abilities: &[Ability::Stomp, Ability::Crush],
            },
            Class::Bomber => ClassSheet {
                name: "Bomber",
                skill_set: "Air Strike",
                mobility: Mobility::Air,
                gray_tile: theme::BOMBER_TILE,
                hp: 70,
                mp: 24,
                attack: 12,
                defense: 5,
                magic: 17,
                resist: 5,
                speed: 9,
                move_range: 8,
                jump: 9,
                evasion: 14,
                can_capture: false,
                attack_range: (1, 2, 9),
                attack_air: 1.0,
                attack_stationary: false,
                abilities: &[Ability::BombRun, Ability::Strafe],
            },
        }
    }

    pub fn tile(self, team: Team) -> u32 {
        team_tile(self.sheet().gray_tile, Some(team))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Ability {
    #[default]
    Attack,
    Burst,
    Suppress,
    Rocket,
    Frag,
    Shell,
    RailShot,
    Resupply,
    FieldRepair,
    Ram,
    Harass,
    Barrage,
    Flak,
    Strafe,
    Rockets,
    Broadside,
    DepthCharge,
    Stomp,
    Crush,
    BombRun,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    Physical,
    Magic,
    Heal,
}

pub struct AbilityShape {
    pub name: &'static str,
    pub mp: i32,
    pub range_min: u32,
    pub range_max: u32,
    pub vertical: u32,
    pub area: u32,
    pub power: i32,
    pub effect: Effect,
    pub uses_class_range: bool,
    pub stationary: bool,
    pub air_factor: f32,
    pub inflicts: Option<Status>,
}

const fn shape(
    name: &'static str,
    mp: i32,
    range: (u32, u32, u32),
    area: u32,
    power: i32,
    effect: Effect,
) -> AbilityShape {
    AbilityShape {
        name,
        mp,
        range_min: range.0,
        range_max: range.1,
        vertical: range.2,
        area,
        power,
        effect,
        uses_class_range: false,
        stationary: false,
        air_factor: 1.0,
        inflicts: None,
    }
}

impl Ability {
    pub fn shape(self) -> AbilityShape {
        match self {
            Ability::Attack => AbilityShape {
                uses_class_range: true,
                ..shape("Attack", 0, (1, 1, 2), 0, 100, Effect::Physical)
            },
            Ability::Burst => shape("Burst Fire", 5, (1, 2, 2), 0, 160, Effect::Physical),
            Ability::Suppress => AbilityShape {
                inflicts: Some(Status::Suppressed),
                ..shape("Suppress", 4, (1, 3, 3), 0, 90, Effect::Physical)
            },
            Ability::Rocket => shape("Rocket", 4, (2, 5, 4), 0, 140, Effect::Physical),
            Ability::Frag => AbilityShape {
                air_factor: 0.0,
                ..shape("Frag Grenade", 7, (2, 4, 4), 1, 80, Effect::Physical)
            },
            Ability::Shell => AbilityShape {
                air_factor: 0.0,
                ..shape("Shell", 8, (1, 3, 3), 1, 130, Effect::Magic)
            },
            Ability::RailShot => shape("Rail Shot", 6, (1, 4, 6), 0, 150, Effect::Magic),
            Ability::Resupply => shape("Resupply", 7, (0, 3, 3), 1, 140, Effect::Heal),
            Ability::FieldRepair => shape("Field Repair", 12, (1, 4, 5), 0, 190, Effect::Heal),
            Ability::Ram => AbilityShape {
                air_factor: 0.0,
                ..shape("Ram", 3, (1, 1, 1), 0, 130, Effect::Physical)
            },
            Ability::Harass => AbilityShape {
                inflicts: Some(Status::Suppressed),
                air_factor: 0.5,
                ..shape("Harass", 5, (1, 2, 2), 0, 80, Effect::Physical)
            },
            Ability::Barrage => AbilityShape {
                stationary: true,
                air_factor: 0.3,
                ..shape("Barrage", 9, (3, 6, 7), 1, 120, Effect::Magic)
            },
            Ability::Flak => AbilityShape {
                air_factor: 2.4,
                ..shape("Flak", 6, (1, 4, 9), 0, 110, Effect::Physical)
            },
            Ability::Strafe => shape("Strafe", 5, (1, 2, 9), 0, 130, Effect::Physical),
            Ability::Rockets => AbilityShape {
                air_factor: 0.0,
                ..shape("Rocket Pods", 8, (2, 3, 9), 1, 100, Effect::Magic)
            },
            Ability::Broadside => shape("Broadside", 7, (2, 4, 4), 0, 150, Effect::Magic),
            Ability::DepthCharge => AbilityShape {
                air_factor: 0.0,
                ..shape("Depth Charge", 8, (1, 2, 2), 1, 120, Effect::Magic)
            },
            Ability::Stomp => AbilityShape {
                air_factor: 0.0,
                ..shape("Stomp", 6, (1, 1, 3), 1, 110, Effect::Physical)
            },
            Ability::Crush => AbilityShape {
                inflicts: Some(Status::Stunned),
                air_factor: 0.0,
                ..shape("Crush", 9, (1, 1, 3), 0, 140, Effect::Physical)
            },
            Ability::BombRun => AbilityShape {
                air_factor: 0.0,
                ..shape("Bomb Run", 11, (1, 3, 9), 1, 160, Effect::Magic)
            },
        }
    }

    pub fn name(self) -> &'static str {
        self.shape().name
    }

    pub fn helps(self) -> bool {
        self.shape().effect == Effect::Heal
    }
}

pub struct Recruit {
    pub name: &'static str,
    pub class: Class,
    pub column: u32,
    pub row: u32,
    pub facing: Facing,
    pub hp_bonus: i32,
    pub attack_bonus: i32,
}

pub(crate) const fn recruit(
    name: &'static str,
    class: Class,
    column: u32,
    row: u32,
    facing: Facing,
    hp_bonus: i32,
    attack_bonus: i32,
) -> Recruit {
    Recruit {
        name,
        class,
        column,
        row,
        facing,
        hp_bonus,
        attack_bonus,
    }
}

pub struct Site {
    pub kind: StructureKind,
    pub owner: Option<Team>,
    pub column: u32,
    pub row: u32,
}

const fn site(kind: StructureKind, owner: Option<Team>, column: u32, row: u32) -> Site {
    Site {
        kind,
        owner,
        column,
        row,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Facing {
    #[default]
    South,
    North,
    East,
    West,
}

impl Facing {
    pub fn delta(self) -> (i32, i32) {
        match self {
            Facing::North => (0, -1),
            Facing::South => (0, 1),
            Facing::East => (1, 0),
            Facing::West => (-1, 0),
        }
    }

    pub fn from_delta(column_delta: i32, row_delta: i32) -> Option<Facing> {
        if column_delta == 0 && row_delta == 0 {
            return None;
        }
        Some(if column_delta.abs() >= row_delta.abs() {
            if column_delta > 0 {
                Facing::East
            } else {
                Facing::West
            }
        } else if row_delta > 0 {
            Facing::South
        } else {
            Facing::North
        })
    }

    pub fn arrow_tip(self) -> u32 {
        match self {
            Facing::North => theme::ARROW_NORTH,
            Facing::South => theme::ARROW_SOUTH,
            Facing::East => theme::ARROW_EAST,
            Facing::West => theme::ARROW_WEST,
        }
    }

    pub fn arrow_shaft(self) -> u32 {
        match self {
            Facing::North | Facing::South => theme::ARROW_VERTICAL,
            Facing::East | Facing::West => theme::ARROW_HORIZONTAL,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Front,
    Flank,
    Back,
}

impl Side {
    pub fn name(self) -> &'static str {
        match self {
            Side::Front => "front",
            Side::Flank => "flank",
            Side::Back => "rear",
        }
    }
}

pub fn side_of_attack(attacker: (u32, u32), target: (u32, u32), target_facing: Facing) -> Side {
    let column_delta = attacker.0 as i32 - target.0 as i32;
    let row_delta = attacker.1 as i32 - target.1 as i32;
    let Some(from) = Facing::from_delta(column_delta, row_delta) else {
        return Side::Front;
    };
    let (facing_column, facing_row) = target_facing.delta();
    let (from_column, from_row) = from.delta();
    let along = facing_column * from_column + facing_row * from_row;
    if along > 0 {
        Side::Front
    } else if along < 0 {
        Side::Back
    } else {
        Side::Flank
    }
}

pub struct Strike {
    pub amount: i32,
    pub hit_chance: i32,
    pub side: Side,
    pub height_gain: i32,
}

pub struct StrikeInput {
    pub attacker_attack: i32,
    pub attacker_magic: i32,
    pub attacker_suppressed: bool,
    pub target_defense: i32,
    pub target_resist: i32,
    pub target_evasion: i32,
    pub target_cover: Feature,
    pub target_air: bool,
    pub height_gain: i32,
    pub side: Side,
    pub hit_bonus: i32,
    pub air_factor: f32,
    pub ability: Ability,
}

pub fn strike(input: StrikeInput) -> Strike {
    let shape = input.ability.shape();
    let defense = input.target_defense + input.target_cover.defense();
    let resist = input.target_resist + input.target_cover.defense() / 2;
    let attack = if input.attacker_suppressed {
        input.attacker_attack * 7 / 10
    } else {
        input.attacker_attack
    };
    let base = match shape.effect {
        Effect::Physical => (attack * 2 - defense).max(2),
        Effect::Magic => (input.attacker_magic * 2 - resist).max(2),
        Effect::Heal => input.attacker_magic * 2,
    };
    let height = (input.height_gain.clamp(-3, 3) * 5) as f32 / 100.0;
    let side_bonus = match input.side {
        Side::Front => 0.0,
        Side::Flank => 0.1,
        Side::Back => 0.25,
    };
    let air = if input.target_air {
        input.air_factor
    } else {
        1.0
    };
    let scale = if shape.effect == Effect::Heal {
        1.0
    } else {
        (1.0 + height + side_bonus) * air
    };
    let amount = ((base * shape.power) as f32 / 100.0 * scale)
        .round()
        .max(1.0) as i32;
    let hit_chance = match shape.effect {
        Effect::Physical => {
            let side_hit = match input.side {
                Side::Front => 0,
                Side::Flank => 8,
                Side::Back => 20,
            };
            let evasion = input.target_evasion + input.target_cover.evasion();
            (94 - evasion + input.height_gain.clamp(-3, 3) * 3 + side_hit + input.hit_bonus)
                .clamp(35, 100)
        }
        Effect::Magic | Effect::Heal => 100,
    };
    Strike {
        amount,
        hit_chance,
        side: input.side,
        height_gain: input.height_gain,
    }
}

pub const MENU_LABELS: [&str; 6] = [
    "Move",
    "Take back the move",
    "Attack",
    "Skills",
    "Capture",
    "Wait",
];
