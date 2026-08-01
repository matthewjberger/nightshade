use crate::components::{
    Cooldowns, Experience, Giver, Health, Level, Mana, MaxHealth, MaxMana, Rank, Recovery, Target,
    Task, Voice, Worth,
};
use crate::contract::Channel;
use crate::data::{
    ABILITIES, ABILITY_COUNT, BANNER_FADE, BANNER_SECONDS, CALLINGS, CHAT_ROWS, Calling, ELITE,
    FRAME_BANDS, FRAME_CORNER, GIVERS, LOW_HEALTH_FRACTION, MINIMAP_DOTS, MINIMAP_RANGE,
    MINIMAP_SIZE, MINIMAP_STEP, Objective, PICKUPS, PLACES, QUESTS,
};
use crate::queries::{coin_text, hour_text, is_alive, level_threshold};
use crate::resources::{Banners, Feel, HudHandles, HudText, Icons, QUEST_ROWS, ThornmereResources};
use crate::systems::quests::turn_in_giver;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_render::data::config::RendererState;
use nightshade_scene::prelude::{LocalTransform, Name};
use nightshade_ui::components::layout_types::WindowLayout;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;

const PANEL_BACKGROUND: Vec4 = Vec4::new(0.05, 0.05, 0.08, 0.92);
const PANEL_BORDER: Vec4 = Vec4::new(0.80, 0.68, 0.40, 0.65);
const BAR_BACKGROUND: Vec4 = Vec4::new(0.10, 0.10, 0.13, 0.95);
const BAR_BORDER: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.8);
const TEXT_BRIGHT: Vec4 = Vec4::new(0.96, 0.96, 0.98, 1.0);
const TEXT_DIM: Vec4 = Vec4::new(0.88, 0.90, 0.94, 1.0);
const HEALTH_COLOR: Vec4 = Vec4::new(0.30, 0.82, 0.32, 1.0);
const MANA_COLOR: Vec4 = Vec4::new(0.30, 0.50, 1.0, 1.0);
const EXPERIENCE_COLOR: Vec4 = Vec4::new(0.62, 0.35, 0.90, 1.0);
const QUEST_COLOR: Vec4 = Vec4::new(0.98, 0.84, 0.42, 1.0);
const TARGET_COLOR: Vec4 = Vec4::new(1.0, 0.55, 0.45, 1.0);
const TARGET_HEALTH_COLOR: Vec4 = Vec4::new(0.85, 0.25, 0.20, 1.0);
const ELITE_COLOR: Vec4 = Vec4::new(1.0, 0.78, 0.30, 1.0);
const READY_COLOR: Vec4 = Vec4::new(0.55, 1.0, 0.65, 1.0);
const COOLING_COLOR: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);
const DRAINED_COLOR: Vec4 = Vec4::new(0.55, 0.62, 0.95, 1.0);
const GOLD_COLOR: Vec4 = Vec4::new(1.0, 0.85, 0.30, 1.0);
const ICON_TINT: Vec4 = Vec4::new(0.95, 0.92, 0.85, 1.0);
const ICON_DRAINED: Vec4 = Vec4::new(0.45, 0.50, 0.75, 1.0);
const SLOT_SHADE: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.62);
const SLOT_FLASH: Vec4 = Vec4::new(1.0, 0.95, 0.6, 0.9);
const WIPE_COLOR: Vec4 = Vec4::new(1.0, 0.93, 0.60, 0.95);
const HURT_COLOR: Vec4 = Vec4::new(0.7, 0.05, 0.02, 1.0);
const COMBAT_COLOR: Vec4 = Vec4::new(1.0, 0.35, 0.30, 1.0);
const MINIMAP_GROUND: Vec4 = Vec4::new(0.16, 0.24, 0.14, 0.92);
const DOT_PLAYER: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);
const DOT_GIVER: Vec4 = Vec4::new(1.0, 0.9, 0.35, 1.0);
const DOT_MOB: Vec4 = Vec4::new(1.0, 0.3, 0.25, 1.0);
const DOT_ELITE: Vec4 = Vec4::new(1.0, 0.7, 0.2, 1.0);
const DOT_PERSON: Vec4 = Vec4::new(0.4, 0.7, 1.0, 1.0);

const MARGIN: f32 = 16.0;
const FRAME_WIDTH: f32 = 300.0;
const FRAME_HEIGHT: f32 = 110.0;
const PORTRAIT: f32 = 48.0;
const BAR_HEIGHT: f32 = 22.0;
const TRACKER_WIDTH: f32 = 330.0;
const TRACKER_ROW: f32 = 26.0;
const EXPERIENCE_WIDTH: f32 = 660.0;
const EXPERIENCE_HEIGHT: f32 = 16.0;
const SLOT_SIZE: f32 = 64.0;
const SLOT_GAP: f32 = 8.0;
const SLOT_LABEL: f32 = 22.0;
const CHAT_WIDTH: f32 = 640.0;
const CHAT_LINE: f32 = 26.0;
const PURSE_WIDTH: f32 = 200.0;
const PURSE_HEIGHT: f32 = 40.0;
const DOT: f32 = 7.0;
const BAR_STEPS: f32 = 200.0;
const SHADE_STEPS: f32 = 60.0;
const WIPE_HEIGHT: f32 = 3.0;

