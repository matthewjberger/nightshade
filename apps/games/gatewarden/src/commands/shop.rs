use crate::data::cards::{ALL_CARDS, CardId};
use crate::data::random::random_below;
use crate::data::relics::{ALL_RELICS, Relic};
use crate::data::shop::{
    CARD_OFFERS, GIFT_DRAWS, Offer, POOL_COST_ABOVE_GATE, RELIC_OFFERS, SERVICE_OFFERS,
};
use crate::theme::OFFER_COUNT;

pub fn gift_card(seed: &mut u64, gate: u32) -> Option<CardId> {
    let ceiling = gate + POOL_COST_ABOVE_GATE;
    let pool: Vec<CardId> = ALL_CARDS
        .iter()
        .copied()
        .filter(|card| card.def().cost <= ceiling)
        .collect();
    let mut best: Option<CardId> = None;
    for _ in 0..GIFT_DRAWS {
        let pick = random_below(seed, pool.len());
        let Some(card) = pool.get(pick).copied() else {
            continue;
        };
        if best.is_none_or(|held| held.def().cost < card.def().cost) {
            best = Some(card);
        }
    }
    best
}

pub fn stock_offers(
    seed: &mut u64,
    owned: &[Relic],
    gate: u32,
    deck: &[CardId],
    forged: &[CardId],
) -> [Option<Offer>; OFFER_COUNT] {
    let mut offers = [None; OFFER_COUNT];
    let ceiling = gate + POOL_COST_ABOVE_GATE;
    let mut pool: Vec<CardId> = ALL_CARDS
        .iter()
        .copied()
        .filter(|card| card.def().cost <= ceiling)
        .collect();
    for slot in offers.iter_mut().take(CARD_OFFERS) {
        let pick = random_below(seed, pool.len());
        *slot = Some(Offer::Card(pool.swap_remove(pick)));
    }
    let mut relics: Vec<Relic> = ALL_RELICS
        .iter()
        .copied()
        .filter(|relic| !owned.contains(relic))
        .collect();
    for slot in offers.iter_mut().skip(CARD_OFFERS).take(RELIC_OFFERS) {
        if relics.is_empty() {
            break;
        }
        let pick = random_below(seed, relics.len());
        *slot = Some(Offer::Relic(relics.swap_remove(pick)));
    }
    for slot in offers
        .iter_mut()
        .skip(CARD_OFFERS + RELIC_OFFERS)
        .take(SERVICE_OFFERS)
    {
        *slot = service_offer(seed, deck, forged);
    }
    offers
}

fn service_offer(seed: &mut u64, deck: &[CardId], forged: &[CardId]) -> Option<Offer> {
    if deck.is_empty() {
        return None;
    }
    let plain: Vec<CardId> = {
        let mut pending = forged.to_vec();
        deck.iter()
            .copied()
            .filter(|card| match pending.iter().position(|held| held == card) {
                Some(index) => {
                    pending.remove(index);
                    false
                }
                None => true,
            })
            .collect()
    };
    if plain.is_empty() {
        return None;
    }
    let pick = random_below(seed, plain.len());
    let card = plain[pick];
    if random_below(seed, 2) == 0 {
        Some(Offer::Forge(card))
    } else {
        Some(Offer::Purge(card))
    }
}
