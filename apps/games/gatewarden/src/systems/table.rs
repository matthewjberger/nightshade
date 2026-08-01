use crate::components::{
    AttackText, Border, Delay, DimOverlay, Dimmed, Generation, HealthText, HeroFrame, Instance,
    Previewed, Settled, ShieldBadge, ShownAttack, ShownHealth, ShownShield, Spot, Tint, Width,
    WrittenAttack, WrittenHealth,
};
use crate::contract::Screen;
use crate::data::battle::{Battle, Event, Pick, Target, Who};
use crate::data::cards::{CardId, Element, Kind};
use crate::data::relics::Relic;
use crate::data::tutorial::Focus;
use crate::queries::battle::{
    attack_targets, can_play, pending_damage, pick_at_cursor, power_rule, targets_for,
};
use crate::queries::table::{board_slot, card_depth, hand_lean, hand_slot};
use crate::queries::tutorial::focus;
use crate::resources::Frames;
use crate::resources::{GatewardenResources, Selection};
use crate::systems::setup::frame;
use crate::theme::*;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_text::prelude::*;
use nightshade_ui::prelude::*;
use std::collections::{HashMap, HashSet};

pub type Numbers<'world, 'item> = Query<
    'world,
    (
        &'item Instance,
        &'item AttackText,
        &'item HealthText,
        &'item ShownAttack,
        &'item mut WrittenAttack,
        &'item ShownHealth,
        &'item mut WrittenHealth,
        &'item mut Previewed,
    ),
>;

pub type Looks<'world, 'item> = Query<
    'world,
    (
        &'item Instance,
        &'item mut Border,
        &'item mut ShownShield,
        &'item ShieldBadge,
        &'item mut Dimmed,
        &'item DimOverlay,
    ),
>;

pub type Spots<'world, 'item> = Query<
    'world,
    (
        &'item Instance,
        &'item Generation,
        &'item mut Spot,
        &'item mut Settled,
        &'item mut Delay,
    ),
>;

nightshade::ecs::query_bundle! {
    pub struct Cards<'world, 'item> {
        pub numbers: Numbers<'world, 'item>,
        pub looks: Looks<'world, 'item>,
        pub spots: Spots<'world, 'item>,
    }
}

const RELIC_ICON: IconGlyph = IconGlyph::lucide(SHARD_SIGIL);
const SHIELD_ICON: IconGlyph = IconGlyph::lucide(SHIELD_SIGIL);

#[derive(Clone, Copy)]
struct Desired {
    lean: f32,
    rank: f32,
    forged: bool,
    id: u32,
    origin: u32,
    card: CardId,
    minion: bool,
    enemy: bool,
    slot: Vec2,
    attack: i32,
    health: i32,
    guard: bool,
    shield: bool,
    ready: bool,
    dimmed: bool,
}

