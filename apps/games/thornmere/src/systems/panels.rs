use crate::commands::audio::cue;
use crate::commands::chat::system_line;
use crate::commands::items::{stow, strip, take_one, take_stack, wear_item};
use crate::components::{
    Cooldowns, Experience, Health, Kills, Level, Mana, MaxHealth, MaxMana, Merchant, Strength,
};
use crate::data::assets::ITEM_ICONS;
use crate::data::items::{
    BAG_SLOTS, ITEMS, LOG_ROWS, MERCHANT_REACH, MERCHANTS, SHEET_ROWS, Sip, WARE_ROWS, WEAR_ORDER,
    WEAR_SLOTS, Wear,
};
use crate::data::{
    Ask, CALLINGS, FRAME_CORNER, GIVERS, Objective, PICKUPS, PLACES, QUESTS, SOUND_ACCEPT,
    SOUND_DENY, SOUND_DONE, SOUND_PLAIN, SOUND_QUIET, SOUND_SLOT,
};
use crate::queries::items::{bag_room, bag_used};
use crate::queries::{
    buy_price, coin_text, flat_distance, gear_totals, item_index, item_note, level_threshold,
};
use crate::resources::{Cues, Icons, PanelHandles, PanelText, Panels, ThornmereResources};
use crate::systems::hud::frame;
use crate::systems::quests::turn_in_giver;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_scene::prelude::LocalTransform;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;

const SHEET_WIDTH: f32 = 400.0;
const SHEET_HEIGHT: f32 = 470.0;
const BAG_WIDTH: f32 = 360.0;
const BAG_HEIGHT: f32 = 460.0;
const SHOP_WIDTH: f32 = 400.0;
const SHOP_HEIGHT: f32 = 440.0;
const LOG_WIDTH: f32 = 400.0;
const LOG_HEIGHT: f32 = 560.0;
const TITLE_HEIGHT: f32 = 34.0;
const ROW_HEIGHT: f32 = 24.0;
const CELL: f32 = 72.0;
const CELL_GAP: f32 = 8.0;
const GRID_COLUMNS: usize = 4;
const WARE_HEIGHT: f32 = 44.0;
const TITLE_SIZE: f32 = 22.0;
const ROW_SIZE: f32 = 16.0;
const COUNT_SIZE: f32 = 14.0;

const PANEL_TITLE: Vec4 = Vec4::new(1.0, 0.88, 0.55, 1.0);
const ROW_TEXT: Vec4 = Vec4::new(0.90, 0.92, 0.96, 1.0);
const DIM_TEXT: Vec4 = Vec4::new(0.66, 0.70, 0.78, 1.0);
const GOLD_TEXT: Vec4 = Vec4::new(1.0, 0.85, 0.30, 1.0);
const PANEL_FILL: Vec4 = Vec4::new(0.05, 0.05, 0.08, 0.95);
const PANEL_EDGE: Vec4 = Vec4::new(0.80, 0.68, 0.40, 0.65);
const SLOT_FILL: Vec4 = Vec4::new(0.09, 0.09, 0.13, 0.95);
const SLOT_EDGE: Vec4 = Vec4::new(0.55, 0.50, 0.35, 0.8);
const EMPTY_ICON: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.0);

fn hidden_panel(
    tree: &mut UiTreeBuilder,
    icons: &Icons,
    position: impl Into<UiValue<Vec2>>,
    size: Vec2,
) -> Entity {
    let entity = tree
        .add_node()
        .window(position, Ab(size), Anchor::Center)
        .with_rect(8.0, 2.0, PANEL_EDGE)
        .color_raw::<UiBase>(PANEL_FILL)
        .without_pointer_events()
        .with_visible(false)
        .entity();
    tree.in_parent(entity, |tree| {
        frame(tree, icons.frame_layer, icons.tint, FRAME_CORNER * 1.2)
    });
    entity
}

