use nightshade::prelude::{Entity, vec2, vec4};
use nightshade_ui::commands::builder::UiTreeBuilder;
use nightshade_ui::components::UiButtonData;
use nightshade_ui::prelude::*;

use crate::commands::ink::BookInk;
use crate::data::demo::DemoControl;
use crate::data::style::{
    CARD_HEADER_HEIGHT, CARD_RADIUS, CHOICE_WIDTH, CONTROL_GAP, CONTROL_HEIGHT, CONTROL_WIDTH,
    HAIRLINE, LABEL_LINE, LABEL_SIZE, SLIDER_WIDTH, SOURCE_BUTTON_HEIGHT, SOURCE_BUTTON_WIDTH,
    SPACE_2, SPACE_3, SPACE_5, STAGE_HEIGHT,
};

pub const EXPAND_LABEL: &str = "Expand";
pub const RESTORE_LABEL: &str = "Restore";

pub struct Figure {
    pub card: Entity,
    pub stage: Entity,
    pub controls: Vec<(&'static str, Entity, ControlKind)>,
    pub source_button: Entity,
    pub expand_button: Entity,
    pub expand_label: Entity,
    pub card_height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlKind {
    Button,
    Slider,
    Toggle,
    Choice,
}

pub fn add_figure(
    tree: &mut UiTreeBuilder,
    number: usize,
    title: &str,
    controls: &[DemoControl],
) -> Figure {
    let theme = tree.active_theme().clone();
    let strip = if controls.is_empty() {
        0.0
    } else {
        CONTROL_HEIGHT + SPACE_3 * 2.0
    };

    let card_height = CARD_HEADER_HEIGHT + STAGE_HEIGHT + strip;
    let card = tree
        .add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, card_height)))
        .with_rect(CARD_RADIUS, theme.border_width, theme.border_color)
        .with_theme_border_color(ThemeColor::Border)
        .color_raw::<UiBase>(vec4(0.0, 0.0, 0.0, 0.0))
        .with_theme_shadow_role::<UiBase>(ThemeShadow::PanelShadow)
        .flow(FlowDirection::Vertical, HAIRLINE, 0.0)
        .with_clip()
        .entity();

    let mut stage = Entity::default();
    let mut built = Vec::new();
    let mut header = Header::default();

    tree.in_parent(card, |tree| {
        header = add_header(tree, number, title);
        stage = add_stage(tree);
        if !controls.is_empty() {
            built = add_controls(tree, controls);
        }
    });

    Figure {
        card,
        stage,
        controls: built,
        source_button: header.source,
        expand_button: header.expand,
        expand_label: header.expand_label,
        card_height,
    }
}

#[derive(Default)]
struct Header {
    source: Entity,
    expand: Entity,
    expand_label: Entity,
}

fn add_header(tree: &mut UiTreeBuilder, number: usize, title: &str) -> Header {
    let mut built = Header::default();
    let header = tree
        .add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, CARD_HEADER_HEIGHT)))
        .rect(0.0)
        .fg(ThemeColor::PanelHeader)
        .flow(FlowDirection::Horizontal, SPACE_3, SPACE_2)
        .align_cross(FlowAlignment::Center)
        .entity();

    tree.in_parent(header, |tree| {
        let label = format!("Figure {number}");
        tree.add_node()
            .flow_child(Ab(vec2(0.0, LABEL_LINE)))
            .auto_size(AutoSizeMode::Width)
            .with_text(&label, LABEL_SIZE)
            .text_left()
            .ink();

        if title.is_empty() {
            tree.add_node()
                .flow_child(Ab(vec2(0.0, LABEL_LINE)))
                .flex_grow(1.0);
        } else {
            tree.add_node()
                .flow_child(Ab(vec2(0.0, LABEL_LINE)))
                .flex_grow(1.0)
                .with_clip()
                .with_text(title, LABEL_SIZE)
                .text_left()
                .ink_dim();
        }

        let (expand, expand_label) = add_strip_button(
            tree,
            nightshade_ui::data::icons::material::CROP_FREE,
            EXPAND_LABEL,
        );
        built.expand = expand;
        built.expand_label = expand_label;
        built.source =
            add_strip_button(tree, nightshade_ui::data::icons::material::CODE, "Source").0;
    });

    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, HAIRLINE)))
        .rect(0.0)
        .fg(ThemeColor::Border);

    built
}

