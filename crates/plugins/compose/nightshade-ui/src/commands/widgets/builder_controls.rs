use nalgebra_glm;
use nalgebra_glm::{Vec2, Vec4};

use crate::commands::builder::UiTreeBuilder;
use crate::components::layout_types::FlowDirection;
use crate::components::*;
use crate::data::state::{UiBase, UiFocused, UiHover, UiPressed};
use crate::data::units::{Ab, Rl};
use nightshade_paint::data::Anchor;
use nightshade_text::contract::data::text_data::{TextAlignment, VerticalAlignment};

impl<'a, 'c> UiTreeBuilder<'a, 'c> {
    pub fn add_slider(&mut self, min: f32, max: f32, initial: f32) -> nightshade::ecs::Entity {
        self.add_slider_configured(crate::components::SliderConfig::new(min, max, initial))
    }

    pub fn add_slider_configured(
        &mut self,
        config: crate::components::SliderConfig<'_>,
    ) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let slider_height = theme.slider_height;
        let corner_radius = theme.corner_radius;
        let font_size = theme.font_size;

        let normalized = if config.logarithmic && config.min > 0.0 && config.max > config.min {
            ((config.initial / config.min).ln() / (config.max / config.min).ln()).clamp(0.0, 1.0)
        } else {
            ((config.initial - config.min) / (config.max - config.min)).clamp(0.0, 1.0)
        };
        let text_slot = self.text_state_mut().cache.add_text(format!(
            "{}{:.prec$}{}",
            config.prefix,
            config.initial,
            config.suffix,
            prec = config.precision
        ));

