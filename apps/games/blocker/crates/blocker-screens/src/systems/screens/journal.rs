use crate::resources::JournalPanel;
use crate::resources::Screens;
use crate::theme;
use blocker_hud::resources::ArtLayers;
use blocker_hud::resources::ShellHandles;
use blocker_pocket::resources::Inventory;
use blocker_quests::data::quests::{Objective, QUESTS};
use blocker_quests::data::spots::PLACES;
use blocker_quests::data::trade::GIVERS;
use blocker_quests::queries::objective_progress;
use blocker_quests::resources::Journal;
use blocker_screen::data::screen::Screen;
use nalgebra_glm::vec2;
use nightshade::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui::commands::builder::{UiBuild, UiTreeBuilder};
use nightshade_ui::components::TextOverflow;
use nightshade_ui::components::layout_types::FlowDirection;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;
use nightshade_ui_menu::prelude::{
    CardStyle, add_card, add_cell_text, add_footer_buttons, add_title_bar,
};
use nightshade_ui_menu::prelude::{Menus, menu_show_on};

struct Entry {
    quest: usize,
    progress: String,
    ready: bool,
    done: bool,
}

pub fn build_journal(
    style: Res<CardStyle>,
    art: Res<ArtLayers>,
    shell: Res<ShellHandles>,
    mut menus: ResMut<Menus<Screen>>,
    mut journal_panel: ResMut<JournalPanel>,
    mut build: UiBuild,
) {
    let root = shell.root;
    let mut built_tree = UiTreeBuilder::from_parent(build.context(), root);
    let tree = &mut built_tree;
    journal_panel.panel = add_card(
        tree,
        &style,
        art.frame,
        blocker_hud::theme::MENU_WIDTH,
        theme::JOURNAL_HEIGHT,
        blocker_hud::theme::PANEL_COLOR,
    );
    let column = theme::JOURNAL_HEIGHT
        - blocker_hud::theme::TITLE_HEIGHT
        - blocker_hud::theme::FOOTER_HEIGHT;
    tree.in_parent(journal_panel.panel, |tree| {
        add_title_bar(
            tree,
            &style,
            blocker_hud::theme::MENU_WIDTH,
            "Quest journal",
        );
        let body = tree
            .add_node()
            .size_px(blocker_hud::theme::MENU_WIDTH, column)
            .flow(
                FlowDirection::Horizontal,
                blocker_hud::theme::BODY_PADDING,
                10.0,
            )
            .entity();
        tree.in_parent(body, |tree| {
            let list = tree
                .add_node()
                .size_px(theme::JOURNAL_LIST_WIDTH, column - 20.0)
                .flow(FlowDirection::Vertical, 0.0, blocker_hud::theme::ROW_GAP)
                .entity();
            tree.in_parent(list, |tree| {
                journal_panel.rows.clear();
                for _ in 0..theme::JOURNAL_ROWS {
                    let holder = tree
                        .add_node()
                        .size_px(
                            theme::JOURNAL_LIST_WIDTH,
                            blocker_hud::theme::ACTION_HEIGHT + 6.0,
                        )
                        .flow(FlowDirection::Horizontal, 0.0, 0.0)
                        .with_visible(false)
                        .entity();
                    let button = tree.in_parent(holder, |tree| tree.add_button(""));
                    journal_panel.rows.push((holder, button));
                }
            });
            let detail = tree
                .add_node()
                .size_px(theme::JOURNAL_DETAIL_WIDTH, column - 20.0)
                .flow(FlowDirection::Vertical, 8.0, 6.0)
                .entity();
            tree.in_parent(detail, |tree| {
                journal_panel.detail_title = add_cell_text(
                    tree,
                    &style,
                    "",
                    theme::JOURNAL_DETAIL_WIDTH - 20.0,
                    blocker_hud::theme::HEADING_SIZE,
                    blocker_hud::theme::ACCENT_COLOR,
                );
                journal_panel.detail_giver = add_cell_text(
                    tree,
                    &style,
                    "",
                    theme::JOURNAL_DETAIL_WIDTH - 20.0,
                    blocker_hud::theme::SMALL_SIZE,
                    blocker_hud::theme::DIM_TEXT_COLOR,
                );
                journal_panel.detail_story = tree
                    .add_node()
                    .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 132.0)))
                    .with_text("", blocker_hud::theme::BODY_SIZE)
                    .with_text_overflow(TextOverflow::Wrap)
                    .text_left()
                    .color_raw::<UiBase>(blocker_hud::theme::TEXT_BRIGHT)
                    .entity();
                journal_panel.detail_next = tree
                    .add_node()
                    .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 92.0)))
                    .with_text("", blocker_hud::theme::BODY_SIZE)
                    .with_text_overflow(TextOverflow::Wrap)
                    .text_left()
                    .color_raw::<UiBase>(blocker_hud::theme::TEXT_COLOR)
                    .entity();
                journal_panel.detail_objective = add_cell_text(
                    tree,
                    &style,
                    "",
                    theme::JOURNAL_DETAIL_WIDTH - 20.0,
                    blocker_hud::theme::BODY_SIZE,
                    theme::READY_COLOR,
                );
                journal_panel.detail_reward = add_cell_text(
                    tree,
                    &style,
                    "",
                    theme::JOURNAL_DETAIL_WIDTH - 20.0,
                    blocker_hud::theme::SMALL_SIZE,
                    blocker_hud::theme::DIM_TEXT_COLOR,
                );
            });
        });
        let buttons = add_footer_buttons(tree, &style, blocker_hud::theme::MENU_WIDTH, &["Close"]);
        journal_panel.close = buttons[0];
    });
    menu_show_on(&mut menus, Screen::Journal, journal_panel.panel);
    built_tree.finish();
}

