use crate::commands::screens::set_screen;
use crate::components::{BarFill, HeroFrame};
use crate::contract::Screen;
use crate::data::climb::{COLUMNS, LANES, Stop, gate_at, step_at};
use crate::data::decks::warden;
use crate::data::frames::{BUTTON_FRAME, CARD_FRAME, FRAME_BANDS, PANEL_FRAME};
use crate::data::tutorial::ECHO;
use crate::resources::{Frames, GatewardenResources, UiHandles};
use crate::theme::*;
use nightshade::prelude::*;
use nightshade_camera::commands::scene::spawn_camera;
use nightshade_camera::prelude::*;
use nightshade_camera::queries::camera_look::camera_look_rotation;
use nightshade_graphics::prelude::*;
use nightshade_platform::prelude::Window;
use nightshade_render::data::decoded_image::decode_to_rgba8;
use nightshade_text::prelude::*;
use nightshade_ui::prelude::*;

pub fn initialize(
    mut resources: ResMut<GatewardenResources>,
    mut settings: ResMut<RenderSettings>,
    mut window: ResMut<Window>,
    mut dirty: ResMut<nightshade_ui::resources::RetainedUiDirty>,
    mut commands: Commands,
) {
    window.min_window_size = Some(WINDOW_MIN_SIZE);
    settings.atmosphere = Atmosphere::None;
    spawn_sun(&mut commands);

    let camera = spawn_camera(&mut commands, CAMERA_EYE, "Main Camera".to_string());
    commands.queue(move |storage, resources| {
        if let Some(transform) = storage.get_mut::<LocalTransform>(camera) {
            transform.rotation = camera_look_rotation(CAMERA_EYE, CAMERA_TARGET, 0.0);
        }
        resources.res_mut::<ActiveCamera>().0 = Some(camera);
    });
    resources.camera = Some(camera);

    set_screen(&mut resources, &mut dirty, &mut commands, Screen::Title);
}

pub fn build_overlays(
    mut resources: ResMut<GatewardenResources>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    let mut tree = UiTreeBuilder::new(build.context());
    let frames = Frames {
        panel: upload_frame(&mut tree, PANEL_FRAME),
        button: upload_frame(&mut tree, BUTTON_FRAME),
        card: upload_frame(&mut tree, CARD_FRAME),
    };
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();

    let mut handles = tree.in_parent(root, |tree| {
        let mut handles = UiHandles {
            frames,
            ..UiHandles::default()
        };
        build_arena(tree, &mut handles);
        build_heroes(tree, &mut handles);
        build_controls(tree, &mut handles);
        build_preview(tree, &mut handles);
        build_log(tree, &mut handles);
        build_strip(tree, &mut handles);
        build_relics(tree, &mut handles);
        build_banner(tree, &mut handles);
        build_lesson(tree, &mut handles);
        build_title(tree, &mut handles);
        build_scene(tree, &mut handles);
        build_map(tree, &mut handles);
        build_shop(tree, &mut handles);
        build_over(tree, &mut handles);
        build_pause(tree, &mut handles);
        handles
    });

    handles.layout_root = Some(tree.finish());
    resources.ui = handles;
}

pub fn upload_frame(tree: &mut UiTreeBuilder, bytes: &[u8]) -> Option<u32> {
    let decoded = decode_to_rgba8(bytes).ok()?;
    let (width, height) = (decoded.width, decoded.height);
    let (images, runtime) = tree.images_and_runtime();
    ui_upload_image(images, runtime, &decoded.rgba, width, height).map(|upload| upload.layer)
}

pub fn frame(tree: &mut UiTreeBuilder, layer: Option<u32>, corner: f32, tint: Vec4) {
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

fn build_arena(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let arena = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -HAND_BOTTOM_MARGIN)),
            Ab(vec2(HAND_WIDTH, ARENA_HEIGHT)),
            Anchor::BottomCenter,
        )
        .entity();
    handles.arena = Some(arena);
    tree.in_parent(arena, |tree| {
        handles.table_surface = Some(
            tree.add_node()
                .window(
                    Ab(vec2(-TABLE_PADDING, ENEMY_ROW_Y - LANE_PADDING - 12.0)),
                    Ab(vec2(
                        HAND_WIDTH + TABLE_PADDING * 2.0,
                        PLAYER_ROW_Y + MINION_HEIGHT + LANE_PADDING + 12.0 - ENEMY_ROW_Y
                            + LANE_PADDING
                            + 12.0,
                    )),
                    Anchor::TopLeft,
                )
                .with_rect(26.0, 1.5, TABLE_BORDER)
                .color_raw::<UiBase>(TABLE_BG)
                .with_effect(RectEffect::VerticalGradient {
                    secondary: TABLE_BG_BOTTOM,
                })
                .with_shadow(CARD_SHADOW, vec2(0.0, 8.0), 24.0, 0.0)
                .without_pointer_events()
                .entity(),
        );
        lane(tree, ENEMY_ROW_Y, "Warden's line");
        handles.divider = Some(lane(tree, PLAYER_ROW_Y, "Your line"));
        handles.hand_surface = Some(
            tree.add_node()
                .window(
                    Ab(vec2(-TABLE_PADDING, HAND_PANEL_TOP)),
                    Ab(vec2(HAND_WIDTH + TABLE_PADDING * 2.0, HAND_PANEL_HEIGHT)),
                    Anchor::TopLeft,
                )
                .with_rect(22.0, 1.5, TABLE_BORDER)
                .color_raw::<UiBase>(PANEL_BG)
                .with_effect(RectEffect::VerticalGradient {
                    secondary: PANEL_BG_BOTTOM,
                })
                .with_shadow(CARD_SHADOW, vec2(0.0, 8.0), 20.0, 0.0)
                .without_pointer_events()
                .entity(),
        );
        build_arrow(tree, handles);
    });
}