fn item_icon(tree: &mut UiTreeBuilder, icons: &Icons, at: Vec2, size: f32) -> Entity {
    let (uv_min, uv_max) = icons
        .item_cells
        .first()
        .copied()
        .unwrap_or((vec2(0.0, 0.0), vec2(1.0, 1.0)));
    tree.add_node()
        .window(Ab(at), Ab(vec2(size, size)), Anchor::TopLeft)
        .with_image_uv(icons.item_layer.unwrap_or(0), uv_min, uv_max)
        .color_raw::<UiBase>(EMPTY_ICON)
        .without_pointer_events()
        .entity()
}

fn slot_box(tree: &mut UiTreeBuilder, at: Vec2, size: Vec2) -> Entity {
    tree.add_node()
        .window(Ab(at), Ab(size), Anchor::TopLeft)
        .with_rect(6.0, 1.0, SLOT_EDGE)
        .color_raw::<UiBase>(SLOT_FILL)
        .with_interaction()
        .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
        .entity()
}

fn text_line(tree: &mut UiTreeBuilder, at: Vec2, width: f32, size: f32, color: Vec4) -> Entity {
    tree.add_node()
        .window(Ab(at), Ab(vec2(width, size + 8.0)), Anchor::TopLeft)
        .with_text("", size)
        .text_left()
        .color_raw::<UiBase>(color)
        .without_pointer_events()
        .entity()
}

fn title_line(tree: &mut UiTreeBuilder, width: f32, text: &str) -> Entity {
    tree.add_node()
        .window(
            Ab(vec2(0.0, 12.0)),
            Ab(vec2(width, TITLE_HEIGHT)),
            Anchor::TopLeft,
        )
        .with_text(text, TITLE_SIZE)
        .color_raw::<UiBase>(PANEL_TITLE)
        .without_pointer_events()
        .entity()
}