        let track_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, slider_height)))
            .with_rect(corner_radius, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
            .with_theme_color::<UiBase>(ThemeColor::InputBackground)
            .with_theme_color::<UiHover>(ThemeColor::InputBackgroundFocused)
            .with_theme_effect_role(crate::components::ThemeEffect::InputEffect)
            .with_theme_shadow_role::<UiFocused>(crate::components::ThemeShadow::FocusGlow)
            .with_interaction()
            .with_transition::<UiHover>(8.0, 6.0)
            .with_transition::<UiFocused>(8.0, 6.0)
            .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
            .entity();

        let fill_entity = self.in_parent(track_entity, |tree| {
            let fill_entity = tree
                .add_node()
                .boundary(
                    Rl(Vec2::new(0.0, 0.0)),
                    Rl(Vec2::new(normalized * 100.0, 100.0)),
                )
                .with_rect(corner_radius, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                .with_theme_color::<UiBase>(ThemeColor::SliderFill)
                .entity();

            tree.add_node()
                .with_text_slot(text_slot, font_size * 0.85)
                .with_theme_color::<UiBase>(ThemeColor::Text)
                .entity();

            fill_entity
        });

        self.commands().set(
            track_entity,
            UiSliderData {
                min: config.min,
                max: config.max,
                value: config.initial,
                changed: false,
                fill_entity,
                text_slot,
                logarithmic: config.logarithmic,
                precision: config.precision,
                prefix: config.prefix.to_string(),
                suffix: config.suffix.to_string(),
            },
        );
        self.edit::<crate::components::UiNodeInteraction>(track_entity, |interaction| {
            interaction.accessible_role = Some(AccessibleRole::Slider);
        });
        self.assign_tab_index(track_entity);

        track_entity
    }

    pub fn add_range_slider(
        &mut self,
        min: f32,
        max: f32,
        initial_low: f32,
        initial_high: f32,
    ) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let slider_height = theme.slider_height;
        let corner_radius = theme.corner_radius;
        let font_size = theme.font_size;
        let thumb_size = slider_height;

        let range = max - min;
        let low_normalized = if range.abs() > f32::EPSILON {
            ((initial_low - min) / range).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let high_normalized = if range.abs() > f32::EPSILON {
            ((initial_high - min) / range).clamp(0.0, 1.0)
        } else {
            1.0
        };

        let text_slot = self
            .text_state_mut()
            .cache
            .add_text(format!("{:.1} - {:.1}", initial_low, initial_high));

        let track_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, slider_height)))
            .with_rect(corner_radius, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
            .with_theme_color::<UiBase>(ThemeColor::InputBackground)
            .with_theme_color::<UiHover>(ThemeColor::InputBackgroundFocused)
            .with_interaction()
            .with_transition::<UiHover>(8.0, 6.0)
            .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
            .entity();

        let (fill_entity, low_thumb_entity, high_thumb_entity) =
            self.in_parent(track_entity, |tree| {
                let fill_entity = tree
                    .add_node()
                    .boundary(
                        Rl(Vec2::new(low_normalized * 100.0, 0.0)),
                        Rl(Vec2::new(high_normalized * 100.0, 100.0)),
                    )
                    .with_rect(corner_radius, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                    .with_theme_color::<UiBase>(ThemeColor::SliderFill)
                    .entity();

                let low_thumb_entity = tree
                    .add_node()
                    .boundary(
                        Rl(Vec2::new(low_normalized * 100.0, 0.0))
                            + Ab(Vec2::new(-thumb_size * 0.5, 0.0)),
                        Rl(Vec2::new(low_normalized * 100.0, 100.0))
                            + Ab(Vec2::new(thumb_size * 0.5, 0.0)),
                    )
                    .with_rect(thumb_size * 0.5, 1.0, Vec4::new(1.0, 1.0, 1.0, 0.3))
                    .with_theme_color::<UiBase>(ThemeColor::Text)
                    .entity();

                let high_thumb_entity = tree
                    .add_node()
                    .boundary(
                        Rl(Vec2::new(high_normalized * 100.0, 0.0))
                            + Ab(Vec2::new(-thumb_size * 0.5, 0.0)),
                        Rl(Vec2::new(high_normalized * 100.0, 100.0))
                            + Ab(Vec2::new(thumb_size * 0.5, 0.0)),
                    )
                    .with_rect(thumb_size * 0.5, 1.0, Vec4::new(1.0, 1.0, 1.0, 0.3))
                    .with_theme_color::<UiBase>(ThemeColor::Text)
                    .entity();

                tree.add_node()
                    .with_text_slot(text_slot, font_size * 0.85)
                    .with_theme_color::<UiBase>(ThemeColor::Text)
                    .entity();

                (fill_entity, low_thumb_entity, high_thumb_entity)
            });

        self.commands().set(
            track_entity,
            UiRangeSliderData {
                min,
                max,
                low_value: initial_low,
                high_value: initial_high,
                changed: false,
                fill_entity,
                low_thumb_entity,
                high_thumb_entity,
                text_slot,
                active_thumb: None,
                precision: 1,
                thumb_half_size: thumb_size * 0.5,
            },
        );
        self.edit::<crate::components::UiNodeInteraction>(track_entity, |interaction| {
            interaction.accessible_role = Some(AccessibleRole::Slider);
        });
        self.assign_tab_index(track_entity);

        track_entity
    }

    pub fn add_canvas(&mut self, size: Vec2) -> nightshade::ecs::Entity {
        let canvas_entity = self.add_node().flow_child(Ab(size)).with_clip().entity();

        self.commands().set(
            canvas_entity,
            UiCanvasData {
                commands: Vec::new(),
                hit_test_enabled: false,
                command_bounds: Vec::new(),
            },
        );

        canvas_entity
    }

    pub fn add_canvas_interactive(&mut self, size: Vec2) -> nightshade::ecs::Entity {
        let canvas_entity = self
            .add_node()
            .flow_child(Ab(size))
            .with_clip()
            .with_interaction()
            .entity();

        self.commands().set(
            canvas_entity,
            UiCanvasData {
                commands: Vec::new(),
                hit_test_enabled: true,
                command_bounds: Vec::new(),
            },
        );

        canvas_entity
    }

    pub fn add_toggle(&mut self, initial: bool) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let bg_color = if initial {
            theme.accent_color
        } else {
            theme.background_color
        };
        let toggle_width = theme.toggle_width;
        let toggle_height = theme.toggle_height;
        let knob_padding = 2.0;
        let knob_size = toggle_height - knob_padding * 2.0;
        let knob_travel = toggle_width - knob_size - knob_padding * 2.0;

        let initial_x = if initial {
            knob_padding + knob_travel
        } else {
            knob_padding
        };
        let hover_color = Vec4::new(
            (bg_color.x + 0.05).min(1.0),
            (bg_color.y + 0.05).min(1.0),
            (bg_color.z + 0.05).min(1.0),
            bg_color.w,
        );
        let pressed_color = Vec4::new(
            (bg_color.x - 0.05).max(0.0),
            (bg_color.y - 0.05).max(0.0),
            (bg_color.z - 0.05).max(0.0),
            bg_color.w,
        );
        let toggle_entity = self
            .add_node()
            .flow_child(Ab(Vec2::new(toggle_width, toggle_height)))
            .with_rect(toggle_height / 2.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
            .color_raw::<UiBase>(bg_color)
            .color_raw::<UiHover>(hover_color)
            .color_raw::<UiPressed>(pressed_color)
            .with_theme_shadow_role::<UiBase>(crate::components::ThemeShadow::ButtonShadow)
            .with_theme_shadow_role::<UiHover>(crate::components::ThemeShadow::ButtonShadowHover)
            .with_theme_shadow_role::<UiPressed>(
                crate::components::ThemeShadow::ButtonShadowPressed,
            )
            .with_theme_shadow_role::<UiFocused>(crate::components::ThemeShadow::FocusGlow)
            .with_interaction()
            .with_transition::<UiHover>(10.0, 6.0)
            .with_transition::<UiPressed>(14.0, 8.0)
            .with_transition::<UiFocused>(8.0, 6.0)
            .with_state_scale::<UiHover>(1.04)
            .with_state_scale::<UiPressed>(0.96)
            .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
            .entity();

        let knob_entity = self.in_parent(toggle_entity, |tree| {
            tree.add_node()
                .window(
                    Ab(Vec2::new(initial_x, knob_padding)),
                    Ab(Vec2::new(knob_size, knob_size)),
                    Anchor::TopLeft,
                )
                .with_rect(knob_size / 2.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                .with_theme_color::<UiBase>(ThemeColor::Text)
                .entity()
        });

        self.commands().set(
            toggle_entity,
            UiToggleData {
                value: initial,
                changed: false,
                animated_position: if initial { 1.0 } else { 0.0 },
                knob_entity,
            },
        );
        self.edit::<crate::components::UiNodeInteraction>(toggle_entity, |interaction| {
            interaction.accessible_role = Some(AccessibleRole::Toggle);
        });
        self.assign_tab_index(toggle_entity);

        toggle_entity
    }

    pub fn add_checkbox(&mut self, label: &str, initial: bool) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let font_size = theme.font_size;
        let corner_radius = theme.corner_radius;
        let border_color = theme.border_color;

        let text_slot = self.text_state_mut().cache.add_text(label);

        let row_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.8)))
            .flow(FlowDirection::Horizontal, 0.0, 8.0)
            .entity();

        self.push_parent(row_entity);

        let box_entity = self
            .add_node()
            .flow_child(Ab(Vec2::new(20.0, 20.0)))
            .with_rect(corner_radius, 1.0, border_color)
            .with_theme_border_color(ThemeColor::Border)
            .with_theme_color::<UiBase>(ThemeColor::InputBackground)
            .with_theme_color::<UiHover>(ThemeColor::InputBackgroundFocused)
            .with_theme_color::<UiPressed>(ThemeColor::BackgroundActive)
            .with_theme_color::<UiFocused>(ThemeColor::InputBackgroundFocused)
            .with_theme_effect_role(crate::components::ThemeEffect::InputEffect)
            .with_interaction()
            .with_transition::<UiHover>(10.0, 6.0)
            .with_transition::<UiPressed>(14.0, 8.0)
            .with_transition::<UiFocused>(8.0, 6.0)
            .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
            .entity();

        let inner_entity = self.in_parent(box_entity, |tree| {
            tree.add_node()
                .boundary(
                    Ab(Vec2::new(3.0, 3.0)),
                    Ab(Vec2::new(-3.0, -3.0)) + Rl(Vec2::new(100.0, 100.0)),
                )
                .with_rect(corner_radius * 0.5, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                .with_theme_color::<UiBase>(ThemeColor::Accent)
                .with_visible(initial)
                .entity()
        });

        self.add_node()
            .flow_child(Ab(Vec2::new(0.0, 20.0)))
            .flex_grow(1.0)
            .with_text_slot(text_slot, font_size)
            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
            .with_theme_color::<UiBase>(ThemeColor::Text)
            .entity();

        self.pop_parent();

        self.commands().set(
            box_entity,
            UiCheckboxData {
                value: initial,
                changed: false,
                inner_entity,
            },
        );
        self.edit::<crate::components::UiNodeInteraction>(box_entity, |interaction| {
            interaction.accessible_role = Some(AccessibleRole::Checkbox);
        });
        self.assign_tab_index(box_entity);

        box_entity
    }

    pub fn add_radio(
        &mut self,
        label: &str,
        group_id: u32,
        option_index: usize,
    ) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let font_size = theme.font_size;
        let border_color = theme.border_color;

        let text_slot = self.text_state_mut().cache.add_text(label);
        let selected = option_index == 0;

        let row_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 1.8)))
            .flow(FlowDirection::Horizontal, 0.0, 8.0)
            .entity();

        self.push_parent(row_entity);

        let circle_entity = self
            .add_node()
            .flow_child(Ab(Vec2::new(20.0, 20.0)))
            .with_rect(10.0, 1.0, border_color)
            .with_theme_border_color(ThemeColor::Border)
            .with_theme_color::<UiBase>(ThemeColor::InputBackground)
            .with_theme_color::<UiHover>(ThemeColor::InputBackgroundFocused)
            .with_theme_color::<UiPressed>(ThemeColor::BackgroundActive)
            .with_interaction()
            .with_transition::<UiHover>(10.0, 6.0)
            .with_transition::<UiPressed>(14.0, 8.0)
            .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
            .entity();

        let inner_entity = self.in_parent(circle_entity, |tree| {
            tree.add_node()
                .boundary(
                    Ab(Vec2::new(5.0, 5.0)),
                    Ab(Vec2::new(-5.0, -5.0)) + Rl(Vec2::new(100.0, 100.0)),
                )
                .with_rect(5.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                .with_theme_color::<UiBase>(ThemeColor::Accent)
                .with_visible(selected)
                .entity()
        });

        self.add_node()
            .flow_child(Ab(Vec2::new(0.0, 20.0)))
            .flex_grow(1.0)
            .with_text_slot(text_slot, font_size)
            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
            .with_theme_color::<UiBase>(ThemeColor::Text)
            .entity();

        self.pop_parent();

        self.commands().set(
            circle_entity,
            UiRadioData {
                group_id,
                option_index,
                selected,
                changed: false,
                inner_entity,
            },
        );
        self.edit::<crate::components::UiNodeInteraction>(circle_entity, |interaction| {
            interaction.accessible_role = Some(AccessibleRole::Radio);
        });
        self.groups_mut()
            .radio
            .entry(group_id)
            .or_default()
            .push(circle_entity);
        self.assign_tab_index(circle_entity);

        circle_entity
    }

    pub fn add_progress_bar(&mut self, initial: f32) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let slider_height = theme.slider_height;
        let corner_radius = theme.corner_radius;

        let normalized = initial.clamp(0.0, 1.0);

        let bar_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, slider_height)))
            .with_rect(corner_radius, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
            .with_theme_color::<UiBase>(ThemeColor::InputBackground)
            .entity();

        let fill_entity = self.in_parent(bar_entity, |tree| {
            tree.add_node()
                .boundary(
                    Rl(Vec2::new(0.0, 0.0)),
                    Rl(Vec2::new(normalized * 100.0, 100.0)),
                )
                .with_rect(corner_radius, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                .with_theme_color::<UiBase>(ThemeColor::Accent)
                .entity()
        });

        self.commands().set(
            bar_entity,
            UiProgressBarData {
                value: initial,
                fill_entity,
            },
        );
        self.commands().set(
            bar_entity,
            UiNodeInteraction {
                accessible_role: Some(AccessibleRole::ProgressBar),
                ..UiNodeInteraction::default()
            },
        );

        bar_entity
    }
}