pub fn sync(
    mut resources: ResMut<GatewardenResources>,
    time: Res<Time>,
    mut build: nightshade_ui::commands::builder::UiBuild,
    mut nodes: Query<(&mut UiLayoutNode,)>,
    mut interactions: Query<(&UiNodeInteraction,)>,
    mut contents: Query<(&mut UiNodeContent,)>,
    mut cards: Cards<'_, '_>,
    mut frames: Query<(&mut HeroFrame,)>,
    mut guided: Query<(&Tint, &Width)>,
) {
    let resources = &mut *resources;

    if !resources.table.retired.is_empty() {
        let mut context = build.context();
        for entity in resources.table.retired.drain(..) {
            ui_despawn_node(&mut context, entity);
        }
    }

    let Some(arena) = resources.ui.arena else {
        return;
    };
    resources.table.hovered_card = None;

    if resources.screen != Screen::Battle || resources.battle.is_none() {
        let (released, _) =
            release_guides(&mut guided, &mut contents, build.context().commands, None);
        if released {
            ui_mark_render_dirty(build.dirty());
        }
        return;
    }

    let Some(battle) = resources.battle.as_ref() else {
        return;
    };
    let card_frames = resources.ui.frames;
    let targets = current_targets(battle, resources.selection);
    let selected_id = match resources.selection {
        Selection::Attacker(id) | Selection::Spell(id) => Some(id),
        _ => None,
    };
    let mut desired = desired_nodes(battle, &resources.theater.revealed, &targets, selected_id);
    let dying_player = row_death_pending(battle, Who::Player);
    let dying_enemy = row_death_pending(battle, Who::Enemy);
    if dying_player || dying_enemy {
        for entry in desired.iter_mut().filter(|entry| entry.minion) {
            let dying = if entry.enemy {
                dying_enemy
            } else {
                dying_player
            };
            if dying && let Some(slot) = resources.table.held_slots.get(&entry.id) {
                entry.slot = *slot;
            }
        }
    } else {
        let held = &mut resources.table.held_slots;
        held.clear();
        for entry in desired.iter().filter(|entry| entry.minion) {
            held.insert(entry.id, entry.slot);
        }
    }
    let wanted: HashMap<u32, Desired> = desired.iter().map(|entry| (entry.id, *entry)).collect();
    let generation = resources.table.generation;

    let mut stale = Vec::new();
    let mut present = HashSet::new();
    cards
        .spots
        .for_each_mut(|entity, (id, card_generation, spot, _, _)| {
            let gone = !wanted.contains_key(&id.0) && !death_pending(battle, id.0);
            if card_generation.0 != generation || gone {
                stale.push((entity, id.0, spot.0));
            } else {
                present.insert(id.0);
            }
        });
    let table = &mut resources.table;
    if !stale.is_empty() {
        let commands = build.context().commands;
        for (entity, id, position) in stale {
            table.last_positions.insert(id, position);
            table.retired.push(entity);
            commands.remove::<(
                Instance,
                Generation,
                AttackText,
                HealthText,
                Spot,
                Settled,
                Delay,
                Border,
            )>(entity);
            commands.remove::<(
                ShownAttack,
                ShownHealth,
                WrittenAttack,
                WrittenHealth,
                Dimmed,
                Previewed,
                ShieldBadge,
                DimOverlay,
                ShownShield,
            )>(entity);
        }
    }

    let missing: Vec<Desired> = desired
        .iter()
        .copied()
        .filter(|entry| !present.contains(&entry.id))
        .collect();
    if !missing.is_empty() {
        let mut tree = UiTreeBuilder::from_parent(build.context(), arena);
        let mut dealt = 0;
        for entry in missing {
            let start = match table.last_positions.remove(&entry.origin) {
                Some(position) => position,
                None if entry.minion && entry.enemy => {
                    vec2(HAND_WIDTH * 0.5 - MINION_WIDTH * 0.5, ENEMY_SPAWN_Y)
                }
                None if entry.minion => vec2(HAND_WIDTH * 0.5 - MINION_WIDTH * 0.5, HAND_Y),
                None => vec2(HAND_WIDTH + DEAL_OFFSCREEN, HAND_Y),
            };
            let delay = if entry.minion {
                0.0
            } else {
                dealt as f32 * 0.07
            };
            if !entry.minion {
                dealt += 1;
            }
            let stats = match entry.card.def().kind {
                Kind::Minion { .. } => Some((entry.attack, entry.health)),
                Kind::Spell { .. } => None,
            };
            let depth = card_depth(entry.rank);
            let node = if entry.minion {
                spawn_minion_node(&mut tree, card_frames, &entry, start, depth)
            } else {
                spawn_hand_node(
                    &mut tree,
                    card_frames,
                    entry.card,
                    start,
                    stats,
                    depth,
                    entry.forged,
                )
            };
            let commands = tree.commands();
            commands.set(node.root, Instance(entry.id));
            commands.set(node.root, Generation(generation));
            commands.set(node.root, AttackText(node.attack_text));
            commands.set(node.root, HealthText(node.health_text));
            commands.set(node.root, Spot(start));
            commands.set(node.root, Settled(false));
            commands.set(node.root, Delay(delay));
            commands.set(node.root, Border::Normal);
            commands.set(node.root, ShownAttack(entry.attack));
            commands.set(node.root, ShownHealth(entry.health));
            commands.set(node.root, WrittenAttack(entry.attack));
            commands.set(node.root, WrittenHealth(entry.health));
            commands.set(node.root, Dimmed(false));
            commands.set(node.root, Previewed(false));
            commands.set(node.root, ShieldBadge(node.shield_badge));
            commands.set(node.root, DimOverlay(node.dim_overlay));
            commands.set(node.root, ShownShield(entry.shield));
        }
        tree.finish();
    }

    let player_turn = battle.active == Who::Player && battle.outcome.is_none();
    let cursor_pick = if resources.cursor.active {
        pick_at_cursor(battle, resources.cursor)
    } else {
        None
    };
    let cursor_id = match cursor_pick {
        Some(Pick::Hand(id) | Pick::Mine(id) | Pick::Foe(id)) => Some(id),
        _ => None,
    };

    let blend = 1.0 - (-DEAL_SPEED * time.delta_time).exp();
    let delta_time = time.delta_time;
    let mut moved = false;
    let mut render_dirty = false;
    let lunge = resources.theater.lunge;
    let mut hovered_card = None;
    let pending = pending_damage(battle, resources.selection);
    let guide = focus(resources.tutorial);
    let ui = resources.ui;
    let wanted_guide = match guide {
        Focus::EndTurn => ui
            .end_turn_button
            .map(|entity| (entity, BUTTON_BORDER, 1.5)),
        Focus::Power => ui.power_button.map(|entity| (entity, BUTTON_BORDER, 1.5)),
        Focus::HandSurface => ui.hand_surface.map(|entity| (entity, TABLE_BORDER, 1.5)),
        _ => None,
    };
    let commands = build.context().commands;
    let (released, held) = release_guides(
        &mut guided,
        &mut contents,
        commands,
        wanted_guide.map(|wanted| wanted.0),
    );
    render_dirty |= released;
    if let Some((entity, color, width)) = wanted_guide
        && !held
    {
        set_border(&mut contents, entity, BORDER_GUIDE, 3.0);
        commands.set(entity, Tint(color));
        commands.set(entity, Width(width));
        render_dirty = true;
    }

    let mut hovered_nodes = HashSet::new();
    interactions.for_each_mut(|entity, (interaction,)| {
        if interaction.hovered {
            hovered_nodes.insert(entity);
        }
    });
    let mut border_updates = Vec::new();
    let mut shield_updates = Vec::new();
    let mut dim_updates = Vec::new();
    let mut preview_updates: Vec<(Entity, Vec4)> = Vec::new();
    let mut placements = Vec::new();
    cards.numbers.for_each_mut(
        |entity,
         (
            id,
            attack_text,
            health_text,
            shown_attack,
            written_attack,
            shown_health,
            written_health,
            previewed,
        )| {
            if shown_attack.0 != written_attack.0
                && let Some(text) = attack_text.0
            {
                ui_set_text(commands, text, &shown_attack.0.to_string());
                written_attack.0 = shown_attack.0;
            }
            if shown_health.0 != written_health.0
                && !previewed.0
                && let Some(text) = health_text.0
            {
                ui_set_text(commands, text, &shown_health.0.max(0).to_string());
                written_health.0 = shown_health.0;
            }
            let Some(entry) = wanted.get(&id.0) else {
                return;
            };
            let hovered = hovered_nodes.contains(&entity);
            let target = if entry.enemy {
                Target::Minion(Who::Enemy, id.0)
            } else {
                Target::Minion(Who::Player, id.0)
            };
            let aimed =
                pending > 0 && targets.contains(&target) && (hovered || cursor_id == Some(id.0));
            if aimed != previewed.0
                && let Some(text) = health_text.0
            {
                if aimed {
                    let left = (shown_health.0 - pending).max(0);
                    ui_set_text(commands, text, &left.to_string());
                    preview_updates.push((
                        text,
                        if left == 0 {
                            PREVIEW_KILL
                        } else {
                            PREVIEW_HURT
                        },
                    ));
                } else {
                    ui_set_text(commands, text, &shown_health.0.max(0).to_string());
                    preview_updates.push((text, TEXT_PRIMARY));
                    written_health.0 = shown_health.0;
                }
                previewed.0 = aimed;
                render_dirty = true;
            }
        },
    );

    cards.looks.for_each_mut(
        |entity, (id, edge, shown_shield, shield_badge, dimmed, dim_overlay)| {
            let Some(entry) = wanted.get(&id.0) else {
                return;
            };
            let hovered = hovered_nodes.contains(&entity);
            if hovered || cursor_id == Some(id.0) {
                hovered_card = Some(entry.card);
            }
            let target = if entry.enemy {
                Target::Minion(Who::Enemy, id.0)
            } else {
                Target::Minion(Who::Player, id.0)
            };
            let guided_card = match guide {
                Focus::Hand(wanted) => !entry.minion && entry.card == wanted,
                Focus::Mine(wanted) => entry.minion && !entry.enemy && entry.card == wanted,
                Focus::Foe(wanted) => entry.minion && entry.enemy && entry.card == wanted,
                _ => false,
            };
            let border = if selected_id == Some(id.0) {
                Border::Selected
            } else if targets.contains(&target) {
                Border::Target
            } else if guided_card {
                Border::Guide
            } else if cursor_id == Some(id.0) {
                Border::Cursor
            } else if entry.shield {
                Border::Shield
            } else if entry.guard {
                Border::Guard
            } else if entry.ready && !entry.enemy && player_turn {
                Border::Ready
            } else {
                Border::Normal
            };
            if border != *edge {
                let (color, width) = border_style(border, entry.card.def().element);
                border_updates.push((entity, color, width));
                *edge = border;
                render_dirty = true;
            }

            if entry.shield != shown_shield.0 {
                shown_shield.0 = entry.shield;
                if let Some(badge) = shield_badge.0 {
                    shield_updates.push((badge, entry.shield));
                }
            }

            if entry.dimmed != dimmed.0 {
                dimmed.0 = entry.dimmed;
                if let Some(overlay) = dim_overlay.0 {
                    dim_updates.push((overlay, entry.dimmed));
                }
            }
        },
    );

    cards
        .spots
        .for_each_mut(|entity, (id, _, spot, settled, delay)| {
            let Some(entry) = wanted.get(&id.0) else {
                return;
            };
            let hovered = hovered_nodes.contains(&entity);
            if delay.0 > 0.0 {
                delay.0 -= delta_time;
                return;
            }

            let mut goal = entry.slot;
            if !entry.minion {
                if selected_id == Some(id.0) {
                    goal.y -= SELECT_RAISE;
                } else if hovered || cursor_id == Some(id.0) {
                    goal.y -= HOVER_RAISE;
                }
            } else if hovered || cursor_id == Some(id.0) {
                goal.y -= 6.0;
            }

            let next = match lunge {
                Some(lunge) if lunge.id == id.0 => {
                    let swing = (lunge.progress * std::f32::consts::PI).sin() * LUNGE_REACH;
                    lunge.from + (lunge.to - lunge.from) * swing
                }
                _ => {
                    let delta = goal - spot.0;
                    if delta.magnitude() < 0.25 {
                        if settled.0 {
                            return;
                        }
                        settled.0 = true;
                        goal
                    } else {
                        settled.0 = false;
                        spot.0 + delta * blend
                    }
                }
            };
            spot.0 = next;
            let size = if entry.minion {
                vec2(MINION_WIDTH, MINION_HEIGHT)
            } else {
                vec2(CARD_WIDTH, CARD_HEIGHT)
            };
            let lifted = selected_id == Some(id.0) || hovered || cursor_id == Some(id.0);
            let lean = if lifted { 0.0 } else { entry.lean };
            let rank = match lunge {
                Some(lunge) if lunge.id == id.0 => CARD_RANK_LUNGE,
                _ if lifted => CARD_RANK_LIFTED,
                _ => entry.rank,
            };
            placements.push((entity, next, size, lean, card_depth(rank)));
            moved = true;
        });

    table.hovered_card = hovered_card;
    for (entity, color, width) in border_updates {
        set_border(&mut contents, entity, color, width);
    }
    for (entity, position, size, lean, depth) in placements {
        place(&mut nodes, entity, position, size, lean, depth);
    }

    let hero_targets = [
        targets.contains(&Target::Hero(Who::Player)),
        targets.contains(&Target::Hero(Who::Enemy))
            || matches!(cursor_pick, Some(Pick::Hero(Who::Enemy))),
    ];
    let hero_nodes = [ui.player_hero, ui.enemy_hero];
    for (index, entity) in hero_nodes.iter().enumerate() {
        let Some(entity) = *entity else {
            continue;
        };
        let hero_guided = match guide {
            Focus::You => index == 0,
            Focus::Warden => index == 1,
            _ => false,
        };
        let border = if hero_targets[index] {
            Border::Target
        } else if hero_guided {
            Border::Guide
        } else {
            Border::Normal
        };
        let mut changed = false;
        frames.for_entity(entity, |(frame,)| {
            changed = frame.border != border;
            frame.border = border;
        });
        if !changed {
            continue;
        }
        let (color, width) = match border {
            Border::Target => (BORDER_TARGET, CARD_BORDER_WIDTH_STRONG),
            Border::Guide => (BORDER_GUIDE, CARD_BORDER_WIDTH_STRONG),
            _ => (PANEL_BORDER, CARD_BORDER_WIDTH),
        };
        set_border(&mut contents, entity, color, width);
        render_dirty = true;
    }

    if moved {
        ui_mark_layout_dirty(build.dirty());
    }
    if !preview_updates.is_empty() {
        let commands = build.context().commands;
        for (text, color) in preview_updates {
            ui_set_color::<UiBase>(commands, text, color);
        }
    }
    if !shield_updates.is_empty() || !dim_updates.is_empty() {
        let context = build.context();
        for (badge, visible) in shield_updates {
            ui_set_visible(context.dirty, context.commands, badge, visible);
        }
        for (overlay, visible) in dim_updates {
            ui_set_visible(context.dirty, context.commands, overlay, visible);
        }
    }
    if render_dirty {
        ui_mark_render_dirty(build.dirty());
    }
}

