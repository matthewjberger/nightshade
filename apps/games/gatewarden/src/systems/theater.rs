use crate::components::{Instance, ShownAttack, ShownHealth, Spot};
use crate::contract::Screen;
use crate::data::battle::{Battle, Event, Target, Who};
use crate::data::effects::{
    block_style, death_style, defeat_style, heal_style, hit_style, shield_style, victory_style,
};
use crate::queries::view::screen_to_world;
use crate::resources::{GatewardenResources, LogLine, Lunge, Theater};
use crate::systems::effects::spawn_burst;
use crate::theme::*;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_ui::components::UiLayoutNode;
use std::collections::HashMap;

type Cards<'world, 'item> = Query<
    'world,
    (
        &'item Instance,
        &'item Spot,
        &'item mut ShownAttack,
        &'item mut ShownHealth,
    ),
>;

pub fn update(
    time: Res<Time>,
    window: Res<Window>,
    mut resources: ResMut<GatewardenResources>,
    mut commands: Commands,
    mut nodes: Query<&UiLayoutNode>,
    mut transforms: Query<&LocalTransform>,
    mut cards: Cards<'_, '_>,
) {
    let resources = &mut *resources;
    if resources.paused {
        return;
    }
    let theater = &mut resources.theater;
    theater.banner_timer = (theater.banner_timer - time.delta_time).max(0.0);
    if let Some(lunge) = theater.lunge.as_mut() {
        lunge.progress += time.delta_time / LUNGE_SECONDS;
        if lunge.progress >= 1.0 {
            theater.lunge = None;
        }
    }
    if resources.screen != Screen::Battle {
        return;
    }
    let Some(battle) = resources.battle.as_mut() else {
        return;
    };
    theater.delay -= time.delta_time;
    if theater.delay > 0.0 {
        return;
    }
    let Some(event) = battle.events.pop_front() else {
        theater.delay = 0.0;
        return;
    };

    let mut camera = LocalTransform::default();
    if let Some(entity) = resources.camera {
        transforms.for_entity(entity, |transform| camera = *transform);
    }
    let hero_nodes = [resources.ui.player_hero, resources.ui.enemy_hero];
    let table = &mut resources.table;

    match event {
        Event::Turn { who, turn } => {
            theater.banner = match who {
                Who::Player => "Your turn".to_string(),
                Who::Enemy => format!("{} takes its turn", battle.enemy.name),
            };
            let line = match who {
                Who::Player => format!("Turn {turn}: your turn"),
                Who::Enemy => format!("Turn {turn}: {} acts", battle.enemy.name),
            };
            record_turn(theater, line);
            theater.banner_timer = BANNER_SECONDS;
            theater.delay = 0.45;
        }
        Event::Play { who, card } => {
            let name = card.def().name;
            if who == Who::Enemy {
                theater.banner = format!("{} plays {name}", battle.enemy.name);
                record(theater, format!("{} plays {name}", battle.enemy.name));
                theater.banner_timer = BANNER_SECONDS;
                theater.delay = 0.4;
            } else {
                record(theater, format!("You play {name}"));
                theater.delay = 0.12;
            }
        }
        Event::Attack { attacker, target } => {
            record(
                theater,
                format!(
                    "{} attacks {}",
                    subject_of(battle, attacker),
                    name_of(battle, target)
                ),
            );
            if let Target::Minion(_, id) = attacker
                && let Some((_, from)) = card_of(&mut cards, id)
            {
                let to = anchor_of(&mut cards, target);
                theater.lunge = Some(Lunge {
                    id,
                    from,
                    to,
                    progress: 0.0,
                });
            }
            theater.delay = LUNGE_SECONDS * 0.5;
        }
        Event::Hit { target, amount } => {
            let hero = matches!(target, Target::Hero(_));
            let (label, color) = if amount > 0 {
                (format!("-{amount}"), FLOAT_DAMAGE)
            } else if hero {
                ("Armor".to_string(), FLOAT_SHIELD)
            } else {
                ("Shield".to_string(), FLOAT_SHIELD)
            };
            record(
                theater,
                if amount > 0 {
                    format!(
                        "{} {} {amount}",
                        subject_of(battle, target),
                        verb_of(target, "take")
                    )
                } else if hero {
                    format!("{} armor holds", subject_of(battle, target))
                } else {
                    format!(
                        "{} {} {} shield",
                        subject_of(battle, target),
                        verb_of(target, "lose"),
                        owned_by(target)
                    )
                },
            );
            let anchor = floater_anchor(&mut cards, &table.last_positions, target);
            table
                .pending_floaters
                .push((floater_card(target), anchor, label, color));
            if let Some(center) = screen_of(&mut cards, &mut nodes, hero_nodes, target) {
                let origin = screen_to_world(&camera, &window, center);
                let style = if amount > 0 {
                    hit_style(amount)
                } else {
                    block_style()
                };
                spawn_burst(&mut commands, origin, style);
            }
            refresh(theater, &mut cards, battle, target);
            theater.delay = 0.28;
        }
        Event::Heal { target, amount } => {
            let (label, color) = if amount > 0 {
                (format!("+{amount}"), FLOAT_HEAL)
            } else {
                ("Shield".to_string(), FLOAT_SHIELD)
            };
            record(
                theater,
                if amount > 0 {
                    format!(
                        "{} {} {amount}",
                        subject_of(battle, target),
                        verb_of(target, "heal")
                    )
                } else {
                    format!(
                        "{} {} a shield",
                        subject_of(battle, target),
                        verb_of(target, "gain")
                    )
                },
            );
            let anchor = floater_anchor(&mut cards, &table.last_positions, target);
            table
                .pending_floaters
                .push((floater_card(target), anchor, label, color));
            if let Some(center) = screen_of(&mut cards, &mut nodes, hero_nodes, target) {
                let origin = screen_to_world(&camera, &window, center);
                let style = if amount > 0 {
                    heal_style()
                } else {
                    shield_style()
                };
                spawn_burst(&mut commands, origin, style);
            }
            refresh(theater, &mut cards, battle, target);
            theater.delay = 0.22;
        }
        Event::Died { target, card } => match target {
            Target::Minion(..) => {
                let name = card.map(|card| card.def().name).unwrap_or("A minion");
                record(theater, format!("{name} falls"));
                let anchor = floater_anchor(&mut cards, &table.last_positions, target);
                table.pending_floaters.push((
                    floater_card(target),
                    anchor,
                    "falls".to_string(),
                    FLOAT_FALLS,
                ));
                if let Some(center) = screen_of(&mut cards, &mut nodes, hero_nodes, target) {
                    let origin = screen_to_world(&camera, &window, center);
                    spawn_burst(&mut commands, origin, death_style());
                }
                theater.delay = 0.22;
            }
            Target::Hero(who) => {
                theater.banner = match who {
                    Who::Enemy => format!("{} falls!", battle.enemy.name),
                    Who::Player => "You fall".to_string(),
                };
                record(
                    theater,
                    match who {
                        Who::Enemy => format!("{} falls", battle.enemy.name),
                        Who::Player => "You fall".to_string(),
                    },
                );
                theater.banner_timer = OUTCOME_HOLD_SECONDS;
                if let Some(center) = screen_of(&mut cards, &mut nodes, hero_nodes, target) {
                    let origin = screen_to_world(&camera, &window, center);
                    let style = if who == Who::Enemy {
                        victory_style()
                    } else {
                        defeat_style()
                    };
                    spawn_burst(&mut commands, origin, style);
                }
                theater.delay = 0.6;
            }
        },
        Event::Summon { who, id } => {
            theater.revealed.insert(id);
            if battle.side(who).minion(id).is_some() {
                record(
                    theater,
                    format!(
                        "{} enters the line",
                        subject_of(battle, Target::Minion(who, id))
                    ),
                );
            }
            theater.delay = 0.3;
        }
        Event::Armor { who, amount } => {
            let target = Target::Hero(who);
            record(
                theater,
                format!(
                    "{} {} {amount} armor",
                    subject_of(battle, target),
                    verb_of(target, "gain")
                ),
            );
            let anchor = floater_anchor(&mut cards, &table.last_positions, target);
            table.pending_floaters.push((
                floater_card(target),
                anchor,
                format!("+{amount}"),
                FLOAT_SHIELD,
            ));
            theater.delay = 0.18;
        }
        Event::Chain { who, element } => {
            if who == Who::Player {
                record(theater, format!("{} chain", element.name()));
            }
            theater.delay = 0.1;
        }
        Event::Draw { .. } => {
            theater.delay = 0.08;
        }
        Event::Fatigue { who, amount } => {
            theater.banner = match who {
                Who::Player => format!("Fatigue! Your empty deck deals {amount}"),
                Who::Enemy => format!("Fatigue! The warden takes {amount}"),
            };
            record(
                theater,
                match who {
                    Who::Player => format!("Your empty deck deals {amount} to you"),
                    Who::Enemy => format!("Fatigue deals {amount} to {}", battle.enemy.name),
                },
            );
            theater.banner_timer = BANNER_SECONDS;
            theater.delay = 0.4;
        }
        Event::Note(text) => {
            record(theater, text.clone());
            theater.banner = text;
            theater.banner_timer = BANNER_SECONDS;
            theater.delay = 0.45;
        }
    }
}

