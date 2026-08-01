use crate::commands::decode_atlas;
use crate::commands::random::random_name;
use crate::data::assets::HUD_ICONS;
use crate::data::{
    ART_ITEMS, BlockKind, CALLINGS, FRAME_IMAGE, Item, ItemStack, RECIPES, SHOP, art_color,
    cell_uv_exact,
};
use crate::queries::{calling_sheet, can_craft, coin_text, inventory_count, selected_stack};
use crate::resources::{
    Aim, Chests, CraftRowHandles, Creation, Draft, Hero, HudHandles, Icons, Inventory, Prompt,
    Save, Screen, Settings, ShopRowHandles, SlotHandles, Toast, UiHandles, VoxelWorld, WorldPlan,
};
use crate::systems::hud;
use crate::systems::worldmap;
use crate::theme;
use nalgebra_glm::{Vec2, Vec4, vec2, vec4};
use nightshade::prelude::*;
use nightshade_kit::prelude::{ICONS, MAP_ICONS};
use nightshade_platform::prelude::Time;
use nightshade_state::prelude::*;
use nightshade_ui::commands::builder::{UiBuild, UiTreeBuilder};
use nightshade_ui::components::layout_types::FlowDirection;
use nightshade_ui::components::{TextOverflow, UiNodeContent, UiScrollAreaData};
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;
use nightshade_ui_menu::prelude::*;

pub fn build(
    mut handles: ResMut<UiHandles>,
    mut hud_handles: ResMut<HudHandles>,
    mut icons: ResMut<Icons>,
    mut creation: ResMut<Creation>,
    mut hero: ResMut<Hero>,
    mut build: UiBuild,
) {
    let mut tree = UiTreeBuilder::new(build.context());
    let (atlas_pixels, atlas_width, atlas_height) = decode_atlas();
    let (tool_pixels, tool_width, tool_height) = art_sheet();
    let (atlas_upload, tool_upload) = {
        let (images, runtime) = tree.images_and_runtime();
        let atlas = ui_upload_image(images, runtime, &atlas_pixels, atlas_width, atlas_height);
        let tools = ui_upload_image(images, runtime, &tool_pixels, tool_width, tool_height);
        (atlas, tools)
    };
    load_icons(&mut tree, &mut icons);
    let mut built = UiHandles {
        atlas_layer: atlas_upload.as_ref().map(|upload| upload.layer),
        tools_layer: tool_upload.as_ref().map(|upload| upload.layer),
        ..Default::default()
    };

    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();

    let name = random_name(&mut hero.random);
    tree.in_parent(root, |tree| {
        *hud_handles = hud::build(tree, &icons, &CALLINGS[creation.class]);
        let hud_root = hud_handles.root;
        tree.in_parent(hud_root, |tree| {
            worldmap::build(tree, &icons, &mut hud_handles);
        });
        let hud = tree
            .add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .with_visible(false)
            .entity();
        built.hud = hud;
        tree.in_parent(hud, |tree| {
            build_hotbar(tree, &mut built, &atlas_upload);
        });
        build_rate(tree, &mut built);
        build_prompt(tree, &mut built);
        build_toast(tree, &mut built);
        build_pocket(tree, &mut built);
        build_crafting(tree, &mut built);
        build_shop(tree, &mut built);
        build_chest(tree, &mut built);
        build_settings(tree, &mut built);
        build_title(tree, &mut built);
        build_new_world(tree, &mut built);
        build_creation(tree, &mut creation, &name);
    });

    built.root = tree.root_entity();
    tree.finish();
    *handles = built;
}

fn pack_atlas(
    names: &[&str],
    source: &[(&str, &[u8])],
    cell: u32,
    side: u32,
) -> (Vec<u8>, Vec<(Vec2, Vec2)>) {
    let side = side as usize;
    let cell = cell as usize;
    let per_row = side / cell;
    let mut pixels = vec![0u8; side * side * 4];
    let mut cells = Vec::new();
    for (slot, name) in names.iter().enumerate() {
        let column = slot % per_row;
        let row = slot / per_row;
        let origin_x = column * cell;
        let origin_y = row * cell;
        if let Some((_, bytes)) = source.iter().find(|(icon, _)| icon == name)
            && let Ok(decoded) = image::load_from_memory(bytes)
        {
            let picture = decoded.to_rgba8();
            let fit = (cell as u32 - 8).min(picture.width().max(picture.height()));
            let scaled = image::imageops::resize(
                &picture,
                fit * picture.width() / picture.width().max(picture.height()),
                fit * picture.height() / picture.width().max(picture.height()),
                image::imageops::FilterType::Triangle,
            );
            let offset_x = origin_x + (cell - scaled.width() as usize) / 2;
            let offset_y = origin_y + (cell - scaled.height() as usize) / 2;
            for (x, y, pixel) in scaled.enumerate_pixels() {
                let index = ((offset_y + y as usize) * side + offset_x + x as usize) * 4;
                pixels[index..index + 4].copy_from_slice(&pixel.0);
            }
        }
        cells.push((
            vec2(origin_x as f32 / side as f32, origin_y as f32 / side as f32),
            vec2(
                (origin_x + cell) as f32 / side as f32,
                (origin_y + cell) as f32 / side as f32,
            ),
        ));
    }
    (pixels, cells)
}

fn load_icons(tree: &mut UiTreeBuilder<'_, '_>, icons: &mut Icons) {
    let (pixels, cells) = pack_atlas(&HUD_ICONS, &ICONS, theme::ICON_CELL, theme::ICON_ATLAS);
    let upload = {
        let (images, runtime) = tree.images_and_runtime();
        ui_upload_image(
            images,
            runtime,
            &pixels,
            theme::ICON_ATLAS,
            theme::ICON_ATLAS,
        )
    };
    if let Some(upload) = upload {
        icons.layer = Some(upload.layer);
        icons.cells = cells
            .iter()
            .map(|(low, high)| {
                (
                    vec2(low.x * upload.uv_max.x, low.y * upload.uv_max.y),
                    vec2(high.x * upload.uv_max.x, high.y * upload.uv_max.y),
                )
            })
            .collect();
    }
    let mark_names: Vec<&str> = MAP_ICONS.iter().map(|(name, _)| *name).collect();
    let (pixels, cells) = pack_atlas(
        &mark_names,
        &MAP_ICONS,
        theme::MAP_ICON_CELL,
        theme::MAP_ATLAS,
    );
    let upload = {
        let (images, runtime) = tree.images_and_runtime();
        ui_upload_image(images, runtime, &pixels, theme::MAP_ATLAS, theme::MAP_ATLAS)
    };
    if let Some(upload) = upload {
        icons.mark_layer = Some(upload.layer);
        icons.marks = cells
            .iter()
            .map(|(low, high)| {
                (
                    vec2(low.x * upload.uv_max.x, low.y * upload.uv_max.y),
                    vec2(high.x * upload.uv_max.x, high.y * upload.uv_max.y),
                )
            })
            .collect();
    }
    if let Some((_, bytes)) = ICONS.iter().find(|(icon, _)| *icon == FRAME_IMAGE)
        && let Ok(decoded) = image::load_from_memory(bytes)
    {
        let picture = decoded.to_rgba8();
        let (width, height) = picture.dimensions();
        let (images, runtime) = tree.images_and_runtime();
        icons.frame_layer = ui_upload_image(images, runtime, picture.as_raw(), width, height)
            .map(|upload| upload.layer);
    }
    icons.tint = CALLINGS[0].accent;
}