pub const HUD_ICONS: [&str; 18] = [
    "cross",
    "exclamation",
    "locked",
    "power",
    "checkmark",
    "warning",
    "fightFist",
    "star",
    "diamond",
    "plus",
    "return",
    "fastForward",
    "coin",
    "massiveMultiplayer",
    "figurine",
    "target",
    "trophy",
    "panel-border-000",
];

fn icon_slot(name: &str) -> usize {
    HUD_ICONS.iter().position(|icon| *icon == name).unwrap_or(0)
}

fn image(
    tree: &mut UiTreeBuilder,
    icons: &Icons,
    name: &str,
    at: Vec2,
    size: f32,
    tint: Vec4,
) -> Entity {
    let slot = icon_slot(name);
    let (uv_min, uv_max) = icons
        .cells
        .get(slot)
        .copied()
        .unwrap_or((vec2(0.0, 0.0), vec2(1.0, 1.0)));
    tree.add_node()
        .window(Ab(at), Ab(vec2(size, size)), Anchor::TopLeft)
        .with_image_uv(icons.layer.unwrap_or(0), uv_min, uv_max)
        .color_raw::<UiBase>(tint)
        .without_pointer_events()
        .entity()
}

pub fn build(tree: &mut UiTreeBuilder, icons: &Icons, class: &Calling) -> HudHandles {
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();

    let mut handles = HudHandles::default();

    tree.in_parent(root, |tree| {
        handles.hurt_overlay = tree
            .add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
            .color_raw::<UiBase>(Vec4::new(HURT_COLOR.x, HURT_COLOR.y, HURT_COLOR.z, 0.0))
            .without_pointer_events()
            .entity();

        let player = panel(
            tree,
            icons,
            Ab(vec2(MARGIN, MARGIN)),
            vec2(FRAME_WIDTH, FRAME_HEIGHT),
            Anchor::TopLeft,
        );
        tree.in_parent(player, |tree| {
            image(
                tree,
                icons,
                "figurine",
                vec2(10.0, 10.0),
                PORTRAIT,
                ICON_TINT,
            );
            handles.player_name = line_at(
                tree,
                PORTRAIT + 20.0,
                8.0,
                22.0,
                FRAME_WIDTH - PORTRAIT - 20.0,
                TEXT_BRIGHT,
                "",
            );
            let (fill, text) = bar(
                tree,
                PORTRAIT + 20.0,
                42.0,
                FRAME_WIDTH - PORTRAIT - 30.0,
                HEALTH_COLOR,
            );
            handles.player_health_fill = fill;
            handles.player_health = text;
            let (fill, text) = bar(
                tree,
                PORTRAIT + 20.0,
                72.0,
                FRAME_WIDTH - PORTRAIT - 30.0,
                MANA_COLOR,
            );
            handles.player_mana_fill = fill;
            handles.player_mana = text;
            handles.combat_label = tree
                .add_node()
                .window(
                    Ab(vec2(8.0, 74.0)),
                    Ab(vec2(PORTRAIT + 8.0, 24.0)),
                    Anchor::TopLeft,
                )
                .with_text("Combat", 14.0)
                .color_raw::<UiBase>(COMBAT_COLOR)
                .without_pointer_events()
                .with_visible(false)
                .entity();
        });

        handles.target_panel = tree
            .add_node()
            .window(
                Ab(vec2(MARGIN + FRAME_WIDTH + 10.0, MARGIN)),
                Ab(vec2(FRAME_WIDTH, FRAME_HEIGHT - 28.0)),
                Anchor::TopLeft,
            )
            .with_rect(6.0, 1.0, PANEL_BORDER)
            .color_raw::<UiBase>(PANEL_BACKGROUND)
            .without_pointer_events()
            .with_visible(false)
            .entity();
        tree.in_parent(handles.target_panel, |tree| {
            frame(tree, icons.frame_layer, icons.tint, FRAME_CORNER);
            image(tree, icons, "target", vec2(10.0, 10.0), PORTRAIT, ICON_TINT);
            handles.target_name = line_at(
                tree,
                PORTRAIT + 20.0,
                8.0,
                22.0,
                FRAME_WIDTH - PORTRAIT - 20.0,
                TARGET_COLOR,
                "",
            );
            let (fill, text) = bar(
                tree,
                PORTRAIT + 20.0,
                46.0,
                FRAME_WIDTH - PORTRAIT - 30.0,
                TARGET_HEALTH_COLOR,
            );
            handles.target_health_fill = fill;
            handles.target_health = text;
        });

        let map_top = MARGIN + 40.0;
        let zone_panel = panel(
            tree,
            icons,
            Rl(vec2(100.0, 0.0)) + Ab(vec2(-MARGIN, MARGIN)),
            vec2(MINIMAP_SIZE, 34.0),
            Anchor::TopRight,
        );
        tree.in_parent(zone_panel, |tree| {
            handles.zone_name = tree
                .add_node()
                .window(
                    Ab(vec2(0.0, 3.0)),
                    Ab(vec2(MINIMAP_SIZE, 28.0)),
                    Anchor::TopLeft,
                )
                .with_text("", 19.0)
                .color_raw::<UiBase>(QUEST_COLOR)
                .without_pointer_events()
                .entity();
        });
        handles.minimap = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-MARGIN, map_top)),
                Ab(vec2(MINIMAP_SIZE, MINIMAP_SIZE)),
                Anchor::TopRight,
            )
            .with_rect(MINIMAP_SIZE / 2.0, 2.0, PANEL_BORDER)
            .color_raw::<UiBase>(MINIMAP_GROUND)
            .without_pointer_events()
            .entity();
        tree.in_parent(handles.minimap, |tree| {
            for _ in 0..MINIMAP_DOTS {
                let dot = tree
                    .add_node()
                    .window(Ab(vec2(0.0, 0.0)), Ab(vec2(DOT, DOT)), Anchor::TopLeft)
                    .with_rect(DOT / 2.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                    .color_raw::<UiBase>(DOT_MOB)
                    .without_pointer_events()
                    .with_visible(false)
                    .entity();
                handles.minimap_dots.push(dot);
            }
            let center = MINIMAP_SIZE / 2.0 - DOT / 2.0;
            tree.add_node()
                .window(
                    Ab(vec2(center, center)),
                    Ab(vec2(DOT + 2.0, DOT + 2.0)),
                    Anchor::TopLeft,
                )
                .with_rect(DOT / 2.0 + 1.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                .color_raw::<UiBase>(DOT_PLAYER)
                .without_pointer_events()
                .entity();
        });
        let clock_top = map_top + MINIMAP_SIZE + 10.0;
        handles.zone_clock = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-MARGIN, clock_top)),
                Ab(vec2(MINIMAP_SIZE, 24.0)),
                Anchor::TopRight,
            )
            .with_text("", 16.0)
            .color_raw::<UiBase>(TEXT_DIM)
            .without_pointer_events()
            .entity();

        let tracker_height = TRACKER_ROW * (QUEST_ROWS as f32 + 1.0) + 16.0;
        let tracker = panel(
            tree,
            icons,
            Rl(vec2(100.0, 0.0)) + Ab(vec2(-MARGIN, clock_top + 34.0)),
            vec2(TRACKER_WIDTH, tracker_height),
            Anchor::TopRight,
        );
        tree.in_parent(tracker, |tree| {
            line(tree, 8.0, 20.0, TRACKER_WIDTH, QUEST_COLOR, "Objectives");
            for index in 0..QUEST_ROWS {
                let top = 8.0 + TRACKER_ROW * (index as f32 + 1.0);
                handles.quest_rows[index] = line(tree, top, 17.0, TRACKER_WIDTH, TEXT_DIM, "");
            }
        });

        let bar_bottom = MARGIN + SLOT_SIZE + SLOT_LABEL + 10.0;
        let experience = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -bar_bottom)),
                Ab(vec2(EXPERIENCE_WIDTH, EXPERIENCE_HEIGHT)),
                Anchor::BottomCenter,
            )
            .with_rect(4.0, 1.0, BAR_BORDER)
            .color_raw::<UiBase>(BAR_BACKGROUND)
            .without_pointer_events()
            .entity();
        tree.in_parent(experience, |tree| {
            handles.experience_fill = tree
                .add_node()
                .window(
                    Ab(vec2(0.0, 0.0)),
                    Ab(vec2(0.0, EXPERIENCE_HEIGHT)),
                    Anchor::TopLeft,
                )
                .with_rect(4.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                .color_raw::<UiBase>(EXPERIENCE_COLOR)
                .without_pointer_events()
                .entity();
        });
        handles.experience_line = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -bar_bottom - EXPERIENCE_HEIGHT - 2.0)),
                Ab(vec2(EXPERIENCE_WIDTH, 22.0)),
                Anchor::BottomCenter,
            )
            .with_text("", 15.0)
            .color_raw::<UiBase>(TEXT_DIM)
            .without_pointer_events()
            .entity();

        let picks = class.abilities;
        let bar_width = SLOT_SIZE * ABILITY_COUNT as f32 + SLOT_GAP * (ABILITY_COUNT as f32 - 1.0);
        for (index, &pick) in picks.iter().enumerate() {
            let ability = &ABILITIES[pick];
            let offset = -bar_width / 2.0 + SLOT_SIZE / 2.0 + index as f32 * (SLOT_SIZE + SLOT_GAP);
            let slot = tree
                .add_node()
                .window(
                    Rl(vec2(50.0, 100.0)) + Ab(vec2(offset, -MARGIN - SLOT_LABEL)),
                    Ab(vec2(SLOT_SIZE, SLOT_SIZE)),
                    Anchor::BottomCenter,
                )
                .with_rect(6.0, 2.0, PANEL_BORDER)
                .color_raw::<UiBase>(PANEL_BACKGROUND)
                .without_pointer_events()
                .entity();
            handles.slot_frames[index] = slot;
            tree.in_parent(slot, |tree| {
                handles.slot_names[index] = image(
                    tree,
                    icons,
                    ability.icon,
                    vec2(7.0, 7.0),
                    SLOT_SIZE - 14.0,
                    ICON_TINT,
                );
                handles.slot_shades[index] = tree
                    .add_node()
                    .window(
                        Ab(vec2(0.0, 0.0)),
                        Ab(vec2(SLOT_SIZE, 0.0)),
                        Anchor::TopLeft,
                    )
                    .with_rect(6.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                    .color_raw::<UiBase>(SLOT_SHADE)
                    .without_pointer_events()
                    .entity();
                handles.slot_wipes[index] = tree
                    .add_node()
                    .window(
                        Ab(vec2(0.0, 0.0)),
                        Ab(vec2(SLOT_SIZE, WIPE_HEIGHT)),
                        Anchor::TopLeft,
                    )
                    .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                    .color_raw::<UiBase>(WIPE_COLOR)
                    .without_pointer_events()
                    .with_visible(false)
                    .entity();
                handles.slot_status[index] = tree
                    .add_node()
                    .window(
                        Ab(vec2(0.0, SLOT_SIZE / 2.0 - 13.0)),
                        Ab(vec2(SLOT_SIZE, 26.0)),
                        Anchor::TopLeft,
                    )
                    .with_text("", 21.0)
                    .color_raw::<UiBase>(COOLING_COLOR)
                    .without_pointer_events()
                    .entity();
            });
            tree.add_node()
                .window(
                    Rl(vec2(50.0, 100.0)) + Ab(vec2(offset, -MARGIN)),
                    Ab(vec2(SLOT_SIZE + SLOT_GAP, SLOT_LABEL)),
                    Anchor::BottomCenter,
                )
                .with_text(ability.name, 12.0)
                .color_raw::<UiBase>(TEXT_BRIGHT)
                .without_pointer_events()
                .entity();
        }

        let chat_height = CHAT_LINE * CHAT_ROWS as f32 + 18.0;
        let chat = tree
            .add_node()
            .window(
                Rl(vec2(0.0, 100.0)) + Ab(vec2(MARGIN, -MARGIN)),
                Ab(vec2(CHAT_WIDTH, chat_height)),
                Anchor::BottomLeft,
            )
            .with_rect(6.0, 1.0, PANEL_BORDER)
            .color_raw::<UiBase>(PANEL_BACKGROUND)
            .entity();
        handles.chat_panel = chat;
        tree.in_parent(chat, |tree| {
            frame(tree, icons.frame_layer, icons.tint, FRAME_CORNER);
            for index in 0..CHAT_ROWS {
                let top = 8.0 + CHAT_LINE * index as f32;
                handles.chat_rows[index] = line(tree, top, 17.0, CHAT_WIDTH, TEXT_DIM, "");
            }
        });

        let purse = panel(
            tree,
            icons,
            Rl(vec2(100.0, 100.0)) + Ab(vec2(-MARGIN, -MARGIN)),
            vec2(PURSE_WIDTH, PURSE_HEIGHT),
            Anchor::BottomRight,
        );
        tree.in_parent(purse, |tree| {
            image(tree, icons, "coin", vec2(8.0, 7.0), 26.0, GOLD_COLOR);
            handles.purse = line_at(tree, 40.0, 8.0, 19.0, PURSE_WIDTH - 44.0, GOLD_COLOR, "");
        });

        handles.banner_title = tree
            .add_node()
            .window(Rl(vec2(50.0, 18.0)), Ab(vec2(900.0, 58.0)), Anchor::Center)
            .with_text("", 44.0)
            .color_raw::<UiBase>(Vec4::new(GOLD_COLOR.x, GOLD_COLOR.y, GOLD_COLOR.z, 0.0))
            .without_pointer_events()
            .entity();
        handles.banner_subtitle = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 18.0)) + Ab(vec2(0.0, 36.0)),
                Ab(vec2(900.0, 32.0)),
                Anchor::Center,
            )
            .with_text("", 22.0)
            .color_raw::<UiBase>(Vec4::new(TEXT_BRIGHT.x, TEXT_BRIGHT.y, TEXT_BRIGHT.z, 0.0))
            .without_pointer_events()
            .entity();
    });

    handles.root = root;
    handles
}