fn build_arrow(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let canvas = tree
        .add_node()
        .window(Ab(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)), Anchor::TopLeft)
        .with_depth(UiDepthMode::Set(ARROW_DEPTH))
        .without_pointer_events()
        .entity();
    tree.commands().set(
        canvas,
        UiCanvasData {
            commands: Vec::new(),
            hit_test_enabled: false,
            command_bounds: Vec::new(),
        },
    );
    handles.arrow_canvas = Some(canvas);
}

fn lane(tree: &mut UiTreeBuilder, row_y: f32, label: &str) -> Entity {
    let entity = tree
        .add_node()
        .window(
            Ab(vec2(-TABLE_PADDING + 14.0, row_y - LANE_PADDING)),
            Ab(vec2(
                HAND_WIDTH + TABLE_PADDING * 2.0 - 28.0,
                MINION_HEIGHT + LANE_PADDING * 2.0,
            )),
            Anchor::TopLeft,
        )
        .with_rect(16.0, 1.0, LANE_BORDER)
        .color_raw::<UiBase>(LANE_BG)
        .without_pointer_events()
        .entity();
    tree.in_parent(entity, |tree| {
        tree.add_node()
            .window(Ab(vec2(14.0, 6.0)), Ab(vec2(120.0, 18.0)), Anchor::TopLeft)
            .with_text(label, SMALL_SIZE - 1.0)
            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
            .color_raw::<UiBase>(Vec4::new(0.85, 0.75, 0.55, 0.55))
            .without_pointer_events()
            .entity();
    });
    entity
}

fn build_heroes(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let enemy = hero_panel(
        tree,
        handles.frames,
        Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, TOP_MARGIN)),
        vec2(ENEMY_PANEL_WIDTH, ENEMY_PANEL_HEIGHT),
        Anchor::TopCenter,
    );
    handles.enemy_hero = Some(enemy);
    tree.in_parent(enemy, |tree| {
        let first = warden(1, 0);
        let (frame, sigil) = portrait(
            tree,
            ENEMY_PORTRAIT,
            element_color(first.element),
            first.sigil,
        );
        handles.enemy_portrait = Some(frame);
        handles.enemy_sigil = Some(sigil);
        let column = hero_column(
            tree,
            vec2(ENEMY_PANEL_WIDTH, ENEMY_PANEL_HEIGHT),
            ENEMY_PORTRAIT,
        );
        tree.in_parent(column, |tree| {
            handles.enemy_name = Some(centered_row(tree, "Warden", LABEL_SIZE, TEXT_GOLD));
            handles.enemy_title = Some(centered_row(tree, "", SMALL_SIZE, TEXT_DIM));
            handles.enemy_shard = Some(centered_row(tree, "", SMALL_SIZE - 1.0, TEXT_GOLD));
            handles.enemy_health = Some(centered_row(tree, "30", HERO_HEALTH_SIZE, HEALTH_RED));
            handles.enemy_health_fill = Some(health_bar(tree));
            handles.enemy_line = Some(centered_row(tree, "", SMALL_SIZE, TEXT_DIM));
            handles.enemy_intent = Some(centered_row(tree, "", SMALL_SIZE, INTENT_CALM));
            handles.gate_line = Some(centered_row(tree, "", SMALL_SIZE - 1.0, TEXT_DIM));
        });
    });

    let backs = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 0.0)) + Ab(vec2(ENEMY_PANEL_WIDTH * 0.5 + 16.0, TOP_MARGIN + 8.0)),
            Ab(vec2(8.0 * (HAND_BACK_WIDTH + 5.0), HAND_BACK_HEIGHT)),
            Anchor::TopLeft,
        )
        .flow(FlowDirection::Horizontal, 0.0, 5.0)
        .without_pointer_events()
        .entity();
    handles.enemy_hand_row = Some(backs);
    tree.in_parent(backs, |tree| {
        for index in 0..handles.enemy_hand_backs.len() {
            handles.enemy_hand_backs[index] = Some(
                tree.add_node()
                    .flow_child(Ab(vec2(HAND_BACK_WIDTH, HAND_BACK_HEIGHT)))
                    .with_rect(5.0, 1.5, CARD_BACK_BORDER)
                    .color_raw::<UiBase>(CARD_BACK)
                    .with_effect(RectEffect::VerticalGradient {
                        secondary: Vec4::new(0.08, 0.1, 0.24, 1.0),
                    })
                    .with_shadow(CARD_SHADOW, vec2(0.0, 3.0), 6.0, 0.0)
                    .with_visible(false)
                    .without_pointer_events()
                    .entity(),
            );
        }
    });

    let player = hero_panel(
        tree,
        handles.frames,
        Rl(vec2(50.0, 100.0)) + Ab(vec2(-SIDE_OFFSET, -HAND_BOTTOM_MARGIN)),
        vec2(HERO_PANEL_WIDTH, HERO_PANEL_HEIGHT),
        Anchor::BottomRight,
    );
    handles.player_hero = Some(player);
    tree.in_parent(player, |tree| {
        portrait(tree, PLAYER_PORTRAIT, TEXT_GOLD, PLAYER_SIGIL);
        let column = hero_column(
            tree,
            vec2(HERO_PANEL_WIDTH, HERO_PANEL_HEIGHT),
            PLAYER_PORTRAIT,
        );
        tree.in_parent(column, |tree| {
            handles.player_name = Some(centered_row(tree, "You", LABEL_SIZE, TEXT_GOLD));
            handles.player_health = Some(centered_row(tree, "30", HERO_HEALTH_SIZE, HEALTH_RED));
            handles.player_health_fill = Some(health_bar(tree));
            spacer(tree, 4.0);
            let (fill, label) = mana_bar(tree);
            handles.player_mana_fill = Some(fill);
            handles.player_mana = Some(label);
            handles.player_line = Some(centered_row(tree, "", SMALL_SIZE, TEXT_DIM));
        });
    });
}

