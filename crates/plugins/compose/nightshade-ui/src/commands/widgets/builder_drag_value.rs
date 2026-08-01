use nalgebra_glm;
use nalgebra_glm::{Vec2, Vec4};

use crate::commands::builder::UiTreeBuilder;
use crate::components::layout_types::FlowDirection;
use crate::components::*;
use crate::data::state::{UiBase, UiFocused, UiHover};
use crate::data::units::{Ab, Rl};
use nightshade_paint::data::Anchor;
use nightshade_text::contract::data::text_data::{TextAlignment, VerticalAlignment};

impl<'a, 'c> UiTreeBuilder<'a, 'c> {
    pub fn add_drag_value(&mut self, min: f32, max: f32, initial: f32) -> nightshade::ecs::Entity {
        self.add_drag_value_configured(DragValueConfig::new(min, max, initial))
    }

    pub fn add_drag_value_configured(
        &mut self,
        config: DragValueConfig<'_>,
    ) -> nightshade::ecs::Entity {
        let initial = config.initial;
        let min = config.min;
        let max = config.max;
        let speed = config.speed;
        let precision = config.precision;
        let prefix = config.prefix;
        let suffix = config.suffix;
        let show_arrows = config.show_arrows;
        let theme = self.active_theme();
        let font_size = theme.font_size;
        let accent_color = theme.accent_color;
        let corner_radius = theme.corner_radius;
        let border_color = theme.border_color;
        let input_height = theme.button_height;

        let display_text = format!("{prefix}{initial:.prec$}{suffix}", prec = precision);
        let text_slot = self.text_state_mut().cache.add_text(&display_text);
        let wrapper_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, input_height)))
            .flow(FlowDirection::Horizontal, 0.0, 0.0)
            .entity();

        self.push_parent(wrapper_entity);

        let root_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(0.0, 100.0)) + Ab(Vec2::new(0.0, 0.0)))
            .flex_grow(1.0)
            .with_rect(corner_radius, 1.0, border_color)
            .with_theme_border_color(ThemeColor::Border)
            .with_theme_color::<UiBase>(ThemeColor::InputBackground)
            .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
            .with_theme_color::<UiFocused>(ThemeColor::InputBackgroundFocused)
            .with_theme_effect_role(crate::components::ThemeEffect::InputEffect)
            .with_theme_shadow_role::<UiFocused>(crate::components::ThemeShadow::FocusGlow)
            .with_interaction()
            .with_transition::<UiHover>(10.0, 6.0)
            .with_transition::<UiFocused>(8.0, 6.0)
            .with_cursor_icon(nightshade_platform::CursorIcon::EwResize)
            .with_clip()
            .entity();

        let (selection_entity, cursor_entity) = self.in_parent(root_entity, |tree| {
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

            (selection_entity, cursor_entity)
        });

        let mut up_entity = None;
        let mut down_entity = None;
        if show_arrows {
            let arrow_width = 18.0;
            let half_height = input_height / 2.0;

            let arrow_col = self
                .add_node()
                .flow_child(Ab(Vec2::new(arrow_width, input_height)))
                .flow(FlowDirection::Vertical, 0.0, 0.0)
                .entity();
            self.push_parent(arrow_col);

            let up_slot = self.text_state_mut().cache.add_text("\u{25B2}");
            up_entity = Some(
                self.add_node()
                    .flow_child(Ab(Vec2::new(arrow_width, half_height)))
                    .with_rect(corner_radius, 1.0, border_color)
                    .with_theme_border_color(ThemeColor::Border)
                    .with_theme_color::<UiBase>(ThemeColor::InputBackground)
                    .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
                    .with_interaction()
                    .with_transition::<UiHover>(8.0, 6.0)
                    .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
                    .with_children(|tree| {
                        tree.add_node()
                            .boundary(Ab(Vec2::zeros()), Rl(Vec2::new(100.0, 100.0)))
                            .with_text_slot(up_slot, font_size * 0.5)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .with_theme_color::<UiBase>(ThemeColor::Text)
                            .entity();
                    })
                    .entity(),
            );

            let down_slot = self.text_state_mut().cache.add_text("\u{25BC}");
            down_entity = Some(
                self.add_node()
                    .flow_child(Ab(Vec2::new(arrow_width, half_height)))
                    .with_rect(corner_radius, 1.0, border_color)
                    .with_theme_border_color(ThemeColor::Border)
                    .with_theme_color::<UiBase>(ThemeColor::InputBackground)
                    .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
                    .with_interaction()
                    .with_transition::<UiHover>(8.0, 6.0)
                    .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
                    .with_children(|tree| {
                        tree.add_node()
                            .boundary(Ab(Vec2::zeros()), Rl(Vec2::new(100.0, 100.0)))
                            .with_text_slot(down_slot, font_size * 0.5)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .with_theme_color::<UiBase>(ThemeColor::Text)
                            .entity();
                    })
                    .entity(),
            );

            self.pop_parent();
        }

        self.pop_parent();

        let step_val = speed * 10.0;
        self.commands().set(
            root_entity,
            UiDragValueData {
                value: initial,
                min,
                max,
                speed,
                precision,
                prefix: prefix.to_string(),
                suffix: suffix.to_string(),
                changed: false,
                text_slot,
                editing: false,
                cursor_entity,
                selection_entity,
                edit_text: String::new(),
                cursor_position: 0,
                selection_start: None,
                cursor_blink_timer: 0.0,
                scroll_offset: 0.0,
                drag_start_value: initial,
                undo_stack: UndoStack::new(100),
                up_entity,
                down_entity,
                step: step_val,
            },
        );
        self.edit::<crate::components::UiNodeInteraction>(root_entity, |interaction| {
            interaction.accessible_role = Some(AccessibleRole::Slider);
        });
        self.assign_tab_index(root_entity);

        root_entity
    }
}
