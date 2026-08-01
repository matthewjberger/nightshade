use crate::resources::PocketPanel;
use crate::theme;
use blocker_hud::commands::{paint_slot, set_border};
use blocker_hud::resources::ArtLayers;
use blocker_hud::resources::ShellHandles;
use blocker_pocket::commands::{select_slot, swap_slots};
use blocker_pocket::queries::{selected_stack, stack_at};
use blocker_pocket::resources::Inventory;
use blocker_screen::data::screen::Screen;
use nightshade::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui::commands::builder::{UiBuild, UiTreeBuilder};
use nightshade_ui::components::layout_types::FlowDirection;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;
use nightshade_ui_menu::prelude::{
    CardStyle, add_body_column, add_card, add_footer_bar, add_section_label, add_slot_row,
    add_title_bar,
};
use nightshade_ui_menu::prelude::{Menus, menu_show_on};

pub fn build_pocket(
    style: Res<CardStyle>,
    art: Res<ArtLayers>,
    shell: Res<ShellHandles>,
    mut menus: ResMut<Menus<Screen>>,
    mut pocket_panel: ResMut<PocketPanel>,
    mut build: UiBuild,
) {
    let root = shell.root;
    let mut built_tree = UiTreeBuilder::from_parent(build.context(), root);
    let tree = &mut built_tree;
    pocket_panel.panel = add_card(
        tree,
        &style,
        art.frame,
        blocker_hud::theme::MENU_WIDTH,
        theme::POCKET_HEIGHT,
        blocker_hud::theme::PANEL_COLOR,
    );
    let panel = pocket_panel.panel;
    tree.in_parent(panel, |tree| {
        add_title_bar(tree, &style, blocker_hud::theme::MENU_WIDTH, "Pocket");
        let body = add_body_column(
            tree,
            &style,
            blocker_hud::theme::MENU_WIDTH,
            theme::POCKET_HEIGHT
                - blocker_hud::theme::TITLE_HEIGHT
                - blocker_hud::theme::FOOTER_HEIGHT,
        );
        tree.in_parent(body, |tree| {
            add_section_label(
                tree,
                &style,
                "Click a slot, then another, to swap them",
                blocker_hud::theme::ROW_WIDTH,
            );
            let mut cells = Vec::new();
            for _ in 0..blocker_blocks::data::items::INVENTORY_ROWS {
                cells.extend(add_slot_row(
                    tree,
                    &style,
                    blocker_hud::theme::MENU_WIDTH - blocker_hud::theme::BODY_PADDING * 2.0,
                    blocker_blocks::data::items::HOTBAR_SLOTS,
                ));
            }
            pocket_panel.slots = cells;
            pocket_panel.detail = tree
                .add_node()
                .size_px(
                    blocker_hud::theme::MENU_WIDTH - blocker_hud::theme::BODY_PADDING * 2.0,
                    34.0,
                )
                .with_text("", blocker_hud::theme::BODY_SIZE)
                .text_center()
                .color_raw::<UiBase>(blocker_hud::theme::TEXT_COLOR)
                .entity();
            pocket_panel.hint = tree
                .add_node()
                .size_px(
                    blocker_hud::theme::MENU_WIDTH - blocker_hud::theme::BODY_PADDING * 2.0,
                    24.0,
                )
                .with_text(
                    "1 to 9 picks a slot for the hand.  I or Escape closes.",
                    blocker_hud::theme::SMALL_SIZE,
                )
                .text_center()
                .color_raw::<UiBase>(blocker_hud::theme::DIM_TEXT_COLOR)
                .entity();
        });
        let footer = add_footer_bar(tree, &style, blocker_hud::theme::MENU_WIDTH);
        tree.in_parent(footer, |tree| {
            let width = (blocker_hud::theme::MENU_WIDTH - 38.0) * 0.5;
            let left = tree
                .add_node()
                .size_px(width, blocker_hud::theme::ACTION_HEIGHT)
                .flow(FlowDirection::Horizontal, 0.0, 0.0)
                .entity();
            pocket_panel.craft = tree.in_parent(left, |tree| tree.add_button("Craft by hand"));
            let right = tree
                .add_node()
                .size_px(width, blocker_hud::theme::ACTION_HEIGHT)
                .flow(FlowDirection::Horizontal, 0.0, 0.0)
                .entity();
            pocket_panel.close = tree.in_parent(right, |tree| tree.add_button("Close"));
        });
    });
    menu_show_on(&mut menus, Screen::Inventory, pocket_panel.panel);
    built_tree.finish();
}