fn build_creation(tree: &mut UiTreeBuilder<'_, '_>, creation: &mut Creation, name: &str) {
    let panel = tree
        .add_node()
        .window(
            Rl(vec2(theme::CREATION_LEFT, 50.0)) + Ab(vec2(0.0, -theme::CREATION_HEIGHT / 2.0)),
            Ab(vec2(theme::CREATION_WIDTH, theme::CREATION_HEIGHT)),
            Anchor::TopLeft,
        )
        .with_rect(12.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .flow(FlowDirection::Vertical, 18.0, 8.0)
        .with_visible(false)
        .entity();
    let first = calling_sheet(creation.class);
    tree.in_parent(panel, |tree| {
        add_menu_text(
            tree,
            theme::TITLE,
            theme::HEADING_SIZE + 8.0,
            theme::ACCENT_COLOR,
        );
        add_menu_text(
            tree,
            "Choose a calling, take a name, and walk out into the blocks.",
            theme::BODY_SIZE,
            theme::DIM_TEXT_COLOR,
        );
        for (index, class) in CALLINGS.iter().enumerate() {
            creation.class_buttons[index] = tree.add_button(class.name);
        }
        creation.title =
            add_menu_text(tree, &first.title, theme::HEADING_SIZE, theme::ACCENT_COLOR);
        creation.blurb = add_menu_text(tree, &first.blurb, theme::BODY_SIZE, theme::TEXT_COLOR);
        for (line, text) in creation.stats.iter_mut().zip(first.stats.iter()) {
            *line = add_menu_text(tree, text, theme::BODY_SIZE, theme::TEXT_COLOR);
        }
        for (line, text) in creation.abilities.iter_mut().zip(first.abilities.iter()) {
            *line = add_menu_text(tree, text, theme::BODY_SIZE, theme::ACCENT_COLOR);
        }
        creation.name_input = tree.add_text_input_with_value("Name", name);
        creation.roll = tree.add_button("Another name");
        creation.enter = tree.add_button("Walk into Hollowmere");
    });
    creation.panel = panel;
}

fn build_hotbar(
    tree: &mut UiTreeBuilder<'_, '_>,
    built: &mut UiHandles,
    atlas_upload: &Option<UiImageUpload>,
) {
    let pitch = theme::SLOT_SIZE + theme::SLOT_GAP;
    let total = pitch * theme::HOTBAR_SLOTS as f32 - theme::SLOT_GAP;
    let mut slots = [SlotHandles::default(); theme::HOTBAR_SLOTS];
    for (index, slot) in slots.iter_mut().enumerate() {
        let left = -total * 0.5 + index as f32 * pitch;
        slot.frame = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 100.0)) + Ab(vec2(left, -theme::HOTBAR_BOTTOM_MARGIN)),
                Ab(vec2(theme::SLOT_SIZE, theme::SLOT_SIZE)),
                Anchor::BottomLeft,
            )
            .with_rect(6.0, 2.0, theme::SLOT_BORDER)
            .color_raw::<UiBase>(theme::SLOT_COLOR)
            .with_interaction()
            .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
            .entity();
        tree.commands().set(
            slot.frame,
            UiButtonData {
                clicked: false,
                text_slot: None,
            },
        );
        tree.in_parent(slot.frame, |tree| {
            let inset = (theme::SLOT_SIZE - theme::ICON_SIZE) * 0.5;
            let uv = atlas_upload
                .as_ref()
                .map(|upload| (upload.layer, upload.uv_min, upload.uv_max))
                .unwrap_or((0, vec2(0.0, 0.0), vec2(1.0, 1.0)));
            slot.icon = tree
                .add_node()
                .window(
                    Ab(vec2(inset, 4.0)),
                    Ab(vec2(theme::ICON_SIZE, theme::ICON_SIZE)),
                    Anchor::TopLeft,
                )
                .with_image_uv(uv.0, uv.1, uv.2)
                .color_raw::<UiBase>(vec4(1.0, 1.0, 1.0, 1.0))
                .with_visible(false)
                .without_pointer_events()
                .entity();
            slot.count = tree
                .add_node()
                .window(
                    Rl(vec2(100.0, 100.0)) + Ab(vec2(-5.0, -1.0)),
                    Ab(vec2(30.0, theme::COUNT_SIZE)),
                    Anchor::BottomRight,
                )
                .with_text("", theme::COUNT_SIZE)
                .text_right()
                .with_text_outline(vec4(0.0, 0.0, 0.0, 1.0), 1.0)
                .color_raw::<UiBase>(theme::TEXT_COLOR)
                .without_pointer_events()
                .entity();
        });
    }
    built.slots = slots;
}

fn build_rate(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    built.rate_panel = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Rl(vec2(100.0, 100.0)) + Ab(vec2(-14.0, -70.0)),
            Ab(vec2(theme::RATE_CARD_WIDTH, theme::STATUS_HEIGHT)),
            Anchor::BottomRight,
        )
        .flow(FlowDirection::Vertical, 5.0, 0.0)
        .with_visible(false)
        .entity();
    tree.in_parent(built.rate_panel, |tree| {
        built.rate_label = tree
            .add_node()
            .size_px(theme::RATE_CARD_WIDTH - 10.0, theme::STATUS_HEIGHT - 10.0)
            .with_text("", theme::BODY_SIZE)
            .text_center()
            .color_raw::<UiBase>(theme::TEXT_COLOR)
            .entity();
    });
}