pub fn frame(tree: &mut UiTreeBuilder, layer: Option<u32>, tint: Vec4, corner: f32) {
    let Some(layer) = layer else {
        return;
    };
    let band = 1.0 / FRAME_BANDS;
    for row in 0..3 {
        for column in 0..3 {
            if row == 1 && column == 1 {
                continue;
            }
            let position = Rl(vec2(
                if column == 2 { 100.0 } else { 0.0 },
                if row == 2 { 100.0 } else { 0.0 },
            )) + Ab(vec2(
                match column {
                    0 => 0.0,
                    1 => corner,
                    _ => -corner,
                },
                match row {
                    0 => 0.0,
                    1 => corner,
                    _ => -corner,
                },
            ));
            let size = Rl(vec2(
                if column == 1 { 100.0 } else { 0.0 },
                if row == 1 { 100.0 } else { 0.0 },
            )) + Ab(vec2(
                if column == 1 { -2.0 * corner } else { corner },
                if row == 1 { -2.0 * corner } else { corner },
            ));
            tree.add_node()
                .window(position, size, Anchor::TopLeft)
                .with_image_uv(
                    layer,
                    vec2(column as f32 * band, row as f32 * band),
                    vec2((column + 1) as f32 * band, (row + 1) as f32 * band),
                )
                .color_raw::<UiBase>(tint)
                .without_pointer_events()
                .entity();
        }
    }
}

