use nalgebra_glm;
use nalgebra_glm::{Vec2, Vec4};

use crate::commands::builder::UiTreeBuilder;
use crate::data::state::{UiBase, UiFocused, UiHover};
use crate::data::units::{Ab, Rl};
use nightshade_paint::data::Anchor;
use nightshade_text::contract::data::text_data::{TextAlignment, VerticalAlignment};

use crate::prelude::*;

struct MultilineTextScaffold {
    input_entity: nightshade::ecs::Entity,
    text_slot: usize,
    cursor_entity: nightshade::ecs::Entity,
    selection_pool: Vec<nightshade::ecs::Entity>,
    placeholder_entity: Option<nightshade::ecs::Entity>,
    line_height: f32,
}

impl<'a, 'c> UiTreeBuilder<'a, 'c> {
    pub fn add_text_input(&mut self, placeholder: &str) -> nightshade::ecs::Entity {
        self.add_text_input_inner(placeholder, "")
    }

    pub fn add_text_input_with_value(
        &mut self,
        placeholder: &str,
        initial_text: &str,
    ) -> nightshade::ecs::Entity {
        self.add_text_input_inner(placeholder, initial_text)
    }

    fn add_text_input_inner(
        &mut self,
        placeholder: &str,
        initial_text: &str,
    ) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let font_size = theme.font_size;
        let accent_color = theme.accent_color;
        let corner_radius = theme.corner_radius;
        let border_color = theme.border_color;
        let input_height = theme.button_height;

        let text_slot = self.text_state_mut().cache.add_text(initial_text);
        let placeholder_text = placeholder.to_string();
        let has_placeholder = !placeholder_text.is_empty();