fn build_settings(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    built.settings_panel = menu_card(tree, theme::SETTINGS_HEIGHT);
    let panel = built.settings_panel;
    tree.in_parent(panel, |tree| {
        title_bar(tree, "Settings");
        let body = body_column(
            tree,
            theme::SETTINGS_HEIGHT - theme::TITLE_HEIGHT - theme::FOOTER_HEIGHT,
        );
        tree.in_parent(body, |tree| {
            section_label(tree, "What the screen shows while you play");
            let strip = row_strip(tree, 0);
            tree.in_parent(strip, |tree| {
                cell_text(
                    tree,
                    "Frame rate in the corner",
                    theme::SETTING_NAME_WIDTH,
                    theme::BODY_SIZE,
                    theme::TEXT_COLOR,
                );
                built.rate_toggle = action_button(tree, "Off");
            });
            section_label(tree, "Your world on disk");
            let strip = row_strip(tree, 0);
            tree.in_parent(strip, |tree| {
                cell_text(
                    tree,
                    "The world saves itself while you play",
                    theme::SETTING_NAME_WIDTH,
                    theme::BODY_SIZE,
                    theme::TEXT_COLOR,
                );
                built.settings_save = action_button(tree, "Save now");
            });
        });
        let buttons = footer_buttons(
            tree,
            &[
                "Back to the world",
                "Save and go to the menu",
                "Save and leave",
            ],
        );
        built.settings_close = buttons[0];
        built.settings_title = buttons[1];
        built.settings_leave = buttons[2];
    });
}

fn footer_buttons(tree: &mut UiTreeBuilder<'_, '_>, labels: &[&str]) -> Vec<Entity> {
    let footer = footer_bar(tree);
    let count = labels.len().max(1) as f32;
    let room = theme::MENU_WIDTH - 28.0 - theme::ROW_SPACING * (count - 1.0);
    let width = room / count;
    tree.in_parent(footer, |tree| {
        labels
            .iter()
            .map(|label| {
                let holder = tree
                    .add_node()
                    .size_px(width, theme::ACTION_HEIGHT)
                    .flow(FlowDirection::Horizontal, 0.0, 0.0)
                    .entity();
                tree.in_parent(holder, |tree| tree.add_button(label))
            })
            .collect()
    })
}

fn wide_button(tree: &mut UiTreeBuilder<'_, '_>, text: &str) -> Entity {
    let holder = tree
        .add_node()
        .size_px(theme::ROW_WIDTH, theme::ACTION_HEIGHT + 6.0)
        .flow(FlowDirection::Horizontal, 0.0, 0.0)
        .entity();
    tree.in_parent(holder, |tree| tree.add_button(text))
}

fn setting_row(tree: &mut UiTreeBuilder<'_, '_>, index: usize, name: &str, value: &str) -> Entity {
    let strip = row_strip(tree, index);
    tree.in_parent(strip, |tree| {
        cell_text(
            tree,
            name,
            theme::SETTING_NAME_WIDTH,
            theme::BODY_SIZE,
            theme::TEXT_COLOR,
        );
        action_button(tree, value)
    })
}

fn build_title(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    built.title_panel = menu_card(tree, theme::TITLE_MENU_HEIGHT);
    let panel = built.title_panel;
    tree.in_parent(panel, |tree| {
        title_bar(tree, theme::TITLE);
        let body = body_column(
            tree,
            theme::TITLE_MENU_HEIGHT - theme::TITLE_HEIGHT - theme::FOOTER_HEIGHT,
        );
        tree.in_parent(body, |tree| {
            section_label(tree, "A block world with a town, a guild, and work to do");
            built.title_continue = wide_button(tree, "Go on with your world");
            built.title_new = wide_button(tree, "Make a new world");
        });
        built.title_leave = footer_buttons(tree, &["Leave"])[0];
    });
}

fn build_new_world(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    built.new_world_panel = menu_card(tree, theme::NEW_WORLD_HEIGHT);
    let panel = built.new_world_panel;
    let plan = WorldPlan::default();
    tree.in_parent(panel, |tree| {
        title_bar(tree, "A new world");
        let body = body_column(
            tree,
            theme::NEW_WORLD_HEIGHT - theme::TITLE_HEIGHT - theme::FOOTER_HEIGHT,
        );
        tree.in_parent(body, |tree| {
            let scroll = scrolling_body(
                tree,
                theme::NEW_WORLD_HEIGHT
                    - theme::TITLE_HEIGHT
                    - theme::FOOTER_HEIGHT
                    - theme::BODY_PADDING * 2.0,
            );
            tree.in_parent(scroll, |tree| {
                section_label(tree, "The ground the world is made of");
                built.land_button = setting_row(tree, 0, "The shape of the land", plan.land.name());
                let strip = row_strip(tree, 1);
                built.land_told = tree.in_parent(strip, |tree| {
                    cell_text(
                        tree,
                        plan.land.told(),
                        theme::ROW_WIDTH - theme::ROW_PADDING * 2.0,
                        theme::SMALL_SIZE,
                        theme::DIM_TEXT_COLOR,
                    )
                });
                let strip = row_strip(tree, 2);
                tree.in_parent(strip, |tree| {
                    cell_text(
                        tree,
                        "The number the world grows from",
                        theme::SETTING_NAME_WIDTH - theme::SEED_WIDTH,
                        theme::BODY_SIZE,
                        theme::TEXT_COLOR,
                    );
                    built.seed_label = cell_text(
                        tree,
                        &theme::SEED.to_string(),
                        theme::SEED_WIDTH,
                        theme::BODY_SIZE,
                        theme::ACCENT_COLOR,
                    );
                    built.seed_button = action_button(tree, "Roll");
                });
                built.woods_button = setting_row(tree, 3, "Trees", plan.woods.name());
                section_label(tree, "What the world holds");
                built.caves_button = setting_row(tree, 0, "Caves under the ground", "On");
                built.villages_button = setting_row(tree, 1, "Villages", "On");
                built.dungeons_button = setting_row(tree, 2, "Dungeons", "On");
                section_label(tree, "The sky over it");
                built.weather_button = setting_row(tree, 0, "Rain and snow", "On");
                built.start_button = setting_row(tree, 1, "The world starts at", plan.start.name());
            });
        });
        let buttons = footer_buttons(tree, &["Make the world", "Back"]);
        built.new_world_make = buttons[0];
        built.new_world_back = buttons[1];
    });
}

