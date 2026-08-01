use crate::resources::DialoguePanel;
use crate::theme;
use blocker_hud::resources::ArtLayers;
use blocker_hud::resources::ShellHandles;
use blocker_player::data::ask::Ask;
use blocker_pocket::resources::Inventory;
use blocker_quests::commands::{accept_quest, complete_quest};
use blocker_quests::resources::Journal;
use blocker_screen::data::notice::Notice;
use blocker_screen::data::screen::Screen;
use blocker_talk::resources::{Deed, Deeds, Objective};
use nalgebra_glm::{vec2, vec4};
use nightshade::ecs::system_param::EventWriter;
use nightshade::prelude::*;
use nightshade_dialogue::prelude::{
    Conversation, close, fully_shown, last_line, reveal_all, shown, step,
};
use nightshade_input::Actions;
use nightshade_state::prelude::*;
use nightshade_ui::commands::builder::{UiBuild, UiTreeBuilder};
use nightshade_ui::components::TextOverflow;
use nightshade_ui::components::layout_types::FlowDirection;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;
use nightshade_ui_menu::prelude::{CardStyle, add_corner_card};
use nightshade_ui_menu::prelude::{Menus, menu_show_on};

pub fn build_dialogue(
    style: Res<CardStyle>,
    art: Res<ArtLayers>,
    shell: Res<ShellHandles>,
    mut menus: ResMut<Menus<Screen>>,
    mut dialogue_panel: ResMut<DialoguePanel>,
    mut build: UiBuild,
) {
    let root = shell.root;
    let mut built_tree = UiTreeBuilder::from_parent(build.context(), root);
    let tree = &mut built_tree;
    dialogue_panel.panel = add_corner_card(
        tree,
        &style,
        art.frame,
        theme::DIALOGUE_LEFT,
        theme::DIALOGUE_TOP,
        theme::DIALOGUE_WIDTH,
        theme::DIALOGUE_HEIGHT,
    );
    tree.in_parent(dialogue_panel.panel, |tree| {
        let bar = tree
            .add_node()
            .size_px(theme::DIALOGUE_WIDTH, blocker_hud::theme::TITLE_HEIGHT)
            .with_rect(10.0, 0.0, vec4(0.0, 0.0, 0.0, 0.0))
            .color_raw::<UiBase>(blocker_hud::theme::TITLE_COLOR)
            .flow(
                FlowDirection::Horizontal,
                blocker_hud::theme::BODY_PADDING,
                8.0,
            )
            .entity();
        tree.in_parent(bar, |tree| {
            let width = theme::DIALOGUE_WIDTH - blocker_hud::theme::BODY_PADDING * 2.0;
            dialogue_panel.name = tree
                .add_node()
                .size_px(
                    width,
                    blocker_hud::theme::TITLE_HEIGHT - blocker_hud::theme::BODY_PADDING,
                )
                .with_text("", blocker_hud::theme::HEADING_SIZE)
                .text_left()
                .color_raw::<UiBase>(blocker_hud::theme::ACCENT_COLOR)
                .entity();
        });
        let body = tree
            .add_node()
            .size_px(
                theme::DIALOGUE_WIDTH,
                theme::DIALOGUE_HEIGHT - blocker_hud::theme::TITLE_HEIGHT,
            )
            .flow(
                FlowDirection::Vertical,
                blocker_hud::theme::BODY_PADDING,
                blocker_hud::theme::ROW_GAP,
            )
            .entity();
        tree.in_parent(body, |tree| {
            dialogue_panel.scroll = tree.add_scroll_area(vec2(0.0, theme::DIALOGUE_BODY_HEIGHT));
            let content = tree
                .component::<UiScrollAreaData>(dialogue_panel.scroll)
                .map(|data| data.content_entity)
                .unwrap_or(dialogue_panel.scroll);
            dialogue_panel.body = tree.in_parent(content, |tree| {
                tree.add_node()
                    .flow_child(Rl(vec2(100.0, 0.0)))
                    .auto_height()
                    .with_text("", blocker_hud::theme::BODY_SIZE)
                    .with_text_overflow(TextOverflow::Wrap)
                    .text_left()
                    .color_raw::<UiBase>(blocker_hud::theme::TEXT_BRIGHT)
                    .entity()
            });
            dialogue_panel.objective = tree
                .add_node()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 44.0)))
                .with_text("", blocker_hud::theme::SMALL_SIZE)
                .with_text_overflow(TextOverflow::Wrap)
                .text_left()
                .color_raw::<UiBase>(blocker_hud::theme::ACCENT_COLOR)
                .entity();
            let mut rows = Vec::new();
            let mut choices = Vec::new();
            for _ in 0..theme::DIALOGUE_CHOICES {
                let holder = tree
                    .add_node()
                    .size_px(
                        theme::DIALOGUE_ROW_WIDTH,
                        blocker_hud::theme::ACTION_HEIGHT + 6.0,
                    )
                    .flow(FlowDirection::Horizontal, 0.0, 0.0)
                    .with_visible(false)
                    .entity();
                let button = tree.in_parent(holder, |tree| tree.add_button(""));
                rows.push(holder);
                choices.push(button);
            }
            dialogue_panel.rows = rows;
            dialogue_panel.choices = choices;
        });
    });
    menu_show_on(&mut menus, Screen::Dialogue, dialogue_panel.panel);
    built_tree.finish();
}

