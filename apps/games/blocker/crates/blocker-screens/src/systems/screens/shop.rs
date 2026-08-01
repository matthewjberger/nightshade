use crate::resources::ShopPanel;
use crate::theme;
use blocker_blocks::queries::item_value;
use blocker_hud::resources::ArtLayers;
use blocker_hud::resources::ShellHandles;
use blocker_hud::systems::widgets::item_icon;
use blocker_pocket::queries::selected_stack;
use blocker_pocket::resources::Inventory;
use blocker_quests::commands::{buy_good, sell_selected};
use blocker_quests::data::trade::{STOCK, TRADERS};
use blocker_quests::resources::Journal;
use blocker_screen::data::notice::Notice;
use blocker_screen::data::screen::Screen;
use blocker_talk::resources::Shopping;
use nightshade::ecs::system_param::EventWriter;
use nightshade::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui::commands::builder::{UiBuild, UiTreeBuilder};
use nightshade_ui::prelude::*;
use nightshade_ui_menu::prelude::{
    CardStyle, add_action_button, add_card, add_cell_text, add_footer_buttons, add_row_strip,
    add_scrolling_body, add_section_label, add_title_bar,
};
use nightshade_ui_menu::prelude::{Menus, menu_show_on};

pub fn build_shop(
    style: Res<CardStyle>,
    art: Res<ArtLayers>,
    shell: Res<ShellHandles>,
    mut menus: ResMut<Menus<Screen>>,
    mut shop_panel: ResMut<ShopPanel>,
    mut build: UiBuild,
) {
    let root = shell.root;
    let mut built_tree = UiTreeBuilder::from_parent(build.context(), root);
    let tree = &mut built_tree;
    shop_panel.panel = add_card(
        tree,
        &style,
        art.frame,
        blocker_hud::theme::MENU_WIDTH,
        theme::SHOP_HEIGHT,
        blocker_hud::theme::PANEL_COLOR,
    );
    let panel = shop_panel.panel;
    tree.in_parent(panel, |tree| {
        add_title_bar(tree, &style, blocker_hud::theme::MENU_WIDTH, "Trader");
        shop_panel.coin = tree
            .add_node()
            .size_px(blocker_hud::theme::MENU_WIDTH, 26.0)
            .with_text("", blocker_hud::theme::SMALL_SIZE)
            .text_center()
            .color_raw::<UiBase>(blocker_hud::theme::ACCENT_COLOR)
            .entity();
        let body = add_scrolling_body(
            tree,
            theme::SHOP_HEIGHT
                - blocker_hud::theme::TITLE_HEIGHT
                - blocker_hud::theme::FOOTER_HEIGHT
                - 30.0,
        );
        tree.in_parent(body, |tree| {
            add_section_label(
                tree,
                &style,
                "Coin buys light and rare blocks",
                blocker_hud::theme::ROW_WIDTH,
            );
            let mut rows = Vec::new();
            for (index, good) in STOCK.iter().enumerate() {
                let strip = add_row_strip(tree, &style, index, blocker_hud::theme::ROW_WIDTH);
                let button = tree.in_parent(strip, |tree| {
                    item_icon(tree, &art, good.item, theme::ROW_ICON);
                    add_cell_text(
                        tree,
                        &style,
                        &format!("{} x{}", good.item.name(), good.count),
                        theme::NAME_WIDTH,
                        blocker_hud::theme::SMALL_SIZE,
                        blocker_hud::theme::TEXT_COLOR,
                    );
                    add_cell_text(
                        tree,
                        &style,
                        &format!("{} coin", good.price),
                        theme::NEEDS_WIDTH,
                        blocker_hud::theme::SMALL_SIZE,
                        blocker_hud::theme::DIM_TEXT_COLOR,
                    );
                    add_action_button(tree, &style, "Buy")
                });
                rows.push(button);
            }
            shop_panel.rows = rows;
        });
        let buttons = add_footer_buttons(
            tree,
            &style,
            blocker_hud::theme::MENU_WIDTH,
            &["Sell held stack", "Close"],
        );
        shop_panel.sell = buttons[0];
        shop_panel.close = buttons[1];
    });
    menu_show_on(&mut menus, Screen::Shop, shop_panel.panel);
    built_tree.finish();
}

pub fn show_the_shop(
    journal: Res<Journal>,
    inventory: Res<Inventory>,
    shopping: Res<Shopping>,
    mut shop_panel: ResMut<ShopPanel>,
    mut commands: Commands,
) {
    if shop_panel.rows.is_empty() {
        return;
    }
    let held = selected_stack(&inventory);
    let sell = held.map(|stack| {
        (
            stack.item.name().to_string(),
            stack.count,
            item_value(stack.item) * stack.count,
        )
    });
    let trader = TRADERS.get(shopping.trader).copied().unwrap_or("Trader");
    let key = (shopping.trader, journal.coin, sell);
    if shop_panel.shown.as_ref() == Some(&key) {
        return;
    }
    let sell = &key.2;
    shop_panel.shown = Some(key.clone());
    ui_set_text(
        &mut commands,
        shop_panel.coin,
        &format!("{trader}   -   Your coin: {}", journal.coin),
    );
    let sell_label = match sell {
        Some((name, count, gained)) => format!("Sell {name} x{count} (+{gained})"),
        None => "Sell held stack".to_string(),
    };
    ui_set_text(&mut commands, shop_panel.sell, &sell_label);
}

pub fn shop_buttons(
    shop_panel: Res<ShopPanel>,
    mut inventory: ResMut<Inventory>,
    mut journal: ResMut<Journal>,
    mut notices: EventWriter<Notice>,
    mut next: ResMut<NextState<Screen>>,
    mut buttons: Query<(&UiButtonData,)>,
) {
    if ui_clicked(&mut buttons, shop_panel.close) {
        next.pending = Some(Screen::Playing);
        return;
    }
    if ui_clicked(&mut buttons, shop_panel.sell) {
        sell_selected(&mut inventory, &mut journal, &mut notices);
        return;
    }
    for index in 0..shop_panel.rows.len() {
        if ui_clicked(&mut buttons, shop_panel.rows[index]) {
            buy_good(&mut inventory, &mut journal, &mut notices, index);
            return;
        }
    }
}