fn build_prompt(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    built.prompt_panel = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Rl(vec2(50.0, 50.0)) + Ab(vec2(0.0, 60.0)),
            Ab(vec2(380.0, 44.0)),
            Anchor::Center,
        )
        .flow(FlowDirection::Vertical, 8.0, 0.0)
        .with_visible(false)
        .entity();
    tree.in_parent(built.prompt_panel, |tree| {
        built.prompt_label = tree
            .add_node()
            .fill_width()
            .auto_height()
            .with_text("", theme::BODY_SIZE)
            .text_center()
            .color_raw::<UiBase>(theme::ACCENT_COLOR)
            .entity();
    });
}

fn build_toast(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    built.toast_panel = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 130.0)),
            Ab(vec2(460.0, 44.0)),
            Anchor::TopCenter,
        )
        .flow(FlowDirection::Vertical, 8.0, 0.0)
        .with_visible(false)
        .entity();
    tree.in_parent(built.toast_panel, |tree| {
        built.toast_label = tree
            .add_node()
            .fill_width()
            .auto_height()
            .with_text("", theme::BODY_SIZE)
            .text_center()
            .color_raw::<UiBase>(theme::TEXT_COLOR)
            .entity();
    });
}

fn menu_card(tree: &mut UiTreeBuilder<'_, '_>, height: f32) -> Entity {
    tree.add_node()
        .window(
            Rl(vec2(50.0, 50.0)),
            Ab(vec2(theme::MENU_WIDTH, height)),
            Anchor::Center,
        )
        .with_rect(12.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .flow(FlowDirection::Vertical, 0.0, 0.0)
        .with_visible(false)
        .entity()
}

fn title_bar(tree: &mut UiTreeBuilder<'_, '_>, title: &str) -> Entity {
    let bar = tree
        .add_node()
        .size_px(theme::MENU_WIDTH, theme::TITLE_HEIGHT)
        .with_rect(10.0, 0.0, vec4(0.0, 0.0, 0.0, 0.0))
        .color_raw::<UiBase>(theme::TITLE_COLOR)
        .flow(FlowDirection::Horizontal, theme::BODY_PADDING, 8.0)
        .entity();
    tree.in_parent(bar, |tree| {
        let width = theme::MENU_WIDTH - theme::BODY_PADDING * 2.0 - theme::COINS_WIDTH - 8.0;
        tree.add_node()
            .size_px(width, theme::TITLE_HEIGHT - theme::BODY_PADDING)
            .with_text(title, theme::HEADING_SIZE)
            .text_left()
            .color_raw::<UiBase>(theme::ACCENT_COLOR);
        tree.add_node()
            .size_px(
                theme::COINS_WIDTH,
                theme::TITLE_HEIGHT - theme::BODY_PADDING,
            )
            .with_text("", theme::BODY_SIZE)
            .text_right()
            .color_raw::<UiBase>(theme::ACCENT_COLOR)
            .entity()
    })
}

fn body_column(tree: &mut UiTreeBuilder<'_, '_>, height: f32) -> Entity {
    tree.add_node()
        .size_px(theme::MENU_WIDTH, height)
        .flow(FlowDirection::Vertical, theme::BODY_PADDING, theme::ROW_GAP)
        .entity()
}

fn scrolling_body(tree: &mut UiTreeBuilder<'_, '_>, height: f32) -> Entity {
    let area = tree.add_scroll_area(vec2(0.0, height));
    tree.component::<UiScrollAreaData>(area)
        .map(|data| data.content_entity)
        .unwrap_or(area)
}

fn footer_bar(tree: &mut UiTreeBuilder<'_, '_>) -> Entity {
    tree.add_node()
        .size_px(theme::MENU_WIDTH, theme::FOOTER_HEIGHT)
        .with_rect(10.0, 0.0, vec4(0.0, 0.0, 0.0, 0.0))
        .color_raw::<UiBase>(theme::FOOTER_COLOR)
        .flow(FlowDirection::Horizontal, 14.0, 10.0)
        .entity()
}

fn section_label(tree: &mut UiTreeBuilder<'_, '_>, text: &str) {
    tree.add_node()
        .size_px(theme::ROW_WIDTH, theme::SMALL_SIZE + 10.0)
        .with_text(text, theme::SMALL_SIZE)
        .text_left()
        .color_raw::<UiBase>(theme::DIM_TEXT_COLOR);
}

fn cell_text(
    tree: &mut UiTreeBuilder<'_, '_>,
    text: &str,
    width: f32,
    size: f32,
    color: Vec4,
) -> Entity {
    tree.add_node()
        .size_px(width, theme::ROW_HEIGHT - theme::ROW_PADDING * 2.0)
        .with_text(text, size)
        .with_text_overflow(TextOverflow::Ellipsis)
        .text_left()
        .color_raw::<UiBase>(color)
        .entity()
}

fn row_strip(tree: &mut UiTreeBuilder<'_, '_>, index: usize) -> Entity {
    let color = if index.is_multiple_of(2) {
        theme::ROW_COLOR
    } else {
        theme::ROW_ALT_COLOR
    };
    tree.add_node()
        .size_px(theme::ROW_WIDTH, theme::ROW_HEIGHT)
        .with_rect(6.0, 0.0, vec4(0.0, 0.0, 0.0, 0.0))
        .color_raw::<UiBase>(color)
        .flow(
            FlowDirection::Horizontal,
            theme::ROW_PADDING,
            theme::ROW_SPACING,
        )
        .entity()
}

fn item_icon(tree: &mut UiTreeBuilder<'_, '_>, built: &UiHandles, item: Item, size: f32) -> Entity {
    let node = tree
        .add_node()
        .size_px(size, size)
        .color_raw::<UiBase>(vec4(1.0, 1.0, 1.0, 1.0));
    match icon_region(built, item) {
        Some((layer, uv_min, uv_max)) => node.with_image_uv(layer, uv_min, uv_max).entity(),
        None => node.with_visible(false).entity(),
    }
}

fn action_button(tree: &mut UiTreeBuilder<'_, '_>, text: &str) -> Entity {
    let holder = tree
        .add_node()
        .size_px(theme::ACTION_WIDTH, theme::ACTION_HEIGHT)
        .flow(FlowDirection::Horizontal, 0.0, 0.0)
        .entity();
    tree.in_parent(holder, |tree| tree.add_button(text))
}

fn slot_cell(tree: &mut UiTreeBuilder<'_, '_>) -> SlotHandles {
    let frame = tree
        .add_node()
        .size_px(theme::CELL_SIZE, theme::CELL_SIZE)
        .with_rect(6.0, 2.0, theme::CELL_BORDER)
        .color_raw::<UiBase>(theme::CELL_COLOR)
        .with_interaction()
        .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
        .entity();
    tree.commands().set(
        frame,
        UiButtonData {
            clicked: false,
            text_slot: None,
        },
    );
    let (icon, count) = tree.in_parent(frame, |tree| {
        let inset = (theme::CELL_SIZE - theme::CELL_ICON) * 0.5;
        let icon = tree
            .add_node()
            .window(
                Ab(vec2(inset, 5.0)),
                Ab(vec2(theme::CELL_ICON, theme::CELL_ICON)),
                Anchor::TopLeft,
            )
            .with_image_uv(0, vec2(0.0, 0.0), vec2(1.0, 1.0))
            .color_raw::<UiBase>(vec4(1.0, 1.0, 1.0, 1.0))
            .with_visible(false)
            .entity();
        let count = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 100.0)) + Ab(vec2(-6.0, -2.0)),
                Ab(vec2(34.0, theme::COUNT_SIZE)),
                Anchor::BottomRight,
            )
            .with_text("", theme::COUNT_SIZE)
            .text_right()
            .with_text_outline(vec4(0.0, 0.0, 0.0, 1.0), 1.0)
            .color_raw::<UiBase>(theme::TEXT_COLOR)
            .entity();
        (icon, count)
    });
    SlotHandles { frame, icon, count }
}

