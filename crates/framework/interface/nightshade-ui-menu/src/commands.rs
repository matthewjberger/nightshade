use crate::data::{CardStyle, MenuStyle};
use crate::resources::{MenuHandles, Menus, SlotHandles};
use nightshade::ecs::Entity;
use nightshade::prelude::{Commands, Vec4, vec2, vec4};
use nightshade_text::contract::data::text_data::{TextAlignment, VerticalAlignment};
use nightshade_ui::components::layout_types::{FlowDirection, WindowLayout};
use nightshade_ui::components::{TextOverflow, UiScrollAreaData};
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;

pub fn add_menu_panel(
    tree: &mut UiTreeBuilder,
    style: &MenuStyle,
    width: f32,
    height: f32,
) -> Entity {
    tree.add_node()
        .window(
            Rl(vec2(50.0, 50.0)),
            Ab(vec2(width, height)),
            Anchor::Center,
        )
        .with_rect(style.corner_radius, style.border_width, style.border)
        .color_raw::<UiBase>(style.background)
        .flow(FlowDirection::Vertical, style.padding, style.spacing)
        .with_visible(style.visible)
        .entity()
}

pub fn add_menu_text(tree: &mut UiTreeBuilder, text: &str, size: f32, color: Vec4) -> Entity {
    text_row(
        tree,
        text,
        size,
        color,
        TextAlignment::Left,
        Vec4::zeros(),
        0.0,
    )
}

pub fn add_menu_text_centered(
    tree: &mut UiTreeBuilder,
    text: &str,
    size: f32,
    color: Vec4,
) -> Entity {
    text_row(
        tree,
        text,
        size,
        color,
        TextAlignment::Center,
        Vec4::zeros(),
        0.0,
    )
}

fn text_row(
    tree: &mut UiTreeBuilder,
    text: &str,
    size: f32,
    color: Vec4,
    align: TextAlignment,
    outline: Vec4,
    outline_width: f32,
) -> Entity {
    let mut node = tree
        .add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, size * 1.35)))
        .with_text(text, size)
        .with_text_alignment(align, VerticalAlignment::Middle)
        .color_raw::<UiBase>(color);
    if outline_width > 0.0 {
        node = node.with_text_outline(outline, outline_width);
    }
    node.entity()
}

fn styled_row(
    tree: &mut UiTreeBuilder,
    style: &MenuStyle,
    text: &str,
    size: f32,
    color: Vec4,
) -> Entity {
    text_row(
        tree,
        text,
        size,
        color,
        alignment(style),
        style.outline,
        style.outline_width,
    )
}

pub fn add_menu_heading(tree: &mut UiTreeBuilder, style: &MenuStyle, text: &str) -> Entity {
    styled_row(tree, style, text, style.heading_size, style.heading_color)
}

pub fn add_menu_row(tree: &mut UiTreeBuilder, style: &MenuStyle, text: &str) -> Entity {
    styled_row(tree, style, text, style.row_size, style.row_color)
}

pub fn add_menu(
    tree: &mut UiTreeBuilder,
    style: &MenuStyle,
    width: f32,
    height: f32,
    heading: &str,
    rows: &[&str],
    labels: &[&str],
) -> MenuHandles {
    let panel = add_menu_panel(tree, style, width, height);
    let mut written = Entity::default();
    let mut lines = Vec::with_capacity(rows.len());
    let mut buttons = Vec::with_capacity(labels.len());
    tree.in_parent(panel, |tree| {
        if !heading.is_empty() {
            written = add_menu_heading(tree, style, heading);
        }
        for row in rows {
            lines.push(add_menu_row(tree, style, row));
        }
        for label in labels {
            buttons.push(tree.add_button(label));
        }
    });
    MenuHandles {
        panel,
        heading: written,
        rows: lines,
        buttons,
    }
}

pub fn menu_show_on<S: Copy + PartialEq>(menus: &mut Menus<S>, screen: S, panel: Entity) {
    menus.panels.push((screen, panel));
    menus.shown = None;
}

fn alignment(style: &MenuStyle) -> TextAlignment {
    if style.centered {
        TextAlignment::Center
    } else {
        TextAlignment::Left
    }
}

pub fn add_frame(tree: &mut UiTreeBuilder, style: &CardStyle, layer: Option<u32>) {
    let Some(layer) = layer else {
        return;
    };
    let corner = style.frame_corner;
    let band = 1.0 / 3.0;
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
                .color_raw::<UiBase>(style.frame_tint)
                .without_pointer_events()
                .entity();
        }
    }
}

pub fn add_card(
    tree: &mut UiTreeBuilder,
    style: &CardStyle,
    frame_layer: Option<u32>,
    width: f32,
    height: f32,
    fill: Vec4,
) -> Entity {
    let card = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 50.0)),
            Ab(vec2(width, height)),
            Anchor::Center,
        )
        .with_rect(
            style.card_radius,
            style.card_border_width,
            style.card_border,
        )
        .color_raw::<UiBase>(fill)
        .flow(FlowDirection::Vertical, 0.0, 0.0)
        .with_visible(false)
        .entity();
    tree.in_parent(card, |tree| add_frame(tree, style, frame_layer));
    card
}