fn release_guides(
    guided: &mut Query<'_, (&Tint, &Width)>,
    contents: &mut Query<'_, (&mut UiNodeContent,)>,
    commands: &mut Commands,
    keep: Option<Entity>,
) -> (bool, bool) {
    let mut held = false;
    let mut restore = Vec::new();
    guided.for_each_mut(|entity, (tint, width)| {
        if Some(entity) == keep {
            held = true;
        } else {
            restore.push((entity, tint.0, width.0));
        }
    });
    let released = !restore.is_empty();
    for (entity, color, width) in restore {
        set_border(contents, entity, color, width);
        commands.remove::<(Tint, Width)>(entity);
    }
    (released, held)
}

fn set_border(
    contents: &mut Query<'_, (&mut UiNodeContent,)>,
    entity: Entity,
    color: Vec4,
    width: f32,
) {
    contents.for_entity(entity, |(content,)| {
        if let UiNodeContent::Rect {
            border_width,
            border_color,
            ..
        } = content
        {
            *border_width = width;
            *border_color = color;
        }
    });
}

fn place(
    nodes: &mut Query<'_, (&mut UiLayoutNode,)>,
    entity: Entity,
    position: Vec2,
    size: Vec2,
    lean: f32,
    depth: f32,
) {
    nodes.for_entity(entity, |(layout,)| {
        layout.base_layout = Some(UiLayoutType::Window(WindowLayout {
            position: Ab(position).into(),
            size: Ab(size).into(),
            anchor: Anchor::TopLeft,
        }));
        layout.rotation = lean;
        layout.depth = UiDepthMode::Set(depth);
    });
}