fn record(theater: &mut Theater, text: String) {
    push_line(theater, LogLine { text, turn: false });
}

fn record_turn(theater: &mut Theater, text: String) {
    push_line(theater, LogLine { text, turn: true });
}

fn push_line(theater: &mut Theater, line: LogLine) {
    theater.log.push_back(line);
    while theater.log.len() > LOG_LINES {
        theater.log.pop_front();
    }
}

fn name_of(battle: &Battle, target: Target) -> String {
    match target {
        Target::Hero(Who::Player) => "you".to_string(),
        Target::Hero(Who::Enemy) => battle.enemy.name.clone(),
        Target::Minion(who, id) => {
            let owner = match who {
                Who::Player => "your",
                Who::Enemy => "the enemy",
            };
            match battle.side(who).minion(id) {
                Some(minion) => format!("{owner} {}", minion.card.def().name),
                None => format!("{owner} minion"),
            }
        }
    }
}

fn subject_of(battle: &Battle, target: Target) -> String {
    let name = name_of(battle, target);
    let mut characters = name.chars();
    match characters.next() {
        Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
        None => name,
    }
}

fn verb_of(target: Target, stem: &str) -> String {
    if target == Target::Hero(Who::Player) {
        stem.to_string()
    } else {
        format!("{stem}s")
    }
}