pub fn add_corner_card(
    tree: &mut UiTreeBuilder,
    style: &CardStyle,
    frame_layer: Option<u32>,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
) -> Entity {
    let card = tree
        .add_node()
        .window(
            Rl(vec2(0.0, top)) + Ab(vec2(left, 0.0)),
            Ab(vec2(width, height)),
            Anchor::TopLeft,
        )
        .with_rect(
            style.card_radius,
            style.card_border_width,
            style.card_border,
        )
        .color_raw::<UiBase>(style.card_background)
        .flow(FlowDirection::Vertical, 0.0, 0.0)
        .with_visible(false)
        .entity();
    tree.in_parent(card, |tree| add_frame(tree, style, frame_layer));
    card
}

pub fn add_title_bar(tree: &mut UiTreeBuilder, style: &CardStyle, width: f32, title: &str) {
    let bar = tree
        .add_node()
        .size_px(width, style.title_height)
        .with_rect(10.0, 0.0, vec4(0.0, 0.0, 0.0, 0.0))
        .color_raw::<UiBase>(style.title_color)
        .flow(FlowDirection::Horizontal, style.body_padding, 8.0)
        .entity();
    tree.in_parent(bar, |tree| {
        tree.add_node()
            .size_px(
                width - style.body_padding * 2.0,
                style.title_height - style.body_padding,
            )
            .with_text(title, style.heading_size)
            .text_left()
            .color_raw::<UiBase>(style.heading_color);
    });
}

pub fn add_body_column(
    tree: &mut UiTreeBuilder,
    style: &CardStyle,
    width: f32,
    height: f32,
) -> Entity {
    tree.add_node()
        .size_px(width, height)
        .flow(FlowDirection::Vertical, style.body_padding, style.row_gap)
        .entity()
}

pub fn add_scrolling_body(tree: &mut UiTreeBuilder, height: f32) -> Entity {
    let area = tree.add_scroll_area(vec2(0.0, height));
    tree.component::<UiScrollAreaData>(area)
        .map(|data| data.content_entity)
        .unwrap_or(area)
}

pub fn add_footer_bar(tree: &mut UiTreeBuilder, style: &CardStyle, width: f32) -> Entity {
    tree.add_node()
        .size_px(width, style.footer_height)
        .with_rect(10.0, 0.0, vec4(0.0, 0.0, 0.0, 0.0))
        .color_raw::<UiBase>(style.footer_color)
        .flow(FlowDirection::Horizontal, 14.0, 10.0)
        .entity()
}

pub fn add_footer_buttons(
    tree: &mut UiTreeBuilder,
    style: &CardStyle,
    width: f32,
    labels: &[&str],
) -> Vec<Entity> {
    let footer = add_footer_bar(tree, style, width);
    let count = labels.len().max(1) as f32;
    let room = width - 28.0 - style.row_spacing * (count - 1.0);
    let width = room / count;
    tree.in_parent(footer, |tree| {
        labels
            .iter()
            .map(|label| {
                let holder = tree
                    .add_node()
                    .size_px(width, style.action_height)
                    .flow(FlowDirection::Horizontal, 0.0, 0.0)
                    .entity();
                tree.in_parent(holder, |tree| tree.add_button(label))
            })
            .collect()
    })
}

pub fn add_section_label(tree: &mut UiTreeBuilder, style: &CardStyle, text: &str, width: f32) {
    tree.add_node()
        .size_px(width, style.small_size + 10.0)
        .with_text(text, style.small_size)
        .text_left()
        .color_raw::<UiBase>(style.dim_color);
}

pub fn add_cell_text(
    tree: &mut UiTreeBuilder,
    style: &CardStyle,
    text: &str,
    width: f32,
    size: f32,
    color: Vec4,
) -> Entity {
    tree.add_node()
        .size_px(width, style.row_height - style.row_padding * 2.0)
        .with_text(text, size)
        .with_text_overflow(TextOverflow::Ellipsis)
        .text_left()
        .color_raw::<UiBase>(color)
        .entity()
}

pub fn add_row_strip(
    tree: &mut UiTreeBuilder,
    style: &CardStyle,
    index: usize,
    width: f32,
) -> Entity {
    let color = if index.is_multiple_of(2) {
        style.row_color
    } else {
        style.row_alt_color
    };
    tree.add_node()
        .size_px(width, style.row_height)
        .with_rect(6.0, 0.0, vec4(0.0, 0.0, 0.0, 0.0))
        .color_raw::<UiBase>(color)
        .flow(
            FlowDirection::Horizontal,
            style.row_padding,
            style.row_spacing,
        )
        .entity()
}