pub fn card_stats(card: CardId) -> Option<(i32, i32)> {
    match card.def().kind {
        Kind::Minion { attack, health, .. } => Some((attack, health)),
        Kind::Spell { .. } => None,
    }
}

fn row_death_pending(battle: &Battle, who: Who) -> bool {
    battle.events.iter().any(|event| {
        matches!(
            event,
            Event::Died {
                target: Target::Minion(side, _),
                ..
            } if *side == who
        )
    })
}

fn death_pending(battle: &Battle, id: u32) -> bool {
    battle.events.iter().any(
        |event| matches!(event, Event::Died { target: Target::Minion(_, dead), .. } if *dead == id),
    )
}

fn current_targets(battle: &Battle, selection: Selection) -> Vec<Target> {
    match selection {
        Selection::None => Vec::new(),
        Selection::Attacker(_) => attack_targets(battle, Who::Player),
        Selection::Spell(id) => battle
            .player
            .hand_card(id)
            .map(|instance| targets_for(battle, Who::Player, instance.card.target_rule()))
            .unwrap_or_default(),
        Selection::Power => targets_for(battle, Who::Player, power_rule(battle.player.power)),
    }
}

fn desired_nodes(
    battle: &Battle,
    revealed: &HashSet<u32>,
    targets: &[Target],
    selected: Option<u32>,
) -> Vec<Desired> {
    let mut desired = Vec::new();
    let player_turn = battle.active == Who::Player;
    let targeting = !targets.is_empty();
    let hand = &battle.player.hand;
    for (index, instance) in hand.iter().enumerate() {
        let def = instance.card.def();
        let (attack, health, guard, shield) = match def.kind {
            Kind::Minion {
                attack,
                health,
                guard,
                shield,
                ..
            } => (
                attack,
                health + battle.player.minion_health_bonus,
                guard,
                shield,
            ),
            Kind::Spell { .. } => (0, 0, false, false),
        };
        desired.push(Desired {
            id: instance.id,
            origin: instance.id,
            card: instance.card,
            minion: false,
            enemy: false,
            lean: hand_lean(index, hand.len()),
            rank: CARD_RANK_HAND + index as f32,
            forged: instance.upgraded,
            slot: hand_slot(index, hand.len()),
            attack,
            health,
            guard,
            shield,
            ready: false,
            dimmed: if targeting {
                selected != Some(instance.id)
            } else {
                player_turn && !can_play(battle, Who::Player, instance.id)
            },
        });
    }
    for (who, row_y) in [(Who::Player, PLAYER_ROW_Y), (Who::Enemy, ENEMY_ROW_Y)] {
        let board = &battle.side(who).board;
        let shown: Vec<_> = board
            .iter()
            .filter(|minion| revealed.contains(&minion.id))
            .collect();
        for (index, minion) in shown.iter().enumerate() {
            desired.push(Desired {
                id: minion.id,
                origin: minion.origin,
                card: minion.card,
                minion: true,
                enemy: who == Who::Enemy,
                lean: 0.0,
                forged: false,
                rank: match who {
                    Who::Player => CARD_RANK_PLAYER_ROW + index as f32,
                    Who::Enemy => index as f32,
                },
                slot: board_slot(index, shown.len(), row_y),
                attack: minion.attack,
                health: minion.health,
                guard: minion.guard,
                shield: minion.shield,
                ready: minion.can_attack(),
                dimmed: if targeting {
                    !targets.contains(&Target::Minion(who, minion.id))
                        && selected != Some(minion.id)
                } else {
                    who == Who::Player && player_turn && !minion.can_attack()
                },
            });
        }
    }
    desired
}