fn hero_column(tree: &mut UiTreeBuilder, panel: Vec2, portrait_size: f32) -> Entity {
    tree.add_node()
        .flow_child(Ab(vec2(
            panel.x - PANEL_PADDING * 2.0 - portrait_size - PORTRAIT_GAP,
            panel.y - PANEL_PADDING * 2.0,
        )))
        .flow_with_alignment(
            FlowDirection::Vertical,
            0.0,
            2.0,
            FlowAlignment::Start,
            FlowAlignment::Center,
        )
        .without_pointer_events()
        .entity()
}

fn portrait(tree: &mut UiTreeBuilder, size: f32, color: Vec4, sigil: char) -> (Entity, Entity) {
    let mut icon = None;
    let frame = tree
        .add_node()
        .flow_child(Ab(vec2(size, size)))
        .with_rect(size * 0.5, PORTRAIT_RING_WIDTH, PORTRAIT_RING)
        .color_raw::<UiBase>(portrait_fill(color))
        .with_effect(RectEffect::RadialGlow {
            glow_color: PORTRAIT_GLOW,
            intensity: 0.55,
        })
        .with_shadow(CARD_SHADOW, vec2(0.0, 5.0), 12.0, 0.0)
        .without_pointer_events()
        .with_children(|tree| {
            icon = Some(
                tree.add_node()
                    .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
                    .with_icon(IconGlyph::lucide(sigil), size * 0.5)
                    .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                    .with_text_outline(TEXT_OUTLINE, 1.2)
                    .color_raw::<UiBase>(TEXT_PRIMARY)
                    .without_pointer_events()
                    .entity(),
            );
        })
        .entity();
    (frame, icon.unwrap_or(frame))
}

fn health_bar(tree: &mut UiTreeBuilder) -> Entity {
    let mut fill = None;
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, HEALTH_BAR_HEIGHT)))
        .with_rect(HEALTH_BAR_HEIGHT * 0.5, 1.0, PANEL_BORDER)
        .color_raw::<UiBase>(BAR_TRACK)
        .without_pointer_events()
        .with_children(|tree| {
            fill = Some(
                tree.add_node()
                    .window(Ab(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)), Anchor::TopLeft)
                    .with_rect(HEALTH_BAR_HEIGHT * 0.5, 0.0, HEALTH_RED)
                    .color_raw::<UiBase>(HEALTH_RED)
                    .with_effect(RectEffect::VerticalGradient {
                        secondary: Vec4::new(0.55, 0.12, 0.1, 1.0),
                    })
                    .without_pointer_events()
                    .entity(),
            );
        })
        .entity();
    let fill = fill.unwrap_or_else(|| tree.commands().spawn(()));
    tree.commands().set(fill, BarFill::default());
    fill
}

fn mana_bar(tree: &mut UiTreeBuilder) -> (Entity, Entity) {
    let mut fill = None;
    let mut label = None;
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, MANA_BAR_HEIGHT)))
        .with_rect(MANA_BAR_RADIUS, 1.0, MANA_BAR_EDGE)
        .color_raw::<UiBase>(MANA_SPENT)
        .without_pointer_events()
        .with_children(|tree| {
            fill = Some(
                tree.add_node()
                    .window(Ab(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)), Anchor::TopLeft)
                    .with_rect(MANA_BAR_RADIUS, 0.0, MANA_BLUE)
                    .color_raw::<UiBase>(MANA_BLUE)
                    .with_effect(RectEffect::VerticalGradient {
                        secondary: MANA_BLUE_DEEP,
                    })
                    .without_pointer_events()
                    .entity(),
            );
            label = Some(
                tree.add_node()
                    .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
                    .with_text("1 of 1 mana", MANA_LABEL_SIZE)
                    .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                    .with_text_outline(TEXT_OUTLINE, 1.5)
                    .color_raw::<UiBase>(TEXT_PRIMARY)
                    .without_pointer_events()
                    .entity(),
            );
        })
        .entity();
    let fill = fill.unwrap_or_else(|| tree.commands().spawn(()));
    let label = label.unwrap_or_else(|| tree.commands().spawn(()));
    tree.commands().set(fill, BarFill::default());
    (fill, label)
}

fn hero_panel(
    tree: &mut UiTreeBuilder,
    frames: Frames,
    position: UiValue<Vec2>,
    size: Vec2,
    anchor: Anchor,
) -> Entity {
    let entity = tree
        .add_node()
        .window(position, Ab(size), anchor)
        .with_rect(14.0, 0.0, PANEL_BORDER)
        .color_raw::<UiBase>(HERO_BG)
        .color_raw::<UiHover>(Vec4::new(0.2, 0.16, 0.26, 0.98))
        .with_effect(RectEffect::VerticalGradient {
            secondary: HERO_BG_BOTTOM,
        })
        .with_shadow(CARD_SHADOW, vec2(0.0, 8.0), 20.0, 0.0)
        .with_interaction()
        .with_spring_transition::<UiHover>(Spring::lively())
        .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
        .flow_with_alignment(
            FlowDirection::Horizontal,
            PANEL_PADDING,
            PORTRAIT_GAP,
            FlowAlignment::Start,
            FlowAlignment::Center,
        )
        .with_children(|tree| frame(tree, frames.panel, PANEL_FRAME_CORNER, FRAME_TINT_SOFT))
        .entity();
    tree.commands().set(entity, UiButtonData::default());
    tree.commands().set(entity, HeroFrame::default());
    entity
}