fn add_strip_button(
    tree: &mut UiTreeBuilder,
    icon: nightshade_ui::data::icons::IconGlyph,
    label: &str,
) -> (Entity, Entity) {
    let theme = tree.active_theme().clone();
    let button = tree
        .add_node()
        .flow_child(Ab(vec2(SOURCE_BUTTON_WIDTH, SOURCE_BUTTON_HEIGHT)))
        .with_rect(4.0, theme.border_width, theme.border_color)
        .with_theme_border_color(ThemeColor::Border)
        .with_theme_color::<UiBase>(ThemeColor::InputBackground)
        .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
        .with_interaction()
        .with_transition::<UiHover>(10.0, 8.0)
        .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
        .flow(FlowDirection::Horizontal, SPACE_2, SPACE_2)
        .align_cross(FlowAlignment::Center)
        .entity();

    let mut word = Entity::default();
    tree.in_parent(button, |tree| {
        tree.add_node()
            .flow_child(Ab(vec2(13.0, SOURCE_BUTTON_HEIGHT)))
            .with_icon(icon, 13.0)
            .text_center()
            .ink();
        word = tree
            .add_node()
            .flow_child(Ab(vec2(0.0, SOURCE_BUTTON_HEIGHT)))
            .flex_grow(1.0)
            .with_text(label, LABEL_SIZE)
            .text_left()
            .ink()
            .entity();
    });

    (button, word)
}

fn add_stage(tree: &mut UiTreeBuilder) -> Entity {
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, STAGE_HEIGHT)))
        .with_rect(0.0, 0.0, vec4(0.0, 0.0, 0.0, 0.0))
        .color_raw::<UiBase>(vec4(0.0, 0.0, 0.0, 0.0))
        .with_interaction()
        .with_cursor_icon(nightshade_platform::CursorIcon::Grab)
        .entity()
}

fn add_controls(
    tree: &mut UiTreeBuilder,
    controls: &[DemoControl],
) -> Vec<(&'static str, Entity, ControlKind)> {
    let mut built = Vec::with_capacity(controls.len());

    let strip = tree
        .add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, CONTROL_HEIGHT + SPACE_3 * 2.0)))
        .rect(0.0)
        .fg(ThemeColor::PanelHeader)
        .flow(FlowDirection::Horizontal, SPACE_3, CONTROL_GAP)
        .align_cross(FlowAlignment::Center)
        .entity();

    tree.in_parent(strip, |tree| {
        for control in controls {
            match control {
                DemoControl::Button { label, action } => {
                    let entity = add_button(tree, label);
                    built.push((*action, entity, ControlKind::Button));
                }
                DemoControl::Slider {
                    label,
                    action,
                    lowest,
                    highest,
                    start,
                } => {
                    let entity = add_slider(tree, label, *lowest, *highest, *start);
                    built.push((*action, entity, ControlKind::Slider));
                }
                DemoControl::Toggle {
                    label,
                    action,
                    start,
                } => {
                    let entity = add_toggle(tree, label, *start);
                    built.push((*action, entity, ControlKind::Toggle));
                }
                DemoControl::Choice {
                    label,
                    action,
                    options,
                    start,
                } => {
                    let entity = add_choice(tree, label, options, *start);
                    built.push((*action, entity, ControlKind::Choice));
                }
            }
        }
    });

    built
}