pub fn build(tree: &mut UiTreeBuilder, icons: &Icons, handles: &mut PanelHandles) {
    handles.sheet_root = hidden_panel(
        tree,
        icons,
        Rl(vec2(20.0, 50.0)),
        vec2(SHEET_WIDTH, SHEET_HEIGHT),
    );
    tree.in_parent(handles.sheet_root, |tree| {
        title_line(tree, SHEET_WIDTH, "Character");
        let left = (SHEET_WIDTH - (CELL * WEAR_SLOTS as f32 + CELL_GAP * 3.0)) / 2.0;
        for (place, wear) in WEAR_ORDER.iter().enumerate() {
            let at = vec2(left + place as f32 * (CELL + CELL_GAP), TITLE_HEIGHT + 18.0);
            let frame = slot_box(tree, at, vec2(CELL, CELL));
            handles.wear_frames[place] = frame;
            tree.in_parent(frame, |tree| {
                handles.wear_icons[place] = item_icon(tree, icons, vec2(8.0, 8.0), CELL - 16.0);
            });
            handles.wear_names[place] = tree
                .add_node()
                .window(
                    Ab(vec2(at.x - 6.0, at.y + CELL + 2.0)),
                    Ab(vec2(CELL + 12.0, 20.0)),
                    Anchor::TopLeft,
                )
                .with_text(wear.name(), COUNT_SIZE)
                .color_raw::<UiBase>(DIM_TEXT)
                .without_pointer_events()
                .entity();
        }
        let top = TITLE_HEIGHT + CELL + 52.0;
        for row in 0..SHEET_ROWS {
            handles.sheet_rows[row] = text_line(
                tree,
                vec2(22.0, top + row as f32 * ROW_HEIGHT),
                SHEET_WIDTH - 44.0,
                ROW_SIZE,
                ROW_TEXT,
            );
        }
    });

    handles.bag_root = hidden_panel(
        tree,
        icons,
        Rl(vec2(80.0, 50.0)),
        vec2(BAG_WIDTH, BAG_HEIGHT),
    );
    tree.in_parent(handles.bag_root, |tree| {
        title_line(tree, BAG_WIDTH, "Bag");
        let grid = CELL * GRID_COLUMNS as f32 + CELL_GAP * (GRID_COLUMNS as f32 - 1.0);
        let left = (BAG_WIDTH - grid) / 2.0;
        for slot in 0..BAG_SLOTS {
            let column = slot % GRID_COLUMNS;
            let row = slot / GRID_COLUMNS;
            let at = vec2(
                left + column as f32 * (CELL + CELL_GAP),
                TITLE_HEIGHT + 20.0 + row as f32 * (CELL + CELL_GAP),
            );
            let frame = slot_box(tree, at, vec2(CELL, CELL));
            handles.bag_frames[slot] = frame;
            tree.in_parent(frame, |tree| {
                handles.bag_icons[slot] = item_icon(tree, icons, vec2(8.0, 6.0), CELL - 16.0);
                handles.bag_counts[slot] = tree
                    .add_node()
                    .window(
                        Ab(vec2(0.0, CELL - 22.0)),
                        Ab(vec2(CELL - 6.0, 20.0)),
                        Anchor::TopLeft,
                    )
                    .with_text("", COUNT_SIZE)
                    .color_raw::<UiBase>(ROW_TEXT)
                    .without_pointer_events()
                    .entity();
            });
        }
        handles.bag_note = text_line(
            tree,
            vec2(20.0, BAG_HEIGHT - 54.0),
            BAG_WIDTH - 40.0,
            ROW_SIZE,
            DIM_TEXT,
        );
    });

    handles.shop_root = hidden_panel(
        tree,
        icons,
        Rl(vec2(20.0, 50.0)),
        vec2(SHOP_WIDTH, SHOP_HEIGHT),
    );
    tree.in_parent(handles.shop_root, |tree| {
        handles.shop_title = title_line(tree, SHOP_WIDTH, "");
        for row in 0..WARE_ROWS {
            let at = vec2(18.0, TITLE_HEIGHT + 22.0 + row as f32 * (WARE_HEIGHT + 6.0));
            let frame = slot_box(tree, at, vec2(SHOP_WIDTH - 36.0, WARE_HEIGHT));
            handles.ware_frames[row] = frame;
            tree.in_parent(frame, |tree| {
                handles.ware_icons[row] =
                    item_icon(tree, icons, vec2(6.0, 6.0), WARE_HEIGHT - 12.0);
                handles.ware_names[row] = text_line(
                    tree,
                    vec2(WARE_HEIGHT + 4.0, 10.0),
                    SHOP_WIDTH - WARE_HEIGHT - 50.0,
                    ROW_SIZE,
                    ROW_TEXT,
                );
            });
        }
        handles.shop_note = text_line(
            tree,
            vec2(20.0, SHOP_HEIGHT - 46.0),
            SHOP_WIDTH - 40.0,
            ROW_SIZE,
            GOLD_TEXT,
        );
    });

    handles.log_root = hidden_panel(
        tree,
        icons,
        Rl(vec2(50.0, 50.0)),
        vec2(LOG_WIDTH, LOG_HEIGHT),
    );
    tree.in_parent(handles.log_root, |tree| {
        title_line(tree, LOG_WIDTH, "Quest Log");
        for row in 0..LOG_ROWS {
            handles.log_rows[row] = text_line(
                tree,
                vec2(22.0, TITLE_HEIGHT + 22.0 + row as f32 * ROW_HEIGHT),
                LOG_WIDTH - 44.0,
                ROW_SIZE,
                ROW_TEXT,
            );
        }
    });
}

