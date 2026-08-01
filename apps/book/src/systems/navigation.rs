use nightshade::prelude::*;
use nightshade_platform::contract::input_capture::InputCapture;
use nightshade_platform::prelude::*;
use nightshade_platform::queries::input_capture::keyboard_captured;
use nightshade_ui::components::UiNodeInteraction;
use nightshade_ui::prelude::*;

use crate::components::{PageLink, RailLink};
use crate::queries::book::{link_target, next_page, previous_page};
use crate::resources::{BookState, Library, PagePhase, Search, Shell};

pub fn poll(
    mut state: ResMut<BookState>,
    mut search: ResMut<Search>,
    shell: Res<Shell>,
    library: Res<Library>,
    input: Res<Input>,
    capture: Res<InputCapture>,
    frame: Res<RetainedUiFrame>,
    mut interactions: Query<(&UiNodeInteraction,)>,
    links: Query<(&RailLink, &UiNodeInteraction)>,
    page_links: Query<(&PageLink, &UiNodeInteraction)>,
) {
    if search.open {
        return;
    }

    let mut requested = None;
    let mut jump = None;

    for event in ui_events(&frame) {
        if let UiEvent::TreeNodeSelected { tree, node, .. } = event
            && *tree == shell.nav_view
            && let Some(position) = shell.nav_nodes.iter().position(|held| held == node)
        {
            requested = shell.nav_chapters.get(position).copied();
            if let Some(Some(anchor)) = shell.nav_anchors.get(position) {
                jump = Some(anchor.clone());
            }
        }
    }

    for (position, row) in shell.demo_rows.iter().enumerate() {
        if !clicked(&mut interactions, *row) {
            continue;
        }
        if let Some((chapter, anchor)) = shell.demo_targets.get(position) {
            requested = Some(*chapter);
            jump = anchor.clone();
        }
    }

    if clicked(&mut interactions, shell.previous_button) {
        requested = previous_page(&library, state.current);
    }
    if clicked(&mut interactions, shell.next_button) {
        requested = next_page(&library, state.current);
    }
    if clicked(&mut interactions, shell.search_button) {
        search.open = true;
    }
    if clicked(&mut interactions, shell.expand_all) {
        state.fold_all = Some(true);
    }
    if clicked(&mut interactions, shell.collapse_all) {
        state.fold_all = Some(false);
    }

    let mut top = false;
    links.for_each(|_entity, (link, interaction)| {
        if interaction.clicked {
            match &link.anchor {
                Some(anchor) => jump = Some(anchor.clone()),
                None => top = true,
            }
        }
    });
    if top {
        state.scroll_to = Some(0.0);
    }

    let mut followed = None;
    page_links.for_each(|_entity, (link, interaction)| {
        if interaction.clicked {
            followed = link_target(&library, state.current, &link.target);
        }
    });
    if let Some((page, anchor)) = followed {
        requested = Some(page);
        if anchor.is_some() {
            jump = anchor;
        }
    }

    if !keyboard_captured(&capture) {
        if input.keyboard.just_pressed(KeyCode::ArrowLeft) {
            requested = previous_page(&library, state.current);
        }
        if input.keyboard.just_pressed(KeyCode::ArrowRight) {
            requested = next_page(&library, state.current);
        }
        if input.keyboard.just_pressed(KeyCode::Home) {
            requested = Some(0);
        }
        if input.keyboard.just_pressed(KeyCode::End) {
            requested = library.chapters.len().checked_sub(1);
        }
        if input.keyboard.just_pressed(KeyCode::Slash) {
            search.open = true;
        }
    }

    if let Some(anchor) = jump {
        state.jump_to = Some(anchor);
    }

    if state.phase != PagePhase::Idle || state.requested.is_some() {
        return;
    }
    if let Some(index) = requested
        && index != state.current
    {
        state.requested = Some(index);
    }
}

fn clicked(interactions: &mut Query<(&UiNodeInteraction,)>, entity: Entity) -> bool {
    let mut hit = false;
    interactions.for_entity(entity, |(interaction,)| {
        hit = interaction.clicked && !interaction.disabled;
    });
    hit
}