fn build_pocket(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    built.inventory_panel = menu_card(tree, theme::POCKET_HEIGHT);
    let panel = built.inventory_panel;
    tree.in_parent(panel, |tree| {
        built.inventory_coins = title_bar(tree, "Pocket");
        let body = body_column(
            tree,
            theme::POCKET_HEIGHT - theme::TITLE_HEIGHT - theme::FOOTER_HEIGHT,
        );
        tree.in_parent(body, |tree| {
            section_label(tree, "Click a slot, then another, to swap them");
            let mut cells = Vec::new();
            for _ in 0..theme::INVENTORY_ROWS {
                cells.extend(slot_row(tree, theme::HOTBAR_SLOTS));
            }
            built.pocket_slots = cells;
            built.pocket_detail = tree
                .add_node()
                .size_px(theme::MENU_WIDTH - theme::BODY_PADDING * 2.0, 34.0)
                .with_text("", theme::BODY_SIZE)
                .text_center()
                .color_raw::<UiBase>(theme::TEXT_COLOR)
                .entity();
        });
        let footer = footer_bar(tree);
        tree.in_parent(footer, |tree| {
            let width = (theme::MENU_WIDTH - 38.0) * 0.5;
            let left = tree
                .add_node()
                .size_px(width, theme::ACTION_HEIGHT)
                .flow(FlowDirection::Horizontal, 0.0, 0.0)
                .entity();
            built.inventory_craft = tree.in_parent(left, |tree| tree.add_button("Craft by hand"));
            let right = tree
                .add_node()
                .size_px(width, theme::ACTION_HEIGHT)
                .flow(FlowDirection::Horizontal, 0.0, 0.0)
                .entity();
            built.inventory_close = tree.in_parent(right, |tree| tree.add_button("Close"));
        });
    });
}

fn build_crafting(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    built.crafting_panel = menu_card(tree, theme::CRAFTING_HEIGHT);
    let panel = built.crafting_panel;
    tree.in_parent(panel, |tree| {
        built.crafting_coins = title_bar(tree, "Crafting");
        let body = scrolling_body(
            tree,
            theme::CRAFTING_HEIGHT - theme::TITLE_HEIGHT - theme::FOOTER_HEIGHT,
        );
        tree.in_parent(body, |tree| {
            section_label(tree, "Green means the parts are in your pocket");
            let mut rows = Vec::new();
            for (index, recipe) in RECIPES.iter().enumerate() {
                let strip = row_strip(tree, index);
                let handles = tree.in_parent(strip, |tree| {
                    item_icon(tree, built, recipe.output, theme::ROW_ICON);
                    cell_text(
                        tree,
                        &format!("{} x{}", recipe.output.name(), recipe.count),
                        theme::NAME_WIDTH,
                        theme::SMALL_SIZE,
                        theme::TEXT_COLOR,
                    );
                    let needs = cell_text(
                        tree,
                        "",
                        theme::NEEDS_WIDTH,
                        theme::SMALL_SIZE,
                        theme::DIM_TEXT_COLOR,
                    );
                    let button = action_button(tree, "Craft");
                    CraftRowHandles {
                        index,
                        needs,
                        button,
                    }
                });
                rows.push(handles);
            }
            built.crafting_rows = rows;
        });
        let footer = footer_bar(tree);
        tree.in_parent(footer, |tree| {
            let holder = tree
                .add_node()
                .size_px(theme::MENU_WIDTH - 28.0, theme::ACTION_HEIGHT)
                .flow(FlowDirection::Horizontal, 0.0, 0.0)
                .entity();
            built.crafting_close = tree.in_parent(holder, |tree| tree.add_button("Close"));
        });
    });
}

fn slot_row(tree: &mut UiTreeBuilder<'_, '_>, count: usize) -> Vec<SlotHandles> {
    let width = theme::CELL_SIZE * count as f32 + theme::CELL_GAP * (count as f32 - 1.0);
    let inset = (theme::MENU_WIDTH - theme::BODY_PADDING * 2.0 - width) * 0.5;
    let row = tree
        .add_node()
        .size_px(
            theme::MENU_WIDTH - theme::BODY_PADDING * 2.0,
            theme::CELL_SIZE + 10.0,
        )
        .flow(FlowDirection::Horizontal, inset, theme::CELL_GAP)
        .entity();
    let mut cells = Vec::new();
    tree.in_parent(row, |tree| {
        for _ in 0..count {
            cells.push(slot_cell(tree));
        }
    });
    cells
}

fn build_chest(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    built.chest_panel = menu_card(tree, theme::CHEST_HEIGHT);
    let panel = built.chest_panel;
    tree.in_parent(panel, |tree| {
        built.chest_title = title_bar(tree, "Chest");
        let body = body_column(
            tree,
            theme::CHEST_HEIGHT - theme::TITLE_HEIGHT - theme::FOOTER_HEIGHT,
        );
        tree.in_parent(body, |tree| {
            section_label(
                tree,
                "Click a chest slot to take it, a pocket slot to store it",
            );
            let mut chest = Vec::new();
            for _ in 0..(theme::CHEST_SLOTS / theme::CHEST_ROW) {
                chest.extend(slot_row(tree, theme::CHEST_ROW));
            }
            built.chest_slots = chest;
            section_label(tree, "Pocket");
            let mut pocket = Vec::new();
            for _ in 0..theme::INVENTORY_ROWS {
                pocket.extend(slot_row(tree, theme::HOTBAR_SLOTS));
            }
            built.chest_pocket = pocket;
        });
        let footer = footer_bar(tree);
        tree.in_parent(footer, |tree| {
            built.chest_close = tree.add_button("Close");
        });
    });
}

