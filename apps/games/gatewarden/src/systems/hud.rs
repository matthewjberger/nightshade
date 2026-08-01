use crate::components::{Locked, Seen, Tall, Tone, Words};
use crate::contract::{Outcome, Screen};
use crate::data::battle::Who;
use crate::data::cards::{Element, Kind};
use crate::data::chapters::{CHAPTER_COUNT, chapter};
use crate::data::climb::{LANES, gate_at, step_at};
use crate::data::decks::warden;
use crate::data::gates::GateRule;
use crate::data::shop::{GATE_COUNT, RELIC_SLOTS};
use crate::data::story::{Ending, defeat_line};
use crate::data::tutorial::{Gate, LESSONS, STRAW_TITLE};
use crate::queries::ai::{Plan, warden_plan};
use crate::queries::battle::{can_use_power, play_block};
use crate::queries::run::boon_for;
use crate::queries::run::{can_buy, can_choose, can_strike, gate_pick, scene_text, strike_target};
use crate::queries::tutorial::gate;
use crate::resources::{GatewardenResources, Mode, SceneKind, Selection};
use crate::theme::{
    CLIMB_SIGIL, ECHO_SIGIL, GATES_SIGIL, INTENT_CALM, INTENT_LETHAL, INTENT_THREAT, MARK_AHEAD,
    MARK_CLEARED, MARK_HERE, NEUTRAL, SCENE_HEIGHT, SCENE_HEIGHT_FORK, SCENE_WIDTH, SHARD_SIGIL,
    STRAW_SIGIL, SUMMIT_SIGIL, TEXT_DIM, TEXT_GOLD, TEXT_PRIMARY, WHOLE_SIGIL, element_color,
    portrait_fill,
};
use nightshade::prelude::*;
use nightshade_ui::components::UiLayoutNode;
use nightshade_ui::prelude::{
    Ab, Anchor, Rl, UiBase, UiLayoutType, WindowLayout, ui_mark_layout_dirty, ui_set_color,
    ui_set_disabled, ui_set_text, ui_set_visible,
};
use nightshade_ui::resources::RetainedUiDirty;

pub type Wordings<'world, 'item> = Query<'world, (&'item mut Words,)>;
pub type Showings<'world, 'item> = Query<'world, (&'item mut Seen,)>;
pub type Lockings<'world, 'item> = Query<'world, (&'item mut Locked,)>;
pub type Tonings<'world, 'item> = Query<'world, (&'item mut Tone,)>;
pub type Tallings<'world, 'item> = Query<'world, (&'item mut Tall,)>;

nightshade::ecs::query_bundle! {
    pub struct Memos<'world, 'item> {
        pub words: Wordings<'world, 'item>,
        pub shown: Showings<'world, 'item>,
        pub locked: Lockings<'world, 'item>,
        pub tones: Tonings<'world, 'item>,
        pub talls: Tallings<'world, 'item>,
    }
}

