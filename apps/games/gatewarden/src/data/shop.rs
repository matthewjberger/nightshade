use crate::data::cards::CardId;
use crate::data::relics::Relic;

pub const GATE_COUNT: u32 = 8;
pub const RELIC_SLOTS: usize = 5;
pub const STARTING_GOLD: i64 = 6;
pub const CARD_BASE_PRICE: i64 = 2;
pub const REROLL_COST: i64 = 2;
pub const POOL_COST_ABOVE_GATE: u32 = 3;
pub const DECK_MAX: usize = 30;
pub const CARD_OFFERS: usize = 3;
pub const RELIC_OFFERS: usize = 1;
pub const SERVICE_OFFERS: usize = 1;
pub const FORGE_BASE_PRICE: i64 = 3;
pub const PURGE_BASE_PRICE: i64 = 3;
pub const STRIKE_COST: i64 = 4;
pub const STRIKE_STEP: i64 = 2;
pub const STRIKE_FLOOR: usize = 14;
pub const GIFT_DRAWS: usize = 3;
pub const GATE_REWARD: i64 = 6;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Offer {
    Card(CardId),
    Relic(Relic),
    Forge(CardId),
    Purge(CardId),
}

impl Offer {
    pub fn name(self) -> String {
        match self {
            Offer::Card(card) => card.def().name.to_string(),
            Offer::Relic(relic) => relic.name().to_string(),
            Offer::Forge(card) => format!("Forge {}", card.def().name),
            Offer::Purge(card) => format!("Strike {}", card.def().name),
        }
    }

    pub fn effect(self) -> String {
        match self {
            Offer::Card(card) => {
                let def = card.def();
                match def.kind {
                    crate::data::cards::Kind::Minion { attack, health, .. } => {
                        if def.text.is_empty() {
                            format!(
                                "{} minion  {attack}/{health}  cost {}",
                                def.element.name(),
                                def.cost
                            )
                        } else {
                            format!(
                                "{} minion  {attack}/{health}  {}  cost {}",
                                def.element.name(),
                                def.text,
                                def.cost
                            )
                        }
                    }
                    crate::data::cards::Kind::Spell { .. } => {
                        format!(
                            "{} spell  {}  cost {}",
                            def.element.name(),
                            def.text,
                            def.cost
                        )
                    }
                }
            }
            Offer::Relic(relic) => relic.text().to_string(),
            Offer::Forge(card) => match card.def().kind {
                crate::data::cards::Kind::Minion { .. } => {
                    "One copy becomes a forged 1 stronger minion".to_string()
                }
                crate::data::cards::Kind::Spell { .. } => {
                    "One copy hits 1 harder or costs 1 less".to_string()
                }
            },
            Offer::Purge(card) => {
                format!("Take one {} out of the deck", card.def().name)
            }
        }
    }

    pub fn cost(self) -> i64 {
        match self {
            Offer::Card(card) => card_price(card),
            Offer::Relic(relic) => relic.cost(),
            Offer::Forge(card) => FORGE_BASE_PRICE + i64::from(card.def().cost),
            Offer::Purge(_) => PURGE_BASE_PRICE,
        }
    }
}

pub fn card_price(card: CardId) -> i64 {
    CARD_BASE_PRICE + i64::from(card.def().cost)
}

pub fn gate_reward(gate: u32, relics: &[Relic]) -> i64 {
    let purse = if relics.contains(&Relic::GoldPurse) {
        3
    } else {
        0
    };
    let ledger = if relics.contains(&Relic::TinLedger) {
        2
    } else {
        0
    };
    GATE_REWARD + i64::from(gate) + purse + ledger
}
