use crate::commands::textures::icon_page;
use crate::data::{Errand, Item, ShopKind, Slot};
use crate::queries::clock::clock_text;
use crate::queries::pocket::{count, errand_progress, hand_text, is_done, is_usable};
use crate::queries::sheet::icon_uv;
use crate::queries::talk::shown;
use crate::resources::{HotbarSlot, MealButton, PocketSlot, Screen, ShopPanel, ShopRow, UiHandles};
use crate::theme;
use nalgebra_glm::vec2;
use nightshade::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui::prelude::*;
use nightshade_ui_menu::prelude::*;

pub fn build(mut handles: ResMut<UiHandles>, mut build: UiBuild) {
    let mut tree = UiTreeBuilder::new(build.context());
    let page = {
        let (images, runtime) = tree.images_and_runtime();
        ui_upload_image(
            images,
            runtime,
            &icon_page(),
            theme::ICON_PAGE,
            theme::ICON_PAGE,
        )
        .map(|upload| upload.layer)
    };
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();
    let built = tree.in_parent(root, |tree| {
        let mut built = UiHandles::default();
        build_hud(tree, &mut built);
        build_hotbar(tree, &mut built, page);
        build_prompt(tree, &mut built);
        build_toast(tree, &mut built);
        build_dialogue(tree, &mut built);
        build_pause(tree, &mut built);
        build_title(tree, &mut built, page);
        build_scrim(tree, &mut built);
        build_inventory(tree, &mut built, page);
        for kind in ShopKind::ALL {
            let panel = build_shop(tree, kind, page);
            built.shop_panels.push(panel);
        }
        built
    });
    tree.finish();
    *handles = built;
}

fn icon(tree: &mut UiTreeBuilder<'_, '_>, page: Option<u32>, index: u32, size: f32) -> Entity {
    let (uv_min, uv_max) = icon_uv(index);
    let node = tree.add_node().size_px(size, size);
    match page {
        Some(layer) => node
            .with_image_uv(layer, uv_min, uv_max)
            .color_raw::<UiBase>(theme::ICON_COLOR)
            .entity(),
        None => node.entity(),
    }
}

fn build_hud(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    let hud = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Ab(vec2(20.0, 20.0)),
            Ab(vec2(380.0, 134.0)),
            Anchor::TopLeft,
        )
        .flow(FlowDirection::Vertical, 12.0, 4.0)
        .entity();
    tree.in_parent(hud, |tree| {
        built.clock_label = Some(add_menu_text(
            tree,
            "",
            theme::BODY_SIZE,
            theme::ACCENT_COLOR,
        ));
        built.coins_label = Some(add_menu_text(tree, "", theme::BODY_SIZE, theme::TEXT_COLOR));
        built.hand_label = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::DIM_TEXT_COLOR,
        ));
        built.vigor_label = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::TEXT_COLOR,
        ));
        built.hero_label = Some(add_menu_text(
            tree,
            "",
            theme::SMALL_SIZE,
            theme::TEXT_COLOR,
        ));
    });
    built.hud = Some(hud);
}

fn build_hotbar(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles, page: Option<u32>) {
    let slots = Slot::ALL.len() as f32;
    let width = slots * (theme::HOTBAR_SLOT + theme::HOTBAR_SPACING) + 20.0;
    let bar = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -14.0)),
            Ab(vec2(width, theme::HOTBAR_HEIGHT)),
            Anchor::BottomCenter,
        )
        .flow(FlowDirection::Horizontal, 10.0, theme::HOTBAR_SPACING)
        .entity();
    built.hotbar = Some(bar);
    tree.in_parent(bar, |tree| {
        for slot in Slot::ALL.iter() {
            let frame = tree
                .add_node()
                .size_px(theme::HOTBAR_SLOT, theme::HOTBAR_HEIGHT - 20.0)
                .with_rect(6.0, 1.0, theme::PANEL_BORDER)
                .color_raw::<UiBase>(theme::SLOT_COLOR)
                .flow(FlowDirection::Vertical, 3.0, 1.0)
                .align_cross(FlowAlignment::Center)
                .entity();
            let (icon_entity, label) = tree.in_parent(frame, |tree| {
                let picture = icon(tree, page, slot.icon(), theme::HOTBAR_ICON);
                let label = tree
                    .add_node()
                    .fill_width()
                    .auto_height()
                    .with_text("", theme::TINY_SIZE)
                    .text_center()
                    .color_raw::<UiBase>(theme::DIM_TEXT_COLOR)
                    .entity();
                (picture, label)
            });
            built.hotbar_slots.push(HotbarSlot {
                frame: Some(frame),
                icon: Some(icon_entity),
                label: Some(label),
            });
        }
    });
}