pub fn sync(
    mut resources: ResMut<GatewardenResources>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    mut nodes: Query<&mut UiLayoutNode>,
    mut written: Memos<'_, '_>,
) {
    let resources = &mut *resources;
    let handles = resources.ui;
    let run = &resources.run;
    let screen = resources.screen;
    let theater = &resources.theater;
    let hovered_card = resources.table.hovered_card;
    let mut writer = Writer {
        written: &mut written,
        dirty: &mut dirty,
        commands: &mut commands,
    };

    let teaching = resources.tutorial;
    let story = run.mode == Mode::Story;
    let (gate_line, gold_line) = if teaching.active {
        (
            "The practice yard".to_string(),
            "Nothing at stake".to_string(),
        )
    } else if story {
        (
            format!("Chapter {} of {CHAPTER_COUNT}", run.chapter + 1),
            chapter(run.chapter).foe.to_string(),
        )
    } else {
        (
            format!("Gate {} of {GATE_COUNT}", run.gate),
            format!("{} gold   Deck {}", run.gold, run.deck.len()),
        )
    };
    writer.text(handles.gate_text, &gate_line);
    writer.text(handles.gold_text, &gold_line);
    let shard_line = if run.shards.is_empty() {
        "No shards yet".to_string()
    } else {
        let names: Vec<&str> = run
            .shards
            .iter()
            .map(|shard| {
                shard
                    .name()
                    .trim_start_matches("Shard of ")
                    .trim_start_matches("the ")
            })
            .collect();
        format!("Shards: {}", names.join(", "))
    };
    writer.text(handles.shard_text, &shard_line);

    let scene = scene_text(run);
    writer.text(handles.scene_speaker, scene.speaker);
    let (scene_sigil, scene_color) = match run.scene {
        SceneKind::Prologue => (CLIMB_SIGIL, TEXT_GOLD),
        SceneKind::Yard | SceneKind::YardEnd => (ECHO_SIGIL, TEXT_GOLD),
        SceneKind::Arrival(gate) => {
            let def = warden(gate, gate_pick(run, gate));
            (def.sigil, element_color(def.element))
        }
        SceneKind::Victory { gate, spared } => {
            let def = warden(gate, gate_pick(run, gate));
            let sigil = if spared { def.sigil } else { SHARD_SIGIL };
            (sigil, element_color(def.element))
        }
        SceneKind::Ending(Ending::Summit) => (SUMMIT_SIGIL, TEXT_GOLD),
        SceneKind::Ending(Ending::Whole) => (WHOLE_SIGIL, TEXT_GOLD),
        SceneKind::Ending(Ending::OpenGates) => (GATES_SIGIL, TEXT_GOLD),
        SceneKind::Chapter(index) | SceneKind::ChapterWon(index) => {
            let sheet = chapter(index);
            (sheet.sigil, element_color(sheet.element))
        }
        SceneKind::StoryEnd => (SUMMIT_SIGIL, TEXT_GOLD),
    };
    writer.text(handles.scene_sigil, &scene_sigil.to_string());
    writer.color(handles.scene_portrait, portrait_fill(scene_color));
    for (index, line) in scene.lines.iter().enumerate() {
        writer.text(handles.scene_lines[index], line);
    }
    let tall = scene.fork.is_some();
    match scene.fork {
        Some(fork) => {
            writer.visible(handles.scene_prompt, true);
            writer.visible(handles.scene_first_text, true);
            writer.visible(handles.scene_second_text, true);
            writer.visible(handles.scene_choices, true);
            writer.visible(handles.scene_continue_button, false);
            writer.text(handles.scene_prompt, fork.prompt);
            writer.text(
                handles.scene_first_text,
                &format!("{}: {}", fork.first.label, fork.first.detail),
            );
            writer.text(
                handles.scene_second_text,
                &format!("{}: {}", fork.second.label, fork.second.detail),
            );
            writer.text(handles.scene_first_label, fork.first.label);
            writer.text(handles.scene_second_label, fork.second.label);
            writer.disabled(
                handles.scene_first_button,
                !can_choose(run, fork.first.outcome),
            );
            writer.disabled(
                handles.scene_second_button,
                !can_choose(run, fork.second.outcome),
            );
        }
        None => {
            writer.visible(handles.scene_prompt, false);
            writer.visible(handles.scene_first_text, false);
            writer.visible(handles.scene_second_text, false);
            writer.visible(handles.scene_choices, false);
            writer.visible(handles.scene_continue_button, true);
        }
    }
    if let Some(panel) = handles.scene_panel
        && writer.tall(panel, tall)
    {
        let height = if tall {
            SCENE_HEIGHT_FORK
        } else {
            SCENE_HEIGHT
        };
        nodes.for_entity(panel, |layout| {
            layout.base_layout = Some(UiLayoutType::Window(WindowLayout {
                position: Rl(vec2(50.0, 50.0)).into(),
                size: Ab(vec2(SCENE_WIDTH, height)).into(),
                anchor: Anchor::Center,
            }));
        });
        ui_mark_layout_dirty(writer.dirty);
    }

    let at_gate = gate_at(run.column);
    let at_step = step_at(run.column);
    for index in 0..handles.map_gate_nodes.len() {
        let gate = index as u32 + 1;
        let def = warden(gate, gate_pick(run, gate));
        writer.text(handles.map_gate_sigils[index], &def.sigil.to_string());
        let (fill, tint) = if gate < run.gate {
            (MARK_CLEARED, NEUTRAL)
        } else if at_gate == Some(gate) {
            (MARK_HERE, element_color(def.element))
        } else {
            (MARK_AHEAD, element_color(def.element))
        };
        writer.color(handles.map_gate_nodes[index], fill);
        writer.color(handles.map_gate_frames[index], portrait_fill(tint));
    }
    for step in 0..handles.map_stop_nodes.len() {
        let road = run.roads[step];
        for lane in 0..LANES {
            let stop = run.stops[step][lane];
            writer.text(
                handles.map_stop_sigils[step][lane],
                &stop.sigil().to_string(),
            );
            writer.text(handles.map_stop_names[step][lane], stop.name());
            let taken = road == Some(lane as u8);
            let (fill, tint, text) = if taken {
                (MARK_CLEARED, NEUTRAL, TEXT_PRIMARY)
            } else if road.is_some() {
                (MARK_AHEAD, NEUTRAL, TEXT_DIM)
            } else if at_step == Some(step) {
                (MARK_HERE, TEXT_GOLD, TEXT_PRIMARY)
            } else {
                (MARK_AHEAD, TEXT_GOLD, TEXT_DIM)
            };
            writer.color(handles.map_stop_nodes[step][lane], fill);
            writer.color(handles.map_stop_frames[step][lane], portrait_fill(tint));
            writer.color(handles.map_stop_names[step][lane], text);
        }
    }
    let map_line = match (at_gate, at_step) {
        (Some(gate), _) => format!(
            "Gate {gate} of {GATE_COUNT}: {} holds it",
            warden(gate, gate_pick(run, gate)).name
        ),
        (_, Some(step)) => {
            let roads: Vec<String> = (0..LANES)
                .map(|lane| {
                    let stop = run.stops[step][lane];
                    format!("{} {}", lane + 1, stop.text())
                })
                .collect();
            format!("Choose your road: {}", roads.join("   "))
        }
        _ => String::new(),
    };
    writer.text(handles.map_line, &map_line);
    writer.text(handles.map_note, &run.trail);
    writer.visible(handles.map_climb_button, at_gate.is_some());
    if let Some(gate) = at_gate {
        writer.text(handles.map_climb_label, &format!("Enter gate {gate}"));
    }

    for index in 0..RELIC_SLOTS {
        let relic = run.relics.get(index).copied();
        writer.visible(handles.relic_slots[index], relic.is_some());
        if let Some(relic) = relic {
            writer.text(handles.relic_names[index], relic.name());
            writer.text(handles.relic_texts[index], relic.text());
        }
    }

    if let Some(battle) = resources.battle.as_ref() {
        let busy = theater.delay > 0.0 || !battle.events.is_empty() || theater.lunge.is_some();
        let player_turn = battle.active == Who::Player && battle.outcome.is_none();
        let warden_def = warden(run.gate, gate_pick(run, run.gate));

        let (enemy_title, enemy_sigil, enemy_color) = if teaching.active {
            (STRAW_TITLE, STRAW_SIGIL, NEUTRAL)
        } else {
            (
                warden_def.title,
                warden_def.sigil,
                element_color(warden_def.element),
            )
        };
        writer.text(handles.enemy_name, &battle.enemy.name);
        writer.text(handles.enemy_title, enemy_title);
        let shard_line = match battle.enemy.shards.first() {
            Some(shard) => format!("{}: {}", shard.name(), shard.text()),
            None => "Holds no shard".to_string(),
        };
        writer.text(handles.enemy_shard, &shard_line);
        writer.text(handles.enemy_sigil, &enemy_sigil.to_string());
        writer.color(handles.enemy_portrait, portrait_fill(enemy_color));
        writer.text(
            handles.enemy_health,
            &health_line(
                theater.shown_enemy_health,
                battle.enemy.hero_max,
                battle.enemy.armor,
            ),
        );
        let plan = warden_plan(battle);
        let (intent, intent_color) = intent_line(&plan, battle.active == Who::Enemy);
        writer.text(handles.enemy_intent, &intent);
        writer.color(handles.enemy_intent, intent_color);
        let rule = battle.rule;
        writer.text(
            handles.gate_line,
            &if rule == GateRule::None {
                String::new()
            } else {
                format!("{}: {}", rule.name(), rule.text())
            },
        );
        writer.text(
            handles.enemy_line,
            &format!(
                "Hand {}   Deck {}   Mana {}/{}   Power: {}",
                battle.enemy.hand.len(),
                battle.enemy.deck.len(),
                battle.enemy.mana,
                battle.enemy.max_mana,
                battle.enemy.power.name()
            ),
        );
        for (index, back) in handles.enemy_hand_backs.iter().enumerate() {
            writer.visible(*back, index < battle.enemy.hand.len());
        }

        writer.text(
            handles.player_health,
            &health_line(
                theater.shown_player_health,
                battle.player.hero_max,
                battle.player.armor,
            ),
        );
        writer.text(
            handles.player_mana,
            &format!("{} of {} mana", battle.player.mana, battle.player.max_mana),
        );
        writer.text(
            handles.player_line,
            &match battle.player.turn_element {
                Some(element) if element != Element::Neutral => format!(
                    "Deck {}   Hand {}   Chain {}",
                    battle.player.deck.len(),
                    battle.player.hand.len(),
                    element.name()
                ),
                _ => format!(
                    "Deck {}   Hand {}",
                    battle.player.deck.len(),
                    battle.player.hand.len()
                ),
            },
        );

        let turn_text = match resources.selection {
            Selection::Attacker(_) => "Choose a target to attack".to_string(),
            Selection::Spell(_) => "Choose a target for the spell".to_string(),
            Selection::Power => "Choose a target for your power".to_string(),
            Selection::None if player_turn => format!("Turn {}: your turn", battle.turn),
            Selection::None => format!("Turn {}: the warden's turn", battle.turn),
        };
        writer.text(handles.turn_text, &turn_text);
        writer.disabled(handles.end_turn_button, !player_turn || busy);
        writer.text(
            handles.power_label,
            &format!(
                "{}   {} mana",
                battle.player.power.name(),
                battle.player.power_cost
            ),
        );
        writer.text(handles.power_text, battle.player.power.text());
        writer.disabled(
            handles.power_button,
            !player_turn || busy || !can_use_power(battle, Who::Player),
        );

        let preview_visible = screen == Screen::Battle && hovered_card.is_some();
        writer.visible(handles.preview_panel, preview_visible);
        if let Some(card) = hovered_card {
            let def = card.def();
            writer.text(handles.preview_name, def.name);
            let (meta, stats) = match def.kind {
                Kind::Minion { attack, health, .. } => (
                    format!("{} minion   {} mana", def.element.name(), def.cost),
                    format!("{attack} attack   {health} health"),
                ),
                Kind::Spell { .. } => (
                    format!("{} spell   {} mana", def.element.name(), def.cost),
                    String::new(),
                ),
            };
            writer.text(handles.preview_meta, &meta);
            writer.text(handles.preview_stats, &stats);
            let text = match def.text {
                "Guard" => "Guard: enemies must attack this first",
                "Rush" => "Rush: can attack the turn it is played",
                "Shield" => "Shield: blocks the first hit",
                "Venom" => "Venom: destroys any minion it damages",
                "Drain" => "Drain: heals you for the damage it deals",
                other => other,
            };
            writer.text(handles.preview_text, text);
            let note = battle
                .player
                .hand
                .iter()
                .find(|instance| instance.card == card)
                .and_then(|instance| play_block(battle, Who::Player, instance.id))
                .unwrap_or_default();
            writer.text(handles.preview_note, &note);
        }

        let written = theater.log.len();
        for (index, slot) in handles.log_lines.iter().enumerate() {
            let line = written
                .checked_sub(index + 1)
                .and_then(|position| theater.log.get(position));
            let text = line.map(|line| line.text.as_str()).unwrap_or("");
            let turn = line.is_some_and(|line| line.turn);
            writer.visible(*slot, index < written);
            writer.text(*slot, text);
            let color = if turn {
                TEXT_GOLD
            } else if index == 0 {
                TEXT_PRIMARY
            } else {
                TEXT_DIM
            };
            writer.color(*slot, color);
        }
    }

    let lesson_visible = screen == Screen::Battle && (teaching.active || story);
    writer.visible(handles.lesson_panel, lesson_visible);
    if lesson_visible && story {
        writer.text(handles.lesson_text, chapter(run.chapter).lesson);
        writer.visible(handles.lesson_next_button, false);
        writer.text(handles.lesson_hint, "");
    }
    if lesson_visible && teaching.active {
        let current = gate(teaching);
        let text = LESSONS.get(teaching.step).map_or("", |lesson| lesson.text);
        writer.text(handles.lesson_text, text);
        writer.visible(handles.lesson_next_button, current == Some(Gate::Next));
        let hint = match current {
            Some(Gate::Next) => "Enter, Space, or A also continues",
            Some(Gate::Play(_)) => "Click the lit card, or press its number",
            Some(Gate::Attack(..)) => "Click the lit minion, then the target",
            Some(Gate::Cast(..)) => "Click the lit card, then the target",
            Some(Gate::Power(_)) => "Q or the lit button, then the target",
            Some(Gate::EndTurn) => "E or the lit button",
            Some(Gate::EnemyTurn) | Some(Gate::Win) | None => "",
        };
        writer.text(handles.lesson_hint, hint);
    }

    let banner_visible = screen == Screen::Battle && theater.banner_timer > 0.0;
    writer.visible(handles.banner, banner_visible);
    if banner_visible {
        writer.text(handles.banner_text, &theater.banner);
    }

    writer.text(
        handles.shop_gold_text,
        &format!(
            "{} gold   Deck {} cards   Relics {}/{RELIC_SLOTS}   Click a card or its button to buy",
            run.gold,
            run.deck.len(),
            run.relics.len()
        ),
    );
    for (index, offer) in run.offers.iter().enumerate() {
        match offer {
            Some(offer) => {
                writer.text(handles.offer_names[index], &offer.name());
                writer.text(handles.offer_effects[index], &offer.effect());
                writer.text(
                    handles.offer_labels[index],
                    &format!("Buy for {} gold", offer.cost()),
                );
                writer.disabled(handles.offer_buttons[index], !can_buy(run, *offer));
            }
            None => {
                writer.text(handles.offer_names[index], "Sold");
                writer.text(handles.offer_effects[index], "");
                writer.text(handles.offer_labels[index], "Sold");
                writer.disabled(handles.offer_buttons[index], true);
            }
        }
    }
    writer.text(
        handles.reroll_label,
        &format!("Reroll {} gold", run.reroll_cost),
    );
    writer.disabled(handles.reroll_button, run.gold < run.reroll_cost);
    let strike_line = match strike_target(run) {
        Some(card) => format!("Strike {}, {} gold", card.def().name, run.strike_cost),
        None => "The deck is lean enough".to_string(),
    };
    writer.text(handles.strike_label, &strike_line);
    writer.disabled(handles.strike_button, !can_strike(run));

    let (title, summary) = if story {
        (
            format!(
                "{} held chapter {}",
                chapter(run.chapter).foe,
                run.chapter + 1
            ),
            chapter(run.chapter).lesson.to_string(),
        )
    } else {
        match run.outcome {
            Outcome::Won => (
                match run.ending {
                    Ending::Whole => "Whole again at the summit".to_string(),
                    Ending::OpenGates => "The gates stay open".to_string(),
                    Ending::Summit => format!("You passed all {GATE_COUNT} gates"),
                },
                format!(
                    "{} shards   {} gold   {} cards in the deck",
                    run.shards.len(),
                    run.gold,
                    run.deck.len()
                ),
            ),
            Outcome::Lost => (
                format!(
                    "{} held gate {}",
                    warden(run.final_gate, gate_pick(run, run.final_gate)).name,
                    run.final_gate
                ),
                defeat_line(run.final_gate).to_string(),
            ),
        }
    };
    writer.visible(handles.pause_scrim, resources.paused);
    writer.text(handles.over_title, &title);
    writer.text(handles.over_text, &summary);
    let report = &run.report;
    writer.text(
        handles.over_stats,
        &if story {
            String::new()
        } else {
            format!(
                "Gate {} of {GATE_COUNT}   {} turns at the last gate",
                report.gate, report.turns
            )
        },
    );
    writer.text(
        handles.over_deck,
        &if story {
            String::new()
        } else {
            format!(
                "{} cards   {} gold   {} trinkets   {} shards",
                run.deck.len(),
                run.gold,
                run.relics.len(),
                run.shards.len()
            )
        },
    );
    writer.text(
        handles.over_boon,
        &if story {
            String::new()
        } else {
            match boon_for(resources.best_gate) {
                Some(relic) => format!("The next climb starts with {}", relic.name()),
                None => "Reach gate three to carry a trinket into the next climb".to_string(),
            }
        },
    );
    writer.text(
        handles.over_new_run_label,
        if story {
            "Take the chapter again"
        } else {
            "Climb again"
        },
    );
}

