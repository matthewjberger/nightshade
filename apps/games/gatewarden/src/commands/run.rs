use crate::commands::battle::{BattleSetup, new_battle};
use crate::commands::chapters::chapter_battle;
use crate::commands::shop::{gift_card, stock_offers};
use crate::commands::tutorial::training_battle;
use crate::contract::{Outcome, Screen};
use crate::data::battle::{Battle, Who};
use crate::data::chapters::CHAPTER_COUNT;
use crate::data::climb::{ALL_STOPS, HEARTH_HEAL, LANES, STEP_COLUMNS, Stop, TROVE_GOLD, step_at};
use crate::data::decks::{STARTING_HEALTH, WARDEN_CHOICES};
use crate::data::random::random_below;
use crate::data::shop::{
    DECK_MAX, GATE_COUNT, Offer, RELIC_SLOTS, REROLL_COST, STRIKE_STEP, gate_reward,
};
use crate::data::story::{
    BORROW_WARDEN_HEALTH, ForkOutcome, SPARE_CARDS, THIN_COUNT, THIN_WARDEN_HEALTH, shard_for,
};
use crate::queries::run::warden_pick;
use crate::queries::run::{
    boon_for, can_buy, can_choose, can_strike, decide_ending, strike_target,
};
use crate::resources::{
    Cursor, DEFAULT_SEED, GatewardenResources, Mode, Report, Run, SceneKind, Selection, Theater,
    Tutorial,
};
use std::collections::HashSet;

pub enum BattleEnd {
    Won,
    Lost,
}

pub fn start_run(resources: &mut GatewardenResources, entropy: u64) {
    let mut seed = resources.run.seed ^ entropy.rotate_left(29);
    if seed == 0 {
        seed = DEFAULT_SEED;
    }
    let mut roster = [0u8; GATE_COUNT as usize];
    for slot in roster.iter_mut() {
        *slot = random_below(&mut seed, WARDEN_CHOICES) as u8;
    }
    let mut stops = [[Stop::Bazaar; LANES]; STEP_COLUMNS];
    for column in stops.iter_mut() {
        let mut pool = ALL_STOPS.to_vec();
        for lane in column.iter_mut() {
            let pick = random_below(&mut seed, pool.len());
            *lane = pool.swap_remove(pick);
        }
    }
    let mut relics = Vec::with_capacity(RELIC_SLOTS);
    relics.extend(boon_for(resources.best_gate));
    resources.run = Run {
        seed,
        scene: SceneKind::Prologue,
        roster,
        stops,
        relics,
        ..Run::default()
    };
    resources.tutorial = Tutorial::default();
    clear_table(resources);
    resources.battle = None;
}

pub fn start_story(resources: &mut GatewardenResources) {
    resources.run = Run {
        mode: Mode::Story,
        scene: SceneKind::Chapter(0),
        ..Run::default()
    };
    resources.tutorial = Tutorial::default();
    clear_table(resources);
    resources.battle = None;
}

pub fn restart_chapter(resources: &mut GatewardenResources) {
    let index = resources.run.chapter;
    resources.run = Run {
        mode: Mode::Story,
        chapter: index,
        scene: SceneKind::Chapter(index),
        ..Run::default()
    };
    clear_table(resources);
    resources.battle = None;
}

pub fn start_chapter(resources: &mut GatewardenResources, index: usize) {
    clear_table(resources);
    let run = &mut resources.run;
    run.chapter = index;
    run.seed = run
        .seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    let battle = chapter_battle(index, run.seed);
    install_battle(resources, battle);
}

pub fn start_tutorial(resources: &mut GatewardenResources) {
    resources.run = Run {
        scene: SceneKind::Yard,
        ..Run::default()
    };
    resources.tutorial = Tutorial {
        active: true,
        ..Tutorial::default()
    };
    clear_table(resources);
    resources.battle = None;
}

pub fn start_battle(resources: &mut GatewardenResources) {
    clear_table(resources);
    let run = &mut resources.run;
    run.seed = run
        .seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    let battle: Battle = if resources.tutorial.active {
        training_battle(run.seed)
    } else {
        new_battle(BattleSetup {
            gate: run.gate,
            pick: warden_pick(run),
            deck: &run.deck,
            forged: &run.forged,
            health: match run.mode {
                Mode::Climb => Some(run.health),
                Mode::Story => None,
            },
            relics: &run.relics,
            shards: &run.shards,
            borrowed_bolt: run.borrowed_bolt,
            warden_bonus: run.warden_bonus,
            seed: run.seed,
        })
    };
    run.borrowed_bolt = false;
    run.warden_bonus = 0;
    install_battle(resources, battle);
}