fn owned_by(target: Target) -> &'static str {
    if target == Target::Hero(Who::Player) {
        "your"
    } else {
        "its"
    }
}

fn card_of(cards: &mut Cards<'_, '_>, id: u32) -> Option<(Entity, Vec2)> {
    let mut found = None;
    cards.for_each_mut(|entity, (card_id, spot, _, _)| {
        if card_id.0 == id {
            found = Some((entity, spot.0));
        }
    });
    found
}

fn anchor_of(cards: &mut Cards<'_, '_>, target: Target) -> Vec2 {
    match target {
        Target::Minion(_, id) => card_of(cards, id)
            .map(|(_, position)| position)
            .unwrap_or_else(|| vec2(HAND_WIDTH * 0.5, PLAYER_ROW_Y)),
        Target::Hero(Who::Enemy) => {
            vec2(HAND_WIDTH * 0.5 - MINION_WIDTH * 0.5, ENEMY_SPAWN_Y + 40.0)
        }
        Target::Hero(Who::Player) => vec2(-HERO_PANEL_WIDTH * 0.5, PLAYER_ROW_Y + 40.0),
    }
}

fn screen_of(
    cards: &mut Cards<'_, '_>,
    nodes: &mut Query<'_, &UiLayoutNode>,
    hero_nodes: [Option<Entity>; 2],
    target: Target,
) -> Option<Vec2> {
    let entity = match target {
        Target::Minion(_, id) => card_of(cards, id).map(|(entity, _)| entity),
        Target::Hero(Who::Player) => hero_nodes[0],
        Target::Hero(Who::Enemy) => hero_nodes[1],
    }?;
    let mut center = None;
    nodes.for_entity(entity, |node| center = Some(node.computed_rect.center()));
    center
}

fn floater_card(target: Target) -> Option<u32> {
    match target {
        Target::Minion(_, id) => Some(id),
        Target::Hero(_) => None,
    }
}

fn floater_anchor(cards: &mut Cards<'_, '_>, last: &HashMap<u32, Vec2>, target: Target) -> Vec2 {
    match target {
        Target::Minion(_, id) => card_of(cards, id)
            .map(|(_, position)| position)
            .or_else(|| last.get(&id).copied())
            .map(|position| position + FLOATER_CARD_OFFSET)
            .unwrap_or_else(|| vec2(HAND_WIDTH * 0.5 - 40.0, PLAYER_ROW_Y)),
        Target::Hero(Who::Enemy) => vec2(HAND_WIDTH * 0.5 - 40.0, -70.0),
        Target::Hero(Who::Player) => vec2(
            -SIDE_OFFSET - HERO_PANEL_WIDTH * 0.5 - 40.0,
            ARENA_HEIGHT - HERO_PANEL_HEIGHT * 0.5 - 40.0,
        ),
    }
}

fn refresh(theater: &mut Theater, cards: &mut Cards<'_, '_>, battle: &Battle, target: Target) {
    match target {
        Target::Hero(Who::Player) => theater.shown_player_health = battle.player.hero_health,
        Target::Hero(Who::Enemy) => theater.shown_enemy_health = battle.enemy.hero_health,
        Target::Minion(who, id) => {
            let (attack, health) = match battle.side(who).minion(id) {
                Some(minion) => (Some(minion.attack), minion.health),
                None => (None, 0),
            };
            cards.for_each_mut(|_, (card_id, _, shown_attack, shown_health)| {
                if card_id.0 != id {
                    return;
                }
                shown_health.0 = health;
                if let Some(attack) = attack {
                    shown_attack.0 = attack;
                }
            });
        }
    }
}