pub fn panel(
    tree: &mut UiTreeBuilder,
    icons: &Icons,
    position: impl Into<UiValue<Vec2>>,
    size: Vec2,
    anchor: Anchor,
) -> Entity {
    let entity = tree
        .add_node()
        .window(position, Ab(size), anchor)
        .with_rect(6.0, 1.0, PANEL_BORDER)
        .color_raw::<UiBase>(PANEL_BACKGROUND)
        .without_pointer_events()
        .entity();
    tree.in_parent(entity, |tree| {
        frame(tree, icons.frame_layer, icons.tint, FRAME_CORNER)
    });
    entity
}

fn line(
    tree: &mut UiTreeBuilder,
    top: f32,
    size: f32,
    width: f32,
    color: Vec4,
    text: &str,
) -> Entity {
    line_at(tree, 10.0, top, size, width - 20.0, color, text)
}

fn line_at(
    tree: &mut UiTreeBuilder,
    left: f32,
    top: f32,
    size: f32,
    width: f32,
    color: Vec4,
    text: &str,
) -> Entity {
    tree.add_node()
        .window(
            Ab(vec2(left, top)),
            Ab(vec2(width, size + 8.0)),
            Anchor::TopLeft,
        )
        .with_text(text, size)
        .text_left()
        .color_raw::<UiBase>(color)
        .without_pointer_events()
        .entity()
}