fn install_battle(resources: &mut GatewardenResources, battle: Battle) {
    let standing: HashSet<u32> = battle
        .player
        .board
        .iter()
        .chain(battle.enemy.board.iter())
        .map(|minion| minion.id)
        .collect();
    resources.theater = Theater {
        revealed: standing,
        shown_player_health: battle.player.hero_health,
        shown_enemy_health: battle.enemy.hero_health,
        ..Theater::default()
    };
    for minion in battle.player.board.iter().chain(battle.enemy.board.iter()) {
        resources.theater.revealed.insert(minion.id);
    }
    resources.battle = Some(battle);
    resources.selection = Selection::None;
    resources.cursor = Cursor::default();
}

fn clear_table(resources: &mut GatewardenResources) {
    let table = &mut resources.table;
    table.generation += 1;
    table.last_positions.clear();
}

pub fn finish_battle(resources: &mut GatewardenResources) -> BattleEnd {
    let Some(battle) = resources.battle.take() else {
        return BattleEnd::Lost;
    };
    clear_table(resources);
    resources.best_gate = resources.best_gate.max(resources.run.gate);
    let run = &mut resources.run;
    run.final_gate = run.gate;
    run.health = battle.player.hero_health.max(1);
    run.report = Report {
        gate: run.gate,
        turns: battle.turn,
        warden: battle.enemy.name.clone(),
        deck: run.deck.len(),
        gold: run.gold,
    };
    if battle.outcome != Some(Who::Player) {
        run.outcome = Outcome::Lost;
        resources.tutorial.active = false;
        return BattleEnd::Lost;
    }
    if resources.tutorial.active {
        run.scene = SceneKind::YardEnd;
        return BattleEnd::Won;
    }
    if run.mode == Mode::Story {
        run.scene = SceneKind::ChapterWon(run.chapter);
        return BattleEnd::Won;
    }
    run.gold += gate_reward(run.gate, &run.relics);
    if let Some(shard) = shard_for(run.gate) {
        run.shards.push(shard);
    }
    run.scene = SceneKind::Victory {
        gate: run.gate,
        spared: false,
    };
    BattleEnd::Won
}

pub fn continue_scene(resources: &mut GatewardenResources) -> Screen {
    let run = &mut resources.run;
    match run.scene {
        SceneKind::Prologue => {
            run.scene = SceneKind::Arrival(run.gate);
            Screen::Map
        }
        SceneKind::Arrival(_) => {
            start_battle(resources);
            Screen::Battle
        }
        SceneKind::Victory { gate, .. } => {
            if gate >= GATE_COUNT {
                run.outcome = Outcome::Won;
                run.ending = decide_ending(run.shards.len(), run.mercy);
                run.scene = SceneKind::Ending(run.ending);
                Screen::Scene
            } else {
                run.gate += 1;
                run.column += 1;
                run.trail = String::new();
                Screen::Map
            }
        }
        SceneKind::Ending(_) => Screen::Over,
        SceneKind::Yard => {
            start_battle(resources);
            Screen::Battle
        }
        SceneKind::YardEnd => {
            resources.tutorial.active = false;
            Screen::Title
        }
        SceneKind::Chapter(index) => {
            start_chapter(resources, index);
            Screen::Battle
        }
        SceneKind::ChapterWon(index) => {
            let next = index + 1;
            if next >= CHAPTER_COUNT {
                run.scene = SceneKind::StoryEnd;
            } else {
                run.chapter = next;
                run.scene = SceneKind::Chapter(next);
            }
            Screen::Scene
        }
        SceneKind::StoryEnd => Screen::Title,
    }
}

