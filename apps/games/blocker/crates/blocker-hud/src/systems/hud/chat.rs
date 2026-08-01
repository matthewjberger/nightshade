use crate::resources::ChatEntry;
use crate::resources::ChatPanel;
use crate::resources::HudBuild;
use crate::systems::widgets::channel_color;
use crate::theme;
use blocker_chat::commands::{npc_say, respond_to_keywords};
use blocker_chat::data::chat::{CHAT_TABS, Channel};
use blocker_chat::resources::{Chat, Chatter, Roster};
use blocker_player::data::ask::Ask;
use blocker_player::resources::Profile;
use blocker_quests::resources::Journal;
use blocker_screen::data::notice::Notice;
use blocker_screen::queries::wrap_the_line;
use nalgebra_glm::{Vec4, vec2};
use nightshade::ecs::system_param::EventWriter;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::Input;
use nightshade_random::Random;
use nightshade_ui::commands::builder::UiTreeBuilder;
use nightshade_ui::components::TextOverflow;
use nightshade_ui::components::layout_types::{FlowAlignment, FlowDirection};
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiFrame;
use nightshade_ui_menu::prelude::add_frame;

pub fn build_chat(tree: &mut UiTreeBuilder<'_, '_>, hud: &mut HudBuild<'_>) {
    let body_height = theme::CHAT_LINE * blocker_chat::data::chat::CHAT_ROWS as f32;
    let height = theme::FRAME_CORNER * 2.0
        + theme::CHAT_TAB_HEIGHT
        + theme::CHAT_GAP * 2.0
        + body_height
        + theme::CHAT_ENTRY_HEIGHT;
    let tab_width = (theme::CHAT_WIDTH
        - theme::FRAME_CORNER * 2.0
        - theme::CHAT_TAB_GAP * (CHAT_TABS.len() as f32 - 1.0))
        / CHAT_TABS.len() as f32;
    hud.chat.panel = tree
        .add_node()
        .window(
            Rl(vec2(0.0, 100.0)) + Ab(vec2(theme::CHAT_MARGIN, -theme::CHAT_MARGIN)),
            Ab(vec2(theme::CHAT_WIDTH, height)),
            Anchor::BottomLeft,
        )
        .with_rect(6.0, 1.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_BACKGROUND)
        .flow_with_alignment(
            FlowDirection::Vertical,
            theme::FRAME_CORNER,
            theme::CHAT_GAP,
            FlowAlignment::Start,
            FlowAlignment::Start,
        )
        .entity();
    let frame_layer = hud.art.frame;
    tree.in_parent(hud.chat.panel, |tree| {
        add_frame(tree, hud.style, frame_layer)
    });
    let tabs_row = tree.in_parent(hud.chat.panel, |tree| {
        tree.add_node()
            .fill_width()
            .auto_height()
            .flow(FlowDirection::Horizontal, 0.0, theme::CHAT_TAB_GAP)
            .entity()
    });
    let mut tabs = Vec::new();
    let mut marks = Vec::new();
    for tab in CHAT_TABS.iter() {
        let cell = tree.in_parent(tabs_row, |tree| {
            tree.add_node()
                .size_px(tab_width, theme::CHAT_TAB_HEIGHT)
                .flow(FlowDirection::Vertical, 0.0, 2.0)
                .entity()
        });
        tree.in_parent(cell, |tree| {
            tabs.push(tree.add_button(tab.name));
            marks.push(
                tree.add_node()
                    .size_px(tab_width, 3.0)
                    .with_rect(1.0, 0.0, theme::PANEL_BORDER)
                    .color_raw::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.0))
                    .entity(),
            );
        });
    }
    hud.chat.tabs = tabs;
    hud.chat.marks = marks;
    let lines = tree.in_parent(hud.chat.panel, |tree| {
        tree.add_node()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, body_height)))
            .flow(FlowDirection::Vertical, 0.0, 0.0)
            .entity()
    });
    let mut rows = Vec::new();
    tree.in_parent(lines, |tree| {
        for _ in 0..blocker_chat::data::chat::CHAT_ROWS {
            rows.push(
                tree.add_node()
                    .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, theme::CHAT_LINE)))
                    .with_text("", theme::SMALL_SIZE)
                    .with_text_overflow(TextOverflow::Ellipsis)
                    .text_left()
                    .color_raw::<UiBase>(theme::TEXT_BRIGHT)
                    .entity(),
            );
        }
    });
    hud.chat.rows = rows;
    hud.chat.entry = tree.in_parent(hud.chat.panel, |tree| {
        tree.add_text_input("Press Enter to chat")
    });
}

pub fn open_the_chat(
    actions: Res<Actions<Ask>>,
    input: Res<Input>,
    chat: Res<Chat>,
    chat_panel: Res<ChatPanel>,
    mut entry: ResMut<ChatEntry>,
    mut interaction: ResMut<RetainedUiInteraction>,
) {
    if chat_panel.entry == Entity::default() {
        return;
    }
    let typing = interaction.focused_entity == Some(chat_panel.entry);
    if !typing {
        if actions.just_released(Ask::Chat) {
            entry.scroll = 0;
            interaction.focused_entity = Some(chat_panel.entry);
        }
        return;
    }
    let wheel = input.mouse.wheel_delta.y;
    if wheel.abs() > 0.01 {
        let reach = chat
            .lines
            .len()
            .saturating_sub(blocker_chat::data::chat::CHAT_ROWS);
        entry.scroll = if wheel > 0.0 {
            (entry.scroll + 1).min(reach)
        } else {
            entry.scroll.saturating_sub(1)
        };
    }
}