struct Writer<'a, 'c, 'w, 'i> {
    written: &'a mut Memos<'w, 'i>,
    dirty: &'a mut RetainedUiDirty,
    commands: &'a mut Commands<'c>,
}

impl Writer<'_, '_, '_, '_> {
    fn text(&mut self, entity: Option<Entity>, text: &str) {
        let Some(entity) = entity else {
            return;
        };
        let mut present = false;
        let mut same = false;
        self.written.words.for_entity(entity, |(state,)| {
            present = true;
            same = state.0.as_deref() == Some(text);
            if !same {
                state.0 = Some(text.to_string());
            }
        });
        if same {
            return;
        }
        if !present {
            self.commands.set(entity, Words(Some(text.to_string())));
        }
        ui_set_text(self.commands, entity, text);
    }

    fn visible(&mut self, entity: Option<Entity>, visible: bool) {
        let Some(entity) = entity else {
            return;
        };
        let mut present = false;
        let mut same = false;
        self.written.shown.for_entity(entity, |(state,)| {
            present = true;
            same = state.0 == Some(visible);
            state.0 = Some(visible);
        });
        if same {
            return;
        }
        if !present {
            self.commands.set(entity, Seen(Some(visible)));
        }
        ui_set_visible(self.dirty, self.commands, entity, visible);
    }

