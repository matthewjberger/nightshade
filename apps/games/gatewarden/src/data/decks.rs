use crate::data::cards::{CardId, Element};
use crate::data::shop::POOL_COST_ABOVE_GATE;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum HeroPower {
    #[default]
    Spark,
    Mend,
    Sprout,
    Bolt,
}

impl HeroPower {
    pub fn name(self) -> &'static str {
        match self {
            HeroPower::Spark => "Spark",
            HeroPower::Mend => "Mend",
            HeroPower::Sprout => "Sprout",
            HeroPower::Bolt => "Bolt",
        }
    }

    pub fn text(self) -> &'static str {
        match self {
            HeroPower::Spark => "1 damage to a target",
            HeroPower::Mend => "Restore 1 health, rests a turn",
            HeroPower::Sprout => "Summon a 1/1 Sapling",
            HeroPower::Bolt => "1 damage to a random foe",
        }
    }

    pub fn cost(self) -> u32 {
        2
    }
}

pub struct WardenDef {
    pub name: &'static str,
    pub title: &'static str,
    pub element: Element,
    pub health: i32,
    pub power: HeroPower,
    pub aggressive: bool,
    pub sigil: char,
}

pub const STARTING_HEALTH: i32 = 30;

pub fn starting_deck() -> Vec<CardId> {
    vec![
        CardId::Spearhand,
        CardId::Spearhand,
        CardId::Shieldbearer,
        CardId::Shieldbearer,
        CardId::Mercenary,
        CardId::Mercenary,
        CardId::Knight,
        CardId::Knight,
        CardId::Champion,
        CardId::EmberMote,
        CardId::EmberMote,
        CardId::FlameHound,
        CardId::EmberBolt,
        CardId::EmberBolt,
        CardId::TideSurge,
        CardId::ReefGuardian,
        CardId::Growth,
        CardId::GroveWarden,
        CardId::SkyDarter,
        CardId::Lightning,
    ]
}

pub const WARDEN_CHOICES: usize = 3;

pub struct WardenChoice {
    pub name: &'static str,
    pub element: Element,
    pub power: HeroPower,
    pub aggressive: bool,
    pub sigil: char,
}

const fn choice(
    name: &'static str,
    element: Element,
    power: HeroPower,
    aggressive: bool,
    sigil: char,
) -> WardenChoice {
    WardenChoice {
        name,
        element,
        power,
        aggressive,
        sigil,
    }
}

pub fn warden_pool(gate: u32) -> [WardenChoice; WARDEN_CHOICES] {
    match gate {
        1 => [
            choice(
                "Ash Whelp",
                Element::Ember,
                HeroPower::Spark,
                true,
                '\u{e0d2}',
            ),
            choice(
                "Thorn Cub",
                Element::Grove,
                HeroPower::Sprout,
                true,
                '\u{e1eb}',
            ),
            choice(
                "Gale Sylph",
                Element::Storm,
                HeroPower::Bolt,
                true,
                '\u{e1b0}',
            ),
        ],
        2 => [
            choice(
                "Reef Sentinel",
                Element::Tide,
                HeroPower::Mend,
                false,
                '\u{e4f9}',
            ),
            choice(
                "Salt Crawler",
                Element::Tide,
                HeroPower::Spark,
                true,
                '\u{e4f8}',
            ),
            choice(
                "Kelp Stalker",
                Element::Grove,
                HeroPower::Bolt,
                true,
                '\u{e2de}',
            ),
        ],
        3 => [
            choice(
                "Bramble Knight",
                Element::Grove,
                HeroPower::Sprout,
                false,
                '\u{e2ee}',
            ),
            choice(
                "Fen Warden",
                Element::Grove,
                HeroPower::Mend,
                false,
                '\u{e092}',
            ),
            choice(
                "Ash Ranger",
                Element::Ember,
                HeroPower::Spark,
                true,
                '\u{e0be}',
            ),
        ],
        4 => [
            choice(
                "Skyvoice",
                Element::Storm,
                HeroPower::Bolt,
                true,
                '\u{e08c}',
            ),
            choice(
                "Thunder Priest",
                Element::Storm,
                HeroPower::Mend,
                false,
                '\u{e1b4}',
            ),
            choice(
                "Squall Rider",
                Element::Tide,
                HeroPower::Bolt,
                true,
                '\u{e218}',
            ),
        ],
        5 => [
            choice(
                "Cinder Baron",
                Element::Ember,
                HeroPower::Spark,
                true,
                '\u{e1d6}',
            ),
            choice(
                "Forge Tyrant",
                Element::Ember,
                HeroPower::Sprout,
                true,
                '\u{e242}',
            ),
            choice(
                "Ash Sovereign",
                Element::Ember,
                HeroPower::Bolt,
                true,
                '\u{e176}',
            ),
        ],
        6 => [
            choice(
                "Tide Matriarch",
                Element::Tide,
                HeroPower::Mend,
                false,
                '\u{e11e}',
            ),
            choice(
                "Deep Herald",
                Element::Tide,
                HeroPower::Bolt,
                false,
                '\u{e03f}',
            ),
            choice(
                "Abyss Warden",
                Element::Tide,
                HeroPower::Spark,
                true,
                '\u{e3a6}',
            ),
        ],
        7 => [
            choice(
                "Elder of the Grove",
                Element::Grove,
                HeroPower::Sprout,
                false,
                '\u{e2f3}',
            ),
            choice(
                "Heartwood Sage",
                Element::Grove,
                HeroPower::Mend,
                false,
                '\u{e2f4}',
            ),
            choice(
                "Bramble Colossus",
                Element::Grove,
                HeroPower::Spark,
                true,
                '\u{e231}',
            ),
        ],
        _ => [
            choice(
                "Storm Dragon",
                Element::Storm,
                HeroPower::Bolt,
                true,
                '\u{e221}',
            ),
            choice(
                "Sky Tyrant",
                Element::Storm,
                HeroPower::Spark,
                true,
                '\u{e3e0}',
            ),
            choice(
                "Tempest Oracle",
                Element::Storm,
                HeroPower::Mend,
                false,
                '\u{e412}',
            ),
        ],
    }
}