fn build_shop(tree: &mut UiTreeBuilder<'_, '_>, built: &mut UiHandles) {
    built.shop_panel = menu_card(tree, theme::SHOP_HEIGHT);
    let panel = built.shop_panel;
    tree.in_parent(panel, |tree| {
        built.shop_coins = title_bar(tree, "Fenn's Ledger");
        let body = scrolling_body(
            tree,
            theme::SHOP_HEIGHT - theme::TITLE_HEIGHT - theme::FOOTER_HEIGHT,
        );
        tree.in_parent(body, |tree| {
            let mut rows = Vec::new();
            for pass in 0..2 {
                section_label(tree, if pass == 0 { "For sale" } else { "Fenn buys" });
                for (index, row) in SHOP.iter().enumerate() {
                    if row.buy.is_some() != (pass == 0) {
                        continue;
                    }
                    let strip = row_strip(tree, rows.len());
                    let handles = tree.in_parent(strip, |tree| {
                        item_icon(tree, built, row.item, theme::ROW_ICON);
                        cell_text(
                            tree,
                            row.item.name(),
                            theme::NAME_WIDTH,
                            theme::SMALL_SIZE,
                            theme::TEXT_COLOR,
                        );
                        let price = cell_text(
                            tree,
                            &price_label(row.buy, row.sell),
                            theme::PRICE_WIDTH,
                            theme::SMALL_SIZE,
                            theme::ACCENT_COLOR,
                        );
                        let have = cell_text(
                            tree,
                            "",
                            theme::HAVE_WIDTH,
                            theme::SMALL_SIZE,
                            theme::DIM_TEXT_COLOR,
                        );
                        let button =
                            action_button(tree, if row.buy.is_some() { "Buy" } else { "Sell" });
                        ShopRowHandles {
                            index,
                            price,
                            have,
                            button,
                        }
                    });
                    rows.push(handles);
                }
            }
            built.shop_rows = rows;
        });
        let footer = footer_bar(tree);
        tree.in_parent(footer, |tree| {
            let holder = tree
                .add_node()
                .size_px(theme::MENU_WIDTH - 28.0, theme::ACTION_HEIGHT)
                .flow(FlowDirection::Horizontal, 0.0, 0.0)
                .entity();
            built.shop_close = tree.in_parent(holder, |tree| tree.add_button("Leave"));
        });
    });
}

fn price_label(buy: Option<u32>, sell: Option<u32>) -> String {
    match (buy, sell) {
        (Some(price), _) => coin_text(price),
        (_, Some(price)) => coin_text(price),
        _ => String::new(),
    }
}

pub fn refresh(
    mut handles: ResMut<UiHandles>,
    inventory: Res<Inventory>,
    settings: Res<Settings>,
    time: Res<Time>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    if handles.root == Entity::default() {
        return;
    }
    show_the_rate(&mut handles, &settings, &time, &mut dirty, &mut commands);
    for index in 0..theme::HOTBAR_SLOTS {
        let stack = inventory.slots[index];
        let slot = handles.slots[index];
        if handles.shown[index] != stack {
            let was = handles.shown[index];
            handles.shown[index] = stack;
            paint_slot(&handles, &slot, stack, was, &mut dirty, &mut commands);
        }
    }

    if handles.shown_selected != Some(inventory.selected) {
        if let Some(previous) = handles.shown_selected {
            let frame = handles.slots[previous].frame;
            ui_set_color::<UiBase>(&mut commands, frame, theme::SLOT_COLOR);
            set_border(&mut commands, frame, theme::SLOT_BORDER);
        }
        let frame = handles.slots[inventory.selected].frame;
        ui_set_color::<UiBase>(&mut commands, frame, theme::SLOT_SELECTED_COLOR);
        set_border(&mut commands, frame, theme::SLOT_SELECTED_BORDER);
        handles.shown_selected = Some(inventory.selected);
    }
}

fn show_the_rate(
    handles: &mut UiHandles,
    settings: &Settings,
    time: &Time,
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
) {
    if handles.shown_rate != Some(settings.show_rate) {
        handles.shown_rate = Some(settings.show_rate);
        ui_set_visible(dirty, commands, handles.rate_panel, settings.show_rate);
        ui_set_text(
            commands,
            handles.rate_toggle,
            if settings.show_rate { "On" } else { "Off" },
        );
    }
    if !settings.show_rate {
        return;
    }
    let rate = time.frames_per_second;
    let text = format!("{rate:.0} fps");
    if text == handles.shown_rate_text {
        return;
    }
    handles.shown_rate_text = text;
    ui_set_text(commands, handles.rate_label, &handles.shown_rate_text);
    let color = if rate >= theme::RATE_GOOD {
        theme::RATE_GOOD_COLOR
    } else if rate >= theme::RATE_FAIR {
        theme::RATE_FAIR_COLOR
    } else {
        theme::RATE_POOR_COLOR
    };
    if handles.shown_rate_color != Some(color) {
        handles.shown_rate_color = Some(color);
        ui_set_color::<UiBase>(commands, handles.rate_label, color);
    }
}

pub fn show_the_draft(
    draft: Res<Draft>,
    save: Res<Save>,
    world: Res<VoxelWorld>,
    mut handles: ResMut<UiHandles>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    if handles.root == Entity::default() {
        return;
    }
    let found = save.found.is_some() || !world.chunks.is_empty();
    if handles.shown_found != Some(found) {
        handles.shown_found = Some(found);
        let button = handles.title_continue;
        ui_set_visible(&mut dirty, &mut commands, button, found);
    }
    let plan = draft.plan;
    if handles.shown_draft == Some((draft.seed, plan)) {
        return;
    }
    handles.shown_draft = Some((draft.seed, plan));
    let told = |held: bool| if held { "On" } else { "Off" };
    let rows = [
        (handles.land_button, plan.land.name().to_string()),
        (handles.land_told, plan.land.told().to_string()),
        (handles.seed_label, draft.seed.to_string()),
        (handles.woods_button, plan.woods.name().to_string()),
        (handles.caves_button, told(plan.caves).to_string()),
        (handles.villages_button, told(plan.villages).to_string()),
        (handles.dungeons_button, told(plan.dungeons).to_string()),
        (handles.weather_button, told(plan.weather).to_string()),
        (handles.start_button, plan.start.name().to_string()),
    ];
    for (entity, text) in rows {
        ui_set_text(&mut commands, entity, &text);
    }
}