fn add_button(tree: &mut UiTreeBuilder, label: &str) -> Entity {
    let theme = tree.active_theme().clone();
    let text_slot = tree.text_state_mut().cache.add_text(label);

    let button = tree
        .add_node()
        .flow_child(Ab(vec2(CONTROL_WIDTH, CONTROL_HEIGHT)))
        .with_rect(4.0, theme.border_width, theme.border_color)
        .with_theme_border_color(ThemeColor::Border)
        .with_theme_color::<UiBase>(ThemeColor::Background)
        .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
        .with_theme_color::<UiPressed>(ThemeColor::BackgroundActive)
        .with_interaction()
        .with_transition::<UiHover>(10.0, 8.0)
        .with_transition::<UiPressed>(14.0, 10.0)
        .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
        .with_children(|tree| {
            tree.add_node()
                .boundary(Ab(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
                .with_text_slot(text_slot, LABEL_SIZE)
                .text_center()
                .ink();
        })
        .entity();

    tree.commands().set(
        button,
        UiButtonData {
            clicked: false,
            text_slot: Some(text_slot),
        },
    );
    button
}

fn add_slider(
    tree: &mut UiTreeBuilder,
    label: &str,
    lowest: f32,
    highest: f32,
    start: f32,
) -> Entity {
    let holder = tree
        .add_node()
        .flow_child(Ab(vec2(SLIDER_WIDTH, CONTROL_HEIGHT)))
        .flow(FlowDirection::Horizontal, 0.0, SPACE_2)
        .entity();

    let mut slider = Entity::default();
    tree.in_parent(holder, |tree| {
        tree.add_node()
            .flow_child(Ab(vec2(0.0, CONTROL_HEIGHT)))
            .auto_size(AutoSizeMode::Width)
            .with_text(label, LABEL_SIZE)
            .text_left()
            .ink_dim();

        let track = tree
            .add_node()
            .flow_child(Ab(vec2(0.0, CONTROL_HEIGHT)))
            .flex_grow(1.0)
            .flow(FlowDirection::Vertical, 0.0, 0.0)
            .entity();
        tree.in_parent(track, |tree| {
            slider = tree.add_slider(lowest, highest, start);
        });
    });
    slider
}

fn add_choice(tree: &mut UiTreeBuilder, label: &str, options: &[&str], start: usize) -> Entity {
    let holder = tree
        .add_node()
        .flow_child(Ab(vec2(CHOICE_WIDTH, CONTROL_HEIGHT)))
        .flow(FlowDirection::Horizontal, 0.0, SPACE_2)
        .align_cross(FlowAlignment::Center)
        .entity();

    let mut dropdown = Entity::default();
    tree.in_parent(holder, |tree| {
        tree.add_node()
            .flow_child(Ab(vec2(0.0, CONTROL_HEIGHT)))
            .auto_size(AutoSizeMode::Width)
            .with_text(label, LABEL_SIZE)
            .text_left()
            .ink_dim();

        let list = tree
            .add_node()
            .flow_child(Ab(vec2(0.0, CONTROL_HEIGHT)))
            .flex_grow(1.0)
            .flow(FlowDirection::Vertical, 0.0, 0.0)
            .entity();
        tree.in_parent(list, |tree| {
            dropdown = tree.add_dropdown(options, start);
        });
    });
    dropdown
}

fn add_toggle(tree: &mut UiTreeBuilder, label: &str, start: bool) -> Entity {
    let holder = tree
        .add_node()
        .flow_child(Ab(vec2(CONTROL_WIDTH + SPACE_5, CONTROL_HEIGHT)))
        .flow(FlowDirection::Horizontal, 0.0, SPACE_2)
        .entity();

    let mut toggle = Entity::default();
    tree.in_parent(holder, |tree| {
        toggle = tree.add_toggle(start);
        tree.add_node()
            .flow_child(Ab(vec2(0.0, CONTROL_HEIGHT)))
            .auto_size(AutoSizeMode::Width)
            .with_text(label, LABEL_SIZE)
            .text_left()
            .ink_dim();
    });
    toggle
}