fn border_style(border: Border, element: Element) -> (Vec4, f32) {
    match border {
        Border::Normal => (element_color(element), CARD_BORDER_WIDTH),
        Border::Ready => (BORDER_READY, CARD_BORDER_WIDTH + 0.5),
        Border::Guard => (BORDER_TAUNT, 3.0),
        Border::Shield => (BORDER_SHIELD, 3.0),
        Border::Cursor => (BORDER_CURSOR, CARD_BORDER_WIDTH_STRONG),
        Border::Target => (BORDER_TARGET, CARD_BORDER_WIDTH_STRONG),
        Border::Selected => (BORDER_SELECTED, CARD_BORDER_WIDTH_STRONG),
        Border::Guide => (BORDER_GUIDE, CARD_BORDER_WIDTH_STRONG),
    }
}

fn wash(tree: &mut UiTreeBuilder, color: Vec4) {
    let tint = Vec4::new(color.x, color.y, color.z, CARD_WASH_ALPHA);
    tree.add_node()
        .window(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)), Anchor::TopLeft)
        .with_rect(10.0, 0.0, tint)
        .color_raw::<UiBase>(tint)
        .with_effect(RectEffect::VerticalGradient {
            secondary: Vec4::new(color.x, color.y, color.z, CARD_WASH_ALPHA * 0.35),
        })
        .without_pointer_events()
        .entity();
}

fn art_plate(
    tree: &mut UiTreeBuilder,
    center_y: f32,
    size: f32,
    icon_size: f32,
    color: Vec4,
    sigil: char,
) {
    tree.add_node()
        .window(
            Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, center_y)),
            Ab(vec2(size, size)),
            Anchor::Center,
        )
        .with_rect(size * 0.5, PORTRAIT_RING_WIDTH - 1.0, PORTRAIT_RING)
        .color_raw::<UiBase>(portrait_fill(color))
        .with_effect(RectEffect::RadialGlow {
            glow_color: PORTRAIT_GLOW,
            intensity: 0.45,
        })
        .with_shadow(CARD_SHADOW, vec2(0.0, 3.0), 8.0, 0.0)
        .without_pointer_events()
        .with_children(|tree| {
            tree.add_node()
                .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
                .with_icon(IconGlyph::lucide(sigil), icon_size)
                .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                .with_text_outline(TEXT_OUTLINE, 1.0)
                .color_raw::<UiBase>(TEXT_PRIMARY)
                .without_pointer_events()
                .entity();
        })
        .entity();
}