pub fn gate_health(gate: u32) -> i32 {
    match gate {
        1 => 20,
        2 => 24,
        3 => 27,
        4 => 30,
        5 => 33,
        6 => 32,
        7 => 39,
        _ => 44,
    }
}

pub fn gate_title(gate: u32) -> &'static str {
    match gate {
        1 => "Warden of the first gate",
        2 => "Warden of the second gate",
        3 => "Warden of the third gate",
        4 => "Warden of the fourth gate",
        5 => "Warden of the fifth gate",
        6 => "Warden of the sixth gate",
        7 => "Warden of the seventh gate",
        _ => "Warden of the last gate",
    }
}

pub fn warden(gate: u32, pick: u8) -> WardenDef {
    let pool = warden_pool(gate);
    let index = usize::from(pick).min(WARDEN_CHOICES - 1);
    let choice = &pool[index];
    WardenDef {
        name: choice.name,
        title: gate_title(gate),
        element: choice.element,
        health: gate_health(gate),
        power: choice.power,
        aggressive: choice.aggressive,
        sigil: choice.sigil,
    }
}

pub fn warden_deck(gate: u32, element: Element) -> Vec<CardId> {
    let element_cards: &[CardId] = match element {
        Element::Ember => &[
            CardId::EmberMote,
            CardId::FlameHound,
            CardId::CinderKnight,
            CardId::BlazeDrake,
            CardId::EmberBolt,
            CardId::Wildfire,
            CardId::DragonBreath,
        ],
        Element::Tide => &[
            CardId::SaltWard,
            CardId::Wavesinger,
            CardId::ReefGuardian,
            CardId::DeepWard,
            CardId::Kraken,
            CardId::TideSurge,
            CardId::MistVeil,
            CardId::Shieldbearer,
        ],
        Element::Grove => &[
            CardId::Bulwark,
            CardId::Sprout,
            CardId::GroveWarden,
            CardId::ElderBough,
            CardId::Growth,
            CardId::Seedlings,
            CardId::Knight,
            CardId::Giant,
        ],
        Element::Storm | Element::Neutral => &[
            CardId::StormSprite,
            CardId::SkyDarter,
            CardId::Thunderwright,
            CardId::TempestLord,
            CardId::Lightning,
            CardId::Thunderfall,
            CardId::StormScroll,
        ],
    };
    let neutrals = [
        CardId::Spearhand,
        CardId::WardStone,
        CardId::Shieldbearer,
        CardId::Mercenary,
        CardId::Knight,
        CardId::Champion,
        CardId::Giant,
    ];
    let ceiling = gate + POOL_COST_ABOVE_GATE;
    let mut deck = Vec::with_capacity(20);
    for card in element_cards {
        deck.push(*card);
        if card.def().cost <= ceiling {
            deck.push(*card);
        }
    }
    for card in neutrals {
        let allowed = match card {
            CardId::Spearhand | CardId::Shieldbearer | CardId::Mercenary | CardId::Knight => true,
            CardId::Champion => gate >= 6,
            _ => gate >= 7,
        };
        if allowed {
            deck.push(card);
        }
    }
    deck.truncate(20);
    deck
}
