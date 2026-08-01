use crate::components::{Age, BarFill, Base, Card, Instance, Rise, Spot};
use crate::resources::GatewardenResources;
use crate::theme::*;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_text::prelude::*;
use nightshade_ui::prelude::*;
use std::collections::HashMap;

pub fn sync(
    mut resources: ResMut<GatewardenResources>,
    time: Res<Time>,
    mut build: nightshade_ui::commands::builder::UiBuild,
    mut nodes: Query<(&mut UiLayoutNode,)>,
    mut floaters: Query<(&mut Base, &mut Rise, &mut Age, &Card)>,
    mut fills: Query<(&mut BarFill,)>,
    mut cards: Query<(&Instance, &Spot)>,
) {
    let resources = &mut *resources;
    let Some(arena) = resources.ui.arena else {
        return;
    };
    let mut moved = false;
    let mut spots: HashMap<u32, Vec2> = HashMap::new();
    cards.for_each_mut(|_, (id, spot)| {
        spots.insert(id.0, spot.0);
    });
    moved |= animate_floaters(
        resources,
        arena,
        &time,
        &spots,
        &mut build,
        &mut nodes,
        &mut floaters,
    );
    moved |= sync_bars(resources, &mut nodes, &mut fills);
    if moved {
        ui_mark_layout_dirty(build.dirty());
    }
}

fn animate_floaters(
    resources: &mut GatewardenResources,
    arena: Entity,
    time: &Time,
    spots: &HashMap<u32, Vec2>,
    build: &mut nightshade_ui::commands::builder::UiBuild,
    nodes: &mut Query<'_, (&mut UiLayoutNode,)>,
    floaters: &mut Query<'_, (&mut Base, &mut Rise, &mut Age, &Card)>,
) -> bool {
    let table = &mut resources.table;
    if !table.pending_floaters.is_empty() {
        let mut tree = UiTreeBuilder::from_parent(build.context(), arena);
        for (card, base, label, color) in table.pending_floaters.drain(..) {
            let root = tree
                .add_node()
                .window(Ab(base), Ab(FLOATER_BOX), Anchor::TopLeft)
                .with_text(&label, FLOATER_SIZE)
                .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                .with_text_outline(TEXT_OUTLINE, 1.5)
                .color_raw::<UiBase>(color)
                .without_pointer_events()
                .entity();
            let commands = tree.commands();
            commands.set(root, Base(base));
            commands.set(root, Rise(0.0));
            commands.set(root, Age(0.0));
            commands.set(root, Card(card));
        }
        tree.finish();
    }
    let delta_time = time.delta_time;
    let mut expired = Vec::new();
    let mut placements = Vec::new();
    floaters.for_each_mut(|entity, (base, rise, age, card)| {
        age.0 += delta_time;
        rise.0 += FLOATER_RISE * delta_time;
        if age.0 >= FLOATER_SECONDS {
            expired.push(entity);
            return;
        }
        if let Some(id) = card.0
            && let Some(spot) = spots.get(&id)
        {
            base.0 = *spot + FLOATER_CARD_OFFSET;
        }
        placements.push((entity, base.0 - vec2(0.0, rise.0)));
    });
    let moved = !placements.is_empty();
    for (entity, position) in placements {
        nodes.for_entity(entity, |(layout,)| {
            layout.base_layout = Some(UiLayoutType::Window(WindowLayout {
                position: Ab(position).into(),
                size: Ab(FLOATER_BOX).into(),
                anchor: Anchor::TopLeft,
            }));
        });
    }
    if !expired.is_empty() {
        let commands = build.context().commands;
        for entity in &expired {
            commands.remove::<(Base, Rise, Age, Card)>(*entity);
        }
        table.retired.extend(expired);
    }
    moved
}

fn sync_bars(
    resources: &GatewardenResources,
    nodes: &mut Query<'_, (&mut UiLayoutNode,)>,
    fills: &mut Query<'_, (&mut BarFill,)>,
) -> bool {
    let Some(battle) = resources.battle.as_ref() else {
        return false;
    };
    let theater = &resources.theater;
    let bars = [
        (
            resources.ui.player_health_fill,
            theater.shown_player_health,
            battle.player.hero_max,
        ),
        (
            resources.ui.enemy_health_fill,
            theater.shown_enemy_health,
            battle.enemy.hero_max,
        ),
        (
            resources.ui.player_mana_fill,
            battle.player.mana as i32,
            battle.player.max_mana.max(1) as i32,
        ),
    ];
    let mut moved = false;
    for (fill, health, max) in bars {
        let Some(fill) = fill else {
            continue;
        };
        let percent = (health as f32 / max.max(1) as f32).clamp(0.0, 1.0) * 100.0;
        let mut changed = false;
        fills.for_entity(fill, |(state,)| {
            changed = (state.percent - percent).abs() >= 0.01;
            state.percent = percent;
        });
        if !changed {
            continue;
        }
        nodes.for_entity(fill, |(layout,)| {
            layout.base_layout = Some(UiLayoutType::Window(WindowLayout {
                position: Ab(vec2(0.0, 0.0)).into(),
                size: Rl(vec2(percent, 100.0)).into(),
                anchor: Anchor::TopLeft,
            }));
        });
        moved = true;
    }
    moved
}