fn bar(tree: &mut UiTreeBuilder, left: f32, top: f32, width: f32, color: Vec4) -> (Entity, Entity) {
    let frame = tree
        .add_node()
        .window(
            Ab(vec2(left, top)),
            Ab(vec2(width, BAR_HEIGHT)),
            Anchor::TopLeft,
        )
        .with_rect(4.0, 1.0, BAR_BORDER)
        .color_raw::<UiBase>(BAR_BACKGROUND)
        .without_pointer_events()
        .entity();
    let mut fill = Entity::default();
    let mut text = Entity::default();
    tree.in_parent(frame, |tree| {
        fill = tree
            .add_node()
            .window(
                Ab(vec2(0.0, 0.0)),
                Ab(vec2(width, BAR_HEIGHT)),
                Anchor::TopLeft,
            )
            .with_rect(4.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
            .color_raw::<UiBase>(color)
            .without_pointer_events()
            .entity();
        text = tree
            .add_node()
            .window(
                Ab(vec2(0.0, 0.0)),
                Ab(vec2(width, BAR_HEIGHT)),
                Anchor::TopLeft,
            )
            .with_text("", 15.0)
            .color_raw::<UiBase>(TEXT_BRIGHT)
            .without_pointer_events()
            .entity();
    });
    (fill, text)
}

fn set_fill(
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
    fill: Entity,
    width: f32,
    height: f32,
    fraction: f32,
) {
    ui_set_window_layout(
        dirty,
        commands,
        fill,
        WindowLayout {
            position: Ab(vec2(0.0, 0.0)).into(),
            size: Ab(vec2(width * fraction.clamp(0.0, 1.0), height)).into(),
            anchor: Anchor::TopLeft,
        },
    );
}

fn quantize(fraction: f32) -> u32 {
    (fraction.clamp(0.0, 1.0) * BAR_STEPS).round() as u32
}

pub fn channel_color(channel: Channel) -> Vec4 {
    match channel {
        Channel::General => Vec4::new(1.0, 0.78, 0.78, 1.0),
        Channel::Trade => Vec4::new(1.0, 0.72, 0.60, 1.0),
        Channel::Lfg => Vec4::new(0.82, 0.95, 0.68, 1.0),
        Channel::Whisper => Vec4::new(1.0, 0.55, 1.0, 1.0),
        Channel::Say => Vec4::new(1.0, 1.0, 1.0, 1.0),
        Channel::System => Vec4::new(1.0, 1.0, 0.45, 1.0),
        Channel::Combat => Vec4::new(1.0, 0.62, 0.42, 1.0),
    }
}

type PlayerFrame<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item Health,
        &'item MaxHealth,
        &'item Mana,
        &'item MaxMana,
        &'item Level,
        &'item Experience,
        &'item Cooldowns,
        &'item Target,
    ),
>;