fn build_prompt(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    let panel = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -(theme::HOTBAR_HEIGHT + 26.0))),
            Ab(vec2(400.0, 44.0)),
            Anchor::BottomCenter,
        )
        .flow(FlowDirection::Vertical, 8.0, 0.0)
        .with_visible(false)
        .entity();
    tree.in_parent(panel, |tree| {
        built.prompt_label = Some(
            tree.add_node()
                .fill_width()
                .auto_height()
                .with_text("", theme::BODY_SIZE)
                .text_center()
                .color_raw::<UiBase>(theme::ACCENT_COLOR)
                .entity(),
        );
    });
    built.prompt_panel = Some(panel);
}

fn build_toast(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    let panel = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 24.0)),
            Ab(vec2(460.0, 44.0)),
            Anchor::TopCenter,
        )
        .flow(FlowDirection::Vertical, 8.0, 0.0)
        .with_visible(false)
        .entity();
    tree.in_parent(panel, |tree| {
        built.toast_label = Some(
            tree.add_node()
                .fill_width()
                .auto_height()
                .with_text("", theme::BODY_SIZE)
                .text_center()
                .color_raw::<UiBase>(theme::TEXT_COLOR)
                .entity(),
        );
    });
    built.toast_panel = Some(panel);
}

fn build_dialogue(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    let panel = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -(theme::HOTBAR_HEIGHT + 26.0))),
            Ab(vec2(theme::DIALOGUE_WIDTH, theme::DIALOGUE_HEIGHT)),
            Anchor::BottomCenter,
        )
        .flow(FlowDirection::Vertical, 16.0, 6.0)
        .with_visible(false)
        .entity();
    tree.in_parent(panel, |tree| {
        built.speaker_label = Some(add_menu_text(
            tree,
            "",
            theme::BODY_SIZE,
            theme::ACCENT_COLOR,
        ));
        built.body_label = Some(
            tree.add_node()
                .fill_width()
                .size_px(theme::DIALOGUE_WIDTH - 32.0, 52.0)
                .with_text("", theme::BODY_SIZE)
                .with_text_wrap()
                .text_left()
                .color_raw::<UiBase>(theme::TEXT_COLOR)
                .entity(),
        );
    });
    built.dialogue_panel = Some(panel);
}

fn fixed_button(tree: &mut UiTreeBuilder<'_, '_>, text: &str, width: f32) -> Entity {
    let holder = tree
        .add_node()
        .size_px(width, theme::MENU_BUTTON_HEIGHT)
        .entity();
    tree.in_parent(holder, |tree| tree.add_button(text))
}

fn heading_row(tree: &mut UiTreeBuilder<'_, '_>, title: &str, width: f32) -> Entity {
    let row = tree
        .add_node()
        .size_px(width, 34.0)
        .flow(FlowDirection::Horizontal, 0.0, 10.0)
        .align_cross(FlowAlignment::Center)
        .entity();
    tree.in_parent(row, |tree| {
        tree.add_node()
            .size_px(width * 0.6, 30.0)
            .with_text(title, theme::HEADING_SIZE)
            .text_left()
            .color_raw::<UiBase>(theme::ACCENT_COLOR)
            .entity();
        tree.add_node()
            .size_px(width * 0.4 - 10.0, 30.0)
            .with_text("", theme::BODY_SIZE)
            .text_right()
            .color_raw::<UiBase>(theme::TEXT_COLOR)
            .entity()
    })
}