fn rule(tree: &mut UiTreeBuilder, y: f32, width: f32) {
    tree.add_node()
        .window(
            Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, y)),
            Ab(vec2(width, RULE_HEIGHT)),
            Anchor::TopCenter,
        )
        .with_rect(RULE_HEIGHT * 0.5, 0.0, RULE_COLOR)
        .color_raw::<UiBase>(RULE_COLOR)
        .without_pointer_events()
        .entity();
}

fn keyword_marks(tree: &mut UiTreeBuilder, card: CardId, top: f32, size: f32) {
    let Kind::Minion {
        guard,
        rush,
        venom,
        drain,
        ..
    } = card.def().kind
    else {
        return;
    };
    let marks: Vec<char> = [
        (guard, SHIELD_SIGIL),
        (rush, RUSH_SIGIL),
        (venom, VENOM_SIGIL),
        (drain, DRAIN_SIGIL),
    ]
    .into_iter()
    .filter(|(carried, _)| *carried)
    .map(|(_, glyph)| glyph)
    .collect();
    if marks.is_empty() {
        return;
    }
    let span = marks.len() as f32 * size + marks.len().saturating_sub(1) as f32 * KEYWORD_MARK_GAP;
    for (index, glyph) in marks.iter().enumerate() {
        let offset = -span * 0.5 + size * 0.5 + index as f32 * (size + KEYWORD_MARK_GAP);
        tree.add_node()
            .window(
                Rl(vec2(50.0, 0.0)) + Ab(vec2(offset, top + size * 0.5)),
                Ab(vec2(size, size)),
                Anchor::Center,
            )
            .with_rect(size * 0.5, 1.0, GEM_RING)
            .color_raw::<UiBase>(KEYWORD_MARK_FILL)
            .without_pointer_events()
            .with_children(|tree| {
                tree.add_node()
                    .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
                    .with_icon(IconGlyph::lucide(*glyph), size - 5.0)
                    .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                    .with_text_outline(TEXT_OUTLINE, 1.0)
                    .color_raw::<UiBase>(TEXT_PRIMARY)
                    .without_pointer_events()
                    .entity();
            })
            .entity();
    }
}

fn card_frame(tree: &mut UiTreeBuilder, frames: Frames, corner: f32) {
    let inner = tree
        .add_node()
        .window(
            Ab(vec2(CARD_FRAME_INSET, CARD_FRAME_INSET)),
            Rl(vec2(100.0, 100.0)) + Ab(vec2(-2.0 * CARD_FRAME_INSET, -2.0 * CARD_FRAME_INSET)),
            Anchor::TopLeft,
        )
        .without_pointer_events()
        .entity();
    tree.in_parent(inner, |tree| {
        frame(tree, frames.card, corner, CARD_FRAME_TINT);
    });
}

fn dim_overlay(tree: &mut UiTreeBuilder) -> Entity {
    tree.add_node()
        .window(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)), Anchor::TopLeft)
        .with_rect(10.0, 0.0, CARD_DIM)
        .color_raw::<UiBase>(CARD_DIM)
        .without_pointer_events()
        .with_visible(false)
        .entity()
}

fn minion_plate(tree: &mut UiTreeBuilder) {
    tree.add_node()
        .window(
            Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, 2.0)),
            Ab(vec2(
                MINION_WIDTH + MINION_PLATE_SPREAD,
                MINION_PLATE_HEIGHT,
            )),
            Anchor::Center,
        )
        .with_rect(MINION_PLATE_HEIGHT * 0.5, 0.0, MINION_PLATE_COLOR)
        .color_raw::<UiBase>(MINION_PLATE_COLOR)
        .with_shadow(CARD_SHADOW, vec2(0.0, 3.0), 10.0, 0.0)
        .without_pointer_events()
        .entity();
}

pub struct CardNode {
    pub root: Entity,
    pub attack_text: Option<Entity>,
    pub health_text: Option<Entity>,
    pub shield_badge: Option<Entity>,
    pub dim_overlay: Option<Entity>,
}