fn build_controls(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let panel = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 100.0)) + Ab(vec2(SIDE_OFFSET, -HAND_BOTTOM_MARGIN)),
            Ab(vec2(HERO_PANEL_WIDTH, CONTROLS_HEIGHT)),
            Anchor::BottomLeft,
        )
        .with_rect(14.0, 1.5, PANEL_BORDER)
        .color_raw::<UiBase>(PANEL_BG)
        .with_effect(RectEffect::VerticalGradient {
            secondary: PANEL_BG_BOTTOM,
        })
        .with_shadow(CARD_SHADOW, vec2(0.0, 8.0), 20.0, 0.0)
        .flow_with_alignment(
            FlowDirection::Vertical,
            12.0,
            6.0,
            FlowAlignment::Start,
            FlowAlignment::Center,
        )
        .entity();
    handles.controls = Some(panel);
    let panel_frame = handles.frames.panel;
    tree.in_parent(panel, |tree| {
        frame(tree, panel_frame, PANEL_FRAME_CORNER, FRAME_TINT_SOFT);
        handles.turn_text = Some(centered_row(tree, "Your turn", LABEL_SIZE, TEXT_GOLD));
        let (end_turn, end_label) = button(
            tree,
            handles.frames,
            "End turn",
            HERO_PANEL_WIDTH - 24.0,
            BUTTON_END,
        );
        handles.end_turn_button = Some(end_turn);
        handles.end_turn_label = Some(end_label);
        let (power, power_label) = button(
            tree,
            handles.frames,
            "Spark  2 mana",
            HERO_PANEL_WIDTH - 24.0,
            BUTTON_POWER,
        );
        handles.power_button = Some(power);
        handles.power_label = Some(power_label);
        handles.power_text = Some(centered_row(tree, "", SMALL_SIZE - 1.0, TEXT_PRIMARY));
        handles.hint_text = Some(centered_row(
            tree,
            "Q power   E end turn   1 to 8 cards",
            SMALL_SIZE - 2.0,
            TEXT_DIM,
        ));
    });
}

fn build_preview(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let panel = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 100.0))
                + Ab(vec2(
                    -SIDE_OFFSET,
                    -HAND_BOTTOM_MARGIN - HERO_PANEL_HEIGHT - 12.0,
                )),
            Ab(vec2(HERO_PANEL_WIDTH, PREVIEW_HEIGHT)),
            Anchor::BottomRight,
        )
        .with_rect(14.0, 1.5, PANEL_BORDER)
        .color_raw::<UiBase>(PANEL_BG)
        .with_effect(RectEffect::VerticalGradient {
            secondary: PANEL_BG_BOTTOM,
        })
        .with_shadow(CARD_SHADOW, vec2(0.0, 8.0), 20.0, 0.0)
        .flow_with_alignment(
            FlowDirection::Vertical,
            12.0,
            2.0,
            FlowAlignment::Start,
            FlowAlignment::Center,
        )
        .without_pointer_events()
        .with_visible(false)
        .entity();
    handles.preview_panel = Some(panel);
    let panel_frame = handles.frames.panel;
    tree.in_parent(panel, |tree| {
        frame(tree, panel_frame, PANEL_FRAME_CORNER, FRAME_TINT_SOFT);
        handles.preview_name = Some(centered_row(tree, "", LABEL_SIZE, TEXT_GOLD));
        handles.preview_meta = Some(centered_row(tree, "", SMALL_SIZE, TEXT_DIM));
        handles.preview_stats = Some(centered_row(tree, "", LABEL_SIZE, TEXT_PRIMARY));
        handles.preview_text = Some(centered_row(tree, "", SMALL_SIZE, TEXT_PRIMARY));
        handles.preview_note = Some(centered_row(tree, "", SMALL_SIZE, TEXT_WARN));
    });
}

fn build_log(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let panel = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 0.0)) + Ab(vec2(SIDE_OFFSET, LOG_TOP)),
            Ab(vec2(HERO_PANEL_WIDTH, LOG_HEIGHT)),
            Anchor::TopLeft,
        )
        .with_rect(14.0, 1.5, PANEL_BORDER)
        .color_raw::<UiBase>(PANEL_BG)
        .with_effect(RectEffect::VerticalGradient {
            secondary: PANEL_BG_BOTTOM,
        })
        .with_shadow(CARD_SHADOW, vec2(0.0, 8.0), 20.0, 0.0)
        .flow_with_alignment(
            FlowDirection::Vertical,
            PANEL_PADDING,
            2.0,
            FlowAlignment::Start,
            FlowAlignment::Center,
        )
        .without_pointer_events()
        .with_visible(false)
        .entity();
    handles.log_panel = Some(panel);
    let panel_frame = handles.frames.panel;
    tree.in_parent(panel, |tree| {
        frame(tree, panel_frame, PANEL_FRAME_CORNER, FRAME_TINT_SOFT);
        text_row(tree, "What happened", SMALL_SIZE, TEXT_GOLD);
        let scroll = tree.add_scroll_area(vec2(0.0, LOG_VIEW_HEIGHT));
        let Some(content) = tree
            .component::<UiScrollAreaData>(scroll)
            .map(|area| area.content_entity)
        else {
            return;
        };
        tree.in_parent(content, |tree| {
            for slot in handles.log_lines.iter_mut() {
                *slot = Some(text_row(tree, "", LOG_LINE_SIZE, TEXT_DIM));
            }
        });
    });
}

