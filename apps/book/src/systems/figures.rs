use nightshade::prelude::*;
use nightshade_ui::components::UiLayoutNode;
use nightshade_ui::prelude::*;

use crate::commands::figure::{EXPAND_LABEL, RESTORE_LABEL};
use crate::data::style::{PAGE_PADDING, STAGE_HEIGHT};
use crate::queries::screen::to_units;
use crate::resources::{BookState, PageStage, Reading, Shell};

pub fn expand(
    mut page_stage: ResMut<PageStage>,
    mut state: ResMut<BookState>,
    reading: Res<Reading>,
    shell: Res<Shell>,
    window: Res<nightshade_platform::prelude::Window>,
    mut build: nightshade_ui::commands::builder::UiBuild,
    mut nodes: Query<(&mut UiLayoutNode,)>,
    mut interactions: Query<(&UiNodeInteraction,)>,
) {
    let was = page_stage.expanded;

    for index in 0..page_stage.figures.len() {
        let button = page_stage.figures[index].expand_button;
        if clicked(&mut interactions, button) {
            page_stage.expanded = if was == Some(index) {
                None
            } else {
                Some(index)
            };
        }
    }

    let jumping = state.jump_to.is_some();
    if jumping {
        page_stage.expanded = None;
    }

    if page_stage
        .expanded
        .is_some_and(|index| index >= page_stage.figures.len())
    {
        page_stage.expanded = None;
    }

    if page_stage.expanded == was {
        return;
    }

    let standing = page_stage.expanded;
    let Some(root) = state.page_root else {
        return;
    };

    let mut room = 0.0;
    nodes.for_entity(shell.content_area, |(node,)| {
        room = to_units(node.computed_rect.height(), &window) - PAGE_PADDING * 2.0;
    });

    let blocks: Vec<Entity> = build.context().hierarchy.children(root).to_vec();
    let card = standing.map(|index| page_stage.figures[index].card);
    for block in blocks {
        nodes.for_entity(block, |(node,)| {
            node.visible = card.is_none_or(|card| card == block);
        });
    }

    for (index, figure) in page_stage.figures.iter().enumerate() {
        let standing = standing == Some(index);
        let card_height = if standing {
            room.max(figure.card_height)
        } else {
            figure.card_height
        };
        let stage_height = card_height - (figure.card_height - STAGE_HEIGHT);
        set_height(&mut nodes, figure.card, card_height);
        set_height(&mut nodes, figure.node, stage_height);

        let label = if standing {
            RESTORE_LABEL
        } else {
            EXPAND_LABEL
        };
        ui_set_text(build.context().commands, figure.expand_label, label);
    }

    ui_mark_layout_dirty(build.context().dirty);

    if standing.is_some() {
        page_stage.reading_from = reading.offset;
        state.scroll_to = Some(0.0);
    } else if !jumping {
        state.scroll_to = Some(page_stage.reading_from);
    }
}

fn set_height(nodes: &mut Query<(&mut UiLayoutNode,)>, entity: Entity, height: f32) {
    nodes.for_entity(entity, |(node,)| {
        node.flow_child_size = Some(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, height)));
    });
}

fn clicked(interactions: &mut Query<(&UiNodeInteraction,)>, entity: Entity) -> bool {
    let mut hit = false;
    interactions.for_entity(entity, |(interaction,)| hit = interaction.clicked);
    hit
}
