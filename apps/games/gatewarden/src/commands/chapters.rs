use crate::commands::battle::{BattleSetup, draw, new_battle, summon};
use crate::data::battle::{Battle, Event, Instance, OPENING_HAND, SECOND_PLAYER_BONUS, Who};
use crate::data::cards::CardId;
use crate::data::chapters::{DECK_DEPTH, FOE_DECK_DEPTH, chapter};

fn stack(cards: &[CardId], depth: usize) -> Vec<CardId> {
    let mut out = Vec::with_capacity(depth);
    while out.len() < depth && !cards.is_empty() {
        for card in cards {
            if out.len() == depth {
                break;
            }
            out.push(*card);
        }
    }
    out
}

pub fn chapter_battle(index: usize, seed: u64) -> Battle {
    let sheet = chapter(index);
    let mut battle = new_battle(BattleSetup {
        gate: 1,
        pick: 0,
        deck: &[],
        forged: &[],
        health: None,
        relics: &[],
        shards: &[],
        borrowed_bolt: false,
        warden_bonus: 0,
        seed,
    });
    battle.enemy.name = sheet.foe.to_string();
    battle.enemy.hero_health = sheet.health;
    battle.enemy.hero_max = sheet.health;
    battle.enemy.power = sheet.power;
    battle.enemy.power_cost = sheet.power.cost();
    battle.enemy.aggressive = sheet.aggressive;
    battle.enemy.shards.clear();
    battle.enemy.spell_bonus = 0;
    battle.player.hand.clear();
    battle.player.deck.clear();
    battle.enemy.hand.clear();
    battle.enemy.deck.clear();
    for card in sheet.hand {
        let id = battle.allocate();
        battle.player.hand.push(Instance {
            id,
            card: *card,
            upgraded: false,
        });
    }
    for card in stack(sheet.deck, DECK_DEPTH).iter().rev() {
        let id = battle.allocate();
        battle.player.deck.push(Instance {
            id,
            card: *card,
            upgraded: false,
        });
    }
    for card in stack(sheet.foe_deck, FOE_DECK_DEPTH).iter().rev() {
        let id = battle.allocate();
        battle.enemy.deck.push(Instance {
            id,
            card: *card,
            upgraded: false,
        });
    }
    for _ in 0..OPENING_HAND + SECOND_PLAYER_BONUS {
        draw(&mut battle, Who::Enemy);
    }
    for card in sheet.board {
        let origin = battle.allocate();
        summon(&mut battle, Who::Player, *card, origin, 0);
    }
    for card in sheet.foe_board {
        let origin = battle.allocate();
        summon(&mut battle, Who::Enemy, *card, origin, 0);
    }
    for minion in battle.player.board.iter_mut() {
        minion.ready = true;
    }
    if sheet.mana > 0 {
        battle.player.max_mana = sheet.mana;
        battle.player.mana = sheet.mana;
    }
    battle.events.clear();
    battle.events.push_back(Event::Turn {
        who: Who::Player,
        turn: battle.turn,
    });
    battle
}