pub fn add_stat_row(
    tree: &mut UiTreeBuilder,
    style: &CardStyle,
    index: usize,
    label: &str,
    width: f32,
    name_width: f32,
) -> Entity {
    let row = add_row_strip(tree, style, index, width);
    let mut value = Entity::default();
    tree.in_parent(row, |tree| {
        add_cell_text(
            tree,
            style,
            label,
            name_width,
            style.body_size,
            style.dim_color,
        );
        let rest = width - name_width - style.row_spacing - style.row_padding * 2.0;
        value = add_cell_text(tree, style, "", rest, style.body_size, style.bright_color);
    });
    value
}

pub fn add_action_button(tree: &mut UiTreeBuilder, style: &CardStyle, text: &str) -> Entity {
    let holder = tree
        .add_node()
        .size_px(style.action_width, style.action_height)
        .flow(FlowDirection::Horizontal, 0.0, 0.0)
        .entity();
    tree.in_parent(holder, |tree| tree.add_button(text))
}

pub fn add_wide_button(
    tree: &mut UiTreeBuilder,
    style: &CardStyle,
    width: f32,
    text: &str,
) -> Entity {
    let holder = tree
        .add_node()
        .size_px(width, style.action_height + 6.0)
        .flow(FlowDirection::Horizontal, 0.0, 0.0)
        .entity();
    tree.in_parent(holder, |tree| tree.add_button(text))
}

pub fn add_bar(
    tree: &mut UiTreeBuilder,
    style: &CardStyle,
    left: f32,
    top: f32,
    width: f32,
    color: Vec4,
) -> (Entity, Entity) {
    let frame = tree
        .add_node()
        .window(
            Ab(vec2(left, top)),
            Ab(vec2(width, style.bar_height)),
            Anchor::TopLeft,
        )
        .with_rect(4.0, 1.0, style.bar_border)
        .color_raw::<UiBase>(style.bar_background)
        .entity();
    let mut fill = Entity::default();
    let mut text = Entity::default();
    tree.in_parent(frame, |tree| {
        fill = tree
            .add_node()
            .window(
                Ab(vec2(0.0, 0.0)),
                Ab(vec2(width, style.bar_height)),
                Anchor::TopLeft,
            )
            .with_rect(4.0, 0.0, vec4(0.0, 0.0, 0.0, 0.0))
            .color_raw::<UiBase>(color)
            .entity();
        text = tree
            .add_node()
            .window(
                Ab(vec2(0.0, 0.0)),
                Ab(vec2(width, style.bar_height)),
                Anchor::TopLeft,
            )
            .with_text("", style.small_size)
            .text_center()
            .color_raw::<UiBase>(style.bright_color)
            .entity();
    });
    (fill, text)
}

pub fn set_bar_fill(
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
    style: &CardStyle,
    fill: Entity,
    width: f32,
    fraction: f32,
) {
    ui_set_window_layout(
        dirty,
        commands,
        fill,
        WindowLayout {
            position: Ab(vec2(0.0, 0.0)).into(),
            size: Ab(vec2(width * fraction.clamp(0.0, 1.0), style.bar_height)).into(),
            anchor: Anchor::TopLeft,
        },
    );
}

fn add_slot_cell(tree: &mut UiTreeBuilder, style: &CardStyle) -> SlotHandles {
    let frame = tree
        .add_node()
        .size_px(style.cell_size, style.cell_size)
        .with_rect(6.0, 2.0, style.cell_border)
        .color_raw::<UiBase>(style.cell_color)
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
        let inset = (style.cell_size - style.cell_icon) * 0.5;
        let icon = tree
            .add_node()
            .window(
                Ab(vec2(inset, 5.0)),
                Ab(vec2(style.cell_icon, style.cell_icon)),
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
                Ab(vec2(34.0, style.count_size)),
                Anchor::BottomRight,
            )
            .with_text("", style.count_size)
            .text_right()
            .with_text_outline(vec4(0.0, 0.0, 0.0, 1.0), 1.0)
            .color_raw::<UiBase>(style.text_color)
            .entity();
        (icon, count)
    });
    SlotHandles { frame, icon, count }
}

pub fn add_slot_row(
    tree: &mut UiTreeBuilder,
    style: &CardStyle,
    row_width: f32,
    count: usize,
) -> Vec<SlotHandles> {
    let width = style.cell_size * count as f32 + style.cell_gap * (count as f32 - 1.0);
    let inset = (row_width - width) * 0.5;
    let row = tree
        .add_node()
        .size_px(row_width, style.cell_size + 10.0)
        .flow(FlowDirection::Horizontal, inset, style.cell_gap)
        .entity();
    let mut cells = Vec::new();
    tree.in_parent(row, |tree| {
        for _ in 0..count {
            cells.push(add_slot_cell(tree, style));
        }
    });
    cells
}
