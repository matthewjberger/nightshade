use crate::resources::CraftRowHandles;
use crate::resources::CraftingPanel;
use crate::theme;
use blocker_blocks::data::items::RECIPES;
use blocker_hud::resources::ArtLayers;
use blocker_hud::resources::ShellHandles;
use blocker_hud::systems::widgets::item_icon;
use blocker_pocket::commands::craft;
use blocker_pocket::queries::{can_craft, inventory_count};
use blocker_pocket::resources::Inventory;
use blocker_screen::data::notice::Notice;
use blocker_screen::data::screen::Screen;
use nightshade::ecs::system_param::EventWriter;
use nightshade::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui::commands::builder::{UiBuild, UiTreeBuilder};
use nightshade_ui::components::layout_types::FlowDirection;
use nightshade_ui::prelude::*;
use nightshade_ui_menu::prelude::{
    CardStyle, add_action_button, add_card, add_cell_text, add_footer_bar, add_row_strip,
    add_scrolling_body, add_section_label, add_title_bar,
};
use nightshade_ui_menu::prelude::{Menus, menu_show_on};

pub fn build_crafting(
    style: Res<CardStyle>,
    art: Res<ArtLayers>,
    shell: Res<ShellHandles>,
    mut menus: ResMut<Menus<Screen>>,
    mut crafting_panel: ResMut<CraftingPanel>,
    mut build: UiBuild,
) {
    let root = shell.root;
    let mut built_tree = UiTreeBuilder::from_parent(build.context(), root);
    let tree = &mut built_tree;
    crafting_panel.panel = add_card(
        tree,
        &style,
        art.frame,
        blocker_hud::theme::MENU_WIDTH,
        theme::CRAFTING_HEIGHT,
        blocker_hud::theme::PANEL_COLOR,
    );
    let panel = crafting_panel.panel;
    tree.in_parent(panel, |tree| {
        add_title_bar(tree, &style, blocker_hud::theme::MENU_WIDTH, "Crafting");
        let body = add_scrolling_body(
            tree,
            theme::CRAFTING_HEIGHT
                - blocker_hud::theme::TITLE_HEIGHT
                - blocker_hud::theme::FOOTER_HEIGHT,
        );
        tree.in_parent(body, |tree| {
            add_section_label(
                tree,
                &style,
                "Green means the parts are in your pocket",
                blocker_hud::theme::ROW_WIDTH,
            );
            let mut rows = Vec::new();
            for (index, recipe) in RECIPES.iter().enumerate() {
                let strip = add_row_strip(tree, &style, index, blocker_hud::theme::ROW_WIDTH);
                let handles = tree.in_parent(strip, |tree| {
                    item_icon(tree, &art, recipe.output, theme::ROW_ICON);
                    add_cell_text(
                        tree,
                        &style,
                        &format!("{} x{}", recipe.output.name(), recipe.count),
                        theme::NAME_WIDTH,
                        blocker_hud::theme::SMALL_SIZE,
                        blocker_hud::theme::TEXT_COLOR,
                    );
                    let needs = add_cell_text(
                        tree,
                        &style,
                        "",
                        theme::NEEDS_WIDTH,
                        blocker_hud::theme::SMALL_SIZE,
                        blocker_hud::theme::DIM_TEXT_COLOR,
                    );
                    let button = add_action_button(tree, &style, "Craft");
                    CraftRowHandles {
                        index,
                        needs,
                        button,
                    }
                });
                rows.push(handles);
            }
            crafting_panel.rows = rows;
        });
        let footer = add_footer_bar(tree, &style, blocker_hud::theme::MENU_WIDTH);
        tree.in_parent(footer, |tree| {
            let holder = tree
                .add_node()
                .size_px(
                    blocker_hud::theme::MENU_WIDTH - 28.0,
                    blocker_hud::theme::ACTION_HEIGHT,
                )
                .flow(FlowDirection::Horizontal, 0.0, 0.0)
                .entity();
            crafting_panel.close = tree.in_parent(holder, |tree| tree.add_button("Close"));
        });
    });
    menu_show_on(&mut menus, Screen::Crafting, crafting_panel.panel);
    built_tree.finish();
}

pub(crate) fn refresh_crafting(
    inventory: &Inventory,
    crafting_panel: &mut CraftingPanel,
    commands: &mut Commands,
) {
    if crafting_panel.shown.len() != crafting_panel.rows.len() {
        crafting_panel.shown = vec![false; crafting_panel.rows.len()];
    }
    for place in 0..crafting_panel.rows.len() {
        let row = crafting_panel.rows[place];
        let recipe = &RECIPES[row.index];
        let ready = can_craft(inventory, recipe);
        let parts: Vec<String> = recipe
            .inputs
            .iter()
            .map(|(item, count)| {
                format!(
                    "{count} {} ({})",
                    item.name(),
                    inventory_count(inventory, *item)
                )
            })
            .collect();
        let text = parts.join(", ");
        if crafting_panel.shown[place] == ready && !text.is_empty() {
            continue;
        }
        crafting_panel.shown[place] = ready;
        ui_set_text(commands, row.needs, &text);
        let color = if ready {
            theme::READY_COLOR
        } else {
            blocker_hud::theme::DIM_TEXT_COLOR
        };
        ui_set_color::<UiBase>(commands, row.needs, color);
    }
}

pub fn crafting_buttons(
    crafting_panel: Res<CraftingPanel>,
    mut inventory: ResMut<Inventory>,
    mut notices: EventWriter<Notice>,
    mut next: ResMut<NextState<Screen>>,
    mut buttons: Query<(&UiButtonData,)>,
) {
    if ui_clicked(&mut buttons, crafting_panel.close) {
        next.pending = Some(Screen::Playing);
        return;
    }
    for handle in &crafting_panel.rows {
        if !ui_clicked(&mut buttons, handle.button) {
            continue;
        }
        let recipe = &RECIPES[handle.index];
        let message = if craft(&mut inventory, recipe) {
            format!(
                "Made {} {}  (have {})",
                recipe.count,
                recipe.output.name(),
                inventory_count(&inventory, recipe.output)
            )
        } else {
            "Missing the parts, or no room".to_string()
        };
        notices.send(Notice(message));
    }
}
