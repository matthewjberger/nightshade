#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Place {
    #[default]
    Overworld,
    Town,
    Dungeon,
}

impl Place {
    pub const fn name(self) -> &'static str {
        match self {
            Place::Overworld => "Ealdmoor",
            Place::Town => "Wenholt",
            Place::Dungeon => "The Sunken Vault",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Job {
    #[default]
    Freelancer,
    Fighter,
    WhiteMage,
    BlackMage,
    Thief,
    Monk,
}

impl Job {
    pub const LADDER: [Job; 5] = [
        Job::Fighter,
        Job::WhiteMage,
        Job::BlackMage,
        Job::Thief,
        Job::Monk,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Job::Freelancer => "Freelancer",
            Job::Fighter => "Fighter",
            Job::WhiteMage => "White Mage",
            Job::BlackMage => "Black Mage",
            Job::Thief => "Thief",
            Job::Monk => "Monk",
        }
    }

    pub const fn spell(self) -> Option<Spell> {
        match self {
            Job::WhiteMage => Some(Spell::Cure),
            Job::BlackMage => Some(Spell::Fire),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spell {
    Cure,
    Fire,
}

impl Spell {
    pub const fn name(self) -> &'static str {
        match self {
            Spell::Cure => "Cure",
            Spell::Fire => "Fire",
        }
    }

    pub const fn cost(self) -> u32 {
        match self {
            Spell::Cure => 4,
            Spell::Fire => 5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MonsterKind {
    #[default]
    Slime,
    Wolf,
    Imp,
    Bat,
    Spider,
    Ghost,
}

impl MonsterKind {
    pub const ROAMERS: [MonsterKind; 2] = [MonsterKind::Slime, MonsterKind::Wolf];
    pub const VAULT: [MonsterKind; 4] = [
        MonsterKind::Imp,
        MonsterKind::Bat,
        MonsterKind::Spider,
        MonsterKind::Ghost,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            MonsterKind::Slime => "Slime",
            MonsterKind::Wolf => "Dire Wolf",
            MonsterKind::Imp => "Imp",
            MonsterKind::Bat => "Cave Bat",
            MonsterKind::Spider => "Weaver",
            MonsterKind::Ghost => "Wisp",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ware {
    Potion,
    Sword,
    Charm,
}

impl Ware {
    pub const ALL: [Ware; 3] = [Ware::Potion, Ware::Sword, Ware::Charm];

    pub const fn name(self) -> &'static str {
        match self {
            Ware::Potion => "Potion",
            Ware::Sword => "Sharpen the blades",
            Ware::Charm => "Temper the guards",
        }
    }

    pub const fn price(self) -> u32 {
        match self {
            Ware::Potion => 20,
            Ware::Sword => 120,
            Ware::Charm => 150,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BattleCommand {
    Attack,
    Magic,
    Item,
    Run,
}

impl BattleCommand {
    pub const ALL: [BattleCommand; 4] = [
        BattleCommand::Attack,
        BattleCommand::Magic,
        BattleCommand::Item,
        BattleCommand::Run,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            BattleCommand::Attack => "Attack",
            BattleCommand::Magic => "Magic",
            BattleCommand::Item => "Item",
            BattleCommand::Run => "Run",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StationKind {
    #[default]
    Inn,
    Shop,
    Crystal,
    Chest,
}