pub(crate) fn refresh_pocket(
    inventory: &Inventory,
    layers: &ArtLayers,
    pocket_panel: &mut PocketPanel,
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
) {
    if pocket_panel.shown.len() != pocket_panel.slots.len() {
        pocket_panel.shown = vec![None; pocket_panel.slots.len()];
    }
    for index in 0..pocket_panel.slots.len() {
        let cell = pocket_panel.slots[index];
        let stack = stack_at(&inventory.slots, index);
        if pocket_panel.shown[index] == stack {
            continue;
        }
        let was = pocket_panel.shown[index];
        pocket_panel.shown[index] = stack;
        paint_slot(layers, &cell, stack, was, dirty, commands);
    }

    let marks = (pocket_panel.holding, inventory.selected.0);
    if pocket_panel.shown_marks != Some(marks) {
        pocket_panel.shown_marks = Some(marks);
        for (index, cell) in pocket_panel.slots.iter().enumerate() {
            let (color, border) = if Some(index) == marks.0 {
                (theme::HELD_COLOR, theme::HELD_BORDER)
            } else if index == marks.1 {
                (
                    blocker_hud::theme::SLOT_SELECTED_COLOR,
                    blocker_hud::theme::SLOT_SELECTED_BORDER,
                )
            } else {
                (
                    blocker_hud::theme::CELL_COLOR,
                    blocker_hud::theme::CELL_BORDER,
                )
            };
            ui_set_color::<UiBase>(commands, cell.frame, color);
            set_border(commands, cell.frame, border);
        }
    }

    let detail = match pocket_panel.holding {
        Some(index) => match stack_at(&inventory.slots, index) {
            Some(stack) => format!("Holding {} x{}", stack.item.name(), stack.count),
            None => "Holding an empty slot".to_string(),
        },
        None => match selected_stack(inventory) {
            Some(stack) => format!("{} x{}", stack.item.name(), stack.count),
            None => "Empty hand".to_string(),
        },
    };
    if pocket_panel.shown_detail != detail {
        ui_set_text(commands, pocket_panel.detail, &detail);
        pocket_panel.shown_detail = detail;
    }
}

pub fn pocket_buttons(
    mut pocket_panel: ResMut<PocketPanel>,
    mut inventory: ResMut<Inventory>,
    mut next: ResMut<NextState<Screen>>,
    mut buttons: Query<(&UiButtonData,)>,
) {
    if ui_clicked(&mut buttons, pocket_panel.craft) {
        next.pending = Some(Screen::Crafting);
        return;
    }
    if ui_clicked(&mut buttons, pocket_panel.close) {
        next.pending = Some(Screen::Playing);
        return;
    }
    let picked = pocket_panel
        .slots
        .iter()
        .position(|cell| ui_clicked(&mut buttons, cell.frame));
    let Some(index) = picked else {
        return;
    };
    let holding = pocket_panel.holding;
    match holding {
        Some(first) if first == index => {
            pocket_panel.holding = None;
        }
        Some(first) => {
            swap_slots(&mut inventory, first, index);
            pocket_panel.holding = None;
            pocket_panel.shown.clear();
            pocket_panel.shown_marks = None;
        }
        None => {
            select_slot(&mut inventory, index);
            pocket_panel.holding = Some(index);
            pocket_panel.shown_marks = None;
        }
    }
}