fn measure(quest: usize, journal: &Journal, inventory: &Inventory) -> (String, bool) {
    let objective = &QUESTS[quest].objective;
    let (have, want) = objective_progress(objective, inventory, journal);
    let words = match objective {
        Objective::Gather { item, .. } => format!("{} {have} of {want}", item.name()),
        Objective::Explore { .. } => format!("Places found {have} of {want}"),
        Objective::Slay { beast, .. } => format!(
            "{} put down {have} of {want}",
            blocker_beasts::data::beasts::BEASTS[*beast].name
        ),
        Objective::Meet { giver } => format!("Speak with {}", GIVERS[*giver].name),
        Objective::Visit { place } => format!("Go to {}", PLACES[*place].name),
        Objective::Report => "Report what you found".to_string(),
    };
    (words, have >= want)
}

pub fn show_the_journal(
    journal: Res<Journal>,
    inventory: Res<Inventory>,
    mut journal_panel: ResMut<JournalPanel>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    if journal_panel.rows.is_empty() {
        return;
    }
    let mut entries: Vec<Entry> = Vec::new();
    for quest in &journal.active {
        let (progress, ready) = measure(*quest, &journal, &inventory);
        entries.push(Entry {
            quest: *quest,
            progress,
            ready,
            done: false,
        });
    }
    for quest in &journal.finished {
        entries.push(Entry {
            quest: *quest,
            progress: "Done".to_string(),
            ready: true,
            done: true,
        });
    }
    if journal_panel.picked >= entries.len() {
        journal_panel.picked = 0;
    }
    let key = (
        entries
            .iter()
            .map(|entry| (entry.quest, entry.progress.clone(), entry.ready))
            .collect::<Vec<_>>(),
        journal_panel.picked,
    );
    if journal_panel.shown.as_ref() == Some(&key) {
        return;
    }
    journal_panel.shown = Some(key);
    for (index, (holder, button)) in journal_panel.rows.iter().enumerate() {
        match entries.get(index) {
            Some(entry) => {
                ui_set_visible(&mut dirty, &mut commands, *holder, true);
                let mark = if index == journal_panel.picked {
                    ">"
                } else {
                    " "
                };
                ui_set_text(
                    &mut commands,
                    *button,
                    &format!("{mark} {}", QUESTS[entry.quest].title),
                );
            }
            None => ui_set_visible(&mut dirty, &mut commands, *holder, false),
        }
    }
    let Some(entry) = entries.get(journal_panel.picked) else {
        ui_set_text(&mut commands, journal_panel.detail_title, "No quests yet");
        ui_set_text(
            &mut commands,
            journal_panel.detail_giver,
            "Look for folk with a mark over their head.",
        );
        ui_set_text(&mut commands, journal_panel.detail_story, "");
        ui_set_text(&mut commands, journal_panel.detail_next, "");
        ui_set_text(&mut commands, journal_panel.detail_objective, "");
        ui_set_text(&mut commands, journal_panel.detail_reward, "");
        return;
    };
    let quest = &QUESTS[entry.quest];
    let giver = &GIVERS[quest.giver];
    ui_set_text(&mut commands, journal_panel.detail_title, quest.title);
    ui_set_text(
        &mut commands,
        journal_panel.detail_giver,
        &format!("Given by {} the {}", giver.name, giver.title),
    );
    ui_set_text(&mut commands, journal_panel.detail_story, quest.story);
    let next = if entry.done {
        "Finished. Nothing more to do here.".to_string()
    } else if entry.ready {
        format!("Go back to {} and hand it in.", giver.name)
    } else {
        quest.nudge.to_string()
    };
    ui_set_text(&mut commands, journal_panel.detail_next, &next);
    ui_set_text(
        &mut commands,
        journal_panel.detail_objective,
        &entry.progress,
    );
    ui_set_color::<UiBase>(
        &mut commands,
        journal_panel.detail_objective,
        if entry.ready {
            theme::READY_COLOR
        } else {
            blocker_hud::theme::ACCENT_COLOR
        },
    );
    ui_set_text(
        &mut commands,
        journal_panel.detail_reward,
        &format!(
            "Pays {} x{} and {} renown",
            quest.reward.name(),
            quest.reward_count,
            quest.renown
        ),
    );
}

pub fn journal_buttons(
    screens: Res<Screens>,
    mut journal_panel: ResMut<JournalPanel>,
    mut next: ResMut<NextState<Screen>>,
    mut buttons: Query<(&UiButtonData,)>,
) {
    if ui_clicked(&mut buttons, journal_panel.close) {
        next.pending = Some(screens.return_to);
        return;
    }
    let picked = journal_panel
        .rows
        .iter()
        .position(|(_, button)| ui_clicked(&mut buttons, *button));
    if let Some(index) = picked {
        journal_panel.picked = index;
        journal_panel.shown = None;
    }
}