pub fn tend(
    actions: Res<Actions<Ask>>,
    resources: Res<ThornmereResources>,
    mut panels: ResMut<Panels>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    players: Query<(&LocalTransform,), With<Cooldowns>>,
    merchants: Query<(&Merchant, &LocalTransform)>,
) {
    if !panels.built {
        return;
    }
    if actions.just_pressed(Ask::Sheet) {
        panels.sheet_open = !panels.sheet_open;
        panels.note = String::new();
        if panels.sheet_open {
            panels.shop = None;
        }
    }
    if actions.just_pressed(Ask::Bag) {
        panels.bag_open = !panels.bag_open;
        panels.note = String::new();
    }
    if actions.just_pressed(Ask::Log) {
        panels.log_open = !panels.log_open;
    }

    if let Some(shop) = panels.shop {
        let mut position = None;
        players.for_each(|_entity, (transform,)| position = Some(transform.translation));
        let mut spot = None;
        merchants.for_each(|_entity, (merchant, transform)| {
            if merchant.0 == shop {
                spot = Some(transform.translation);
            }
        });
        let near = match (position, spot) {
            (Some(position), Some(spot)) => flat_distance(position, spot) < MERCHANT_REACH * 2.0,
            _ => false,
        };
        if !near {
            panels.shop = None;
        }
    }
    if resources.player.is_none() {
        panels.shop = None;
    }

    let panels = &mut *panels;
    let sheet = panels.sheet_open && panels.shop.is_none();
    if panels.shown_sheet != sheet {
        ui_set_visible(&mut dirty, &mut commands, panels.handles.sheet_root, sheet);
        panels.shown_sheet = sheet;
    }
    if panels.shown_bag != panels.bag_open {
        ui_set_visible(
            &mut dirty,
            &mut commands,
            panels.handles.bag_root,
            panels.bag_open,
        );
        panels.shown_bag = panels.bag_open;
    }
    if panels.shown_log != panels.log_open {
        ui_set_visible(
            &mut dirty,
            &mut commands,
            panels.handles.log_root,
            panels.log_open,
        );
        panels.shown_log = panels.log_open;
    }
    let shop = panels.shop.is_some();
    if panels.shown_shop != shop {
        ui_set_visible(&mut dirty, &mut commands, panels.handles.shop_root, shop);
        panels.shown_shop = shop;
    }
}

type Fighters<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item mut Health,
        &'item MaxHealth,
        &'item mut Mana,
        &'item MaxMana,
        &'item Level,
    ),
    With<Cooldowns>,
>;

pub fn touch(
    frame: Res<RetainedUiFrame>,
    mut resources: ResMut<ThornmereResources>,
    mut panels: ResMut<Panels>,
    mut cues: ResMut<Cues>,
    mut players: Fighters<'_, '_>,
) {
    if !panels.built {
        return;
    }
    let clicks: Vec<Entity> = ui_button_clicks(&frame).collect();
    if clicks.is_empty() {
        return;
    }
    let resources = &mut *resources;
    let panels = &mut *panels;

    let mut player_level = 1;
    players.for_each_mut(|_entity, (_, _, _, _, level)| player_level = level.0);

    if let Some(place) = panels
        .handles
        .wear_frames
        .iter()
        .position(|frame| clicks.contains(frame))
    {
        match strip(&mut resources.bag, &mut resources.gear, place) {
            Some(item) => {
                panels.note = format!("You take off the {}.", ITEMS[item].name);
                cue(&mut cues, SOUND_SLOT, SOUND_QUIET);
            }
            None => {
                cue(&mut cues, SOUND_DENY, SOUND_QUIET);
            }
        }
        return;
    }

    if let Some(row) = panels
        .handles
        .ware_frames
        .iter()
        .position(|frame| clicks.contains(frame))
        && let Some(shop) = panels.shop
    {
        let wares = MERCHANTS[shop].wares;
        let Some(name) = wares.get(row) else {
            return;
        };
        let Some(item) = item_index(name) else {
            return;
        };
        let price = buy_price(item);
        if resources.purse < price {
            panels.note = format!("{} costs {}.", ITEMS[item].name, coin_text(price));
            cue(&mut cues, SOUND_DENY, SOUND_QUIET);
            return;
        }
        if !bag_room(&resources.bag, item) {
            panels.note = "The bag is full.".to_string();
            cue(&mut cues, SOUND_DENY, SOUND_QUIET);
            return;
        }
        resources.purse -= price;
        stow(&mut resources.bag, item, 1);
        panels.note = format!("Bought {} for {}.", ITEMS[item].name, coin_text(price));
        system_line(
            &mut resources.chat,
            format!(
                "You buy {} from {} for {}.",
                ITEMS[item].name,
                MERCHANTS[shop].name,
                coin_text(price)
            ),
        );
        cue(&mut cues, SOUND_ACCEPT, SOUND_PLAIN);
        return;
    }

    let Some(slot) = panels
        .handles
        .bag_frames
        .iter()
        .position(|frame| clicks.contains(frame))
    else {
        return;
    };
    let Some(stack) = resources.bag.slots[slot] else {
        return;
    };
    let item = &ITEMS[stack.item];

    if panels.shop.is_some() {
        let Some(taken) = take_stack(&mut resources.bag, slot) else {
            return;
        };
        let paid = item.value * taken.count;
        resources.purse += paid;
        panels.note = format!(
            "Sold {} x{} for {}.",
            item.name,
            taken.count,
            coin_text(paid)
        );
        system_line(
            &mut resources.chat,
            format!(
                "You sell {} x{} for {}.",
                item.name,
                taken.count,
                coin_text(paid)
            ),
        );
        cue(&mut cues, SOUND_ACCEPT, SOUND_QUIET);
        return;
    }

    match item.sip {
        Sip::Health(amount) => {
            let mut drank = false;
            players.for_each_mut(|_entity, (health, max_health, _, _, _)| {
                if health.0 < max_health.0 {
                    health.0 = (health.0 + amount as f32).min(max_health.0);
                    drank = true;
                }
            });
            if drank {
                take_one(&mut resources.bag, slot);
                panels.note = format!("You drink the {}.", item.name);
                cue(&mut cues, SOUND_DONE, SOUND_QUIET);
            } else {
                panels.note = item_note(stack.item);
                cue(&mut cues, SOUND_DENY, SOUND_QUIET);
            }
            return;
        }
        Sip::Mana(amount) => {
            let mut drank = false;
            players.for_each_mut(|_entity, (_, _, mana, max_mana, _)| {
                if mana.0 < max_mana.0 {
                    mana.0 = (mana.0 + amount as f32).min(max_mana.0);
                    drank = true;
                }
            });
            if drank {
                take_one(&mut resources.bag, slot);
                panels.note = format!("You drink the {}.", item.name);
                cue(&mut cues, SOUND_DONE, SOUND_QUIET);
            } else {
                panels.note = item_note(stack.item);
                cue(&mut cues, SOUND_DENY, SOUND_QUIET);
            }
            return;
        }
        Sip::None => {}
    }

    if item.wear == Wear::Carried {
        panels.note = format!("{}   {}", item.name, item_note(stack.item));
        cue(&mut cues, SOUND_SLOT, SOUND_QUIET);
        return;
    }
    if item.level > player_level {
        panels.note = format!("{} needs level {}.", item.name, item.level);
        cue(&mut cues, SOUND_DENY, SOUND_QUIET);
        return;
    }
    let Some((worn, _)) = wear_item(&mut resources.bag, &mut resources.gear, slot) else {
        cue(&mut cues, SOUND_DENY, SOUND_QUIET);
        return;
    };
    panels.note = format!("You put on the {}.", ITEMS[worn].name);
    system_line(
        &mut resources.chat,
        format!("You put on the {}.", ITEMS[worn].name),
    );
    cue(&mut cues, SOUND_ACCEPT, SOUND_QUIET);
}