fn build_strip(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let strip = tree
        .add_node()
        .window(
            Rl(vec2(100.0, 0.0)) + Ab(vec2(-SIDE_TOP, SIDE_TOP)),
            Ab(vec2(STRIP_WIDTH, STRIP_HEIGHT)),
            Anchor::TopRight,
        )
        .with_rect(12.0, 1.0, PANEL_BORDER)
        .color_raw::<UiBase>(PANEL_BG)
        .with_effect(RectEffect::VerticalGradient {
            secondary: PANEL_BG_BOTTOM,
        })
        .with_shadow(CARD_SHADOW, vec2(0.0, 6.0), 16.0, 0.0)
        .flow(FlowDirection::Vertical, 12.0, 0.0)
        .without_pointer_events()
        .entity();
    handles.strip = Some(strip);
    let panel_frame = handles.frames.panel;
    tree.in_parent(strip, |tree| {
        frame(tree, panel_frame, PANEL_FRAME_CORNER, FRAME_TINT_SOFT);
        handles.gate_text = Some(text_row(tree, "Gate 1 of 8", LABEL_SIZE, TEXT_GOLD));
        handles.gold_text = Some(text_row(tree, "6 gold", LABEL_SIZE, TEXT_PRIMARY));
        handles.shard_text = Some(text_row(tree, "No shards yet", SMALL_SIZE, TEXT_DIM));
    });
}

fn build_relics(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let slots = handles.relic_slots.len() as f32;
    let row = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 0.0)) + Ab(vec2(-SIDE_OFFSET, SIDE_TOP)),
            Ab(vec2(
                RELIC_SLOT_WIDTH,
                slots * (RELIC_SLOT_HEIGHT + RELIC_SLOT_GAP),
            )),
            Anchor::TopRight,
        )
        .flow(FlowDirection::Vertical, 0.0, RELIC_SLOT_GAP)
        .without_pointer_events()
        .entity();
    handles.relic_row = Some(row);
    tree.in_parent(row, |tree| {
        for index in 0..handles.relic_slots.len() {
            let slot = tree
                .add_node()
                .flow_child(Ab(vec2(RELIC_SLOT_WIDTH, RELIC_SLOT_HEIGHT)))
                .with_rect(10.0, 1.5, RELIC_BORDER)
                .color_raw::<UiBase>(RELIC_BG)
                .with_effect(RectEffect::VerticalGradient {
                    secondary: RELIC_BG_BOTTOM,
                })
                .with_shadow(CARD_SHADOW, vec2(0.0, 4.0), 10.0, 0.0)
                .flow(FlowDirection::Vertical, 8.0, 2.0)
                .with_visible(false)
                .without_pointer_events()
                .entity();
            handles.relic_slots[index] = Some(slot);
            tree.in_parent(slot, |tree| {
                handles.relic_names[index] = Some(text_row(tree, "", SMALL_SIZE + 2.0, TEXT_GOLD));
                handles.relic_texts[index] =
                    Some(text_row(tree, "", SMALL_SIZE - 1.0, TEXT_PRIMARY));
            });
        }
    });
}

fn build_banner(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let pill = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, TOP_MARGIN + ENEMY_PANEL_HEIGHT + BANNER_GAP)),
            Ab(vec2(BANNER_WIDTH, BANNER_HEIGHT)),
            Anchor::TopCenter,
        )
        .with_rect(25.0, 1.5, PANEL_BORDER)
        .color_raw::<UiBase>(BANNER_BG)
        .with_effect(RectEffect::VerticalGradient {
            secondary: BANNER_BG_BOTTOM,
        })
        .with_shadow(CARD_SHADOW, vec2(0.0, 8.0), 20.0, 0.0)
        .without_pointer_events()
        .with_visible(false)
        .entity();
    handles.banner = Some(pill);
    let panel_frame = handles.frames.panel;
    tree.in_parent(pill, |tree| {
        frame(tree, panel_frame, PANEL_FRAME_CORNER, FRAME_TINT);
        handles.banner_text = Some(
            tree.add_node()
                .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
                .with_text("", BANNER_SIZE)
                .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                .with_text_outline(TEXT_OUTLINE, 1.2)
                .color_raw::<UiBase>(TEXT_GOLD)
                .without_pointer_events()
                .entity(),
        );
    });
}

fn build_lesson(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let inner = HERO_PANEL_WIDTH - PANEL_PADDING * 2.0;
    let panel = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 0.0)) + Ab(vec2(-SIDE_OFFSET, LESSON_TOP)),
            Ab(vec2(HERO_PANEL_WIDTH, LESSON_HEIGHT)),
            Anchor::TopRight,
        )
        .with_rect(14.0, 1.5, PORTRAIT_RING)
        .color_raw::<UiBase>(PANEL_BG)
        .with_effect(RectEffect::VerticalGradient {
            secondary: PANEL_BG_BOTTOM,
        })
        .with_shadow(CARD_SHADOW, vec2(0.0, 8.0), 20.0, 0.0)
        .flow_with_alignment(
            FlowDirection::Vertical,
            PANEL_PADDING,
            8.0,
            FlowAlignment::Start,
            FlowAlignment::Center,
        )
        .with_visible(false)
        .entity();
    handles.lesson_panel = Some(panel);
    tree.in_parent(panel, |tree| {
        let header = tree
            .add_node()
            .flow_child(Ab(vec2(inner, LESSON_PORTRAIT)))
            .flow_with_alignment(
                FlowDirection::Horizontal,
                0.0,
                10.0,
                FlowAlignment::Start,
                FlowAlignment::Center,
            )
            .without_pointer_events()
            .entity();
        tree.in_parent(header, |tree| {
            portrait(tree, LESSON_PORTRAIT, TEXT_GOLD, ECHO_SIGIL);
            tree.add_node()
                .flow_child(Ab(vec2(inner - LESSON_PORTRAIT - 10.0, LESSON_PORTRAIT)))
                .with_text(ECHO, LABEL_SIZE)
                .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                .color_raw::<UiBase>(TEXT_GOLD)
                .without_pointer_events()
                .entity();
        });
        handles.lesson_text = Some(
            tree.add_node()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, LESSON_TEXT_HEIGHT)))
                .with_text("", LESSON_TEXT_SIZE)
                .with_text_wrap()
                .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
                .color_raw::<UiBase>(TEXT_PRIMARY)
                .without_pointer_events()
                .entity(),
        );
        handles.lesson_next_button =
            Some(button(tree, handles.frames, "Go on", inner, BUTTON_PLAY).0);
        handles.lesson_hint = Some(centered_row(tree, "", SMALL_SIZE - 2.0, TEXT_DIM));
    });
}