pub fn spawn_hand_node(
    tree: &mut UiTreeBuilder,
    frames: Frames,
    card: CardId,
    position: Vec2,
    stats: Option<(i32, i32)>,
    depth: f32,
    forged: bool,
) -> CardNode {
    let def = card.def();
    let name = if forged {
        format!("{}+", def.name)
    } else {
        def.name.to_string()
    };
    let color = element_color(def.element);
    let mut attack_text = None;
    let mut health_text = None;
    let mut overlay = None;
    let root = tree
        .add_node()
        .window(
            Ab(position),
            Ab(vec2(CARD_WIDTH, CARD_HEIGHT)),
            Anchor::TopLeft,
        )
        .with_depth(UiDepthMode::Set(depth))
        .with_rect(10.0, CARD_BORDER_WIDTH, color)
        .color_raw::<UiBase>(CARD_FACE)
        .color_raw::<UiHover>(CARD_FACE_LIT)
        .with_effect(RectEffect::VerticalGradient {
            secondary: CARD_FACE_BOTTOM,
        })
        .with_shadow(CARD_SHADOW, vec2(0.0, 7.0), 16.0, 0.0)
        .with_interaction()
        .with_spring_transition::<UiHover>(Spring::lively())
        .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
        .with_children(|tree| {
            if def.element != Element::Neutral {
                wash(tree, color);
            }
            card_frame(tree, frames, CARD_FRAME_CORNER);
            art_plate(
                tree,
                CARD_ART_Y,
                CARD_ART_SIZE,
                ICON_SIZE,
                color,
                card.sigil(),
            );
            keyword_marks(tree, card, CARD_MARKS_Y, KEYWORD_MARK_SIZE);
            tree.add_node()
                .window(
                    Ab(vec2(CARD_NAME_INSET, CARD_NAME_Y)),
                    Ab(vec2(CARD_WIDTH - CARD_NAME_INSET * 2.0, 16.0)),
                    Anchor::TopLeft,
                )
                .with_text(&name, CARD_NAME_SIZE)
                .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                .color_raw::<UiBase>(if forged { TEXT_FORGED } else { CARD_TEXT_COLOR })
                .without_pointer_events()
                .entity();
            if !def.text.is_empty() {
                rule(tree, CARD_RULE_Y, CARD_WIDTH - 30.0);
                tree.add_node()
                    .window(
                        Ab(vec2(CARD_TEXT_INSET, CARD_TEXT_Y)),
                        Ab(vec2(CARD_WIDTH - CARD_TEXT_INSET * 2.0, CARD_TEXT_HEIGHT)),
                        Anchor::TopLeft,
                    )
                    .with_text(def.text, CARD_TEXT_SIZE)
                    .with_text_wrap()
                    .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                    .color_raw::<UiBase>(CARD_TEXT_COLOR)
                    .without_pointer_events()
                    .entity();
            }
            gem_node(
                tree,
                Ab(vec2(CARD_MANA_INSET, CARD_MANA_INSET)).into(),
                Anchor::TopLeft,
                MANA_BLUE,
                &def.cost.to_string(),
                BADGE_SIZE,
            );
            if let Some((attack, health)) = stats {
                attack_text = Some(gem_node(
                    tree,
                    Rl(vec2(0.0, 100.0)) + Ab(vec2(CARD_MANA_INSET, -CARD_MANA_INSET)),
                    Anchor::BottomLeft,
                    ATTACK_GOLD,
                    &attack.to_string(),
                    BADGE_SIZE,
                ));
                health_text = Some(gem_node(
                    tree,
                    Rl(vec2(100.0, 100.0)) + Ab(vec2(-CARD_MANA_INSET, -CARD_MANA_INSET)),
                    Anchor::BottomRight,
                    HEALTH_RED,
                    &health.to_string(),
                    BADGE_SIZE,
                ));
            }
            overlay = Some(dim_overlay(tree));
        })
        .entity();
    tree.commands().set(root, UiButtonData::default());
    CardNode {
        root,
        attack_text,
        health_text,
        shield_badge: None,
        dim_overlay: overlay,
    }
}

pub fn spawn_relic_node(tree: &mut UiTreeBuilder, relic: Relic) -> Entity {
    tree.add_node()
        .window(
            Ab(vec2(0.0, 0.0)),
            Ab(vec2(CARD_WIDTH, CARD_HEIGHT)),
            Anchor::TopLeft,
        )
        .with_rect(10.0, CARD_BORDER_WIDTH, RELIC_BORDER)
        .color_raw::<UiBase>(RELIC_BG)
        .with_effect(RectEffect::VerticalGradient {
            secondary: RELIC_BG_BOTTOM,
        })
        .with_shadow(CARD_SHADOW, vec2(0.0, 6.0), 14.0, 0.0)
        .with_children(|tree| {
            tree.add_node()
                .window(Rl(vec2(50.0, 40.0)), Ab(vec2(56.0, 56.0)), Anchor::Center)
                .with_icon(RELIC_ICON, ICON_SIZE + 6.0)
                .color_raw::<UiBase>(TEXT_GOLD)
                .without_pointer_events()
                .entity();
            tree.add_node()
                .window(
                    Ab(vec2(4.0, CARD_HEIGHT - 48.0)),
                    Ab(vec2(CARD_WIDTH - 8.0, 40.0)),
                    Anchor::TopLeft,
                )
                .with_text(relic.name(), CARD_NAME_SIZE)
                .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                .color_raw::<UiBase>(TEXT_PRIMARY)
                .without_pointer_events()
                .entity();
        })
        .entity()
}