pub fn refresh(
    mut resources: ResMut<ThornmereResources>,
    renderer_state: Res<RendererState>,
    feel: Res<Feel>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    mut players: PlayerFrame<'_, '_>,
    mut casters: Query<(&Recovery,), With<Cooldowns>>,
    mut mobs: Query<(&Health, &MaxHealth, &Level, &Rank, &Name), With<Worth>>,
) {
    let resources = &mut *resources;
    if resources.hud.root == Entity::default() {
        return;
    }
    let Some(player) = resources.player else {
        return;
    };

    let mut frame = None;
    players.for_entity(
        player,
        |(health, max_health, mana, max_mana, level, experience, cooldowns, target)| {
            frame = Some((
                health.0,
                max_health.0,
                mana.0,
                max_mana.0,
                level.0,
                experience.0,
                cooldowns.0,
                target.0,
            ));
        },
    );
    let Some((health, max_health, mana, max_mana, level, experience, cooldowns, target)) = frame
    else {
        return;
    };
    let mut recovery = 0.0;
    casters.for_entity(player, |(recovering,)| recovery = recovering.0);

    let mut target_visible = false;
    let mut target_name = String::new();
    let mut target_health = String::new();
    let mut target_fraction = 0.0;
    if let Some(target) = target {
        mobs.for_entity(target, |(health, max_health, level, rank, name)| {
            target_visible = true;
            let tag = if rank.0 >= ELITE { "  Elite" } else { "" };
            target_name = format!("{}  Lv {}{tag}", name.0, level.0);
            target_fraction = health.0 / max_health.0;
            target_health = if is_alive(health.0) {
                format!("{} / {}", health.0.ceil() as i32, max_health.0 as i32)
            } else {
                "Dead".to_string()
            };
        });
    }

    let hour = renderer_state.day_night.hour;
    let zone_name = resources
        .clock
        .last_place
        .map(|place| PLACES[place].name.to_string())
        .unwrap_or_else(|| "The Vale".to_string());

    let mut quest_rows = Vec::new();
    for state in &resources.quests.active {
        let quest = &QUESTS[state.quest];
        quest_rows.push(quest.title.to_string());
        let objective = if state.complete {
            format!("   Return to {}", GIVERS[turn_in_giver(state.quest)].name)
        } else {
            match quest.objective {
                Objective::Kill { prey, count } => {
                    format!("   {prey} slain  {}/{count}", state.progress)
                }
                Objective::Gather { pickup, count } => {
                    format!(
                        "   {} gathered  {}/{count}",
                        PICKUPS[pickup].name, state.progress
                    )
                }
                Objective::Deliver { giver } => {
                    format!("   Find {} at {}", GIVERS[giver].name, GIVERS[giver].title)
                }
                Objective::Discover { place } => {
                    format!("   Find {}", PLACES[place].name)
                }
            }
        };
        quest_rows.push(objective);
    }
    if quest_rows.is_empty() {
        quest_rows.push("   No quests. The board in town has work.".to_string());
    }
    quest_rows.truncate(QUEST_ROWS);

    let picks = CALLINGS[resources.class].abilities;
    let mut slot_status = Vec::new();
    let mut slot_shade = Vec::new();
    for (index, &pick) in picks.iter().enumerate() {
        let ability = &ABILITIES[pick];
        let status = if cooldowns[index] > 0.0 {
            (format!("{:.0}", cooldowns[index].ceil()), COOLING_COLOR)
        } else if mana < ability.mana {
            (String::new(), DRAINED_COLOR)
        } else if recovery > 0.0 {
            (String::new(), COOLING_COLOR)
        } else {
            (String::new(), READY_COLOR)
        };
        slot_status.push(status);
        let shade = if cooldowns[index] > 0.0 {
            cooldowns[index] / ability.cooldown
        } else if recovery > 0.0 {
            recovery.min(0.3)
        } else {
            0.0
        };
        slot_shade.push((shade.clamp(0.0, 1.0) * SHADE_STEPS).round() as u32);
    }

    let text = HudText {
        player_name: format!("{}  Lv {level}", resources.player_name),
        player_health: format!("{} / {}", health.ceil().max(0.0) as i32, max_health as i32),
        player_health_fraction: quantize(health / max_health),
        player_mana: format!("{} / {}", mana.floor().max(0.0) as i32, max_mana as i32),
        player_mana_fraction: quantize(mana / max_mana),
        target_visible,
        target_name,
        target_health,
        target_health_fraction: quantize(target_fraction),
        zone_name,
        zone_clock: format!(
            "{}      {} online",
            hour_text(hour),
            resources.clock.online + 1
        ),
        quest_rows,
        experience: format!(
            "Level {level}     {experience} / {}",
            level_threshold(level)
        ),
        experience_fraction: quantize(experience as f32 / level_threshold(level) as f32),
        slot_status,
        slot_shade,
        chat: {
            let lines = &resources.chat.lines;
            let end = lines.len().saturating_sub(resources.chat.scroll);
            let start = end.saturating_sub(CHAT_ROWS);
            lines
                .iter()
                .skip(start)
                .take(end - start)
                .cloned()
                .collect()
        },
        purse: coin_text(resources.purse),
        combat_visible: feel.combat > 0.0,
        minimap: resources.hud_text.minimap.clone(),
        hurt_alpha: 0.0,
        map_open: resources.hud_text.map_open,
        map_player: resources.hud_text.map_player,
        map_marks: resources.hud_text.map_marks.clone(),
        map_note: resources.hud_text.map_note.clone(),
    };

    let handles = resources.hud.clone();
    let shown = &resources.hud_text;
    let bar_width = FRAME_WIDTH - PORTRAIT - 30.0;
    write_changed(
        &mut commands,
        handles.player_name,
        &shown.player_name,
        &text.player_name,
    );
    write_changed(
        &mut commands,
        handles.player_health,
        &shown.player_health,
        &text.player_health,
    );
    if shown.player_health_fraction != text.player_health_fraction {
        set_fill(
            &mut dirty,
            &mut commands,
            handles.player_health_fill,
            bar_width,
            BAR_HEIGHT,
            text.player_health_fraction as f32 / BAR_STEPS,
        );
    }
    write_changed(
        &mut commands,
        handles.player_mana,
        &shown.player_mana,
        &text.player_mana,
    );
    if shown.player_mana_fraction != text.player_mana_fraction {
        set_fill(
            &mut dirty,
            &mut commands,
            handles.player_mana_fill,
            bar_width,
            BAR_HEIGHT,
            text.player_mana_fraction as f32 / BAR_STEPS,
        );
    }
    if shown.target_visible != text.target_visible {
        ui_set_visible(
            &mut dirty,
            &mut commands,
            handles.target_panel,
            text.target_visible,
        );
    }
    write_changed(
        &mut commands,
        handles.target_name,
        &shown.target_name,
        &text.target_name,
    );
    write_changed(
        &mut commands,
        handles.target_health,
        &shown.target_health,
        &text.target_health,
    );
    if shown.target_health_fraction != text.target_health_fraction {
        set_fill(
            &mut dirty,
            &mut commands,
            handles.target_health_fill,
            bar_width,
            BAR_HEIGHT,
            text.target_health_fraction as f32 / BAR_STEPS,
        );
    }
    if shown.target_name != text.target_name {
        let elite = text.target_name.ends_with("Elite");
        ui_set_color::<UiBase>(
            &mut commands,
            handles.target_name,
            if elite { ELITE_COLOR } else { TARGET_COLOR },
        );
        ui_set_color::<UiBase>(
            &mut commands,
            handles.target_health_fill,
            if elite {
                ELITE_COLOR
            } else {
                TARGET_HEALTH_COLOR
            },
        );
    }
    write_changed(
        &mut commands,
        handles.zone_name,
        &shown.zone_name,
        &text.zone_name,
    );
    write_changed(
        &mut commands,
        handles.zone_clock,
        &shown.zone_clock,
        &text.zone_clock,
    );
    write_changed(
        &mut commands,
        handles.experience_line,
        &shown.experience,
        &text.experience,
    );
    if shown.experience_fraction != text.experience_fraction {
        set_fill(
            &mut dirty,
            &mut commands,
            handles.experience_fill,
            EXPERIENCE_WIDTH,
            EXPERIENCE_HEIGHT,
            text.experience_fraction as f32 / BAR_STEPS,
        );
    }
    write_changed(&mut commands, handles.purse, &shown.purse, &text.purse);
    if shown.combat_visible != text.combat_visible {
        ui_set_visible(
            &mut dirty,
            &mut commands,
            handles.combat_label,
            text.combat_visible,
        );
    }

    let empty = String::new();
    for index in 0..QUEST_ROWS {
        let old = shown.quest_rows.get(index).unwrap_or(&empty);
        let new = text.quest_rows.get(index).unwrap_or(&empty);
        write_changed(&mut commands, handles.quest_rows[index], old, new);
        if old != new {
            let color = if new.starts_with("   ") {
                TEXT_DIM
            } else {
                QUEST_COLOR
            };
            ui_set_color::<UiBase>(&mut commands, handles.quest_rows[index], color);
        }
    }

    for index in 0..ABILITY_COUNT {
        let old = shown.slot_status.get(index);
        let new = &text.slot_status[index];
        if old != Some(new) {
            ui_set_text(&mut commands, handles.slot_status[index], &new.0);
            ui_set_color::<UiBase>(&mut commands, handles.slot_status[index], new.1);
            let tint = if new.1 == DRAINED_COLOR {
                ICON_DRAINED
            } else {
                ICON_TINT
            };
            ui_set_color::<UiBase>(&mut commands, handles.slot_names[index], tint);
        }
        if shown.slot_shade.get(index) != Some(&text.slot_shade[index]) {
            let steps = text.slot_shade[index];
            let fraction = steps as f32 / SHADE_STEPS;
            let top = SLOT_SIZE * (1.0 - fraction);
            ui_set_window_layout(
                &mut dirty,
                &mut commands,
                handles.slot_shades[index],
                WindowLayout {
                    position: Ab(vec2(0.0, top)).into(),
                    size: Ab(vec2(SLOT_SIZE, SLOT_SIZE * fraction)).into(),
                    anchor: Anchor::TopLeft,
                },
            );
            ui_set_window_layout(
                &mut dirty,
                &mut commands,
                handles.slot_wipes[index],
                WindowLayout {
                    position: Ab(vec2(0.0, top)).into(),
                    size: Ab(vec2(SLOT_SIZE, WIPE_HEIGHT)).into(),
                    anchor: Anchor::TopLeft,
                },
            );
            let cooling = steps > 0;
            if shown.slot_shade.get(index).is_some_and(|old| *old > 0) != cooling {
                ui_set_visible(
                    &mut dirty,
                    &mut commands,
                    handles.slot_wipes[index],
                    cooling,
                );
            }
        }
        let flash = feel.slot_flash[index];
        let frame_color = if flash > 0.0 {
            Vec4::new(SLOT_FLASH.x, SLOT_FLASH.y, SLOT_FLASH.z, 0.2 + 0.7 * flash)
        } else {
            PANEL_BACKGROUND
        };
        if flash > 0.0
            || resources.hud_text.slot_status.get(index) != Some(&text.slot_status[index])
        {
            ui_set_color::<UiBase>(&mut commands, handles.slot_frames[index], frame_color);
        }
    }

    for index in 0..CHAT_ROWS {
        let old = shown.chat.get(index);
        let new = text.chat.get(index);
        if old != new {
            let (channel, line) = new
                .map(|(channel, line)| (*channel, line.as_str()))
                .unwrap_or((Channel::System, ""));
            ui_set_text(&mut commands, handles.chat_rows[index], line);
            let age = 1.0 - index as f32 / (CHAT_ROWS as f32 - 1.0);
            let color = channel_color(channel);
            ui_set_color::<UiBase>(
                &mut commands,
                handles.chat_rows[index],
                Vec4::new(color.x, color.y, color.z, 1.0 - 0.5 * age),
            );
        }
    }

    let low = health / max_health < LOW_HEALTH_FRACTION && is_alive(health);
    let pulse = if low {
        0.10 + 0.08 * feel.low_pulse.sin()
    } else {
        0.0
    };
    let alpha = (feel.hurt * 0.4 + pulse).min(0.6);
    if alpha > 0.0 || resources.hud_text.hurt_alpha > 0.0 {
        ui_set_color::<UiBase>(
            &mut commands,
            handles.hurt_overlay,
            Vec4::new(HURT_COLOR.x, HURT_COLOR.y, HURT_COLOR.z, alpha),
        );
    }

    let mut text = text;
    text.hurt_alpha = alpha;
    resources.hud_text = text;
}

