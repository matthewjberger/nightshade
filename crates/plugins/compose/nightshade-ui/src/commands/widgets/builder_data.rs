use nalgebra_glm;
use nalgebra_glm::{Vec2, Vec4};

use crate::commands::builder::UiTreeBuilder;
use crate::components::layout_types::{FlowAlignment, FlowDirection};
use crate::data::state::{UiBase, UiHover};
use crate::data::units::{Ab, Rl};
use nightshade_paint::data::{Anchor, Rect};
use nightshade_text::contract::data::text_data::{TextAlignment, VerticalAlignment};

use crate::prelude::*;
impl<'a, 'c> UiTreeBuilder<'a, 'c> {
    pub fn add_property_grid(&mut self, label_width: f32) -> nightshade::ecs::Entity {
        let grid_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 0.0)))
            .flow(FlowDirection::Vertical, 4.0, 2.0)
            .auto_height()
            .with_interaction()
            .entity();

        self.commands().set(
            grid_entity,
            UiPropertyGridData {
                label_width,
                row_entities: Vec::new(),
                label_entities: Vec::new(),
                resize_active: false,
                resize_start_x: 0.0,
                resize_start_width: 0.0,
            },
        );
        self.edit::<crate::components::UiNodeInteraction>(grid_entity, |interaction| {
            interaction.accessible_role = Some(AccessibleRole::Grid);
        });

        grid_entity
    }

    pub fn add_property_row(
        &mut self,
        grid: nightshade::ecs::Entity,
        parent: nightshade::ecs::Entity,
        label: &str,
    ) -> nightshade::ecs::Entity {
        let label_width =
            if let Some(data) = self.component::<crate::components::UiPropertyGridData>(grid) {
                data.label_width
            } else {
                100.0
            };

        let theme = self.active_theme();
        let font_size = theme.font_size;
        let row_height = theme.button_height;

        let label_slot = self.text_state_mut().cache.add_text(label);

        let row_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, row_height)))
            .auto_height()
            .flow_with_alignment(
                FlowDirection::Horizontal,
                4.0,
                8.0,
                FlowAlignment::Start,
                FlowAlignment::Center,
            )
            .entity();

        self.edit::<nightshade::ecs::ChildOf>(row_entity, move |p| {
            *p = nightshade::ecs::ChildOf(parent);
        });

        self.push_parent(row_entity);

        let label_entity = self
            .add_node()
            .flow_child(Ab(Vec2::new(label_width, row_height)))
            .with_text_slot(label_slot, font_size * 0.9)
            .with_text_alignment(TextAlignment::Right, VerticalAlignment::Middle)
            .with_text_overflow(crate::components::TextOverflow::Ellipsis)
            .with_theme_color::<UiBase>(ThemeColor::TextDisabled)
            .entity();

        let widget_area = self
            .add_node()
            .flow_child(Ab(Vec2::new(0.0, row_height)))
            .auto_height()
            .flex_grow(1.0)
            .flow_with_alignment(
                FlowDirection::Horizontal,
                0.0,
                4.0,
                FlowAlignment::Start,
                FlowAlignment::Center,
            )
            .entity();

        self.pop_parent();

        self.edit::<crate::components::UiPropertyGridData>(grid, move |data| {
            data.row_entities.push(row_entity);
            data.label_entities.push(label_entity);
        });

        widget_area
    }

    pub fn add_property_text_input(
        &mut self,
        grid: nightshade::ecs::Entity,
        parent: nightshade::ecs::Entity,
        label: &str,
        placeholder: &str,
    ) -> nightshade::ecs::Entity {
        let widget_area = self.add_property_row(grid, parent, label);
        self.push_parent(widget_area);
        let widget = self.add_text_input(placeholder);
        self.pop_parent();
        widget
    }

    pub fn add_property_dropdown(
        &mut self,
        grid: nightshade::ecs::Entity,
        parent: nightshade::ecs::Entity,
        label: &str,
        options: &[&str],
        initial: usize,
    ) -> nightshade::ecs::Entity {
        let widget_area = self.add_property_row(grid, parent, label);
        self.push_parent(widget_area);
        let widget = self.add_dropdown(options, initial);
        self.pop_parent();
        widget
    }

    pub fn add_property_checkbox(
        &mut self,
        grid: nightshade::ecs::Entity,
        parent: nightshade::ecs::Entity,
        label: &str,
        initial: bool,
    ) -> nightshade::ecs::Entity {
        let widget_area = self.add_property_row(grid, parent, label);
        self.push_parent(widget_area);
        let widget = self.add_checkbox(label, initial);
        self.pop_parent();
        widget
    }

    pub fn add_property_drag_value(
        &mut self,
        grid: nightshade::ecs::Entity,
        parent: nightshade::ecs::Entity,
        label: &str,
        min: f32,
        max: f32,
        initial: f32,
    ) -> nightshade::ecs::Entity {
        let widget_area = self.add_property_row(grid, parent, label);
        self.push_parent(widget_area);
        let widget = self.add_drag_value(min, max, initial);
        self.pop_parent();
        widget
    }

    pub fn add_property_section(
        &mut self,
        parent: nightshade::ecs::Entity,
        label: &str,
    ) -> nightshade::ecs::Entity {
        let header = self.add_collapsing_header(label, true);

        let reparent_entity = self
            .component::<nightshade::ecs::ChildOf>(header)
            .map(|child_of| child_of.0)
            .unwrap_or(header);

        self.edit::<nightshade::ecs::ChildOf>(reparent_entity, move |p| {
            *p = nightshade::ecs::ChildOf(parent);
        });

        widget::<UiCollapsingHeaderData>(self.commands(), header)
            .map(|data| data.content_entity)
            .unwrap_or(header)
    }

    pub fn add_rich_text(&mut self, spans: &[TextSpan]) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let font_size = theme.font_size;
        let text_color = theme.text_color;

        let container = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)))
            .auto_size(crate::components::AutoSizeMode::Height)
            .flow(FlowDirection::Horizontal, 0.0, 0.0)
            .flow_wrap()
            .entity();

        self.push_parent(container);

        let mut span_entities = Vec::with_capacity(spans.len());
        let mut span_text_slots = Vec::with_capacity(spans.len());

        for span in spans {
            let (span_wrapper, text_slot) =
                self.build_text_span(span, font_size, text_color, None, true);
            span_entities.push(span_wrapper);
            span_text_slots.push(text_slot);
        }

        self.pop_parent();

        self.commands().set(
            container,
            UiRichTextData {
                span_entities,
                span_text_slots,
            },
        );

        container
    }

    pub(super) fn build_text_span(
        &mut self,
        span: &TextSpan,
        font_size: f32,
        text_color: Vec4,
        fixed_height: Option<f32>,
        allow_underline: bool,
    ) -> (nightshade::ecs::Entity, usize) {
        let text_slot = self.text_state_mut().cache.add_text(&span.text);
        let mut span_font_size = span.font_size_override.unwrap_or(font_size);
        let span_color = span.color.unwrap_or(text_color);
        let span_font_index = span.font_index.unwrap_or(0);

        if span.bold {
            span_font_size *= 1.05;
        }

        let outline_width = if span.bold { 0.5 } else { 0.0 };
        let outline_color = if span.bold {
            span_color
        } else {
            Vec4::new(0.0, 0.0, 0.0, 0.0)
        };

        let wrapper_height = fixed_height.unwrap_or(span_font_size * 1.5);

        let span_wrapper = self
            .add_node()
            .flow_child(Ab(Vec2::new(0.0, wrapper_height)))
            .auto_size(crate::components::AutoSizeMode::Width)
            .flow(FlowDirection::Vertical, 0.0, 0.0)
            .entity();

        self.push_parent(span_wrapper);

        let entity = self.add_node().entity();
        self.edit::<crate::components::UiNodeContent>(entity, move |content| {
            *content = crate::components::UiNodeContent::Text {
                text_slot,
                font_index: span_font_index,
                font_size_override: Some(span_font_size),
                outline_color,
                outline_width,
                alignment: TextAlignment::Left,
                vertical_alignment: VerticalAlignment::Middle,
                overflow: crate::components::TextOverflow::default(),
                monospace_width: None,
                font_kind: nightshade_text::contract::data::text_data::FontKind::Default,
            };
        });
        self.edit::<crate::components::UiLayoutNode>(entity, move |node| {
            node.flow_child_size = Some(Ab(Vec2::new(0.0, wrapper_height)).into());
            node.auto_size = crate::components::AutoSizeMode::Width;
            node.pointer_events = false;
        });
        self.edit::<crate::components::UiNodeColor>(entity, move |color| {
            color.colors[UiBase::INDEX] = Some(span_color);
        });
        if span.color.is_none() {
            let mut binding = crate::components::UiThemeBinding::default();
            binding.color_roles[UiBase::INDEX] = Some(ThemeColor::Text);
            self.commands().set(entity, binding);
        }

        if allow_underline && span.underline {
            if span.color.is_none() {
                self.add_node()
                    .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 1.0)))
                    .with_rect(0.0, 0.0, Vec4::zeros())
                    .with_theme_color::<UiBase>(ThemeColor::Text)
                    .entity();
            } else {
                self.add_node()
                    .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 1.0)))
                    .with_rect(0.0, 0.0, Vec4::zeros())
                    .color_raw::<UiBase>(span_color)
                    .entity();
            }
        }

        self.pop_parent();

        (span_wrapper, text_slot)
    }

    pub fn add_data_grid(
        &mut self,
        columns: &[DataGridColumn],
        pool_size: usize,
    ) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let font_size = theme.font_size;
        let row_height = font_size * 1.8;

        let total_width: f32 = columns.iter().map(|column| column.width).sum();

        let root = self
            .add_node()
            .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
            .flow_child(Rl(Vec2::new(100.0, 100.0)))
            .with_rect(0.0, 0.0, Vec4::zeros())
            .with_theme_color::<UiBase>(ThemeColor::Panel)
            .entity();

        self.push_parent(root);

        let header_row = self
            .add_node()
            .boundary(
                Rl(Vec2::new(0.0, 0.0)),
                Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, row_height)),
            )
            .flow(FlowDirection::Horizontal, 0.0, 0.0)
            .with_rect(0.0, 0.0, Vec4::zeros())
            .with_theme_color::<UiBase>(ThemeColor::PanelHeader)
            .entity();

        self.push_parent(header_row);

        let mut header_entities = Vec::with_capacity(columns.len());
        let mut header_text_slots = Vec::with_capacity(columns.len());
        for column in columns {
            let text_slot = self.text_state_mut().cache.add_text(&column.label);
            header_text_slots.push(text_slot);

            let header_cell = if column.sortable {
                let entity = self
                    .add_node()
                    .flow_child(Ab(Vec2::new(column.width, row_height)))
                    .with_text_slot(text_slot, font_size)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                    .with_theme_color::<UiBase>(ThemeColor::Text)
                    .with_theme_color::<UiHover>(ThemeColor::TextAccent)
                    .with_interaction()
                    .entity();
                self.edit::<crate::components::UiNodeInteraction>(entity, |interaction| {
                    interaction.cursor_icon = Some(nightshade_platform::CursorIcon::Pointer);
                });
                entity
            } else {
                self.add_node()
                    .flow_child(Ab(Vec2::new(column.width, row_height)))
                    .with_text_slot(text_slot, font_size)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                    .with_theme_color::<UiBase>(ThemeColor::Text)
                    .entity()
            };

            header_entities.push(header_cell);
        }

        self.pop_parent();

        let scroll_wrapper = self
            .add_node()
            .boundary(Ab(Vec2::new(0.0, row_height)), Rl(Vec2::new(100.0, 100.0)))
            .entity();
        self.push_parent(scroll_wrapper);

        let scroll = self.add_scroll_area_fill(0.0, 0.0);
        let scroll_entity = scroll;
        let body_entity = widget::<UiScrollAreaData>(self.commands(), scroll)
            .map(|data| data.content_entity)
            .unwrap_or(scroll);

        self.pop_parent();

        self.push_parent(body_entity);

        let top_spacer = self
            .add_node()
            .flow_child(Ab(Vec2::new(total_width, 0.0)))
            .entity();

        let mut pool_rows = Vec::with_capacity(pool_size);
        let accent_color = self.active_theme().accent_color;
        let selected_bg = Vec4::new(accent_color.x, accent_color.y, accent_color.z, 0.3);
        let has_editable = columns.iter().any(|column| column.editable);

        for _ in 0..pool_size {
            let mut row_builder = self
                .add_node()
                .flow_child(Ab(Vec2::new(total_width, row_height)))
                .flow(FlowDirection::Horizontal, 0.0, 0.0)
                .with_rect(0.0, 0.0, Vec4::zeros())
                .color_raw::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.0))
                .color_raw::<crate::data::state::UiSelected>(selected_bg)
                .with_interaction();
            if has_editable {
                row_builder = row_builder.with_cursor_icon(nightshade_platform::CursorIcon::Cell);
            }
            let row_entity = row_builder.entity();

            self.push_parent(row_entity);

            let mut cell_text_slots = Vec::with_capacity(columns.len());
            let mut cell_entities = Vec::with_capacity(columns.len());
            for column in columns {
                let text_slot = self.text_state_mut().cache.add_text("");

                let cell_entity = self
                    .add_node()
                    .flow_child(Ab(Vec2::new(column.width, row_height)))
                    .with_text_slot(text_slot, font_size)
                    .with_text_alignment(column.alignment, VerticalAlignment::Middle)
                    .with_theme_color::<UiBase>(ThemeColor::Text)
                    .entity();

                cell_text_slots.push(text_slot);
                cell_entities.push(cell_entity);
            }

            self.pop_parent();

            self.edit::<crate::components::UiLayoutNode>(row_entity, |node| {
                node.visible = false;
            });

            pool_rows.push(DataGridPoolRow {
                row_entity,
                cell_text_slots,
                cell_entities,
            });
        }

        let bottom_spacer = self
            .add_node()
            .flow_child(Ab(Vec2::new(total_width, 0.0)))
            .entity();

        self.pop_parent();
        self.pop_parent();

        self.push_parent(root);
        let editing_input_entity = self.add_text_input("");
        self.edit::<crate::components::UiLayoutNode>(editing_input_entity, move |node| {
            node.visible = false;
            node.flow_child_size = None;
            node.base_layout = Some(crate::components::layout_types::UiLayoutType::Window(
                crate::components::layout_types::WindowLayout {
                    position: Ab(Vec2::new(0.0, 0.0)).into(),
                    size: Ab(Vec2::new(100.0, row_height)).into(),
                    anchor: nightshade_paint::data::Anchor::TopLeft,
                },
            ));
        });
        self.pop_parent();

        self.commands().set(
            root,
            UiDataGridData {
                columns: columns.to_vec(),
                row_height,
                pool_size,
                total_rows: 0,
                scroll_entity,
                body_entity,
                header_entities,
                header_text_slots,
                top_spacer,
                bottom_spacer,
                pool_rows,
                visible_start: 0,
                sort_column: None,
                sort_ascending: true,
                sort_changed: false,
                selected_rows: std::collections::HashSet::new(),
                selection_anchor: None,
                selection_changed: false,
                focused: false,
                resize_column: None,
                resize_start_x: 0.0,
                resize_start_width: 0.0,
                filtered_indices: None,
                header_divider_entities: Vec::new(),
                filter_row_entity: None,
                filter_input_entities: Vec::new(),
                filter_texts: Vec::new(),
                editing_cell: None,
                editing_input_entity: Some(editing_input_entity),
            },
        );
        self.edit::<crate::components::UiNodeInteraction>(root, |interaction| {
            interaction.accessible_role = Some(AccessibleRole::Grid);
        });

        root
    }

    pub fn add_tile_container(&mut self, size: Vec2) -> nightshade::ecs::Entity {
        let container = self
            .add_node()
            .window(Ab(Vec2::new(0.0, 0.0)), Ab(size), Anchor::TopLeft)
            .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
            .with_theme_color::<UiBase>(ThemeColor::Panel)
            .with_clip()
            .entity();

        let root_tabs = TileNode::Tabs {
            panes: Vec::new(),
            active: 0,
        };

        let tiles: Vec<Option<TileNode>> = vec![Some(root_tabs)];

        let data = UiTileContainerData {
            tiles,
            root: TileId(0),
            rects: vec![Rect::default()],
            dragging_splitter: None,
            pending_tab_drag: None,
            dragging_tab: None,
            drop_preview: None,
            container_entity: container,
            splitter_width: 4.0,
            tab_bar_height: 26.0,
            next_free: Vec::new(),
            hovered_close: None,
        };

        self.commands().set(container, data);

        container
    }
}