fn spawn_minion_node(
    tree: &mut UiTreeBuilder,
    frames: Frames,
    entry: &Desired,
    position: Vec2,
    depth: f32,
) -> CardNode {
    let def = entry.card.def();
    let color = element_color(def.element);
    let face = if entry.enemy {
        CARD_FACE_ENEMY
    } else {
        CARD_FACE
    };
    let mut attack_text = None;
    let mut health_text = None;
    let mut shield_badge = None;
    let mut overlay = None;
    let root = tree
        .add_node()
        .window(
            Ab(position),
            Ab(vec2(MINION_WIDTH, MINION_HEIGHT)),
            Anchor::TopLeft,
        )
        .with_depth(UiDepthMode::Set(depth))
        .with_rect(10.0, CARD_BORDER_WIDTH, color)
        .color_raw::<UiBase>(face)
        .color_raw::<UiHover>(CARD_FACE_LIT)
        .with_effect(RectEffect::VerticalGradient {
            secondary: CARD_FACE_BOTTOM,
        })
        .with_shadow(CARD_SHADOW, vec2(0.0, 6.0), 14.0, 0.0)
        .with_interaction()
        .with_spring_transition::<UiHover>(Spring::lively())
        .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
        .with_children(|tree| {
            if def.element != Element::Neutral {
                wash(tree, color);
            }
            minion_plate(tree);
            card_frame(tree, frames, MINION_FRAME_CORNER);
            tree.add_node()
                .window(
                    Ab(vec2(MINION_BADGE_INSET, 5.0)),
                    Ab(vec2(MINION_WIDTH - MINION_BADGE_INSET * 2.0, 16.0)),
                    Anchor::TopLeft,
                )
                .with_text(def.name, MINION_NAME_SIZE)
                .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                .color_raw::<UiBase>(CARD_TEXT_COLOR)
                .without_pointer_events()
                .entity();
            art_plate(
                tree,
                MINION_ART_Y,
                MINION_ART_SIZE,
                MINION_ICON_SIZE,
                color,
                entry.card.sigil(),
            );
            keyword_marks(tree, entry.card, MINION_MARKS_Y, KEYWORD_MARK_SIZE);
            attack_text = Some(gem_node(
                tree,
                Rl(vec2(0.0, 100.0)) + Ab(vec2(MINION_BADGE_INSET, -MINION_BADGE_INSET)),
                Anchor::BottomLeft,
                ATTACK_GOLD,
                &entry.attack.to_string(),
                MINION_BADGE_SIZE,
            ));
            health_text = Some(gem_node(
                tree,
                Rl(vec2(100.0, 100.0)) + Ab(vec2(-MINION_BADGE_INSET, -MINION_BADGE_INSET)),
                Anchor::BottomRight,
                HEALTH_RED,
                &entry.health.to_string(),
                MINION_BADGE_SIZE,
            ));
            shield_badge = Some(shield_node(tree, entry.shield));
            overlay = Some(dim_overlay(tree));
        })
        .entity();
    tree.commands().set(root, UiButtonData::default());
    CardNode {
        root,
        attack_text,
        health_text,
        shield_badge,
        dim_overlay: overlay,
    }
}

fn shield_node(tree: &mut UiTreeBuilder, shielded: bool) -> Entity {
    tree.add_node()
        .window(
            Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -MINION_BADGE_INSET)),
            Ab(vec2(SHIELD_BADGE_SIZE, SHIELD_BADGE_SIZE)),
            Anchor::BottomCenter,
        )
        .with_rect(SHIELD_BADGE_SIZE * 0.5, 1.5, BORDER_SHIELD)
        .color_raw::<UiBase>(SHIELD_BADGE_FILL)
        .with_shadow(BORDER_SHIELD, vec2(0.0, 0.0), 9.0, 1.0)
        .with_icon(SHIELD_ICON, SHIELD_BADGE_SIZE - 7.0)
        .without_pointer_events()
        .with_visible(shielded)
        .entity()
}

fn gem_node(
    tree: &mut UiTreeBuilder,
    position: UiValue<Vec2>,
    anchor: Anchor,
    color: Vec4,
    text: &str,
    size: f32,
) -> Entity {
    let glow = Vec4::new(
        (color.x + 0.4).min(1.0),
        (color.y + 0.4).min(1.0),
        (color.z + 0.4).min(1.0),
        1.0,
    );
    let mut label = None;
    tree.add_node()
        .window(position, Ab(vec2(size, size)), anchor)
        .with_rect(size * 0.5, 1.5, GEM_RING)
        .color_raw::<UiBase>(color)
        .with_effect(RectEffect::RadialGlow {
            glow_color: glow,
            intensity: 0.5,
        })
        .with_shadow(CARD_SHADOW, vec2(0.0, 2.0), 6.0, 0.0)
        .without_pointer_events()
        .with_children(|tree| {
            label = Some(
                tree.add_node()
                    .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
                    .with_text(text, STAT_SIZE)
                    .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                    .with_text_outline(TEXT_OUTLINE, 1.0)
                    .color_raw::<UiBase>(TEXT_PRIMARY)
                    .without_pointer_events()
                    .entity(),
            );
        })
        .entity();
    label.unwrap_or_else(|| tree.commands().spawn(()))
}
