use nightshade::prelude::*;
use nightshade_platform::contract::input::mouse::MouseState;
use nightshade_platform::prelude::Input;
use nightshade_ui::components::UiLayoutNode;
use nightshade_ui::prelude::*;

use crate::resources::{Appearance, Shell};

pub fn drive(
    mut appearance: ResMut<Appearance>,
    shell: Res<Shell>,
    mut theme: ResMut<ThemeState>,
    mut dirty: ResMut<RetainedUiDirty>,
    input: Res<Input>,
    mut interactions: Query<(&UiNodeInteraction,)>,
    mut nodes: Query<(&mut UiLayoutNode,)>,
) {
    let was_open = appearance.open;

    let asked = std::mem::take(&mut appearance.asked);
    if asked || clicked(&mut interactions, shell.appearance_button) {
        appearance.open = !appearance.open;
    } else if appearance.open
        && input.mouse.state.contains(MouseState::LEFT_JUST_PRESSED)
        && !over_panel(&mut nodes, &shell, input.mouse.position)
    {
        appearance.open = false;
    }

    if appearance.open {
        match hovered(&mut interactions, &shell.theme_rows) {
            Some(row) => theme.set_preview(Some(row)),
            None => theme.clear_preview(),
        }
        match hovered(&mut interactions, &shell.size_rows) {
            Some(row) => theme.set_type_scale_preview(appearance.scale_at(row)),
            None => theme.clear_type_scale_preview(),
        }

        if let Some(row) = clicked_row(&mut interactions, &shell.theme_rows) {
            theme.select_theme(row);
            mark_selected(&mut nodes, &shell.theme_marks, row);
        }
        if let Some(row) = clicked_row(&mut interactions, &shell.size_rows) {
            theme.select_type_scale(appearance.scale_at(row));
            mark_selected(&mut nodes, &shell.size_marks, row);
        }
    }

    if appearance.open != was_open {
        if !appearance.open {
            theme.clear_preview();
            theme.clear_type_scale_preview();
        }
        set_visible(&mut nodes, shell.appearance_panel, appearance.open);
        dirty.layout_dirty = true;
    }
}

fn over_panel(nodes: &mut Query<(&mut UiLayoutNode,)>, shell: &Shell, pointer: Vec2) -> bool {
    let mut over = false;
    nodes.for_entity(shell.appearance_panel, |(node,)| {
        over = node.visible && node.computed_rect.contains(pointer);
    });
    over
}

fn hovered(interactions: &mut Query<(&UiNodeInteraction,)>, rows: &[Entity]) -> Option<usize> {
    rows.iter().position(|row| {
        let mut over = false;
        interactions.for_entity(*row, |(interaction,)| over = interaction.hovered);
        over
    })
}

fn clicked_row(interactions: &mut Query<(&UiNodeInteraction,)>, rows: &[Entity]) -> Option<usize> {
    rows.iter().position(|row| clicked(interactions, *row))
}

fn clicked(interactions: &mut Query<(&UiNodeInteraction,)>, entity: Entity) -> bool {
    let mut hit = false;
    interactions.for_entity(entity, |(interaction,)| hit = interaction.clicked);
    hit
}

fn set_visible(nodes: &mut Query<(&mut UiLayoutNode,)>, entity: Entity, visible: bool) {
    nodes.for_entity(entity, |(node,)| node.visible = visible);
}

fn mark_selected(nodes: &mut Query<(&mut UiLayoutNode,)>, marks: &[Entity], selected: usize) {
    for (index, mark) in marks.iter().enumerate() {
        set_visible(nodes, *mark, index == selected);
    }
}