pub fn send_the_line(
    frame: Res<RetainedUiFrame>,
    chat_panel: Res<ChatPanel>,
    profile: Res<Profile>,
    roster: Res<Roster>,
    mut chat: ResMut<Chat>,
    mut chatter: ResMut<Chatter>,
    mut random: ResMut<Random>,
    mut journal: ResMut<Journal>,
    mut entry: ResMut<ChatEntry>,
    mut notices: EventWriter<Notice>,
    mut commands: Commands,
    mut fields: Query<(&mut UiTextInputData,)>,
) {
    let field = chat_panel.entry;
    if field == Entity::default() {
        return;
    }
    let submitted = ui_events(&frame).iter().any(
        |event| matches!(event, UiEvent::TextInputSubmitted { entity, .. } if *entity == field),
    );
    if !submitted {
        return;
    }
    let mut said = String::new();
    fields.for_entity(field, |(data,)| {
        said = data.text.trim().to_string();
        data.text.clear();
        data.cursor_position = 0;
    });
    ui_set_text(&mut commands, field, "");
    entry.scroll = 0;
    if said.is_empty() {
        return;
    }
    npc_say(&mut chat, &profile.name, &said);
    respond_to_keywords(
        &mut chat,
        &mut chatter,
        &roster,
        &mut random,
        &mut journal,
        &mut notices,
        &said,
        &profile.name,
    );
}

pub fn choose_chat_tab(
    mut entry: ResMut<ChatEntry>,
    chat_panel: Res<ChatPanel>,
    mut buttons: Query<(&UiButtonData,)>,
) {
    for index in 0..chat_panel.tabs.len() {
        if ui_clicked(&mut buttons, chat_panel.tabs[index]) {
            entry.active_tab = index;
            entry.scroll = 0;
        }
    }
}

fn unread_tabs(chat: &Chat, entry: &ChatEntry) -> Vec<bool> {
    CHAT_TABS
        .iter()
        .enumerate()
        .map(|(index, tab)| {
            index != entry.active_tab
                && tab
                    .channels
                    .iter()
                    .any(|channel| chat.counts[channel.index()] > entry.seen[channel.index()])
        })
        .collect()
}

pub fn show_the_chat(
    chat: Res<Chat>,
    mut entry: ResMut<ChatEntry>,
    mut chat_panel: ResMut<ChatPanel>,
    mut commands: Commands,
) {
    if chat_panel.tabs.len() < CHAT_TABS.len() || chat_panel.marks.len() < CHAT_TABS.len() {
        return;
    }
    let active = entry.active_tab.min(CHAT_TABS.len() - 1);
    let key = (chat.counts.iter().sum::<u64>(), active, entry.scroll);
    if chat_panel.shown_key == Some(key) {
        return;
    }
    chat_panel.shown_key = Some(key);
    let tab = &CHAT_TABS[active];
    let filtered: Vec<(Channel, &str)> = chat
        .lines
        .iter()
        .filter(|(channel, _)| tab.shows(*channel))
        .map(|(channel, text)| (*channel, text.as_str()))
        .collect();
    let mut chosen: Vec<(Channel, String)> = Vec::new();
    let mut used = 0usize;
    for (channel, text) in filtered.iter().rev().skip(entry.scroll) {
        let wrapped = wrap_the_line(text, theme::CHAT_CHARACTERS);
        if !chosen.is_empty() && used + wrapped.len() > blocker_chat::data::chat::CHAT_ROWS {
            break;
        }
        used += wrapped.len();
        for row in wrapped.into_iter().rev() {
            chosen.push((*channel, row));
        }
    }
    chosen.reverse();
    let overflow = chosen
        .len()
        .saturating_sub(blocker_chat::data::chat::CHAT_ROWS);
    chosen.drain(..overflow);
    let body = chosen
        .iter()
        .map(|(channel, row)| format!("{}{row}", channel.index()))
        .collect::<Vec<_>>()
        .join("\u{1f}");
    let tab_changed = chat_panel.shown_tab != Some(active);
    if tab_changed || chat_panel.shown_body != body {
        chat_panel.shown_body = body;
        let start = chat_panel.rows.len().saturating_sub(chosen.len());
        for (index, row) in chat_panel.rows.iter().enumerate() {
            match index.checked_sub(start).and_then(|slot| chosen.get(slot)) {
                Some((channel, text)) => {
                    ui_set_text(&mut commands, *row, text);
                    ui_set_color::<UiBase>(&mut commands, *row, channel_color(*channel));
                }
                None => ui_set_text(&mut commands, *row, ""),
            }
        }
    }
    let unread = unread_tabs(&chat, &entry);
    if tab_changed || chat_panel.shown_unread != unread {
        for index in 0..chat_panel.marks.len() {
            let color = if index == active {
                theme::ACCENT_COLOR
            } else if unread.get(index).copied().unwrap_or(false) {
                theme::CHAT_UNREAD_COLOR
            } else {
                Vec4::new(0.0, 0.0, 0.0, 0.0)
            };
            ui_set_color::<UiBase>(&mut commands, chat_panel.marks[index], color);
        }
        chat_panel.shown_unread = unread;
    }
    chat_panel.shown_tab = Some(active);
    for channel in tab.channels {
        entry.seen[channel.index()] = chat.counts[channel.index()];
    }
}