pub fn show_the_screen(
    state: Res<State<Screen>>,
    hud_handles: Res<HudHandles>,
    creation: Res<Creation>,
    mut handles: ResMut<UiHandles>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    let screen = state.current;
    if handles.shown_screen == Some(screen) {
        return;
    }
    handles.shown_screen = Some(screen);
    let panels = [
        (handles.title_panel, screen == Screen::Title),
        (handles.new_world_panel, screen == Screen::NewWorld),
        (creation.panel, screen == Screen::Creation),
        (handles.inventory_panel, screen == Screen::Inventory),
        (handles.crafting_panel, screen == Screen::Crafting),
        (handles.shop_panel, screen == Screen::Shop),
        (handles.chest_panel, screen == Screen::Chest),
        (handles.settings_panel, screen == Screen::Settings),
        (handles.hud, screen == Screen::Playing),
        (hud_handles.root, screen == Screen::Playing),
    ];
    for (panel, visible) in panels {
        ui_set_visible(&mut dirty, &mut commands, panel, visible);
    }
}

pub fn refresh_panels(
    time: Res<Time>,
    inventory: Res<Inventory>,
    chests: Res<Chests>,
    prompt: Res<Prompt>,
    aim: Res<Aim>,
    mut toast: ResMut<Toast>,
    mut handles: ResMut<UiHandles>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    if handles.root == Entity::default() {
        return;
    }
    let screen = handles.shown_screen.unwrap_or_default();

    let prompt_text = match &*prompt {
        Prompt::None => aim.target.map(|hit| {
            let held = selected_stack(&inventory)
                .map(|stack| stack.item.name())
                .unwrap_or("Empty hand");
            format!("{held}   {}", BlockKind::from_index(hit.kind).info().name)
        }),
        Prompt::Trade(_) => Some("Trade".to_string()),
        Prompt::Harvest(_) => Some("Pick".to_string()),
        Prompt::Door(_, open) => Some(if *open {
            "Close the door".to_string()
        } else {
            "Open the door".to_string()
        }),
        Prompt::Craft => Some("Craft".to_string()),
        Prompt::Chest(_) => Some("Open the chest".to_string()),
        Prompt::Sign(_) => Some("Read".to_string()),
    };
    let wanted = if screen == Screen::Playing {
        prompt_text
    } else {
        None
    };
    if wanted != handles.shown_prompt {
        ui_set_visible(
            &mut dirty,
            &mut commands,
            handles.prompt_panel,
            wanted.is_some(),
        );
        if let Some(text) = &wanted {
            ui_set_text(&mut commands, handles.prompt_label, text);
        }
        handles.shown_prompt = wanted;
    }

    let was_shown = toast.remaining > 0.0;
    toast.remaining -= time.delta_time;
    let shown = toast.remaining > 0.0;
    if shown {
        ui_set_text(&mut commands, handles.toast_label, &toast.text);
    }
    if shown != was_shown
        || (shown && toast.remaining > theme::TOAST_SECONDS - time.delta_time * 2.0)
    {
        ui_set_visible(&mut dirty, &mut commands, handles.toast_panel, shown);
    }

    match screen {
        Screen::Inventory => {
            refresh_pocket(&inventory, &mut handles, &mut dirty, &mut commands);
        }
        Screen::Crafting => {
            refresh_crafting(&inventory, &mut handles, &mut commands);
        }
        Screen::Shop => {
            refresh_shop(&inventory, &mut handles, &mut commands);
        }
        Screen::Chest => {
            refresh_chest(&inventory, &chests, &mut handles, &mut dirty, &mut commands);
        }
        Screen::Playing
        | Screen::Settings
        | Screen::Title
        | Screen::NewWorld
        | Screen::Creation => {}
    }
}

fn refresh_chest(
    inventory: &Inventory,
    chests: &Chests,
    handles: &mut UiHandles,
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
) {
    let empty = Vec::new();
    let held = chests
        .open
        .and_then(|key| chests.slots.get(&key))
        .unwrap_or(&empty);
    if handles.shown_chest.len() != handles.chest_slots.len() {
        handles.shown_chest = vec![None; handles.chest_slots.len()];
    }
    for (index, cell) in handles.chest_slots.clone().iter().enumerate() {
        let stack = held.get(index).copied().flatten();
        if handles.shown_chest[index] == stack {
            continue;
        }
        let was = handles.shown_chest[index];
        handles.shown_chest[index] = stack;
        paint_slot(handles, cell, stack, was, dirty, commands);
    }
    if handles.shown_chest_pocket.len() != handles.chest_pocket.len() {
        handles.shown_chest_pocket = vec![None; handles.chest_pocket.len()];
    }
    for (index, cell) in handles.chest_pocket.clone().iter().enumerate() {
        let stack = inventory.slots.get(index).copied().flatten();
        if handles.shown_chest_pocket[index] == stack {
            continue;
        }
        let was = handles.shown_chest_pocket[index];
        handles.shown_chest_pocket[index] = stack;
        paint_slot(handles, cell, stack, was, dirty, commands);
    }
}

fn paint_slot(
    handles: &UiHandles,
    cell: &SlotHandles,
    stack: Option<ItemStack>,
    was: Option<ItemStack>,
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
) {
    match stack {
        Some(stack) => {
            if was.map(|was| was.item) != Some(stack.item)
                && let Some(region) = icon_region(handles, stack.item)
            {
                set_icon(commands, cell.icon, region);
            }
            if was.is_none() {
                ui_set_visible(dirty, commands, cell.icon, true);
            }
            ui_set_text(commands, cell.count, &count_label(stack));
        }
        None => {
            ui_set_visible(dirty, commands, cell.icon, false);
            ui_set_text(commands, cell.count, "");
        }
    }
}