type Sheets<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item Level,
        &'item MaxHealth,
        &'item MaxMana,
        &'item Strength,
        &'item Kills,
        &'item Experience,
    ),
    With<Cooldowns>,
>;

pub fn refresh(
    mut resources: ResMut<ThornmereResources>,
    mut panels: ResMut<Panels>,
    icons: Res<Icons>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    players: Sheets<'_, '_>,
) {
    if !panels.built {
        return;
    }
    let resources = &mut *resources;
    let panels = &mut *panels;
    let mut wanted = PanelText::default();

    if panels.sheet_open {
        let mut level = 1;
        let mut max_health = 0.0;
        let mut max_mana = 0.0;
        let mut strength = 0.0;
        let mut kills = 0;
        let mut experience = 0;
        players.for_each(|_entity, (rank, health, mana, might, tally, earned)| {
            level = rank.0;
            max_health = health.0;
            max_mana = mana.0;
            strength = might.0;
            kills = tally.0;
            experience = earned.0;
        });
        let (power, guard, spirit) = gear_totals(&resources.gear);
        let quests = resources
            .quests
            .finished
            .iter()
            .filter(|done| **done)
            .count();
        let places = resources
            .clock
            .discovered
            .iter()
            .filter(|seen| **seen)
            .count();
        wanted.sheet = vec![
            (resources.player_name.clone(), PANEL_TITLE),
            (
                format!("Level {level}   {}", CALLINGS[resources.class].name),
                ROW_TEXT,
            ),
            (format!("Health {max_health:.0}"), ROW_TEXT),
            (format!("Mana {max_mana:.0}"), ROW_TEXT),
            (format!("Strength {strength:.0}"), ROW_TEXT),
            (
                format!("Gear   +{power:.0} strength   +{guard:.0} health   +{spirit:.0} mana"),
                DIM_TEXT,
            ),
            (
                format!("Experience {experience} of {}", level_threshold(level)),
                ROW_TEXT,
            ),
            (format!("Kills {kills}"), ROW_TEXT),
            (format!("Coins {}", coin_text(resources.purse)), GOLD_TEXT),
            (
                format!("Quests done {quests} of {}", QUESTS.len()),
                ROW_TEXT,
            ),
            (
                format!("Places found {places} of {}", PLACES.len()),
                ROW_TEXT,
            ),
            (String::new(), ROW_TEXT),
        ];
        wanted.wear = WEAR_ORDER
            .iter()
            .enumerate()
            .map(|(place, wear)| match resources.gear.worn[place] {
                Some(item) => (ITEMS[item].name.to_string(), ITEMS[item].grade.color()),
                None => (wear.name().to_string(), DIM_TEXT),
            })
            .collect();
    }

    if panels.bag_open {
        wanted.bag = (0..BAG_SLOTS)
            .map(|slot| match resources.bag.slots[slot] {
                Some(stack) => (
                    ITEMS[stack.item].icon.to_string(),
                    if stack.count > 1 {
                        stack.count.to_string()
                    } else {
                        String::new()
                    },
                    ITEMS[stack.item].grade.color(),
                ),
                None => (String::new(), String::new(), EMPTY_ICON),
            })
            .collect();
        wanted.bag_note = if panels.note.is_empty() {
            format!("{} of {} full", bag_used(&resources.bag), BAG_SLOTS)
        } else {
            panels.note.clone()
        };
    }

    if let Some(shop) = panels.shop {
        let data = &MERCHANTS[shop];
        wanted.shop_title = format!("{}   {}", data.name, data.title);
        wanted.shop_note = if panels.note.is_empty() {
            coin_text(resources.purse)
        } else {
            panels.note.clone()
        };
        wanted.wares = (0..WARE_ROWS)
            .map(|row| match data.wares.get(row) {
                Some(name) => match item_index(name) {
                    Some(item) => (
                        ITEMS[item].icon.to_string(),
                        format!("{}   {}", ITEMS[item].name, coin_text(buy_price(item))),
                        ITEMS[item].grade.color(),
                    ),
                    None => (String::new(), String::new(), EMPTY_ICON),
                },
                None => (String::new(), String::new(), EMPTY_ICON),
            })
            .collect();
    }

    if panels.log_open {
        let mut rows = Vec::new();
        for state in resources.quests.active.iter() {
            let quest = &QUESTS[state.quest];
            rows.push((quest.title.to_string(), PANEL_TITLE));
            let line = match quest.objective {
                Objective::Kill { prey, count } => {
                    format!("Slay {prey}   {} of {count}", state.progress.min(count))
                }
                Objective::Gather { pickup, count } => format!(
                    "Gather {}   {} of {count}",
                    PICKUPS[pickup].name,
                    state.progress.min(count)
                ),
                Objective::Deliver { giver } => {
                    format!("Speak with {}", GIVERS[giver].name)
                }
                Objective::Discover { place } => format!("Find {}", PLACES[place].name),
            };
            let line = if state.complete {
                format!(
                    "{line}   ready for {}",
                    GIVERS[turn_in_giver(state.quest)].name
                )
            } else {
                line
            };
            rows.push((line, ROW_TEXT));
            rows.push((
                format!(
                    "{} experience   {}",
                    quest.experience,
                    coin_text(quest.coins)
                ),
                GOLD_TEXT,
            ));
            rows.push((String::new(), ROW_TEXT));
        }
        let done = resources
            .quests
            .finished
            .iter()
            .filter(|done| **done)
            .count();
        rows.push((format!("{done} of {} done", QUESTS.len()), DIM_TEXT));
        rows.truncate(LOG_ROWS);
        while rows.len() < LOG_ROWS {
            rows.push((String::new(), ROW_TEXT));
        }
        wanted.log = rows;
    }

    if wanted == panels.shown {
        return;
    }

    write_rows(
        &mut commands,
        &panels.handles.sheet_rows,
        &panels.shown.sheet,
        &wanted.sheet,
    );
    for (place, value) in wanted.wear.iter().enumerate() {
        if panels.shown.wear.get(place) == Some(value) {
            continue;
        }
        ui_set_text(&mut commands, panels.handles.wear_names[place], &value.0);
        ui_set_color::<UiBase>(&mut commands, panels.handles.wear_names[place], value.1);
        let icon = resources.gear.worn[place].map(|item| ITEMS[item].icon);
        set_icon(
            &mut dirty,
            &mut commands,
            &icons,
            panels.handles.wear_icons[place],
            icon,
            value.1,
        );
    }
    for (slot, value) in wanted.bag.iter().enumerate() {
        if panels.shown.bag.get(slot) == Some(value) {
            continue;
        }
        ui_set_text(&mut commands, panels.handles.bag_counts[slot], &value.1);
        let icon = if value.0.is_empty() {
            None
        } else {
            Some(value.0.as_str())
        };
        set_icon(
            &mut dirty,
            &mut commands,
            &icons,
            panels.handles.bag_icons[slot],
            icon,
            value.2,
        );
    }
    if panels.shown.bag_note != wanted.bag_note {
        ui_set_text(&mut commands, panels.handles.bag_note, &wanted.bag_note);
    }
    if panels.shown.shop_title != wanted.shop_title {
        ui_set_text(&mut commands, panels.handles.shop_title, &wanted.shop_title);
    }
    if panels.shown.shop_note != wanted.shop_note {
        ui_set_text(&mut commands, panels.handles.shop_note, &wanted.shop_note);
    }
    for (row, value) in wanted.wares.iter().enumerate() {
        if panels.shown.wares.get(row) == Some(value) {
            continue;
        }
        ui_set_text(&mut commands, panels.handles.ware_names[row], &value.1);
        ui_set_color::<UiBase>(&mut commands, panels.handles.ware_names[row], value.2);
        let icon = if value.0.is_empty() {
            None
        } else {
            Some(value.0.as_str())
        };
        set_icon(
            &mut dirty,
            &mut commands,
            &icons,
            panels.handles.ware_icons[row],
            icon,
            value.2,
        );
    }
    write_rows(
        &mut commands,
        &panels.handles.log_rows,
        &panels.shown.log,
        &wanted.log,
    );
    panels.shown = wanted;
}