fn build_title(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let panel = centered_panel(tree, handles.frames, TITLE_WIDTH, TITLE_HEIGHT);
    handles.title_panel = Some(panel);
    tree.in_parent(panel, |tree| {
        centered_row(tree, "Gatewarden", TITLE_SIZE, TEXT_GOLD);
        centered_row(
            tree,
            "Eight gates. A warden holds each one.",
            LABEL_SIZE,
            TEXT_DIM,
        );
        spacer(tree, 8.0);
        handles.story_button = Some(button(tree, handles.frames, "The road", 240.0, BUTTON_PLAY).0);
        handles.new_run_button =
            Some(button(tree, handles.frames, "The climb", 240.0, BUTTON_BUY).0);
        handles.learn_button = Some(
            button(
                tree,
                handles.frames,
                "The practice yard",
                240.0,
                BUTTON_POWER,
            )
            .0,
        );
        handles.quit_button = Some(button(tree, handles.frames, "Quit", 240.0, BUTTON_NEUTRAL).0);
    });
}

fn build_scene(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let panel = centered_panel(tree, handles.frames, SCENE_WIDTH, SCENE_HEIGHT);
    handles.scene_panel = Some(panel);
    tree.in_parent(panel, |tree| {
        let (frame, sigil) = portrait(tree, SCENE_PORTRAIT, TEXT_GOLD, CLIMB_SIGIL);
        handles.scene_portrait = Some(frame);
        handles.scene_sigil = Some(sigil);
        handles.scene_speaker = Some(centered_row(tree, "", HEADING_SIZE, TEXT_GOLD));
        spacer(tree, 4.0);
        for index in 0..handles.scene_lines.len() {
            handles.scene_lines[index] =
                Some(centered_row(tree, "", SCENE_LINE_SIZE, TEXT_PRIMARY));
        }
        spacer(tree, 6.0);
        handles.scene_prompt = Some(centered_row(tree, "", LABEL_SIZE, TEXT_GOLD));
        handles.scene_first_text = Some(centered_row(tree, "", SMALL_SIZE, TEXT_DIM));
        handles.scene_second_text = Some(centered_row(tree, "", SMALL_SIZE, TEXT_DIM));
        let choices = tree
            .add_node()
            .flow_child(Ab(vec2(640.0, BUTTON_HEIGHT)))
            .flow_with_alignment(
                FlowDirection::Horizontal,
                0.0,
                16.0,
                FlowAlignment::Center,
                FlowAlignment::Center,
            )
            .entity();
        handles.scene_choices = Some(choices);
        tree.in_parent(choices, |tree| {
            let (first, first_label) = button(tree, handles.frames, "", 300.0, BUTTON_BUY);
            handles.scene_first_button = Some(first);
            handles.scene_first_label = Some(first_label);
            let (second, second_label) = button(tree, handles.frames, "", 300.0, BUTTON_PLAY);
            handles.scene_second_button = Some(second);
            handles.scene_second_label = Some(second_label);
        });
        handles.scene_continue_button =
            Some(button(tree, handles.frames, "Continue", 260.0, BUTTON_PLAY).0);
    });
}

fn build_shop(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let panel = centered_panel(tree, handles.frames, SHOP_WIDTH, SHOP_HEIGHT);
    handles.shop_panel = Some(panel);
    tree.in_parent(panel, |tree| {
        centered_row(tree, "The Bazaar", HEADING_SIZE, TEXT_GOLD);
        handles.shop_gold_text = Some(centered_row(tree, "", LABEL_SIZE, TEXT_PRIMARY));

        let offers = tree
            .add_node()
            .flow_child(Ab(vec2(
                OFFER_COUNT as f32 * (OFFER_WIDTH + 12.0),
                OFFER_HEIGHT,
            )))
            .flow(FlowDirection::Horizontal, 0.0, 12.0)
            .entity();
        tree.in_parent(offers, |tree| {
            for index in 0..OFFER_COUNT {
                let offer = tree
                    .add_node()
                    .flow_child(Ab(vec2(OFFER_WIDTH, OFFER_HEIGHT)))
                    .with_rect(12.0, 1.5, PANEL_BORDER)
                    .color_raw::<UiBase>(OFFER_BG)
                    .with_effect(RectEffect::VerticalGradient {
                        secondary: OFFER_BG_BOTTOM,
                    })
                    .with_shadow(CARD_SHADOW, vec2(0.0, 5.0), 12.0, 0.0)
                    .flow_with_alignment(
                        FlowDirection::Vertical,
                        12.0,
                        6.0,
                        FlowAlignment::Start,
                        FlowAlignment::Center,
                    )
                    .entity();
                tree.in_parent(offer, |tree| {
                    handles.offer_names[index] =
                        Some(centered_row(tree, "", LABEL_SIZE, TEXT_GOLD));
                    handles.offer_areas[index] = Some(
                        tree.add_node()
                            .flow_child(Ab(vec2(CARD_WIDTH, CARD_HEIGHT)))
                            .entity(),
                    );
                    handles.offer_effects[index] =
                        Some(centered_row(tree, "", SMALL_SIZE - 2.0, TEXT_PRIMARY));
                    let (entity, label) =
                        button(tree, handles.frames, "Buy", OFFER_WIDTH - 28.0, BUTTON_BUY);
                    handles.offer_buttons[index] = Some(entity);
                    handles.offer_labels[index] = Some(label);
                });
            }
        });

        let actions = tree
            .add_node()
            .flow_child(Ab(vec2(700.0, BUTTON_HEIGHT)))
            .flow_with_alignment(
                FlowDirection::Horizontal,
                0.0,
                12.0,
                FlowAlignment::Center,
                FlowAlignment::Center,
            )
            .entity();
        tree.in_parent(actions, |tree| {
            let (entity, label) =
                button(tree, handles.frames, "Reroll 2 gold", 190.0, BUTTON_NEUTRAL);
            handles.reroll_button = Some(entity);
            handles.reroll_label = Some(label);
            let (entity, label) = button(tree, handles.frames, "Strike a card", 250.0, BUTTON_END);
            handles.strike_button = Some(entity);
            handles.strike_label = Some(label);
            handles.next_button = Some(
                button(
                    tree,
                    handles.frames,
                    "Back to the climb",
                    200.0,
                    BUTTON_PLAY,
                )
                .0,
            );
        });
    });
}

