use nalgebra_glm;
use std::collections::HashSet;

use nalgebra_glm::{Vec2, Vec4};

use crate::commands::builder::UiTreeBuilder;
use crate::components::layer::UiLayer;
use crate::components::layout_types::FlowDirection;
use crate::components::*;
use crate::data::state::{UiBase, UiHover, UiPressed};
use crate::data::units::{Ab, Rl};
use nightshade_paint::data::Anchor;
use nightshade_text::contract::data::text_data::{TextAlignment, VerticalAlignment};

use super::date_utils::{format_month_year, populate_calendar_grid};

const MENU_ITEM_INSET: f32 = 10.0;

impl<'a, 'c> UiTreeBuilder<'a, 'c> {
    fn add_popup_shell(
        &mut self,
        options: &[&str],
        header_text_slot: usize,
        arrow_slot: usize,
    ) -> (nightshade::ecs::Entity, Vec<String>, f32) {
        let theme = self.active_theme();
        let font_size = theme.font_size;
        let corner_radius = theme.corner_radius;
        let border_color = theme.border_color;
        let button_height = theme.button_height;

        let option_strings: Vec<String> = options.iter().map(|text| text.to_string()).collect();
        let popup_height = (options.len() as f32 * button_height).min(crate::MAX_POPUP_HEIGHT);

        let wrapper_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, button_height)))
            .entity();

        self.push_parent(wrapper_entity);

        let header_entity = self
            .add_node()
            .boundary(Ab(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
            .with_rect(corner_radius, 1.0, border_color)
            .with_theme_border_color(ThemeColor::Border)
            .with_theme_color::<UiBase>(ThemeColor::InputBackground)
            .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
            .with_theme_color::<UiPressed>(ThemeColor::BackgroundActive)
            .with_interaction()
            .with_transition::<UiHover>(10.0, 6.0)
            .with_transition::<UiPressed>(14.0, 8.0)
            .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
            .flow(FlowDirection::Horizontal, 8.0, 0.0)
            .with_children(|tree| {
                tree.add_node()
                    .flow_child(Rl(Vec2::new(0.0, 100.0)))
                    .flex_grow(1.0)
                    .with_text_slot(header_text_slot, font_size)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                    .with_theme_color::<UiBase>(ThemeColor::Text)
                    .entity();

                tree.add_node()
                    .flow_child(Ab(Vec2::new(20.0, 0.0)) + Rl(Vec2::new(0.0, 100.0)))
                    .with_text_slot(arrow_slot, font_size * 0.8)
                    .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                    .with_theme_color::<UiBase>(ThemeColor::Text)
                    .entity();
            })
            .entity();

        (header_entity, option_strings, popup_height)
    }

    pub fn add_dropdown(&mut self, options: &[&str], initial: usize) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let font_size = theme.font_size;
        let corner_radius = theme.corner_radius;
        let border_color = theme.border_color;
        let button_height = theme.button_height;
        let accent_color = theme.accent_color;
        let bg_color = theme.input_background_color;

        let selected_text = options.get(initial).copied().unwrap_or("");
        let header_text_slot = self.text_state_mut().cache.add_text(selected_text);
        let arrow_slot = self.text_state_mut().cache.add_text("\u{25BC}");

        let mut popup_entities = Vec::new();
        let (header_entity, option_strings, popup_height) =
            self.add_popup_shell(options, header_text_slot, arrow_slot);

        let popup_container_entity = self
            .add_node()
            .window(
                Rl(Vec2::new(0.0, 100.0)),
                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, popup_height)),
                Anchor::TopLeft,
            )
            .with_rect(corner_radius, 1.0, border_color)
            .with_theme_border_color(ThemeColor::Border)
            .with_theme_color::<UiBase>(ThemeColor::InputBackground)
            .with_layer(UiLayer::Popups)
            .with_depth(UiDepthMode::Set(30.0))
            .with_visible(false)
            .with_clip()
            .flow(FlowDirection::Vertical, 0.0, 0.0)
            .entity();

        self.push_parent(popup_container_entity);

        for (index, option) in options.iter().enumerate() {
            let option_slot = self.text_state_mut().cache.add_text(*option);
            let is_selected = index == initial;

            let option_entity = self
                .add_node()
                .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, button_height)))
                .with_rect(corner_radius, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                .color_raw::<UiBase>(if is_selected { accent_color } else { bg_color })
                .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
                .with_theme_color::<UiPressed>(ThemeColor::BackgroundActive)
                .with_interaction()
                .with_transition::<UiHover>(10.0, 6.0)
                .with_transition::<UiPressed>(14.0, 8.0)
                .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
                .with_children(|tree| {
                    tree.add_node()
                        .boundary(
                            Ab(Vec2::new(8.0, 0.0)),
                            Ab(Vec2::new(-8.0, 0.0)) + Rl(Vec2::new(100.0, 100.0)),
                        )
                        .with_text_slot(option_slot, font_size)
                        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                        .with_theme_color::<UiBase>(ThemeColor::Text)
                        .entity();
                })
                .entity();

            popup_entities.push(option_entity);
        }

        self.pop_parent();
        self.pop_parent();

        self.commands().set(
            header_entity,
            UiDropdownData {
                options: option_strings,
                selected_index: initial,
                changed: false,
                open: false,
                header_text_slot,
                popup_entities,
                popup_container_entity,
                hovered_index: None,
                is_theme_dropdown: false,
                searchable: false,
                filter_text: String::new(),
                filter_input_entity: None,
                filtered_indices: Vec::new(),
            },
        );
        self.edit::<crate::components::UiNodeInteraction>(header_entity, |interaction| {
            interaction.accessible_role = Some(AccessibleRole::Dropdown);
        });
        self.assign_tab_index(header_entity);

        header_entity
    }

    pub fn add_dropdown_searchable(
        &mut self,
        options: &[&str],
        initial: usize,
    ) -> nightshade::ecs::Entity {
        let entity = self.add_dropdown(options, initial);
        let theme = self.active_theme();
        let font_size = theme.font_size;
        let button_height = theme.button_height;
        let corner_radius = theme.corner_radius;
        let border_color = theme.border_color;
        let accent_color = theme.accent_color;

        let popup_container = {
            if let Some(data) = self.component::<crate::components::UiDropdownData>(entity) {
                data.popup_container_entity
            } else {
                return entity;
            }
        };

        let filter_text_slot = self.text_state_mut().cache.add_text("");
        let placeholder_slot = self.text_state_mut().cache.add_text("Search...");

        self.push_parent(popup_container);

        let filter_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, button_height)))
            .with_rect(corner_radius, 1.0, border_color)
            .with_theme_border_color(ThemeColor::Border)
            .with_theme_color::<UiBase>(ThemeColor::InputBackground)
            .with_interaction()
            .with_cursor_icon(nightshade_platform::CursorIcon::Text)
            .with_clip()
            .entity();

        let (selection_entity, cursor_entity, placeholder_entity) =
            self.in_parent(filter_entity, |tree| {
                let selection_entity = tree
                    .add_node()
                    .window(
                        Ab(Vec2::new(8.0, 4.0)),
                        Ab(Vec2::new(0.0, button_height - 8.0)),
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
                    .boundary(
                        Ab(Vec2::new(8.0, 0.0)),
                        Ab(Vec2::new(0.0, button_height)) + Rl(Vec2::new(100.0, 0.0)),
                    )
                    .with_text_slot(filter_text_slot, font_size)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                    .with_theme_color::<UiBase>(ThemeColor::Text)
                    .entity();

                let placeholder_entity = tree
                    .add_node()
                    .boundary(
                        Ab(Vec2::new(8.0, 0.0)),
                        Ab(Vec2::new(0.0, button_height)) + Rl(Vec2::new(100.0, 0.0)),
                    )
                    .with_text_slot(placeholder_slot, font_size)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                    .with_theme_color::<UiBase>(ThemeColor::TextDisabled)
                    .entity();

                let cursor_entity = tree
                    .add_node()
                    .window(
                        Ab(Vec2::new(8.0, 4.0)),
                        Ab(Vec2::new(2.0, button_height - 8.0)),
                        Anchor::TopLeft,
                    )
                    .with_rect(1.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                    .with_theme_color::<UiBase>(ThemeColor::Text)
                    .with_visible(false)
                    .entity();

                (selection_entity, cursor_entity, placeholder_entity)
            });

        self.pop_parent();

        self.commands().queue(move |_, resources| {
            if let Some(children) = resources
                .res_mut::<nightshade::ecs::HierarchyIndex>()
                .children
                .get_mut(&popup_container)
                && let Some(position) = children.iter().position(|entity| *entity == filter_entity)
            {
                children.remove(position);
                children.insert(0, filter_entity);
            }
        });

        let all_indices: Vec<usize> = (0..options.len()).collect();
        self.edit::<crate::components::UiDropdownData>(entity, move |data| {
            data.searchable = true;
            data.filter_input_entity = Some(filter_entity);
            data.filtered_indices = all_indices;
        });

        self.commands().set(
            filter_entity,
            UiTextInputData {
                text: String::new(),
                cursor_position: 0,
                selection_start: None,
                changed: false,
                text_slot: filter_text_slot,
                cursor_entity,
                selection_entity,
                scroll_offset: 0.0,
                cursor_blink_timer: 0.0,
                placeholder_entity: Some(placeholder_entity),
                undo_stack: std::sync::Arc::new(UndoStack::new(50)),
                input_mask: crate::components::InputMask::None,
                max_length: None,
                password: false,
            },
        );

        entity
    }

    pub fn add_multi_select(&mut self, options: &[&str]) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let font_size = theme.font_size;
        let corner_radius = theme.corner_radius;
        let border_color = theme.border_color;
        let button_height = theme.button_height;
        let bg_color = theme.input_background_color;

        let header_text_slot = self.text_state_mut().cache.add_text("0 selected");
        let arrow_slot = self.text_state_mut().cache.add_text("\u{25BC}");

        let mut popup_entities = Vec::new();
        let mut check_entities = Vec::new();
        let (header_entity, option_strings, popup_height) =
            self.add_popup_shell(options, header_text_slot, arrow_slot);

        let popup_container_entity = self
            .add_node()
            .window(
                Rl(Vec2::new(0.0, 100.0)),
                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, popup_height)),
                Anchor::TopLeft,
            )
            .with_rect(corner_radius, 1.0, border_color)
            .with_theme_border_color(ThemeColor::Border)
            .with_theme_color::<UiBase>(ThemeColor::InputBackground)
            .with_layer(UiLayer::Popups)
            .with_depth(UiDepthMode::Set(30.0))
            .with_visible(false)
            .flow(FlowDirection::Vertical, 0.0, 0.0)
            .entity();

        self.push_parent(popup_container_entity);

        for option in options.iter() {
            let option_slot = self.text_state_mut().cache.add_text(*option);
            let check_slot = self.text_state_mut().cache.add_text(" ");

            let option_entity = self
                .add_node()
                .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, button_height)))
                .with_rect(corner_radius, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                .color_raw::<UiBase>(bg_color)
                .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
                .with_interaction()
                .with_transition::<UiHover>(8.0, 6.0)
                .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
                .flow(FlowDirection::Horizontal, 4.0, 0.0)
                .entity();

            let check_entity = self.in_parent(option_entity, |tree| {
                let check_entity = tree
                    .add_node()
                    .flow_child(Ab(Vec2::new(20.0, 0.0)) + Rl(Vec2::new(0.0, 100.0)))
                    .with_text_slot(check_slot, font_size)
                    .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                    .with_theme_color::<UiBase>(ThemeColor::Text)
                    .entity();

                tree.add_node()
                    .flow_child(Rl(Vec2::new(0.0, 100.0)))
                    .flex_grow(1.0)
                    .with_text_slot(option_slot, font_size)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                    .with_theme_color::<UiBase>(ThemeColor::Text)
                    .entity();

                check_entity
            });

            popup_entities.push(option_entity);
            check_entities.push(check_entity);
        }

        self.pop_parent();
        self.pop_parent();

        self.commands().set(
            header_entity,
            UiMultiSelectData {
                options: option_strings,
                selected_indices: HashSet::new(),
                changed: false,
                open: false,
                header_text_slot,
                popup_entities,
                popup_container_entity,
                hovered_index: None,
                check_entities,
            },
        );
        self.assign_tab_index(header_entity);

        header_entity
    }

    pub fn add_date_picker(&mut self, year: i32, month: u32, day: u32) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let font_size = theme.font_size;
        let corner_radius = theme.corner_radius;
        let border_color = theme.border_color;
        let button_height = theme.button_height;
        let accent_color = theme.accent_color;

        let header_text = format!("{year:04}-{month:02}-{day:02}");
        let header_text_slot = self.text_state_mut().cache.add_text(&header_text);
        let arrow_slot = self.text_state_mut().cache.add_text("\u{1F4C5}");
        let month_label = format_month_year(year, month);
        let month_label_slot = self.text_state_mut().cache.add_text(&month_label);

        let cell_size = 30.0;
        let grid_width = cell_size * 7.0;
        let nav_height = button_height;
        let grid_height = cell_size * 6.0;
        let popup_height = nav_height + grid_height;

        let wrapper_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, button_height)))
            .entity();
        self.push_parent(wrapper_entity);

        let header_entity = self
            .add_node()
            .boundary(Ab(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
            .with_rect(corner_radius, 1.0, border_color)
            .with_theme_border_color(ThemeColor::Border)
            .with_theme_color::<UiBase>(ThemeColor::InputBackground)
            .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
            .with_theme_color::<UiPressed>(ThemeColor::BackgroundActive)
            .with_interaction()
            .with_transition::<UiHover>(10.0, 6.0)
            .with_transition::<UiPressed>(14.0, 8.0)
            .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
            .flow(FlowDirection::Horizontal, 8.0, 0.0)
            .with_children(|tree| {
                tree.add_node()
                    .flow_child(Rl(Vec2::new(0.0, 100.0)))
                    .flex_grow(1.0)
                    .with_text_slot(header_text_slot, font_size)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                    .with_theme_color::<UiBase>(ThemeColor::Text)
                    .entity();

                tree.add_node()
                    .flow_child(Ab(Vec2::new(24.0, 0.0)) + Rl(Vec2::new(0.0, 100.0)))
                    .with_text_slot(arrow_slot, font_size * 0.8)
                    .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                    .with_theme_color::<UiBase>(ThemeColor::Text)
                    .entity();
            })
            .entity();

        let popup_entity = self
            .add_node()
            .window(
                Rl(Vec2::new(0.0, 100.0)),
                Ab(Vec2::new(grid_width + 8.0, popup_height + 8.0)),
                Anchor::TopLeft,
            )
            .with_rect(corner_radius, 1.0, border_color)
            .with_theme_border_color(ThemeColor::Border)
            .with_theme_color::<UiBase>(ThemeColor::InputBackground)
            .with_layer(UiLayer::Popups)
            .with_depth(UiDepthMode::Set(30.0))
            .with_visible(false)
            .flow(FlowDirection::Vertical, 4.0, 4.0)
            .entity();

        self.push_parent(popup_entity);

        let nav_row = self
            .add_node()
            .flow_child(Ab(Vec2::new(grid_width, nav_height)))
            .flow(FlowDirection::Horizontal, 0.0, 0.0)
            .entity();
        self.push_parent(nav_row);

        let prev_slot = self.text_state_mut().cache.add_text("\u{25C0}");
        let prev_month_entity = self
            .add_node()
            .flow_child(Ab(Vec2::new(cell_size, nav_height)))
            .with_rect(corner_radius, 0.0, Vec4::zeros())
            .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
            .with_interaction()
            .with_transition::<UiHover>(8.0, 6.0)
            .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
            .with_children(|tree| {
                tree.add_node()
                    .boundary(Ab(Vec2::zeros()), Rl(Vec2::new(100.0, 100.0)))
                    .with_text_slot(prev_slot, font_size)
                    .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                    .with_theme_color::<UiBase>(ThemeColor::Text)
                    .entity();
            })
            .entity();

        self.add_node()
            .flow_child(Rl(Vec2::new(0.0, 100.0)))
            .flex_grow(1.0)
            .with_text_slot(month_label_slot, font_size)
            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
            .with_theme_color::<UiBase>(ThemeColor::Text)
            .entity();

        let next_slot = self.text_state_mut().cache.add_text("\u{25B6}");
        let next_month_entity = self
            .add_node()
            .flow_child(Ab(Vec2::new(cell_size, nav_height)))
            .with_rect(corner_radius, 0.0, Vec4::zeros())
            .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
            .with_interaction()
            .with_transition::<UiHover>(8.0, 6.0)
            .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
            .with_children(|tree| {
                tree.add_node()
                    .boundary(Ab(Vec2::zeros()), Rl(Vec2::new(100.0, 100.0)))
                    .with_text_slot(next_slot, font_size)
                    .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                    .with_theme_color::<UiBase>(ThemeColor::Text)
                    .entity();
            })
            .entity();

        self.pop_parent();

        let mut day_entities = Vec::with_capacity(42);
        let mut day_text_slots = Vec::with_capacity(42);

        let grid_entity = self
            .add_node()
            .flow_child(Ab(Vec2::new(grid_width, grid_height)))
            .flow(FlowDirection::Horizontal, 0.0, 0.0)
            .flow_wrap()
            .entity();
        self.push_parent(grid_entity);

        for _ in 0..42 {
            let slot = self.text_state_mut().cache.add_text("");
            day_text_slots.push(slot);

            let cell = self
                .add_node()
                .flow_child(Ab(Vec2::new(cell_size, cell_size)))
                .with_rect(corner_radius, 0.0, Vec4::zeros())
                .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
                .with_interaction()
                .with_transition::<UiHover>(8.0, 6.0)
                .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
                .with_children(|tree| {
                    tree.add_node()
                        .boundary(Ab(Vec2::zeros()), Rl(Vec2::new(100.0, 100.0)))
                        .with_text_slot(slot, font_size * 0.85)
                        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                        .with_theme_color::<UiBase>(ThemeColor::Text)
                        .entity();
                })
                .entity();
            day_entities.push(cell);
        }

        self.pop_parent();
        self.pop_parent();
        self.pop_parent();

        let (text_state, commands) = self.text_state_and_commands();
        populate_calendar_grid(
            text_state,
            commands,
            year,
            month,
            day,
            &day_text_slots,
            &day_entities,
            accent_color,
        );

        self.commands().set(
            header_entity,
            UiDatePickerData {
                year,
                month,
                day,
                changed: false,
                open: false,
                header_text_slot,
                popup_entity,
                day_entities,
                day_text_slots,
                month_label_slot,
                prev_month_entity,
                next_month_entity,
                selected_day_entity: None,
            },
        );
        self.assign_tab_index(header_entity);

        header_entity
    }

    pub fn add_menu(&mut self, label: &str, items: &[&str]) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let font_size = theme.font_size;
        let corner_radius = theme.corner_radius;
        let button_height = theme.button_height;
        let border_color = theme.border_color;

        let label_slot = self.text_state_mut().cache.add_text(label);

        let item_strings: Vec<String> = items.iter().map(|text| text.to_string()).collect();
        let popup_height = (items.len() as f32 * button_height).min(crate::MAX_POPUP_HEIGHT);

        let popup_width = {
            let font_engine = &mut self.text_state_mut().font_engine;
            items
                .iter()
                .map(|item| crate::data::units::measure_text_width(font_engine, item, font_size))
                .fold(0.0f32, f32::max)
                + MENU_ITEM_INSET * 2.0
        };

        let menu_button = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, button_height)))
            .with_rect(corner_radius, 1.0, border_color)
            .with_theme_border_color(ThemeColor::Border)
            .with_theme_color::<UiBase>(ThemeColor::Background)
            .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
            .with_interaction()
            .with_transition::<UiHover>(8.0, 6.0)
            .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
            .entity();

        let (popup_container_entity, popup_entities) = self.in_parent(menu_button, |tree| {
            let mut popup_entities = Vec::new();

            tree.add_node()
                .with_text_slot(label_slot, font_size)
                .with_theme_color::<UiBase>(ThemeColor::Text)
                .entity();

            let popup_container_entity = tree
                .add_node()
                .window(
                    Rl(Vec2::new(0.0, 100.0)),
                    Ab(Vec2::new(popup_width, popup_height)),
                    Anchor::TopLeft,
                )
                .with_rect(corner_radius, 1.0, border_color)
                .with_theme_border_color(ThemeColor::Border)
                .with_theme_color::<UiBase>(ThemeColor::Background)
                .with_layer(UiLayer::Popups)
                .with_depth(UiDepthMode::Set(30.0))
                .with_visible(false)
                .flow(FlowDirection::Vertical, 0.0, 0.0)
                .entity();

            tree.push_parent(popup_container_entity);

            for item in items {
                let item_slot = tree.text_state_mut().cache.add_text(*item);

                let item_entity = tree
                    .add_node()
                    .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, button_height)))
                    .with_rect(corner_radius, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                    .with_theme_color::<UiBase>(ThemeColor::Background)
                    .with_theme_color::<UiHover>(ThemeColor::Accent)
                    .with_interaction()
                    .with_transition::<UiHover>(8.0, 6.0)
                    .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
                    .flow(FlowDirection::Horizontal, MENU_ITEM_INSET, 0.0)
                    .with_children(|tree| {
                        tree.add_node()
                            .flow_child(Rl(Vec2::new(100.0, 100.0)))
                            .flex_grow(1.0)
                            .with_text_slot(item_slot, font_size)
                            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                            .with_theme_color::<UiBase>(ThemeColor::Text)
                            .entity();
                    })
                    .entity();

                popup_entities.push(item_entity);
            }

            tree.pop_parent();

            (popup_container_entity, popup_entities)
        });

        self.commands().set(
            menu_button,
            UiMenuData {
                items: item_strings,
                clicked_item: None,
                open: false,
                label_text_slot: label_slot,
                popup_entities,
                popup_container_entity,
            },
        );

        menu_button
    }

    pub fn add_theme_dropdown(&mut self) -> nightshade::ecs::Entity {
        let preset_names: Vec<String> = self
            .theme_state_mut()
            .presets
            .iter()
            .map(|theme| theme.name.clone())
            .collect();

        let selected = self.theme_state_mut().selected_preset_index;

        let labels: Vec<&str> = preset_names.iter().map(|name| name.as_str()).collect();
        let entity = self.add_dropdown_searchable(&labels, selected.unwrap_or(0));
        self.edit::<crate::components::UiDropdownData>(entity, |data| {
            data.is_theme_dropdown = true;
        });
        entity
    }
}
