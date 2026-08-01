use crate::commands::battle::{BattleSetup, new_battle};
use crate::data::battle::{Battle, Event, Instance, Who};
use crate::data::tutorial::{
    Deed, PLAYER_DECK, PLAYER_HAND, STRAW_DECK, STRAW_HAND, STRAW_HEALTH, STRAW_WARDEN,
};
use crate::queries::tutorial::{gate, gate_opens};
use crate::resources::Tutorial;

pub fn training_battle(seed: u64) -> Battle {
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
    battle.enemy.name = STRAW_WARDEN.to_string();
    battle.enemy.hero_health = STRAW_HEALTH;
    battle.enemy.hero_max = STRAW_HEALTH;
    battle.enemy.aggressive = false;
    battle.enemy.shards.clear();
    battle.enemy.spell_bonus = 0;
    battle.player.hand.clear();
    battle.player.deck.clear();
    battle.enemy.hand.clear();
    battle.enemy.deck.clear();
    for card in PLAYER_HAND {
        let id = battle.allocate();
        battle.player.hand.push(Instance {
            id,
            card,
            upgraded: false,
        });
    }
    for card in PLAYER_DECK {
        let id = battle.allocate();
        battle.player.deck.push(Instance {
            id,
            card,
            upgraded: false,
        });
    }
    for card in STRAW_HAND {
        let id = battle.allocate();
        battle.enemy.hand.push(Instance {
            id,
            card,
            upgraded: false,
        });
    }
    for card in STRAW_DECK {
        let id = battle.allocate();
        battle.enemy.deck.push(Instance {
            id,
            card,
            upgraded: false,
        });
    }
    battle.events.clear();
    battle.events.push_back(Event::Turn {
        who: Who::Player,
        turn: battle.turn,
    });
    battle
}

pub fn record(tutorial: &mut Tutorial, done: Deed) {
    let Some(current) = gate(*tutorial) else {
        return;
    };
    if gate_opens(current, done) {
        tutorial.step += 1;
    }
}
