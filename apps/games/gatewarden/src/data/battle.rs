use crate::data::cards::{CardId, Element};
use crate::data::decks::HeroPower;
use crate::data::gates::GateRule;
use crate::data::story::Shard;
use std::collections::VecDeque;

pub const BOARD_LIMIT: usize = 6;
pub const OPENING_HAND: usize = 3;
pub const SECOND_PLAYER_BONUS: usize = 1;
pub const POWER_REST_TURNS: u32 = 2;
pub const HAND_LIMIT: usize = 8;
pub const MAX_MANA: u32 = 10;
pub const ARMOR_CAP: i32 = 12;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum Who {
    #[default]
    Player,
    Enemy,
}

pub fn other(who: Who) -> Who {
    match who {
        Who::Player => Who::Enemy,
        Who::Enemy => Who::Player,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Target {
    Hero(Who),
    Minion(Who, u32),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Instance {
    pub id: u32,
    pub card: CardId,
    pub upgraded: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Minion {
    pub id: u32,
    pub origin: u32,
    pub card: CardId,
    pub attack: i32,
    pub health: i32,
    pub max_health: i32,
    pub guard: bool,
    pub shield: bool,
    pub venom: bool,
    pub drain: bool,
    pub frozen: bool,
    pub ready: bool,
    pub attacked: bool,
}

impl Minion {
    pub fn can_attack(&self) -> bool {
        self.ready && !self.attacked && self.attack > 0
    }
}

#[derive(Default)]
pub struct Side {
    pub name: String,
    pub hero_health: i32,
    pub hero_max: i32,
    pub armor: i32,
    pub turn_element: Option<Element>,
    pub mana: u32,
    pub max_mana: u32,
    pub deck: Vec<Instance>,
    pub hand: Vec<Instance>,
    pub board: Vec<Minion>,
    pub fatigue: i32,
    pub power: HeroPower,
    pub power_cost: u32,
    pub power_used: bool,
    pub power_turn: u32,
    pub minion_health_bonus: i32,
    pub minion_attack_bonus: i32,
    pub guard_health_bonus: i32,
    pub turn_heal: i32,
    pub turn_burn: i32,
    pub rush_all: bool,
    pub spell_bonus: i32,
    pub shards: Vec<Shard>,
    pub aggressive: bool,
}

impl Side {
    pub fn minion(&self, id: u32) -> Option<&Minion> {
        self.board.iter().find(|minion| minion.id == id)
    }

    pub fn minion_mut(&mut self, id: u32) -> Option<&mut Minion> {
        self.board.iter_mut().find(|minion| minion.id == id)
    }

    pub fn has_taunt(&self) -> bool {
        self.board.iter().any(|minion| minion.guard)
    }

    pub fn hand_card(&self, id: u32) -> Option<Instance> {
        self.hand.iter().copied().find(|instance| instance.id == id)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Play {
        who: Who,
        card: CardId,
    },
    Attack {
        attacker: Target,
        target: Target,
    },
    Hit {
        target: Target,
        amount: i32,
    },
    Heal {
        target: Target,
        amount: i32,
    },
    Died {
        target: Target,
        card: Option<CardId>,
    },
    Summon {
        who: Who,
        id: u32,
    },
    Draw {
        who: Who,
    },
    Fatigue {
        who: Who,
        amount: i32,
    },
    Armor {
        who: Who,
        amount: i32,
    },
    Chain {
        who: Who,
        element: Element,
    },
    Turn {
        who: Who,
        turn: u32,
    },
    Note(String),
}

#[derive(Default)]
pub struct Battle {
    pub player: Side,
    pub enemy: Side,
    pub rule: GateRule,
    pub played_this_turn: u32,
    pub turn: u32,
    pub active: Who,
    pub next_id: u32,
    pub events: VecDeque<Event>,
    pub outcome: Option<Who>,
    pub seed: u64,
}

impl Battle {
    pub fn side(&self, who: Who) -> &Side {
        match who {
            Who::Player => &self.player,
            Who::Enemy => &self.enemy,
        }
    }

    pub fn side_mut(&mut self, who: Who) -> &mut Side {
        match who {
            Who::Player => &mut self.player,
            Who::Enemy => &mut self.enemy,
        }
    }

    pub fn allocate(&mut self) -> u32 {
        self.next_id += 1;
        self.next_id
    }

    pub fn minion(&self, target: Target) -> Option<&Minion> {
        match target {
            Target::Hero(_) => None,
            Target::Minion(who, id) => self.side(who).minion(id),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pick {
    Hand(u32),
    Mine(u32),
    Foe(u32),
    Hero(Who),
}

pub enum Intent {
    Pick(Pick),
    EndTurn,
    Power,
    Cancel,
}
