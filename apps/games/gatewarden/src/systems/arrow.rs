use crate::components::{Instance, Spot};
use crate::contract::Screen;
use crate::data::battle::{Battle, Pick, Who};
use crate::queries::battle::pick_at_cursor;
use crate::queries::table::curve_point;
use crate::resources::{GatewardenResources, Selection};
use crate::theme::*;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_ui::prelude::*;

pub fn sync(
    resources: Res<GatewardenResources>,
    input: Res<Input>,
    mut canvases: Query<(&mut UiCanvasData,)>,
    mut nodes: Query<(&UiLayoutNode,)>,
    mut cards: Query<(&Instance, &Spot)>,
) {
    let Some(canvas) = resources.ui.arrow_canvas else {
        return;
    };
    canvases.for_entity(canvas, |(data,)| data.commands.clear());

    let Some(battle) = resources.battle.as_ref() else {
        return;
    };
    if resources.screen != Screen::Battle {
        return;
    }
    let Some(source_id) = (match resources.selection {
        Selection::Attacker(id) | Selection::Spell(id) => Some(id),
        _ => None,
    }) else {
        return;
    };

    let mut rect = None;
    nodes.for_entity(canvas, |(layout,)| rect = Some(layout.computed_rect));
    let Some(rect) = rect else {
        return;
    };
    let scale = rect.width() / HAND_WIDTH;
    if scale <= 0.0 {
        return;
    }

    let Some(source) = center_of(&mut cards, battle, source_id) else {
        return;
    };
    let target = match cursor_target(battle, &resources, &mut cards) {
        Some(center) => center * scale,
        None => input.mouse.position - rect.min,
    };
    let source = source * scale;

    let span = target - source;
    let reach = span.magnitude();
    if reach < ARROW_HEAD_LENGTH * scale {
        return;
    }
    let bow = (reach * ARROW_BOW).min(ARROW_BOW_MAX * scale);
    let control = source + span * 0.5 - vec2(0.0, bow);
    let head_length = ARROW_HEAD_LENGTH * scale;
    let head_width = ARROW_HEAD_WIDTH * scale;
    let heading = (target - control).normalize();
    let base = target - heading * head_length;

    canvases.for_entity(canvas, |(data,)| {
        for index in 0..ARROW_SEGMENTS {
            let from = curve_point(source, control, base, index as f32 / ARROW_SEGMENTS as f32);
            let to = curve_point(
                source,
                control,
                base,
                (index + 1) as f32 / ARROW_SEGMENTS as f32,
            );
            let along = (index as f32 + 0.5) / ARROW_SEGMENTS as f32;
            let thickness = (ARROW_LINE_MIN + (ARROW_LINE_MAX - ARROW_LINE_MIN) * along) * scale;
            data.commands.push(CanvasCommand::Line {
                from,
                to,
                thickness,
                color: ARROW_COLOR,
                id: None,
            });
        }
        let across = vec2(-heading.y, heading.x);
        let step = head_length / ARROW_HEAD_LINES as f32;
        for index in 0..ARROW_HEAD_LINES {
            let along = (index as f32 + 0.5) / ARROW_HEAD_LINES as f32;
            let center = target - heading * along * head_length;
            let half = across * along * head_width * 0.5;
            data.commands.push(CanvasCommand::Line {
                from: center - half,
                to: center + half,
                thickness: step + 1.0,
                color: ARROW_HEAD_COLOR,
                id: None,
            });
        }
    });
}

fn cursor_target(
    battle: &Battle,
    resources: &GatewardenResources,
    cards: &mut Query<'_, (&Instance, &Spot)>,
) -> Option<Vec2> {
    if !resources.cursor.active {
        return None;
    }
    let id = match pick_at_cursor(battle, resources.cursor)? {
        Pick::Hand(id) | Pick::Mine(id) | Pick::Foe(id) => id,
        Pick::Hero(_) => return None,
    };
    center_of(cards, battle, id)
}

fn center_of(cards: &mut Query<'_, (&Instance, &Spot)>, battle: &Battle, id: u32) -> Option<Vec2> {
    let mut position = None;
    cards.for_each_mut(|_, (card_id, spot)| {
        if card_id.0 == id {
            position = Some(spot.0);
        }
    });
    let position = position?;
    let minion = [Who::Player, Who::Enemy]
        .iter()
        .any(|who| battle.side(*who).minion(id).is_some());
    let span = if minion {
        vec2(MINION_WIDTH, MINION_HEIGHT)
    } else {
        vec2(CARD_WIDTH, CARD_HEIGHT)
    };
    Some(position + span * 0.5)
}