    fn disabled(&mut self, entity: Option<Entity>, disabled: bool) {
        let Some(entity) = entity else {
            return;
        };
        let mut present = false;
        let mut same = false;
        self.written.locked.for_entity(entity, |(state,)| {
            present = true;
            same = state.0 == Some(disabled);
            state.0 = Some(disabled);
        });
        if same {
            return;
        }
        if !present {
            self.commands.set(entity, Locked(Some(disabled)));
        }
        ui_set_disabled(self.commands, entity, disabled);
    }

    fn color(&mut self, entity: Option<Entity>, color: Vec4) {
        let Some(entity) = entity else {
            return;
        };
        let key = [
            color.x.to_bits(),
            color.y.to_bits(),
            color.z.to_bits(),
            color.w.to_bits(),
        ];
        let mut present = false;
        let mut same = false;
        self.written.tones.for_entity(entity, |(state,)| {
            present = true;
            same = state.0 == Some(key);
            state.0 = Some(key);
        });
        if same {
            return;
        }
        if !present {
            self.commands.set(entity, Tone(Some(key)));
        }
        ui_set_color::<UiBase>(self.commands, entity, color);
    }

    fn tall(&mut self, entity: Entity, tall: bool) -> bool {
        let mut present = false;
        let mut same = false;
        self.written.talls.for_entity(entity, |(state,)| {
            present = true;
            same = state.0 == Some(tall);
            state.0 = Some(tall);
        });
        if !present {
            self.commands.set(entity, Tall(Some(tall)));
        }
        !same
    }
}

fn health_line(shown: i32, max: i32, armor: i32) -> String {
    if armor > 0 {
        format!("{} / {max}   armor {armor}", shown.max(0))
    } else {
        format!("{} / {max}", shown.max(0))
    }
}

fn intent_line(plan: &Plan, acting: bool) -> (String, Vec4) {
    if acting {
        return ("The warden is taking its turn".to_string(), INTENT_CALM);
    }
    if plan.lethal {
        return (
            format!("Next turn: lethal, {} damage", plan.total()),
            INTENT_LETHAL,
        );
    }
    if plan.total() == 0 {
        return ("Next turn: nothing on its line".to_string(), INTENT_CALM);
    }
    if plan.guarded {
        return (
            format!("Next turn: {} damage into your guard", plan.total()),
            INTENT_CALM,
        );
    }
    (
        format!("Next turn: up to {} damage", plan.total()),
        INTENT_THREAT,
    )
}