fn build_title(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles, page: Option<u32>) {
    let inner = theme::MENU_WIDTH - 44.0;
    let panel = tree
        .add_node()
        .with_rect(14.0, 3.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::TITLE_PANEL_COLOR)
        .window(
            Rl(vec2(50.0, 50.0)),
            Ab(vec2(theme::MENU_WIDTH, 372.0)),
            Anchor::Center,
        )
        .flow(FlowDirection::Vertical, 24.0, 12.0)
        .align_cross(FlowAlignment::Center)
        .with_visible(false)
        .entity();
    tree.in_parent(panel, |tree| {
        let strip = tree
            .add_node()
            .size_px(inner, theme::TITLE_ICON)
            .flow(FlowDirection::Horizontal, 0.0, 10.0)
            .align_main(FlowAlignment::Center)
            .align_cross(FlowAlignment::Center)
            .entity();
        tree.in_parent(strip, |tree| {
            for (place, index) in theme::TITLE_STRIP.into_iter().enumerate() {
                let size = if place == 0 {
                    theme::TITLE_ICON
                } else {
                    theme::TITLE_ICON * 0.6
                };
                icon(tree, page, index, size);
            }
        });
        tree.add_node()
            .size_px(inner, 46.0)
            .with_text(theme::TITLE, theme::TITLE_SIZE)
            .text_center()
            .color_raw::<UiBase>(theme::ACCENT_COLOR)
            .entity();
        tree.add_node()
            .size_px(inner, 22.0)
            .with_text(theme::TAGLINE, theme::SMALL_SIZE)
            .text_center()
            .color_raw::<UiBase>(theme::DIM_TEXT_COLOR)
            .entity();
        tree.add_node().size_px(inner, 6.0).entity();
        built.title_play = Some(fixed_button(tree, "Play", theme::MENU_BUTTON_WIDE));
        built.title_quit = Some(fixed_button(
            tree,
            "Quit to desktop",
            theme::MENU_BUTTON_WIDE,
        ));
    });
    built.title_panel = Some(panel);
}

fn build_pause(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    let inner = theme::MENU_WIDTH - 44.0;
    let panel = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Rl(vec2(50.0, 50.0)),
            Ab(vec2(theme::MENU_WIDTH, 216.0)),
            Anchor::Center,
        )
        .flow(FlowDirection::Vertical, 22.0, 12.0)
        .align_cross(FlowAlignment::Center)
        .with_visible(false)
        .entity();
    tree.in_parent(panel, |tree| {
        tree.add_node()
            .size_px(inner, 34.0)
            .with_text("Paused", theme::HEADING_SIZE)
            .text_center()
            .color_raw::<UiBase>(theme::ACCENT_COLOR)
            .entity();
        built.pause_status = Some(
            tree.add_node()
                .size_px(inner, 20.0)
                .with_text("", theme::SMALL_SIZE)
                .text_center()
                .color_raw::<UiBase>(theme::DIM_TEXT_COLOR)
                .entity(),
        );
        built.resume_button = Some(fixed_button(tree, "Resume", theme::MENU_BUTTON_WIDE));
        built.menu_button = Some(fixed_button(tree, "Main menu", theme::MENU_BUTTON_WIDE));
    });
    built.pause_panel = Some(panel);
}

fn build_scrim(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    built.scrim = Some(
        tree.add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .with_rect(0.0, 0.0, theme::CLEAR_COLOR)
            .color_raw::<UiBase>(theme::SCRIM_COLOR)
            .with_visible(false)
            .entity(),
    );
}

