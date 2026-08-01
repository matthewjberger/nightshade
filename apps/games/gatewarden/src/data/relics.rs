#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Relic {
    #[default]
    DeepWell,
    WardingCharm,
    BattleStandard,
    SparkFocus,
    ScholarsQuill,
    GoldPurse,
    Whetstone,
    StoneCollar,
    TideFlask,
    KilnCoal,
    QuickBoots,
    TinLedger,
    GlassLens,
    ThickHide,
}

pub const ALL_RELICS: [Relic; 14] = [
    Relic::DeepWell,
    Relic::WardingCharm,
    Relic::BattleStandard,
    Relic::SparkFocus,
    Relic::ScholarsQuill,
    Relic::GoldPurse,
    Relic::Whetstone,
    Relic::StoneCollar,
    Relic::TideFlask,
    Relic::KilnCoal,
    Relic::QuickBoots,
    Relic::TinLedger,
    Relic::GlassLens,
    Relic::ThickHide,
];

impl Relic {
    pub fn name(self) -> &'static str {
        match self {
            Relic::DeepWell => "Deep Well",
            Relic::WardingCharm => "Warding Charm",
            Relic::BattleStandard => "Battle Standard",
            Relic::SparkFocus => "Spark Focus",
            Relic::ScholarsQuill => "Scholar's Quill",
            Relic::GoldPurse => "Gold Purse",
            Relic::Whetstone => "Whetstone",
            Relic::StoneCollar => "Stone Collar",
            Relic::TideFlask => "Tide Flask",
            Relic::KilnCoal => "Kiln Coal",
            Relic::QuickBoots => "Quick Boots",
            Relic::TinLedger => "Tin Ledger",
            Relic::GlassLens => "Glass Lens",
            Relic::ThickHide => "Thick Hide",
        }
    }

    pub fn text(self) -> &'static str {
        match self {
            Relic::DeepWell => "Start each battle with 1 extra mana",
            Relic::WardingCharm => "Your hero has +5 health",
            Relic::BattleStandard => "Your minions get +1 health",
            Relic::SparkFocus => "Your hero power costs 1",
            Relic::ScholarsQuill => "Draw an extra card at the start",
            Relic::GoldPurse => "+3 gold for each gate",
            Relic::Whetstone => "Your minions get +1 attack",
            Relic::StoneCollar => "Your Guard minions get +2 health",
            Relic::TideFlask => "Restore 2 health each of your turns",
            Relic::KilnCoal => "1 damage to the warden each of your turns",
            Relic::QuickBoots => "Your minions can attack the turn you play them",
            Relic::TinLedger => "+2 gold for each gate",
            Relic::GlassLens => "Your damage spells deal 1 more",
            Relic::ThickHide => "Your hero has +8 health",
        }
    }

    pub fn cost(self) -> i64 {
        match self {
            Relic::QuickBoots => 9,
            Relic::KilnCoal => 8,
            Relic::DeepWell | Relic::SparkFocus | Relic::Whetstone | Relic::TideFlask => 7,
            Relic::WardingCharm
            | Relic::BattleStandard
            | Relic::ScholarsQuill
            | Relic::GlassLens
            | Relic::ThickHide => 6,
            Relic::GoldPurse | Relic::StoneCollar => 5,
            Relic::TinLedger => 4,
        }
    }
}