pub fn choose(resources: &mut GatewardenResources, outcome: ForkOutcome) -> Screen {
    if !can_choose(&resources.run, outcome) {
        return Screen::Scene;
    }
    let run = &mut resources.run;
    match outcome {
        ForkOutcome::Duel => continue_scene(resources),
        ForkOutcome::Tithe { gold } => {
            run.gold -= gold;
            run.mercy += 1;
            if let Some(shard) = shard_for(run.gate) {
                run.shards.push(shard);
            }
            run.final_gate = run.gate;
            run.scene = SceneKind::Victory {
                gate: run.gate,
                spared: true,
            };
            Screen::Scene
        }
        ForkOutcome::Borrow => {
            run.borrowed_bolt = true;
            run.warden_bonus = BORROW_WARDEN_HEALTH;
            start_battle(resources);
            Screen::Battle
        }
        ForkOutcome::Spare => {
            run.deck.extend_from_slice(&SPARE_CARDS);
            run.mercy += 1;
            run.final_gate = run.gate;
            run.scene = SceneKind::Victory {
                gate: run.gate,
                spared: true,
            };
            Screen::Scene
        }
        ForkOutcome::Thin => {
            for _ in 0..THIN_COUNT {
                let Some(index) = run
                    .deck
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, card)| card.def().cost)
                    .map(|(index, _)| index)
                else {
                    break;
                };
                run.deck.remove(index);
            }
            run.warden_bonus = THIN_WARDEN_HEALTH;
            start_battle(resources);
            Screen::Battle
        }
    }
}

pub fn leave_shop(run: &mut Run) -> Screen {
    run.scene = SceneKind::Arrival(run.gate);
    Screen::Map
}

pub fn take_stop(resources: &mut GatewardenResources, lane: usize) -> Screen {
    let run = &mut resources.run;
    let Some(step) = step_at(run.column) else {
        return Screen::Map;
    };
    if lane >= LANES {
        return Screen::Map;
    }
    let stop = run.stops[step][lane];
    run.roads[step] = Some(lane as u8);
    run.column += 1;
    match stop {
        Stop::Bazaar => {
            run.trail = "The bazaar opens its cloth".to_string();
            enter_shop(run);
            Screen::Shop
        }
        Stop::Trove => {
            run.gold += TROVE_GOLD;
            run.trail = format!("You take {TROVE_GOLD} gold from under the stones");
            Screen::Map
        }
        Stop::Camp => {
            match strike_target(run) {
                Some(card) => {
                    if let Some(index) = run.deck.iter().position(|held| *held == card) {
                        run.deck.remove(index);
                    }
                    run.trail = format!("You strike {} from the deck", card.def().name);
                }
                None => run.trail = "The deck is already lean".to_string(),
            }
            Screen::Map
        }
        Stop::Hearth => {
            let before = run.health;
            run.health = (run.health + HEARTH_HEAL).min(STARTING_HEALTH);
            let mended = run.health - before;
            run.trail = if mended > 0 {
                format!("The fire mends {mended} health")
            } else {
                "The fire finds nothing to mend".to_string()
            };
            Screen::Map
        }
        Stop::Echo => {
            match gift_card(&mut run.seed, run.gate) {
                Some(card) if run.deck.len() < DECK_MAX => {
                    run.deck.push(card);
                    run.trail = format!("An echo hands you {}", card.def().name);
                }
                _ => run.trail = "The echo has nothing your deck can hold".to_string(),
            }
            Screen::Map
        }
    }
}

fn enter_shop(run: &mut Run) {
    run.shop_generation += 1;
    run.reroll_cost = REROLL_COST;
    run.offers = stock_offers(&mut run.seed, &run.relics, run.gate, &run.deck, &run.forged);
}

pub fn reroll(run: &mut Run) {
    if run.gold < run.reroll_cost {
        return;
    }
    run.gold -= run.reroll_cost;
    run.reroll_cost += 1;
    run.shop_generation += 1;
    run.offers = stock_offers(&mut run.seed, &run.relics, run.gate, &run.deck, &run.forged);
}

pub fn strike(run: &mut Run) {
    if !can_strike(run) {
        return;
    }
    let Some(card) = strike_target(run) else {
        return;
    };
    let Some(index) = run.deck.iter().position(|held| *held == card) else {
        return;
    };
    run.deck.remove(index);
    run.gold -= run.strike_cost;
    run.strike_cost += STRIKE_STEP;
}

pub fn buy(run: &mut Run, index: usize) {
    let Some(offer) = run.offers.get(index).copied().flatten() else {
        return;
    };
    if !can_buy(run, offer) {
        return;
    }
    run.gold -= offer.cost();
    match offer {
        Offer::Card(card) => run.deck.push(card),
        Offer::Relic(relic) => run.relics.push(relic),
        Offer::Forge(card) => run.forged.push(card),
        Offer::Purge(card) => {
            if let Some(index) = run.deck.iter().position(|held| *held == card) {
                run.deck.remove(index);
            }
            if let Some(index) = run.forged.iter().position(|held| *held == card) {
                run.forged.remove(index);
            }
        }
    }
    run.offers[index] = None;
    run.shop_generation += 1;
}