fn write_rows(
    commands: &mut Commands,
    nodes: &[Entity],
    shown: &[(String, Vec4)],
    wanted: &[(String, Vec4)],
) {
    for (row, value) in wanted.iter().enumerate() {
        if shown.get(row) == Some(value) {
            continue;
        }
        let Some(node) = nodes.get(row) else {
            continue;
        };
        ui_set_text(commands, *node, &value.0);
        ui_set_color::<UiBase>(commands, *node, value.1);
    }
}

fn set_icon(
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
    icons: &Icons,
    node: Entity,
    icon: Option<&str>,
    tint: Vec4,
) {
    let layer = icons.item_layer.unwrap_or(0);
    let (uv_min, uv_max) =
        match icon.and_then(|name| ITEM_ICONS.iter().position(|(item, _)| *item == name)) {
            Some(slot) => icons
                .item_cells
                .get(slot)
                .copied()
                .unwrap_or((vec2(0.0, 0.0), vec2(1.0, 1.0))),
            None => (vec2(0.0, 0.0), vec2(0.0, 0.0)),
        };
    let color = if icon.is_some() {
        Vec4::new(tint.x, tint.y, tint.z, 1.0)
    } else {
        EMPTY_ICON
    };
    ui_mark_render_dirty(dirty);
    commands.queue(move |storage, _| {
        if let Some(content) = storage.get_mut::<UiNodeContent>(node) {
            *content = UiNodeContent::Image {
                texture_index: layer,
                uv_min,
                uv_max,
            };
        }
    });
    ui_set_color::<UiBase>(commands, node, color);
}