fn build_inventory(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles, page: Option<u32>) {
    let inner = theme::POCKET_WIDTH - 44.0;
    let panel = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Rl(vec2(50.0, 50.0)),
            Ab(vec2(theme::POCKET_WIDTH, theme::INVENTORY_HEIGHT)),
            Anchor::Center,
        )
        .flow(FlowDirection::Vertical, 22.0, 10.0)
        .with_visible(false)
        .entity();
    tree.in_parent(panel, |tree| {
        built.inventory_coins = Some(heading_row(tree, "Pocket", inner));
        for chunk in Item::ALL.chunks(theme::POCKET_COLUMNS) {
            let row = tree
                .add_node()
                .size_px(inner, theme::POCKET_CELL_HEIGHT)
                .flow(FlowDirection::Horizontal, 0.0, 10.0)
                .entity();
            tree.in_parent(row, |tree| {
                for item in chunk {
                    let cell = tree
                        .add_node()
                        .size_px(theme::POCKET_CELL, theme::POCKET_CELL_HEIGHT)
                        .with_rect(8.0, 1.0, theme::PANEL_BORDER)
                        .color_raw::<UiBase>(theme::SLOT_COLOR)
                        .flow(FlowDirection::Vertical, 8.0, 2.0)
                        .align_cross(FlowAlignment::Center)
                        .entity();
                    let (picture, held) = tree.in_parent(cell, |tree| {
                        let picture = icon(tree, page, item.icon(), theme::POCKET_ICON);
                        tree.add_node()
                            .size_px(theme::POCKET_CELL - 16.0, 16.0)
                            .with_text(item.name(), theme::TINY_SIZE)
                            .text_center()
                            .color_raw::<UiBase>(theme::DIM_TEXT_COLOR)
                            .entity();
                        let held = tree
                            .add_node()
                            .size_px(theme::POCKET_CELL - 16.0, 18.0)
                            .with_text("x0", theme::SMALL_SIZE)
                            .text_center()
                            .color_raw::<UiBase>(theme::TEXT_COLOR)
                            .entity();
                        (picture, held)
                    });
                    built.pocket_slots.push(PocketSlot {
                        cell: Some(cell),
                        icon: Some(picture),
                        count: Some(held),
                    });
                }
            });
        }
        tree.add_node()
            .size_px(inner, 26.0)
            .with_text("Meals", theme::BODY_SIZE)
            .text_left()
            .color_raw::<UiBase>(theme::ACCENT_COLOR)
            .entity();
        let foods: Vec<Item> = Item::ALL
            .into_iter()
            .filter(|item| item.food() > 0.0)
            .collect();
        for chunk in foods.chunks(theme::MEAL_COLUMNS) {
            let meals = tree
                .add_node()
                .size_px(inner, theme::MENU_BUTTON_HEIGHT + 6.0)
                .flow(FlowDirection::Horizontal, 0.0, 8.0)
                .entity();
            tree.in_parent(meals, |tree| {
                for item in chunk {
                    let button = fixed_button(tree, item.name(), theme::MEAL_BUTTON_WIDTH);
                    built.meal_buttons.push(MealButton {
                        item: *item,
                        button,
                    });
                }
            });
        }
        tree.add_node()
            .size_px(inner, 26.0)
            .with_text("Errands", theme::BODY_SIZE)
            .text_left()
            .color_raw::<UiBase>(theme::ACCENT_COLOR)
            .entity();
        let list = tree
            .add_node()
            .size_px(inner, Errand::ALL.len() as f32 * 20.0 + 16.0)
            .with_rect(8.0, 1.0, theme::PANEL_BORDER)
            .color_raw::<UiBase>(theme::MENU_ROW_DIM)
            .flow(FlowDirection::Vertical, 8.0, 2.0)
            .entity();
        tree.in_parent(list, |tree| {
            for _ in Errand::ALL {
                let row = tree
                    .add_node()
                    .size_px(inner - 16.0, 18.0)
                    .with_text("", theme::SMALL_SIZE)
                    .text_left()
                    .color_raw::<UiBase>(theme::TEXT_COLOR)
                    .entity();
                built.errand_rows.push(row);
            }
        });
    });
    built.inventory_panel = Some(panel);
}