fn build_map(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let panel = centered_panel(tree, handles.frames, MAP_WIDTH, MAP_HEIGHT);
    handles.map_panel = Some(panel);
    tree.in_parent(panel, |tree| {
        centered_row(tree, "The climb", HEADING_SIZE, TEXT_GOLD);
        handles.map_line = Some(centered_row(tree, "", LABEL_SIZE, TEXT_PRIMARY));
        handles.map_note = Some(centered_row(tree, "", SMALL_SIZE, TEXT_DIM));
        let area = tree
            .add_node()
            .flow_child(Ab(vec2(MAP_AREA_WIDTH, MAP_AREA_HEIGHT)))
            .entity();
        tree.in_parent(area, |tree| {
            for column in 0..COLUMNS - 1 {
                roads(tree, column);
            }
            for column in 0..COLUMNS {
                if let Some(gate) = gate_at(column) {
                    let index = gate as usize - 1;
                    let node = map_node(tree, column, 1, CLIMB_SIGIL, &format!("Gate {gate}"));
                    handles.map_gate_nodes[index] = Some(node.panel);
                    handles.map_gate_frames[index] = Some(node.frame);
                    handles.map_gate_sigils[index] = Some(node.sigil);
                }
                if let Some(step) = step_at(column) {
                    for lane in 0..LANES {
                        let node = map_node(tree, column, lane, Stop::Bazaar.sigil(), "");
                        handles.map_stop_nodes[step][lane] = Some(node.panel);
                        handles.map_stop_frames[step][lane] = Some(node.frame);
                        handles.map_stop_sigils[step][lane] = Some(node.sigil);
                        handles.map_stop_names[step][lane] = Some(node.name);
                    }
                }
            }
        });
        spacer(tree, 2.0);
        let (entity, label) = button(tree, handles.frames, "Enter the gate", 280.0, BUTTON_PLAY);
        handles.map_climb_button = Some(entity);
        handles.map_climb_label = Some(label);
    });
}

struct MapNode {
    panel: Entity,
    frame: Entity,
    sigil: Entity,
    name: Entity,
}

fn node_center(column: usize, lane: usize) -> Vec2 {
    vec2(
        NODE_BOX_WIDTH * 0.5 + column as f32 * COLUMN_PITCH,
        NODE_BOX_HEIGHT * 0.5 + lane as f32 * LANE_PITCH,
    )
}

fn circle_center(column: usize, lane: usize) -> Vec2 {
    node_center(column, lane) - vec2(0.0, (NODE_ROW_GAP + NODE_LABEL_HEIGHT) * 0.5)
}

fn map_node(
    tree: &mut UiTreeBuilder,
    column: usize,
    lane: usize,
    sigil: char,
    label: &str,
) -> MapNode {
    let mut frame = None;
    let mut glyph = None;
    let mut name = None;
    let panel = tree
        .add_node()
        .window(
            Ab(node_center(column, lane)),
            Ab(vec2(NODE_BOX_WIDTH, NODE_BOX_HEIGHT)),
            Anchor::Center,
        )
        .with_rect(12.0, 1.0, MARK_BORDER)
        .color_raw::<UiBase>(MARK_AHEAD)
        .color_raw::<UiHover>(MARK_HERE)
        .flow_with_alignment(
            FlowDirection::Vertical,
            0.0,
            NODE_ROW_GAP,
            FlowAlignment::Start,
            FlowAlignment::Center,
        )
        .with_interaction()
        .with_spring_transition::<UiHover>(Spring::lively())
        .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
        .with_children(|tree| {
            let (circle, icon) = portrait(tree, NODE_SIZE, TEXT_GOLD, sigil);
            frame = Some(circle);
            glyph = Some(icon);
            name = Some(centered_row(tree, label, MAP_NAME_SIZE, TEXT_DIM));
        })
        .entity();
    tree.commands().set(panel, UiButtonData::default());
    MapNode {
        panel,
        frame: frame.unwrap_or(panel),
        sigil: glyph.unwrap_or(panel),
        name: name.unwrap_or(panel),
    }
}

fn roads(tree: &mut UiTreeBuilder, column: usize) {
    let from_gate = gate_at(column).is_some();
    for lane in 0..LANES {
        let (start, end) = if from_gate {
            (circle_center(column, 1), circle_center(column + 1, lane))
        } else {
            (circle_center(column, lane), circle_center(column + 1, 1))
        };
        for index in 1..=ROAD_DOTS {
            let along = index as f32 / (ROAD_DOTS + 1) as f32;
            road_dot(tree, start + (end - start) * along);
        }
    }
}

fn road_dot(tree: &mut UiTreeBuilder, at: Vec2) -> Entity {
    tree.add_node()
        .window(Ab(at), Ab(vec2(ROAD_DOT, ROAD_DOT)), Anchor::Center)
        .with_rect(ROAD_DOT * 0.5, 0.0, ROAD_DIM)
        .color_raw::<UiBase>(ROAD_DIM)
        .without_pointer_events()
        .entity()
}