        let input_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, input_height)))
            .with_rect(corner_radius, 1.0, border_color)
            .with_theme_border_color(ThemeColor::Border)
            .with_theme_color::<UiBase>(ThemeColor::InputBackground)
            .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
            .with_theme_color::<UiFocused>(ThemeColor::InputBackgroundFocused)
            .with_theme_effect_role(crate::components::ThemeEffect::InputEffect)
            .with_theme_shadow_role::<crate::data::state::UiFocused>(
                crate::components::ThemeShadow::FocusGlow,
            )
            .with_interaction()
            .with_transition::<UiHover>(10.0, 6.0)
            .with_transition::<UiFocused>(8.0, 6.0)
            .with_cursor_icon(nightshade_platform::CursorIcon::Text)
            .with_clip()
            .entity();

        let (selection_entity, cursor_entity, placeholder_entity_out) =
            self.in_parent(input_entity, |tree| {
                let selection_entity = tree
                    .add_node()
                    .window(
                        Ab(Vec2::new(8.0, 4.0)),
                        Ab(Vec2::new(0.0, input_height - 8.0)),
                        Anchor::TopLeft,
                    )
                    .with_rect(2.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                    .color_raw::<UiBase>(Vec4::new(
                        accent_color.x,
                        accent_color.y,
                        accent_color.z,
                        0.3,
                    ))
                    .with_visible(false)
                    .entity();

                tree.add_node()
                    .window(
                        Ab(Vec2::new(8.0, 0.0)),
                        Ab(Vec2::new(0.0, input_height)) + Rl(Vec2::new(100.0, 0.0)),
                        Anchor::TopLeft,
                    )
                    .with_text_slot(text_slot, font_size)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                    .with_theme_color::<UiBase>(ThemeColor::Text)
                    .entity();

                let placeholder_entity_out = if has_placeholder {
                    let placeholder_slot = tree.text_state_mut().cache.add_text(&placeholder_text);
                    Some(
                        tree.add_node()
                            .window(
                                Ab(Vec2::new(8.0, 0.0)),
                                Ab(Vec2::new(0.0, input_height)) + Rl(Vec2::new(100.0, 0.0)),
                                Anchor::TopLeft,
                            )
                            .with_text_slot(placeholder_slot, font_size)
                            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                            .with_theme_color::<UiBase>(ThemeColor::TextDisabled)
                            .entity(),
                    )
                } else {
                    None
                };

                let cursor_entity = tree
                    .add_node()
                    .window(
                        Ab(Vec2::new(8.0, 4.0)),
                        Ab(Vec2::new(2.0, input_height - 8.0)),
                        Anchor::TopLeft,
                    )
                    .with_rect(1.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                    .with_theme_color::<UiBase>(ThemeColor::Text)
                    .with_visible(false)
                    .entity();

                (selection_entity, cursor_entity, placeholder_entity_out)
            });

        self.commands().set(
            input_entity,
            UiTextInputData {
                text: initial_text.to_string(),
                cursor_position: initial_text.chars().count(),
                selection_start: None,
                changed: false,
                text_slot,
                cursor_entity,
                selection_entity,
                scroll_offset: 0.0,
                cursor_blink_timer: 0.0,
                placeholder_entity: placeholder_entity_out,
                undo_stack: std::sync::Arc::new(UndoStack::new(100)),
                input_mask: crate::components::InputMask::None,
                max_length: None,
                password: false,
            },
        );
        self.edit::<crate::components::UiNodeInteraction>(input_entity, |interaction| {
            interaction.accessible_role = Some(AccessibleRole::TextInput);
        });
        if !initial_text.is_empty()
            && let Some(ph_entity) = placeholder_entity_out
        {
            self.edit::<crate::components::UiLayoutNode>(ph_entity, |node| {
                node.visible = false;
            });
        }
        self.assign_tab_index(input_entity);

        input_entity
    }

    fn build_multiline_text_scaffold(
        &mut self,
        placeholder: &str,
        rows: usize,
    ) -> MultilineTextScaffold {
        let theme = self.active_theme();
        let font_size = theme.font_size;
        let accent_color = theme.accent_color;
        let corner_radius = theme.corner_radius;
        let border_color = theme.border_color;
        let line_height = font_size * 1.4;
        let area_height = line_height * rows as f32 + 16.0;

        let text_slot = self.text_state_mut().cache.add_text("");
        let placeholder_text = placeholder.to_string();
        let has_placeholder = !placeholder_text.is_empty();

        let input_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, area_height)))
            .with_rect(corner_radius, 1.0, border_color)
            .with_theme_border_color(ThemeColor::Border)
            .with_theme_color::<UiBase>(ThemeColor::InputBackground)
            .with_theme_color::<UiFocused>(ThemeColor::InputBackgroundFocused)
            .with_interaction()
            .with_transition::<UiFocused>(8.0, 6.0)
            .with_cursor_icon(nightshade_platform::CursorIcon::Text)
            .with_clip()
            .entity();

        let (cursor_entity, selection_pool, placeholder_entity_out) =
            self.in_parent(input_entity, |tree| {
                let mut selection_pool = Vec::new();
                for _ in 0..rows + 1 {
                    let sel = tree
                        .add_node()
                        .window(
                            Ab(Vec2::new(8.0, 0.0)),
                            Ab(Vec2::new(0.0, line_height)),
                            Anchor::TopLeft,
                        )
                        .with_rect(2.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                        .color_raw::<UiBase>(Vec4::new(
                            accent_color.x,
                            accent_color.y,
                            accent_color.z,
                            0.3,
                        ))
                        .with_visible(false)
                        .entity();
                    selection_pool.push(sel);
                }

                tree.add_node()
                    .window(
                        Ab(Vec2::new(8.0, 8.0)),
                        Rl(Vec2::new(100.0, 100.0)) + Ab(Vec2::new(-16.0, 0.0)),
                        Anchor::TopLeft,
                    )
                    .with_text_slot(text_slot, font_size)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
                    .with_theme_color::<UiBase>(ThemeColor::Text)
                    .entity();

                let placeholder_entity_out = if has_placeholder {
                    let placeholder_slot = tree.text_state_mut().cache.add_text(&placeholder_text);
                    Some(
                        tree.add_node()
                            .window(
                                Ab(Vec2::new(8.0, 8.0)),
                                Rl(Vec2::new(100.0, 100.0)) + Ab(Vec2::new(-16.0, 0.0)),
                                Anchor::TopLeft,
                            )
                            .with_text_slot(placeholder_slot, font_size)
                            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
                            .with_theme_color::<UiBase>(ThemeColor::TextDisabled)
                            .entity(),
                    )
                } else {
                    None
                };

                let cursor_entity = tree
                    .add_node()
                    .window(
                        Ab(Vec2::new(8.0, 8.0)),
                        Ab(Vec2::new(2.0, line_height)),
                        Anchor::TopLeft,
                    )
                    .with_rect(1.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                    .with_theme_color::<UiBase>(ThemeColor::Text)
                    .with_visible(false)
                    .entity();

                (cursor_entity, selection_pool, placeholder_entity_out)
            });

        self.edit::<crate::components::UiNodeInteraction>(input_entity, |interaction| {
            interaction.accessible_role = Some(AccessibleRole::TextArea);
        });
        self.assign_tab_index(input_entity);

        MultilineTextScaffold {
            input_entity,
            text_slot,
            cursor_entity,
            selection_pool,
            placeholder_entity: placeholder_entity_out,
            line_height,
        }
    }

    pub fn add_text_area(&mut self, placeholder: &str, rows: usize) -> nightshade::ecs::Entity {
        let scaffold = self.build_multiline_text_scaffold(placeholder, rows);
        self.commands().set(
            scaffold.input_entity,
            UiTextAreaData {
                text: String::new(),
                cursor_position: 0,
                selection_start: None,
                changed: false,
                text_slot: scaffold.text_slot,
                cursor_entity: scaffold.cursor_entity,
                selection_pool: scaffold.selection_pool,
                scroll_offset_y: 0.0,
                cursor_blink_timer: 0.0,
                placeholder_entity: scaffold.placeholder_entity,
                line_height: scaffold.line_height,
                visible_rows: rows,
                undo_stack: UndoStack::new(100),
                max_length: None,
            },
        );
        scaffold.input_entity
    }

    pub fn add_rich_text_editor(
        &mut self,
        placeholder: &str,
        rows: usize,
    ) -> nightshade::ecs::Entity {
        let scaffold = self.build_multiline_text_scaffold(placeholder, rows);
        self.commands().set(
            scaffold.input_entity,
            UiRichTextEditorData {
                text: String::new(),
                char_styles: Vec::new(),
                current_style: CharStyle::default(),
                cursor_position: 0,
                selection_start: None,
                changed: false,
                text_slot: scaffold.text_slot,
                cursor_entity: scaffold.cursor_entity,
                selection_pool: scaffold.selection_pool,
                scroll_offset_y: 0.0,
                cursor_blink_timer: 0.0,
                line_height: scaffold.line_height,
                visible_rows: rows,
                placeholder_entity: scaffold.placeholder_entity,
                undo_stack: UndoStack::new(100),
            },
        );
        scaffold.input_entity
    }
}
