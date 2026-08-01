#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum Element {
    #[default]
    Neutral,
    Ember,
    Tide,
    Grove,
    Storm,
}

impl Element {
    pub fn name(self) -> &'static str {
        match self {
            Element::Neutral => "Neutral",
            Element::Ember => "Ember",
            Element::Tide => "Tide",
            Element::Grove => "Grove",
            Element::Storm => "Storm",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TargetRule {
    None,
    AnyEnemy,
    EnemyMinion,
    AnyCharacter,
    FriendlyMinion,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Spell {
    Damage(i32),
    Heal(i32),
    Buff(i32, i32),
    Storm(i32),
    Draw(u32),
    Summon(CardId, u32),
    Shield,
    Freeze,
    Armor(i32),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Minion {
        attack: i32,
        health: i32,
        guard: bool,
        rush: bool,
        shield: bool,
        venom: bool,
        drain: bool,
        warcry: Option<Spell>,
    },
    Spell {
        effect: Spell,
        target: TargetRule,
    },
}

#[derive(Clone, Copy)]
pub struct CardDef {
    pub name: &'static str,
    pub cost: u32,
    pub element: Element,
    pub kind: Kind,
    pub text: &'static str,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum CardId {
    #[default]
    Spearhand,
    Shieldbearer,
    Mercenary,
    Knight,
    Champion,
    Giant,
    EmberMote,
    FlameHound,
    CinderKnight,
    BlazeDrake,
    EmberBolt,
    Wildfire,
    DragonBreath,
    Wavesinger,
    ReefGuardian,
    DeepWard,
    Kraken,
    TideSurge,
    MistVeil,
    Sprout,
    GroveWarden,
    ElderBough,
    Growth,
    Seedlings,
    StormSprite,
    SkyDarter,
    Thunderwright,
    TempestLord,
    Lightning,
    Thunderfall,
    StormScroll,
    SpineViper,
    BogLurker,
    AshHerald,
    StoneScribe,
    FrostBind,
    Sapling,
    Bulwark,
    SaltWard,
    WardStone,
}

pub const ALL_CARDS: [CardId; 39] = [
    CardId::Spearhand,
    CardId::Shieldbearer,
    CardId::Mercenary,
    CardId::Knight,
    CardId::Champion,
    CardId::Giant,
    CardId::EmberMote,
    CardId::FlameHound,
    CardId::CinderKnight,
    CardId::BlazeDrake,
    CardId::EmberBolt,
    CardId::Wildfire,
    CardId::DragonBreath,
    CardId::Wavesinger,
    CardId::ReefGuardian,
    CardId::DeepWard,
    CardId::Kraken,
    CardId::TideSurge,
    CardId::MistVeil,
    CardId::Sprout,
    CardId::GroveWarden,
    CardId::ElderBough,
    CardId::Growth,
    CardId::Seedlings,
    CardId::StormSprite,
    CardId::SkyDarter,
    CardId::Thunderwright,
    CardId::TempestLord,
    CardId::Lightning,
    CardId::Thunderfall,
    CardId::StormScroll,
    CardId::SpineViper,
    CardId::BogLurker,
    CardId::AshHerald,
    CardId::StoneScribe,
    CardId::FrostBind,
    CardId::Bulwark,
    CardId::SaltWard,
    CardId::WardStone,
];

const fn minion(
    name: &'static str,
    cost: u32,
    element: Element,
    attack: i32,
    health: i32,
    text: &'static str,
) -> CardDef {
    CardDef {
        name,
        cost,
        element,
        kind: Kind::Minion {
            attack,
            health,
            guard: false,
            rush: false,
            shield: false,
            venom: false,
            drain: false,
            warcry: None,
        },
        text,
    }
}

const fn guard(
    name: &'static str,
    cost: u32,
    element: Element,
    attack: i32,
    health: i32,
) -> CardDef {
    CardDef {
        name,
        cost,
        element,
        kind: Kind::Minion {
            attack,
            health,
            guard: true,
            rush: false,
            shield: false,
            venom: false,
            drain: false,
            warcry: None,
        },
        text: "Guard",
    }
}

const fn rush(
    name: &'static str,
    cost: u32,
    element: Element,
    attack: i32,
    health: i32,
) -> CardDef {
    CardDef {
        name,
        cost,
        element,
        kind: Kind::Minion {
            attack,
            health,
            guard: false,
            rush: true,
            shield: false,
            venom: false,
            drain: false,
            warcry: None,
        },
        text: "Rush",
    }
}

const fn shielded(
    name: &'static str,
    cost: u32,
    element: Element,
    attack: i32,
    health: i32,
) -> CardDef {
    CardDef {
        name,
        cost,
        element,
        kind: Kind::Minion {
            attack,
            health,
            guard: false,
            rush: false,
            shield: true,
            venom: false,
            drain: false,
            warcry: None,
        },
        text: "Shield",
    }
}

const fn venomous(
    name: &'static str,
    cost: u32,
    element: Element,
    attack: i32,
    health: i32,
) -> CardDef {
    CardDef {
        name,
        cost,
        element,
        kind: Kind::Minion {
            attack,
            health,
            guard: false,
            rush: false,
            shield: false,
            venom: true,
            drain: false,
            warcry: None,
        },
        text: "Venom",
    }
}

const fn draining(
    name: &'static str,
    cost: u32,
    element: Element,
    attack: i32,
    health: i32,
) -> CardDef {
    CardDef {
        name,
        cost,
        element,
        kind: Kind::Minion {
            attack,
            health,
            guard: false,
            rush: false,
            shield: false,
            venom: false,
            drain: true,
            warcry: None,
        },
        text: "Drain",
    }
}

const fn herald(
    name: &'static str,
    cost: u32,
    element: Element,
    attack: i32,
    health: i32,
    warcry: Spell,
    text: &'static str,
) -> CardDef {
    CardDef {
        name,
        cost,
        element,
        kind: Kind::Minion {
            attack,
            health,
            guard: false,
            rush: false,
            shield: false,
            venom: false,
            drain: false,
            warcry: Some(warcry),
        },
        text,
    }
}

const fn spell(
    name: &'static str,
    cost: u32,
    element: Element,
    effect: Spell,
    target: TargetRule,
    text: &'static str,
) -> CardDef {
    CardDef {
        name,
        cost,
        element,
        kind: Kind::Spell { effect, target },
        text,
    }
}

impl CardId {
    pub fn def(self) -> CardDef {
        match self {
            CardId::Spearhand => minion("Spearhand", 1, Element::Neutral, 1, 2, ""),
            CardId::Shieldbearer => guard("Shieldbearer", 2, Element::Neutral, 1, 4),
            CardId::Mercenary => minion("Mercenary", 3, Element::Neutral, 3, 3, ""),
            CardId::Knight => minion("Knight", 4, Element::Neutral, 4, 4, ""),
            CardId::Champion => shielded("Champion", 5, Element::Neutral, 5, 5),
            CardId::Giant => minion("Giant", 7, Element::Neutral, 8, 8, ""),
            CardId::EmberMote => minion("Ember Mote", 1, Element::Ember, 2, 1, ""),
            CardId::FlameHound => rush("Flame Hound", 2, Element::Ember, 3, 2),
            CardId::CinderKnight => minion("Cinder Knight", 3, Element::Ember, 4, 2, ""),
            CardId::BlazeDrake => minion("Blaze Drake", 5, Element::Ember, 5, 4, ""),
            CardId::EmberBolt => spell(
                "Ember Bolt",
                2,
                Element::Ember,
                Spell::Damage(3),
                TargetRule::AnyEnemy,
                "Deal 3 damage",
            ),
            CardId::Wildfire => spell(
                "Wildfire",
                4,
                Element::Ember,
                Spell::Storm(2),
                TargetRule::None,
                "2 damage to all enemy minions",
            ),
            CardId::DragonBreath => spell(
                "Dragon Breath",
                6,
                Element::Ember,
                Spell::Damage(6),
                TargetRule::AnyEnemy,
                "Deal 6 damage",
            ),
            CardId::Wavesinger => minion("Wavesinger", 2, Element::Tide, 2, 3, ""),
            CardId::ReefGuardian => guard("Reef Guardian", 4, Element::Tide, 3, 6),
            CardId::DeepWard => shielded("Deep Ward", 3, Element::Tide, 2, 5),
            CardId::Kraken => guard("Kraken", 7, Element::Tide, 7, 7),
            CardId::TideSurge => spell(
                "Tide Surge",
                2,
                Element::Tide,
                Spell::Heal(6),
                TargetRule::AnyCharacter,
                "Restore 6 health",
            ),
            CardId::MistVeil => spell(
                "Mist Veil",
                1,
                Element::Tide,
                Spell::Shield,
                TargetRule::FriendlyMinion,
                "Give a minion Shield",
            ),
            CardId::Sprout => minion("Sprout", 1, Element::Grove, 1, 3, ""),
            CardId::Sapling => minion("Sapling", 1, Element::Grove, 1, 1, ""),
            CardId::GroveWarden => guard("Grove Warden", 3, Element::Grove, 3, 4),
            CardId::ElderBough => guard("Elder Bough", 6, Element::Grove, 6, 7),
            CardId::Growth => spell(
                "Growth",
                2,
                Element::Grove,
                Spell::Buff(2, 2),
                TargetRule::FriendlyMinion,
                "Give a minion +2/+2",
            ),
            CardId::Seedlings => spell(
                "Seedlings",
                3,
                Element::Grove,
                Spell::Summon(CardId::Sprout, 2),
                TargetRule::None,
                "Summon two Sprouts",
            ),
            CardId::StormSprite => rush("Storm Sprite", 1, Element::Storm, 1, 1),
            CardId::SkyDarter => rush("Sky Darter", 2, Element::Storm, 3, 1),
            CardId::Thunderwright => minion("Thunderwright", 4, Element::Storm, 4, 3, ""),
            CardId::TempestLord => shielded("Tempest Lord", 6, Element::Storm, 5, 6),
            CardId::Lightning => spell(
                "Lightning",
                1,
                Element::Storm,
                Spell::Damage(2),
                TargetRule::EnemyMinion,
                "2 damage to a minion",
            ),
            CardId::Thunderfall => spell(
                "Thunderfall",
                5,
                Element::Storm,
                Spell::Storm(3),
                TargetRule::None,
                "3 damage to all enemy minions",
            ),
            CardId::SpineViper => venomous("Spine Viper", 3, Element::Grove, 2, 2),
            CardId::BogLurker => draining("Bog Lurker", 4, Element::Tide, 3, 4),
            CardId::AshHerald => herald(
                "Ash Herald",
                3,
                Element::Ember,
                2,
                2,
                Spell::Storm(1),
                "Warcry: 1 damage to all enemy minions",
            ),
            CardId::StoneScribe => herald(
                "Stone Scribe",
                4,
                Element::Neutral,
                3,
                3,
                Spell::Draw(1),
                "Warcry: draw a card",
            ),
            CardId::FrostBind => spell(
                "Frost Bind",
                1,
                Element::Tide,
                Spell::Freeze,
                TargetRule::EnemyMinion,
                "Freeze a minion",
            ),
            CardId::StormScroll => spell(
                "Storm Scroll",
                2,
                Element::Storm,
                Spell::Draw(2),
                TargetRule::None,
                "Draw two cards",
            ),
            CardId::Bulwark => spell(
                "Bulwark",
                2,
                Element::Grove,
                Spell::Armor(6),
                TargetRule::None,
                "Gain 6 armor",
            ),
            CardId::SaltWard => spell(
                "Salt Ward",
                1,
                Element::Tide,
                Spell::Armor(3),
                TargetRule::None,
                "Gain 3 armor",
            ),
            CardId::WardStone => herald(
                "Ward Stone",
                3,
                Element::Neutral,
                2,
                4,
                Spell::Armor(3),
                "Warcry: gain 3 armor",
            ),
        }
    }

    pub fn forge_number(self) -> bool {
        match self.def().kind {
            Kind::Minion { .. } => true,
            Kind::Spell { effect, .. } => !matches!(effect, Spell::Shield | Spell::Freeze),
        }
    }

    pub fn target_rule(self) -> TargetRule {
        match self.def().kind {
            Kind::Minion { .. } => TargetRule::None,
            Kind::Spell { target, .. } => target,
        }
    }

    pub fn sigil(self) -> char {
        match self {
            CardId::Sapling => '\u{e1eb}',
            CardId::Spearhand => '\u{e2b3}',
            CardId::Shieldbearer => '\u{e158}',
            CardId::Mercenary => '\u{e2b4}',
            CardId::Knight => '\u{e517}',
            CardId::Champion => '\u{e1d6}',
            CardId::Giant => '\u{e231}',
            CardId::EmberMote => '\u{e0d2}',
            CardId::FlameHound => '\u{e4f5}',
            CardId::CinderKnight => '\u{e050}',
            CardId::BlazeDrake => '\u{e59d}',
            CardId::EmberBolt => '\u{e0ac}',
            CardId::Wildfire => '\u{e53a}',
            CardId::DragonBreath => '\u{e178}',
            CardId::Wavesinger => '\u{e0b4}',
            CardId::ReefGuardian => '\u{e4f7}',
            CardId::DeepWard => '\u{e03f}',
            CardId::Kraken => '\u{e3a6}',
            CardId::TideSurge => '\u{e0b5}',
            CardId::MistVeil => '\u{e088}',
            CardId::Sprout => '\u{e1eb}',
            CardId::GroveWarden => '\u{e2f4}',
            CardId::ElderBough => '\u{e2f5}',
            CardId::Growth => '\u{e2de}',
            CardId::Seedlings => '\u{e2d4}',
            CardId::StormSprite => '\u{e47e}',
            CardId::SkyDarter => '\u{e3c5}',
            CardId::Thunderwright => '\u{e08c}',
            CardId::TempestLord => '\u{e218}',
            CardId::Lightning => '\u{e1b4}',
            CardId::Thunderfall => '\u{e1b0}',
            CardId::StormScroll => '\u{e2ed}',
            CardId::SpineViper => '\u{e20c}',
            CardId::BogLurker => '\u{e4f8}',
            CardId::AshHerald => '\u{e0be}',
            CardId::StoneScribe => '\u{e2ed}',
            CardId::FrostBind => '\u{e165}',
            CardId::Bulwark => '\u{e158}',
            CardId::SaltWard => '\u{e03f}',
            CardId::WardStone => '\u{e2ee}',
        }
    }
}