fn refresh_pocket(
    inventory: &Inventory,
    handles: &mut UiHandles,
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
) {
    show_the_coins(inventory, handles, commands, handles.inventory_coins);
    if handles.shown_pocket.len() != handles.pocket_slots.len() {
        handles.shown_pocket = vec![None; handles.pocket_slots.len()];
    }
    for (index, cell) in handles.pocket_slots.clone().iter().enumerate() {
        let stack = inventory.slots.get(index).copied().flatten();
        if handles.shown_pocket[index] == stack {
            continue;
        }
        let was = handles.shown_pocket[index];
        handles.shown_pocket[index] = stack;
        paint_slot(handles, cell, stack, was, dirty, commands);
    }

    let marks = (handles.holding, inventory.selected);
    if handles.shown_marks != Some(marks) {
        handles.shown_marks = Some(marks);
        for (index, cell) in handles.pocket_slots.clone().iter().enumerate() {
            let (color, border) = if Some(index) == marks.0 {
                (theme::HELD_COLOR, theme::HELD_BORDER)
            } else if index == marks.1 {
                (theme::SLOT_SELECTED_COLOR, theme::SLOT_SELECTED_BORDER)
            } else {
                (theme::CELL_COLOR, theme::CELL_BORDER)
            };
            ui_set_color::<UiBase>(commands, cell.frame, color);
            set_border(commands, cell.frame, border);
        }
    }

    let detail = match handles.holding {
        Some(index) => match inventory.slots.get(index).copied().flatten() {
            Some(stack) => format!("Holding {} x{}", stack.item.name(), stack.count),
            None => "Holding an empty slot".to_string(),
        },
        None => match selected_stack(inventory) {
            Some(stack) => format!("{} x{}", stack.item.name(), stack.count),
            None => "Empty hand".to_string(),
        },
    };
    if handles.shown_detail != detail {
        ui_set_text(commands, handles.pocket_detail, &detail);
        handles.shown_detail = detail;
    }
}

fn show_the_coins(
    inventory: &Inventory,
    handles: &mut UiHandles,
    commands: &mut Commands,
    label: Entity,
) {
    if handles.shown_panel_coins == Some((label, inventory.coins)) {
        return;
    }
    handles.shown_panel_coins = Some((label, inventory.coins));
    ui_set_text(commands, label, &coin_text(inventory.coins));
}

fn refresh_crafting(inventory: &Inventory, handles: &mut UiHandles, commands: &mut Commands) {
    show_the_coins(inventory, handles, commands, handles.crafting_coins);
    if handles.shown_craft.len() != handles.crafting_rows.len() {
        handles.shown_craft = vec![false; handles.crafting_rows.len()];
    }
    for (place, row) in handles.crafting_rows.clone().iter().enumerate() {
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
        if handles.shown_craft[place] == ready && !text.is_empty() {
            continue;
        }
        handles.shown_craft[place] = ready;
        ui_set_text(commands, row.needs, &text);
        let color = if ready {
            theme::READY_COLOR
        } else {
            theme::DIM_TEXT_COLOR
        };
        ui_set_color::<UiBase>(commands, row.needs, color);
    }
}

fn refresh_shop(inventory: &Inventory, handles: &mut UiHandles, commands: &mut Commands) {
    show_the_coins(inventory, handles, commands, handles.shop_coins);
    if handles.shown_have.len() != handles.shop_rows.len() {
        handles.shown_have = vec![u32::MAX; handles.shop_rows.len()];
    }
    for (place, row) in handles.shop_rows.clone().iter().enumerate() {
        let ware = SHOP[row.index];
        let have = inventory_count(inventory, ware.item);
        if handles.shown_have[place] == have {
            continue;
        }
        handles.shown_have[place] = have;
        ui_set_text(commands, row.have, &format!("you hold {have}"));
        let ready = match (ware.buy, ware.sell) {
            (Some(price), _) => inventory.coins >= price,
            (_, Some(_)) => have > 0,
            _ => false,
        };
        let color = if ready {
            theme::ACCENT_COLOR
        } else {
            theme::SHORT_COLOR
        };
        ui_set_color::<UiBase>(commands, row.price, color);
    }
}

fn set_icon(commands: &mut Commands, entity: Entity, region: (u32, Vec2, Vec2)) {
    let (layer, uv_min, uv_max) = region;
    commands.queue(move |storage, _| {
        if let Some(UiNodeContent::Image {
            texture_index,
            uv_min: min,
            uv_max: max,
        }) = storage.get_mut::<UiNodeContent>(entity)
        {
            *texture_index = layer;
            *min = uv_min;
            *max = uv_max;
        }
    });
}

fn set_border(commands: &mut Commands, entity: Entity, color: Vec4) {
    commands.queue(move |storage, _| {
        if let Some(UiNodeContent::Rect { border_color, .. }) =
            storage.get_mut::<UiNodeContent>(entity)
        {
            *border_color = color;
        }
    });
}

fn count_label(stack: ItemStack) -> String {
    if stack.count > 1 {
        stack.count.to_string()
    } else {
        String::new()
    }
}

fn icon_region(handles: &UiHandles, item: Item) -> Option<(u32, Vec2, Vec2)> {
    match item {
        Item::Block(kind) => {
            let layer = handles.atlas_layer?;
            let info = kind.info();
            let (min, max) = cell_uv_exact(info.side);
            let scale = vec2(
                1.0,
                theme::ATLAS_ROWS as f32 * theme::TILE_TEXELS as f32
                    / theme::TOOL_SHEET_WIDTH as f32,
            );
            Some((layer, min.component_mul(&scale), max.component_mul(&scale)))
        }
        _ => {
            let layer = handles.tools_layer?;
            let texel = theme::TILE_TEXELS as f32 / theme::TOOL_SHEET_WIDTH as f32;
            let left = item.art_index()? as f32 * texel;
            Some((layer, vec2(left, 0.0), vec2(left + texel, texel)))
        }
    }
}

fn art_sheet() -> (Vec<u8>, u32, u32) {
    let mut pixels = vec![0u8; (theme::TOOL_SHEET_WIDTH * theme::TOOL_SHEET_HEIGHT * 4) as usize];
    for item in ART_ITEMS {
        let Some((art, index)) = item.art().zip(item.art_index()) else {
            continue;
        };
        for (row, line) in art.iter().enumerate() {
            for (column, character) in line.chars().enumerate() {
                let Some(color) = art_color(character) else {
                    continue;
                };
                let x = index as u32 * theme::TILE_TEXELS + column as u32;
                let y = row as u32;
                let offset = ((y * theme::TOOL_SHEET_WIDTH + x) * 4) as usize;
                for channel in 0..4 {
                    pixels[offset + channel] = (color[channel] * 255.0).round() as u8;
                }
            }
        }
    }
    (pixels, theme::TOOL_SHEET_WIDTH, theme::TOOL_SHEET_HEIGHT)
}