fn build_shop(tree: &mut UiTreeBuilder<'_, '_>, kind: ShopKind, page: Option<u32>) -> ShopPanel {
    let wares = kind.wares();
    let mut built = ShopPanel {
        kind: Some(kind),
        ..Default::default()
    };
    let inner = theme::SHOP_WIDTH - 44.0;
    let offered = wares.iter().filter(|ware| ware.sells).count();
    let tallest = offered.max(wares.len() - offered) as f32;
    let body = 32.0 + tallest * (theme::SHOP_ROW_HEIGHT + 8.0);
    let height = body + 34.0 + theme::MENU_BUTTON_HEIGHT + 80.0;
    let panel = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Rl(vec2(50.0, 50.0)),
            Ab(vec2(theme::SHOP_WIDTH, height)),
            Anchor::Center,
        )
        .flow(FlowDirection::Vertical, 22.0, 12.0)
        .with_visible(false)
        .entity();
    tree.in_parent(panel, |tree| {
        built.coins = Some(heading_row(tree, kind.title(), inner));
        let columns = tree
            .add_node()
            .size_px(inner, body)
            .flow(FlowDirection::Horizontal, 0.0, 16.0)
            .entity();
        tree.in_parent(columns, |tree| {
            for buying in [true, false] {
                let column = tree
                    .add_node()
                    .size_px(theme::SHOP_COLUMN, body)
                    .flow(FlowDirection::Vertical, 0.0, 8.0)
                    .entity();
                tree.in_parent(column, |tree| {
                    tree.add_node()
                        .size_px(theme::SHOP_COLUMN, 24.0)
                        .with_text(
                            if buying { "For sale" } else { "They buy" },
                            theme::BODY_SIZE,
                        )
                        .text_left()
                        .color_raw::<UiBase>(theme::ACCENT_COLOR)
                        .entity();
                    for (index, ware) in wares.iter().enumerate() {
                        if ware.sells != buying {
                            continue;
                        }
                        let frame = tree
                            .add_node()
                            .size_px(theme::SHOP_COLUMN, theme::SHOP_ROW_HEIGHT)
                            .with_rect(8.0, 1.0, theme::PANEL_BORDER)
                            .color_raw::<UiBase>(theme::MENU_ROW_COLOR)
                            .flow(FlowDirection::Horizontal, 8.0, 8.0)
                            .align_cross(FlowAlignment::Center)
                            .entity();
                        let row = tree.in_parent(frame, |tree| {
                            icon(tree, page, ware.item.icon(), 30.0);
                            let name = tree
                                .add_node()
                                .size_px(theme::SHOP_COLUMN - 300.0, 18.0)
                                .with_text(ware.item.name(), theme::SMALL_SIZE)
                                .text_left()
                                .color_raw::<UiBase>(theme::TEXT_COLOR)
                                .entity();
                            let note = tree
                                .add_node()
                                .size_px(76.0, 16.0)
                                .with_text("", theme::TINY_SIZE)
                                .text_right()
                                .color_raw::<UiBase>(theme::DIM_TEXT_COLOR)
                                .entity();
                            let price = tree
                                .add_node()
                                .size_px(58.0, 18.0)
                                .with_text("", theme::SMALL_SIZE)
                                .text_right()
                                .color_raw::<UiBase>(theme::ACCENT_COLOR)
                                .entity();
                            let button = fixed_button(
                                tree,
                                if buying { "Buy" } else { "Sell" },
                                theme::MENU_BUTTON,
                            );
                            ShopRow {
                                ware: index,
                                frame: Some(frame),
                                name: Some(name),
                                note: Some(note),
                                price: Some(price),
                                button: Some(button),
                            }
                        });
                        built.rows.push(row);
                    }
                });
            }
        });
        let footer = tree
            .add_node()
            .size_px(inner, theme::MENU_BUTTON_HEIGHT)
            .flow(FlowDirection::Horizontal, 0.0, 0.0)
            .entity();
        built.close_button = Some(tree.in_parent(footer, |tree| {
            tree.add_node()
                .size_px((inner - theme::MENU_BUTTON_WIDE) / 2.0, 4.0)
                .entity();
            fixed_button(tree, "Leave", theme::MENU_BUTTON_WIDE)
        }));
    });
    built.panel = Some(panel);
    built
}

pub fn show_the_screen(
    screen_state: Res<State<Screen>>,
    view: crate::resources::Readout,
    mut commands: Commands,
) {
    let commands = &mut commands;
    let screen = screen_state.current;
    let open = view.open_shop.kind;
    let handles = {
        let handles = &mut *view.handles;
        if handles.shown == Some(screen)
            && (screen != Screen::Shop || handles.shown_shop == Some(open))
        {
            return;
        }
        handles.shown = Some(screen);
        handles.shown_shop = Some(open);
        handles.clone()
    };
    let modal = matches!(
        screen,
        Screen::Title | Screen::Paused | Screen::Inventory | Screen::Shop
    );
    let playing = matches!(screen, Screen::Playing | Screen::Talking);
    view.renderer_state.day_night.auto_cycle = playing;
    let dirty = &mut *view.dirty;
    let panels = [
        (handles.dialogue_panel, screen == Screen::Talking),
        (handles.pause_panel, screen == Screen::Paused),
        (handles.inventory_panel, screen == Screen::Inventory),
        (handles.title_panel, screen == Screen::Title),
        (handles.scrim, modal),
        (handles.hud, playing),
        (handles.hotbar, playing),
    ];
    for (panel, visible) in panels {
        if let Some(panel) = panel {
            ui_set_visible(dirty, commands, panel, visible);
        }
    }
    for shop in &handles.shop_panels {
        if let Some(panel) = shop.panel {
            let visible = screen == Screen::Shop && shop.kind == Some(open);
            ui_set_visible(dirty, commands, panel, visible);
        }
    }
}