fn build_over(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let panel = centered_panel(tree, handles.frames, OVER_WIDTH, OVER_HEIGHT);
    handles.over_panel = Some(panel);
    tree.in_parent(panel, |tree| {
        handles.over_title = Some(centered_row(
            tree,
            "The climb ends",
            HEADING_SIZE,
            TEXT_GOLD,
        ));
        handles.over_text = Some(centered_row(tree, "", LABEL_SIZE, TEXT_PRIMARY));
        handles.over_stats = Some(centered_row(tree, "", SMALL_SIZE, TEXT_DIM));
        handles.over_deck = Some(centered_row(tree, "", SMALL_SIZE, TEXT_DIM));
        handles.over_boon = Some(centered_row(tree, "", SMALL_SIZE, TEXT_GOLD));
        spacer(tree, 6.0);
        let (entity, label) = button(tree, handles.frames, "Climb again", 240.0, BUTTON_PLAY);
        handles.over_new_run_button = Some(entity);
        handles.over_new_run_label = Some(label);
        handles.over_title_button =
            Some(button(tree, handles.frames, "Title", 240.0, BUTTON_NEUTRAL).0);
    });
}

fn build_pause(tree: &mut UiTreeBuilder, handles: &mut UiHandles) {
    let scrim = tree
        .add_node()
        .window(Rl(vec2(50.0, 50.0)), Rl(vec2(100.0, 100.0)), Anchor::Center)
        .with_rect(0.0, 0.0, SCRIM)
        .color_raw::<UiBase>(SCRIM)
        .with_layer(UiLayer::Popups)
        .with_visible(false)
        .entity();
    handles.pause_scrim = Some(scrim);
    tree.in_parent(scrim, |tree| {
        let panel = centered_panel(tree, handles.frames, PAUSE_WIDTH, PAUSE_HEIGHT);
        tree.in_parent(panel, |tree| {
            centered_row(tree, "The climb waits", HEADING_SIZE, TEXT_GOLD);
            centered_row(tree, "Escape goes back to the table", SMALL_SIZE, TEXT_DIM);
            spacer(tree, 6.0);
            handles.pause_resume_button =
                Some(button(tree, handles.frames, "Resume", 260.0, BUTTON_PLAY).0);
            handles.pause_menu_button =
                Some(button(tree, handles.frames, "Main menu", 260.0, BUTTON_NEUTRAL).0);
            handles.pause_quit_button =
                Some(button(tree, handles.frames, "Quit", 260.0, BUTTON_END).0);
        });
    });
}

fn centered_panel(tree: &mut UiTreeBuilder, frames: Frames, width: f32, height: f32) -> Entity {
    tree.add_node()
        .window(
            Rl(vec2(50.0, 50.0)),
            Ab(vec2(width, height)),
            Anchor::Center,
        )
        .with_rect(16.0, 0.0, PANEL_BORDER)
        .color_raw::<UiBase>(PANEL_BG)
        .with_effect(RectEffect::VerticalGradient {
            secondary: PANEL_BG_BOTTOM,
        })
        .with_shadow(CARD_SHADOW, vec2(0.0, 12.0), 28.0, 0.0)
        .flow_with_alignment(
            FlowDirection::Vertical,
            18.0,
            10.0,
            FlowAlignment::Start,
            FlowAlignment::Center,
        )
        .with_children(|tree| frame(tree, frames.panel, FRAME_CORNER, FRAME_TINT))
        .entity()
}

fn text_row(tree: &mut UiTreeBuilder, text: &str, size: f32, color: Vec4) -> Entity {
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, size + 8.0)))
        .with_text(text, size)
        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
        .color_raw::<UiBase>(color)
        .without_pointer_events()
        .entity()
}

fn centered_row(tree: &mut UiTreeBuilder, text: &str, size: f32, color: Vec4) -> Entity {
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, size + 8.0)))
        .with_text(text, size)
        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
        .color_raw::<UiBase>(color)
        .without_pointer_events()
        .entity()
}

fn spacer(tree: &mut UiTreeBuilder, height: f32) -> Entity {
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, height)))
        .without_pointer_events()
        .entity()
}

pub fn button(
    tree: &mut UiTreeBuilder,
    frames: Frames,
    text: &str,
    width: f32,
    color: Vec4,
) -> (Entity, Entity) {
    let hover = Vec4::new(
        (color.x + 0.12).min(1.0),
        (color.y + 0.12).min(1.0),
        (color.z + 0.12).min(1.0),
        color.w,
    );
    let pressed = Vec4::new(color.x * 0.75, color.y * 0.75, color.z * 0.75, color.w);
    let shade = Vec4::new(color.x * 0.6, color.y * 0.6, color.z * 0.6, color.w);
    let mut label = None;
    let entity = tree
        .add_node()
        .flow_child(Ab(vec2(width, BUTTON_HEIGHT)))
        .with_rect(10.0, 0.0, BUTTON_BORDER)
        .color_raw::<UiBase>(color)
        .color_raw::<UiHover>(hover)
        .color_raw::<UiPressed>(pressed)
        .with_effect(RectEffect::VerticalGradient { secondary: shade })
        .with_shadow(CARD_SHADOW, vec2(0.0, 4.0), 10.0, 0.0)
        .with_interaction()
        .with_spring_transition::<UiHover>(Spring::lively())
        .with_spring_transition::<UiPressed>(Spring::snappy())
        .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
        .with_children(|tree| {
            frame(tree, frames.button, BUTTON_FRAME_CORNER, BUTTON_FRAME_TINT);
            label = Some(
                tree.add_node()
                    .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
                    .with_text(text, BUTTON_SIZE)
                    .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                    .with_text_outline(TEXT_OUTLINE, 0.8)
                    .color_raw::<UiBase>(TEXT_PRIMARY)
                    .without_pointer_events()
                    .entity(),
            );
        })
        .entity();
    tree.commands().set(entity, UiButtonData::default());
    (entity, label.unwrap_or(entity))
}