fn write_changed(commands: &mut Commands, node: Entity, old: &str, new: &str) {
    if old == new || node == Entity::default() {
        return;
    }
    ui_set_text(commands, node, new);
}

pub fn banners(
    time: Res<Time>,
    resources: Res<ThornmereResources>,
    mut banners: ResMut<Banners>,
    mut commands: Commands,
) {
    let handles = resources.hud.clone();
    if handles.root == Entity::default() {
        return;
    }
    let banners = &mut *banners;
    if banners.remaining <= 0.0 {
        if banners.shown {
            banners.shown = false;
            ui_set_color::<UiBase>(
                &mut commands,
                handles.banner_title,
                Vec4::new(1.0, 1.0, 1.0, 0.0),
            );
            ui_set_color::<UiBase>(
                &mut commands,
                handles.banner_subtitle,
                Vec4::new(1.0, 1.0, 1.0, 0.0),
            );
        }
        let Some(banner) = banners.pending.pop_front() else {
            return;
        };
        banners.remaining = BANNER_SECONDS;
        banners.shown = true;
        banners.color = banner.color;
        ui_set_text(&mut commands, handles.banner_title, &banner.title);
        ui_set_text(&mut commands, handles.banner_subtitle, &banner.subtitle);
    }
    banners.remaining -= time.delta_time;
    let elapsed = BANNER_SECONDS - banners.remaining;
    let alpha = (elapsed / 0.35)
        .min(1.0)
        .min((banners.remaining / BANNER_FADE).max(0.0));
    let color = banners.color;
    ui_set_color::<UiBase>(
        &mut commands,
        handles.banner_title,
        Vec4::new(color.x, color.y, color.z, alpha),
    );
    ui_set_color::<UiBase>(
        &mut commands,
        handles.banner_subtitle,
        Vec4::new(TEXT_BRIGHT.x, TEXT_BRIGHT.y, TEXT_BRIGHT.z, alpha * 0.9),
    );
}