pub fn refresh(view: crate::resources::Readout, mut commands: Commands) {
    let commands = &mut commands;
    let delta_time = view.time.delta_time;
    let hour = view.renderer_state.day_night.hour;
    let day = view.diorama.day;
    let weather = crate::queries::weather::label(view.weather);
    let open = view.open_shop.kind;
    let handles = view.handles.clone();
    let inventory = std::mem::take(view.inventory);
    let prompt = view.prompt.clone();
    let quests = std::mem::take(view.quests);

    if let Some(label) = handles.clock_label {
        ui_set_text(
            commands,
            label,
            &format!("Day {}  {}   {}", day + 1, clock_text(hour), weather),
        );
    }
    if let Some(label) = handles.pause_status {
        ui_set_text(
            commands,
            label,
            &format!("Day {}   {}   {}", day + 1, clock_text(hour), weather),
        );
    }
    let coins = format!("Coins {}", inventory.coins);
    let shop_coins = handles.shop_panels.iter().filter_map(|panel| panel.coins);
    for label in [handles.coins_label, handles.inventory_coins]
        .into_iter()
        .flatten()
        .chain(shop_coins)
    {
        ui_set_text(commands, label, &coins);
    }
    if let Some(label) = handles.hand_label {
        ui_set_text(commands, label, &hand_text(&inventory));
    }
    if let Some(label) = handles.vigor_label {
        let stamina = view.vigor.stamina;
        ui_set_text(
            commands,
            label,
            &format!(
                "Stamina {}/{}",
                stamina.ceil() as u32,
                theme::MAX_STAMINA as u32
            ),
        );
        ui_set_color::<UiBase>(
            commands,
            label,
            if stamina <= theme::TIRED_STAMINA {
                theme::TIRED_COLOR
            } else {
                theme::TEXT_COLOR
            },
        );
    }
    for meal in &handles.meal_buttons {
        let held = count(&inventory, meal.item);
        ui_set_text(
            commands,
            meal.button,
            &format!("{} x{}", meal.item.name(), held),
        );
    }
    if let Some(label) = handles.hero_label {
        let hero = view.hero;
        let dungeon = view.dungeon;
        let mut text = format!(
            "HP {}/{}   Lv {}   {}",
            hero.hp.ceil() as u32,
            hero.max_hp,
            hero.level,
            hero.weapon.name()
        );
        if dungeon.active {
            text.push_str(&format!("   Hollow depth {}", dungeon.depth));
        }
        ui_set_text(commands, label, &text);
    }

    let playing = handles.shown == Some(Screen::Playing);
    let text = crate::queries::prompt::text(&prompt);
    {
        let dirty = &mut *view.dirty;
        if let (Some(panel), Some(label)) = (handles.prompt_panel, handles.prompt_label) {
            ui_set_visible(dirty, commands, panel, playing && text.is_some());
            if let Some(text) = text {
                ui_set_text(commands, label, &text);
            }
        }
    }

    {
        let remaining = {
            let toast = &mut *view.toast;
            toast.remaining -= delta_time;
            toast.remaining
        };
        let toast_text = &*view.toast.text.clone();
        let dirty = &mut *view.dirty;
        if let (Some(panel), Some(label)) = (handles.toast_panel, handles.toast_label) {
            ui_set_visible(dirty, commands, panel, remaining > 0.0);
            ui_set_text(commands, label, toast_text);
        }
    }

    if handles.shown == Some(Screen::Talking) {
        let dialogue = view.dialogue;
        if let Some(label) = handles.speaker_label {
            ui_set_text(commands, label, &dialogue.speaker);
        }
        if let Some(label) = handles.body_label {
            ui_set_text(commands, label, &shown(dialogue));
        }
    }

    if handles.shown == Some(Screen::Inventory) {
        for (item, slot) in Item::ALL.iter().zip(handles.pocket_slots.iter()) {
            let held = count(&inventory, *item);
            if let Some(label) = slot.count {
                ui_set_text(commands, label, &format!("x{held}"));
                ui_set_color::<UiBase>(
                    commands,
                    label,
                    if held > 0 {
                        theme::TEXT_COLOR
                    } else {
                        theme::DIM_TEXT_COLOR
                    },
                );
            }
            if let Some(picture) = slot.icon {
                ui_set_color::<UiBase>(
                    commands,
                    picture,
                    if held > 0 {
                        theme::ICON_COLOR
                    } else {
                        theme::ICON_DIM_COLOR
                    },
                );
            }
            if let Some(cell) = slot.cell {
                ui_set_color::<UiBase>(
                    commands,
                    cell,
                    if held > 0 {
                        theme::SLOT_COLOR
                    } else {
                        theme::MENU_ROW_DIM
                    },
                );
            }
        }
        for (errand, row) in Errand::ALL.iter().zip(handles.errand_rows.iter()) {
            let (item, wanted) = errand.wants();
            let text = if is_done(&quests, *errand) {
                format!(
                    "{} wanted {} {}: done",
                    errand.asker().name(),
                    wanted,
                    item.name().to_lowercase()
                )
            } else {
                let (have, _) = errand_progress(&inventory, *errand);
                format!(
                    "{} wants {} {}: {}/{}   ({} coins)",
                    errand.asker().name(),
                    wanted,
                    item.name().to_lowercase(),
                    have,
                    wanted,
                    errand.reward()
                )
            };
            ui_set_text(commands, *row, &text);
            let color = if is_done(&quests, *errand) {
                theme::DONE_COLOR
            } else {
                theme::TEXT_COLOR
            };
            ui_set_color::<UiBase>(commands, *row, color);
        }
    }

    if handles.shown == Some(Screen::Shop)
        && let Some(panel) = handles.shop_panels.get(open.index())
    {
        for row in &panel.rows {
            let Some(ware) = open.wares().get(row.ware) else {
                continue;
            };
            let held = count(&inventory, ware.item);
            let ready = if ware.sells {
                inventory.coins >= ware.price
            } else {
                held > 0
            };
            if let Some(name) = row.name {
                ui_set_text(commands, name, ware.item.name());
                ui_set_color::<UiBase>(
                    commands,
                    name,
                    if ready {
                        theme::TEXT_COLOR
                    } else {
                        theme::DIM_TEXT_COLOR
                    },
                );
            }
            if let Some(note) = row.note {
                ui_set_text(commands, note, &format!("you hold {held}"));
            }
            if let Some(price) = row.price {
                ui_set_text(commands, price, &format!("{} c", ware.price));
                ui_set_color::<UiBase>(
                    commands,
                    price,
                    if ready {
                        theme::ACCENT_COLOR
                    } else {
                        theme::DIM_TEXT_COLOR
                    },
                );
            }
            if let Some(frame) = row.frame {
                ui_set_color::<UiBase>(
                    commands,
                    frame,
                    if ready {
                        theme::MENU_ROW_COLOR
                    } else {
                        theme::MENU_ROW_DIM
                    },
                );
            }
        }
    }

    for (index, slot) in handles.hotbar_slots.iter().enumerate() {
        let definition = Slot::ALL[index];
        let picked = inventory.slot == index;
        let usable = is_usable(&inventory, definition);
        if let Some(frame) = slot.frame {
            let color = if picked {
                theme::SLOT_SELECTED_COLOR
            } else {
                theme::SLOT_COLOR
            };
            ui_set_color::<UiBase>(commands, frame, color);
        }
        if let Some(icon_entity) = slot.icon {
            let color = if usable {
                theme::ICON_COLOR
            } else {
                theme::ICON_DIM_COLOR
            };
            ui_set_color::<UiBase>(commands, icon_entity, color);
        }
        if let Some(label) = slot.label {
            let text = match definition {
                Slot::Seed(kind) => format!("x{}", count(&inventory, kind.seed())),
                Slot::Tool(_) => String::new(),
            };
            ui_set_text(commands, label, &text);
        }
    }

    *view.inventory = inventory;
    *view.quests = quests;
}
