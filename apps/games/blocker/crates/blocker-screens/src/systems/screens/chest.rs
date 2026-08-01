use crate::commands::leave_the_chest;
use crate::resources::ChestPanel;
use crate::theme;
use blocker_blocks::data::items::ItemStack;
use blocker_hud::commands::paint_slot;
use blocker_hud::resources::ArtLayers;
use blocker_hud::resources::ShellHandles;
use blocker_pocket::commands::{clear_slot, inventory_add, put_stack, stack_into};
use blocker_pocket::queries::stack_at;
use blocker_pocket::resources::{Chests, Inventory, OpenChest};
use blocker_screen::data::notice::Notice;
use blocker_screen::data::screen::Screen;
use nightshade::ecs::system_param::EventWriter;
use nightshade::prelude::*;
use nightshade_inventory::Slots;
use nightshade_state::prelude::*;
use nightshade_ui::commands::builder::{UiBuild, UiTreeBuilder};
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;
use nightshade_ui_menu::prelude::{
    CardStyle, add_body_column, add_card, add_footer_bar, add_section_label, add_slot_row,
    add_title_bar,
};
use nightshade_ui_menu::prelude::{Menus, menu_show_on};

pub fn build_chest(
    style: Res<CardStyle>,
    art: Res<ArtLayers>,
    shell: Res<ShellHandles>,
    mut menus: ResMut<Menus<Screen>>,
    mut chest_panel: ResMut<ChestPanel>,
    mut build: UiBuild,
) {
    let root = shell.root;
    let mut built_tree = UiTreeBuilder::from_parent(build.context(), root);
    let tree = &mut built_tree;
    chest_panel.panel = add_card(
        tree,
        &style,
        art.frame,
        blocker_hud::theme::MENU_WIDTH,
        theme::CHEST_HEIGHT,
        blocker_hud::theme::PANEL_COLOR,
    );
    let panel = chest_panel.panel;
    tree.in_parent(panel, |tree| {
        add_title_bar(tree, &style, blocker_hud::theme::MENU_WIDTH, "Chest");
        let body = add_body_column(
            tree,
            &style,
            blocker_hud::theme::MENU_WIDTH,
            theme::CHEST_HEIGHT
                - blocker_hud::theme::TITLE_HEIGHT
                - blocker_hud::theme::FOOTER_HEIGHT,
        );
        tree.in_parent(body, |tree| {
            add_section_label(
                tree,
                &style,
                "Click a chest slot to take it, a pocket slot to store it",
                blocker_hud::theme::ROW_WIDTH,
            );
            let mut chest = Vec::new();
            for _ in 0..(blocker_world::data::world::CHEST_SLOTS / theme::CHEST_ROW) {
                chest.extend(add_slot_row(
                    tree,
                    &style,
                    blocker_hud::theme::MENU_WIDTH - blocker_hud::theme::BODY_PADDING * 2.0,
                    theme::CHEST_ROW,
                ));
            }
            chest_panel.slots = chest;
            add_section_label(tree, &style, "Pocket", blocker_hud::theme::ROW_WIDTH);
            let mut pocket = Vec::new();
            for _ in 0..blocker_blocks::data::items::INVENTORY_ROWS {
                pocket.extend(add_slot_row(
                    tree,
                    &style,
                    blocker_hud::theme::MENU_WIDTH - blocker_hud::theme::BODY_PADDING * 2.0,
                    blocker_blocks::data::items::HOTBAR_SLOTS,
                ));
            }
            chest_panel.pocket = pocket;
        });
        let footer = add_footer_bar(tree, &style, blocker_hud::theme::MENU_WIDTH);
        tree.in_parent(footer, |tree| {
            chest_panel.close = tree.add_button("Close");
        });
    });
    menu_show_on(&mut menus, Screen::Chest, chest_panel.panel);
    built_tree.finish();
}

pub(crate) fn refresh_chest(
    inventory: &Inventory,
    chests: &Chests,
    open: &OpenChest,
    layers: &ArtLayers,
    chest_panel: &mut ChestPanel,
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
) {
    let empty = Slots::default();
    let held = open.0.and_then(|key| chests.0.get(&key)).unwrap_or(&empty);
    if chest_panel.shown.len() != chest_panel.slots.len() {
        chest_panel.shown = vec![None; chest_panel.slots.len()];
    }
    for index in 0..chest_panel.slots.len() {
        let cell = chest_panel.slots[index];
        let stack = stack_at(held, index);
        if chest_panel.shown[index] == stack {
            continue;
        }
        let was = chest_panel.shown[index];
        chest_panel.shown[index] = stack;
        paint_slot(layers, &cell, stack, was, dirty, commands);
    }
    if chest_panel.shown_pocket.len() != chest_panel.pocket.len() {
        chest_panel.shown_pocket = vec![None; chest_panel.pocket.len()];
    }
    for index in 0..chest_panel.pocket.len() {
        let cell = chest_panel.pocket[index];
        let stack = stack_at(&inventory.slots, index);
        if chest_panel.shown_pocket[index] == stack {
            continue;
        }
        let was = chest_panel.shown_pocket[index];
        chest_panel.shown_pocket[index] = stack;
        paint_slot(layers, &cell, stack, was, dirty, commands);
    }
}

pub fn chest_buttons(
    mut chest_panel: ResMut<ChestPanel>,
    mut chests: ResMut<Chests>,
    mut open: ResMut<OpenChest>,
    mut inventory: ResMut<Inventory>,
    mut notices: EventWriter<Notice>,
    mut next: ResMut<NextState<Screen>>,
    mut buttons: Query<(&UiButtonData,)>,
) {
    if ui_clicked(&mut buttons, chest_panel.close) {
        leave_the_chest(&mut open, &mut next);
        return;
    }
    let Some(key) = open.0 else {
        return;
    };
    let from_chest = chest_panel
        .slots
        .iter()
        .position(|cell| ui_clicked(&mut buttons, cell.frame));
    if let Some(index) = from_chest {
        let Some(slots) = chests.0.get_mut(&key) else {
            return;
        };
        let Some(stack) = stack_at(slots, index) else {
            return;
        };
        clear_slot(slots, index);
        let left = inventory_add(&mut inventory, stack.item, stack.count);
        if left > 0 {
            put_stack(
                slots,
                index,
                Some(ItemStack {
                    item: stack.item,
                    count: left,
                }),
            );
            notices.send(Notice("No room in the pocket".into()));
        }
        chest_panel.shown.clear();
        chest_panel.shown_pocket.clear();
        return;
    }
    let from_pocket = chest_panel
        .pocket
        .iter()
        .position(|cell| ui_clicked(&mut buttons, cell.frame));
    let Some(index) = from_pocket else {
        return;
    };
    let Some(stack) = stack_at(&inventory.slots, index) else {
        return;
    };
    clear_slot(&mut inventory.slots, index);
    let Some(slots) = chests.0.get_mut(&key) else {
        return;
    };
    let left = stack_into(slots, stack.item, stack.count);
    if left > 0 {
        put_stack(
            &mut inventory.slots,
            index,
            Some(ItemStack {
                item: stack.item,
                count: left,
            }),
        );
        notices.send(Notice("The chest is full".into()));
    }
    chest_panel.shown.clear();
    chest_panel.shown_pocket.clear();
}