pub fn minimap(
    time: Res<Time>,
    mut resources: ResMut<ThornmereResources>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    players: Query<(&LocalTransform,), With<Cooldowns>>,
    givers: Query<(&LocalTransform,), With<Giver>>,
    mobs: Query<(&Health, &Rank, &LocalTransform), With<Worth>>,
    people: Query<(&Task, &LocalTransform), With<Voice>>,
) {
    let resources = &mut *resources;
    if resources.hud.root == Entity::default() {
        return;
    }
    resources.clock.minimap_timer -= time.delta_time;
    if resources.clock.minimap_timer > 0.0 {
        return;
    }
    resources.clock.minimap_timer = MINIMAP_STEP;

    let mut center = None;
    players.for_each(|_entity, (transform,)| center = Some(transform.translation));
    let Some(center) = center else {
        return;
    };

    let mut dots: Vec<(u8, f32, f32)> = Vec::new();
    let half = MINIMAP_SIZE / 2.0;
    let push = |kind: u8, position: Vec3, dots: &mut Vec<(u8, f32, f32)>| {
        let dx = (position.x - center.x) / MINIMAP_RANGE;
        let dz = (position.z - center.z) / MINIMAP_RANGE;
        if dx * dx + dz * dz > 0.92 {
            return;
        }
        dots.push((
            kind,
            half + dx * half - DOT / 2.0,
            half + dz * half - DOT / 2.0,
        ));
    };
    givers.for_each(|_entity, (transform,)| push(0, transform.translation, &mut dots));
    mobs.for_each(|_entity, (health, rank, transform)| {
        if is_alive(health.0) {
            let kind = if rank.0 >= ELITE { 1 } else { 2 };
            push(kind, transform.translation, &mut dots);
        }
    });
    people.for_each(|_entity, (task, transform)| {
        if task.0 != crate::components::Job::Offline {
            push(3, transform.translation, &mut dots);
        }
    });
    dots.sort_by_key(|dot| dot.0);
    dots.truncate(MINIMAP_DOTS);

    let quantized: Vec<(u8, i16, i16)> = dots
        .iter()
        .map(|(kind, x, y)| (*kind, x.round() as i16, y.round() as i16))
        .collect();
    if quantized == resources.hud_text.minimap {
        return;
    }

    let handles = resources.hud.clone();
    let shown = &resources.hud_text.minimap;
    for index in 0..MINIMAP_DOTS {
        let dot = handles.minimap_dots[index];
        match (quantized.get(index), shown.get(index)) {
            (Some(new), old) => {
                if old.is_none() {
                    ui_set_visible(&mut dirty, &mut commands, dot, true);
                }
                if old.map(|old| old.0) != Some(new.0) {
                    let color = match new.0 {
                        0 => DOT_GIVER,
                        1 => DOT_ELITE,
                        2 => DOT_MOB,
                        _ => DOT_PERSON,
                    };
                    ui_set_color::<UiBase>(&mut commands, dot, color);
                }
                if old != Some(new) {
                    ui_set_window_layout(
                        &mut dirty,
                        &mut commands,
                        dot,
                        WindowLayout {
                            position: Ab(vec2(new.1 as f32, new.2 as f32)).into(),
                            size: Ab(vec2(DOT, DOT)).into(),
                            anchor: Anchor::TopLeft,
                        },
                    );
                }
            }
            (None, Some(_)) => ui_set_visible(&mut dirty, &mut commands, dot, false),
            (None, None) => {}
        }
    }
    resources.hud_text.minimap = quantized;
}