pub fn show_the_dialogue(
    conversation: Res<Conversation>,
    objective: Res<Objective>,
    mut dialogue_panel: ResMut<DialoguePanel>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    if dialogue_panel.rows.is_empty() {
        return;
    }
    let last = last_line(&conversation);
    let key = (conversation.speaker.clone(), shown(&conversation), last);
    if dialogue_panel.shown.as_ref() == Some(&key) {
        return;
    }
    ui_set_text(&mut commands, dialogue_panel.name, &key.0);
    ui_set_text(&mut commands, dialogue_panel.body, &key.1);
    ui_scroll_area_set_offset(&mut commands, dialogue_panel.scroll, 0.0);
    ui_set_text(
        &mut commands,
        dialogue_panel.objective,
        if last { objective.0.as_str() } else { "" },
    );
    let count = if last { conversation.replies.len() } else { 1 };
    for index in 0..dialogue_panel.rows.len() {
        let visible = index < count;
        ui_set_visible(
            &mut dirty,
            &mut commands,
            dialogue_panel.rows[index],
            visible,
        );
        if !visible {
            continue;
        }
        let label = if last {
            format!("{}. {}", index + 1, conversation.replies[index])
        } else {
            "1. Go on.".to_string()
        };
        ui_set_text(&mut commands, dialogue_panel.choices[index], &label);
    }
    dialogue_panel.shown = Some(key);
}

pub fn dialogue_buttons(
    actions: Res<Actions<Ask>>,
    deeds: Res<Deeds>,
    dialogue_panel: Res<DialoguePanel>,
    mut conversation: ResMut<Conversation>,
    mut inventory: ResMut<Inventory>,
    mut journal: ResMut<Journal>,
    mut notices: EventWriter<Notice>,
    mut next: ResMut<NextState<Screen>>,
    mut buttons: Query<(&UiButtonData,)>,
) {
    if !fully_shown(&conversation) || !last_line(&conversation) {
        let Some(button) = dialogue_panel.choices.first().copied() else {
            return;
        };
        let typed = Ask::SLOTS
            .first()
            .is_some_and(|slot| actions.just_pressed(*slot));
        if !typed && !ui_clicked(&mut buttons, button) {
            return;
        }
        if fully_shown(&conversation) {
            step(&mut conversation);
        } else {
            reveal_all(&mut conversation);
        }
        return;
    }
    for index in 0..conversation.replies.len() {
        let Some(button) = dialogue_panel.choices.get(index).copied() else {
            continue;
        };
        let typed = Ask::SLOTS
            .get(index)
            .is_some_and(|slot| actions.just_pressed(*slot));
        if !typed && !ui_clicked(&mut buttons, button) {
            continue;
        }
        match deeds.0.get(index).copied() {
            Some(Deed::Accept(quest_index)) => {
                accept_quest(&mut inventory, &mut journal, &mut notices, quest_index)
            }
            Some(Deed::Complete(quest_index)) => {
                complete_quest(&mut inventory, &mut journal, &mut notices, quest_index)
            }
            Some(Deed::Close) | None => {}
        }
        close(&mut conversation);
        next.pending = Some(Screen::Playing);
        return;
    }
}
