use nalgebra_glm;
use nalgebra_glm::{Vec2, Vec4};

use crate::commands::builder::UiTreeBuilder;
use crate::components::layout_types::FlowDirection;
use crate::data::state::{UiBase, UiHover, UiPressed};
use crate::data::units::{Ab, Rl};
use nightshade_paint::data::Anchor;
use nightshade_text::contract::data::text_data::{TextAlignment, VerticalAlignment};

use crate::prelude::*;
impl<'a, 'c> UiTreeBuilder<'a, 'c> {
    pub fn add_collapsing_header(
        &mut self,
        label: &str,
        default_open: bool,
    ) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let font_size = theme.font_size;
        let corner_radius = theme.corner_radius;

        let arrow_text = if default_open { "\u{25BC}" } else { "\u{25B6}" };
        let arrow_slot = self.text_state_mut().cache.add_text(arrow_text);
        let label_slot = self.text_state_mut().cache.add_text(label);

        let root_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 0.0)))
            .flex_grow(0.0)
            .flow(FlowDirection::Vertical, 0.0, 0.0)
            .auto_height()
            .entity();

        self.push_parent(root_entity);

        let header_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, font_size * 2.0)))
            .with_rect(corner_radius, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
            .with_theme_color::<UiBase>(ThemeColor::Background)
            .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
            .with_interaction()
            .with_transition::<UiHover>(8.0, 6.0)
            .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
            .flow(FlowDirection::Horizontal, 8.0, 4.0)
            .with_children(|tree| {
                tree.add_node()
                    .flow_child(Ab(Vec2::new(16.0, 0.0)) + Rl(Vec2::new(0.0, 100.0)))
                    .with_text_slot(arrow_slot, font_size)
                    .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                    .with_theme_color::<UiBase>(ThemeColor::Text)
                    .entity();

                tree.add_node()
                    .flow_child(Rl(Vec2::new(0.0, 100.0)))
                    .flex_grow(1.0)
                    .with_text_slot(label_slot, font_size)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                    .with_theme_color::<UiBase>(ThemeColor::Text)
                    .entity();
            })
            .entity();

        let content_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 0.0)))
            .flex_grow(0.0)
            .flow(FlowDirection::Vertical, 8.0, 4.0)
            .auto_height()
            .with_visible(default_open)
            .entity();

        self.pop_parent();

        self.commands().set(
            header_entity,
            UiCollapsingHeaderData {
                open: default_open,
                changed: false,
                content_entity,
                arrow_text_slot: arrow_slot,
            },
        );
        self.edit::<crate::components::UiNodeInteraction>(header_entity, |interaction| {
            interaction.accessible_role = Some(AccessibleRole::Button);
        });

        header_entity
    }

    pub fn add_scroll_area(&mut self, size: Vec2) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let corner_radius = theme.corner_radius;

        let root_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, size.y)))
            .with_rect(corner_radius, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
            .with_theme_color::<UiBase>(ThemeColor::InputBackground)
            .with_clip()
            .with_interaction()
            .entity();

        let (content_entity, track_entity, thumb_entity) =
            self.in_parent(root_entity, |tree| build_scroll_track(tree, 4.0, 4.0));

        self.commands().set(
            root_entity,
            UiScrollAreaData {
                scroll_offset: 0.0,
                content_entity,
                content_height: 0.0,
                visible_height: size.y,
                thumb_entity,
                track_entity,
                thumb_dragging: false,
                thumb_drag_start_offset: 0.0,
                snap_interval: None,
            },
        );

        root_entity
    }

    pub fn add_scroll_area_fill(&mut self, padding: f32, spacing: f32) -> nightshade::ecs::Entity {
        let root_entity = self
            .add_node()
            .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
            .flow_child(Rl(Vec2::new(100.0, 0.0)))
            .flex_grow(1.0)
            .with_clip()
            .with_interaction()
            .entity();

        let (content_entity, track_entity, thumb_entity) = self.in_parent(root_entity, |tree| {
            build_scroll_track(tree, padding, spacing)
        });

        self.commands().set(
            root_entity,
            UiScrollAreaData {
                scroll_offset: 0.0,
                content_entity,
                content_height: 0.0,
                visible_height: 0.0,
                thumb_entity,
                track_entity,
                thumb_dragging: false,
                thumb_drag_start_offset: 0.0,
                snap_interval: None,
            },
        );

        root_entity
    }

    pub fn add_virtual_list(
        &mut self,
        item_height: f32,
        pool_size: usize,
    ) -> nightshade::ecs::Entity {
        use crate::components::{UiVirtualListData, VirtualListPoolItem};

        let root_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 0.0)))
            .flex_grow(1.0)
            .entity();

        self.push_parent(root_entity);

        let scroll_entity = self.add_scroll_area_fill(0.0, 0.0);
        let body_entity = widget::<UiScrollAreaData>(self.commands(), scroll_entity)
            .map(|data| data.content_entity)
            .unwrap();

        self.push_parent(body_entity);

        let top_spacer = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 0.0)))
            .entity();

        let mut pool_items = Vec::with_capacity(pool_size);
        for _ in 0..pool_size {
            let container = self
                .add_node()
                .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, item_height)))
                .with_interaction()
                .with_visible(false)
                .entity();
            pool_items.push(VirtualListPoolItem {
                container_entity: container,
            });
        }

        let bottom_spacer = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 0.0)))
            .entity();

        self.pop_parent();
        self.pop_parent();

        self.commands().set(
            root_entity,
            UiVirtualListData {
                item_height,
                pool_size,
                total_items: 0,
                visible_start: 0,
                scroll_entity,
                body_entity,
                top_spacer,
                bottom_spacer,
                pool_items,
                selection: None,
                selection_changed: false,
            },
        );

        root_entity
    }

    pub fn add_tab_bar(&mut self, labels: &[&str], initial: usize) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let font_size = theme.font_size;
        let active_bg = theme.accent_color;
        let inactive_bg = theme.background_color;
        let corner_radius = theme.corner_radius;
        let tab_height = theme.button_height;

        let mut tab_entities = Vec::new();
        let mut tab_text_slots = Vec::new();

        let bar_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, tab_height)))
            .flow(FlowDirection::Horizontal, 0.0, 2.0)
            .entity();

        self.push_parent(bar_entity);

        for (index, label) in labels.iter().enumerate() {
            let is_active = index == initial;
            let background = if is_active { active_bg } else { inactive_bg };
            let text_slot = self.text_state_mut().cache.add_text(*label);
            tab_text_slots.push(text_slot);

            let tab_entity = self
                .add_node()
                .flow_child(Ab(Vec2::new(0.0, tab_height)))
                .flex_grow(1.0)
                .with_rect(corner_radius, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
                .color_raw::<UiBase>(background)
                .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
                .with_theme_color::<UiPressed>(ThemeColor::BackgroundActive)
                .with_interaction()
                .with_transition::<UiHover>(10.0, 6.0)
                .with_transition::<UiPressed>(14.0, 8.0)
                .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
                .with_children(|tree| {
                    tree.add_node()
                        .with_text_slot(text_slot, font_size)
                        .with_theme_color::<UiBase>(ThemeColor::Text)
                        .entity();
                })
                .entity();

            self.edit::<crate::components::UiNodeInteraction>(tab_entity, |interaction| {
                interaction.accessible_role = Some(AccessibleRole::Tab);
            });

            tab_entities.push(tab_entity);
        }

        self.pop_parent();

        self.commands().set(
            bar_entity,
            UiTabBarData {
                selected_tab: initial,
                changed: false,
                tab_entities,
                tab_text_slots,
            },
        );

        bar_entity
    }

    pub fn add_breadcrumb(&mut self, segments: &[&str]) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let font_size = theme.font_size;
        let text_color = theme.text_color;
        let accent_color = theme.accent_color;
        let separator_color = Vec4::new(
            text_color.x * 0.5,
            text_color.y * 0.5,
            text_color.z * 0.5,
            text_color.w,
        );

        let mut segment_entities = Vec::new();
        let mut segment_text_slots = Vec::new();

        let bar_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 28.0)))
            .flow(FlowDirection::Horizontal, 0.0, 2.0)
            .entity();

        self.push_parent(bar_entity);

        for (index, segment) in segments.iter().enumerate() {
            if index > 0 {
                let separator_slot = self.text_state_mut().cache.add_text("\u{203A}");
                self.add_node()
                    .with_text_slot(separator_slot, font_size)
                    .color_raw::<UiBase>(separator_color)
                    .flow_child(Ab(Vec2::new(0.0, 28.0)))
                    .auto_size(crate::components::AutoSizeMode::Width)
                    .auto_size_padding(Vec2::new(4.0, 0.0))
                    .entity();
            }

            let text_slot = self.text_state_mut().cache.add_text(*segment);
            segment_text_slots.push(text_slot);

            let is_last = index == segments.len() - 1;
            let color = if is_last { text_color } else { accent_color };

            let segment_entity = self
                .add_node()
                .flow_child(Ab(Vec2::new(0.0, 28.0)))
                .auto_size(crate::components::AutoSizeMode::Width)
                .auto_size_padding(Vec2::new(4.0, 0.0))
                .with_text_slot(text_slot, font_size)
                .color_raw::<UiBase>(color);

            let segment_entity = if is_last {
                segment_entity.entity()
            } else {
                segment_entity
                    .with_interaction()
                    .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
                    .entity()
            };

            segment_entities.push(segment_entity);
        }

        self.pop_parent();

        self.commands().set(
            bar_entity,
            crate::components::UiBreadcrumbData {
                segments: segments.iter().map(|text| text.to_string()).collect(),
                clicked_segment: None,
                changed: false,
                segment_entities,
                segment_text_slots,
            },
        );

        bar_entity
    }

    pub fn add_splitter(
        &mut self,
        direction: crate::components::SplitDirection,
        initial_ratio: f32,
    ) -> nightshade::ecs::Entity {
        use crate::components::SplitDirection;

        let theme = self.active_theme();
        let divider_color = theme.border_color;
        let divider_hover_color = theme.accent_color;
        let divider_thickness = 4.0;
        let ratio = initial_ratio.clamp(0.05, 0.95);

        let container_entity = self
            .add_node()
            .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
            .entity();

        self.push_parent(container_entity);

        let (
            first_pane_min,
            first_pane_max,
            divider_min,
            divider_max,
            second_pane_min,
            second_pane_max,
        ) = match direction {
            SplitDirection::Horizontal => (
                Rl(Vec2::new(0.0, 0.0)),
                Rl(Vec2::new(ratio * 100.0, 100.0)) + Ab(Vec2::new(-divider_thickness * 0.5, 0.0)),
                Rl(Vec2::new(ratio * 100.0, 0.0)) + Ab(Vec2::new(-divider_thickness * 0.5, 0.0)),
                Rl(Vec2::new(ratio * 100.0, 100.0)) + Ab(Vec2::new(divider_thickness * 0.5, 0.0)),
                Rl(Vec2::new(ratio * 100.0, 0.0)) + Ab(Vec2::new(divider_thickness * 0.5, 0.0)),
                Rl(Vec2::new(100.0, 100.0)),
            ),
            SplitDirection::Vertical => (
                Rl(Vec2::new(0.0, 0.0)),
                Rl(Vec2::new(100.0, ratio * 100.0)) + Ab(Vec2::new(0.0, -divider_thickness * 0.5)),
                Rl(Vec2::new(0.0, ratio * 100.0)) + Ab(Vec2::new(0.0, -divider_thickness * 0.5)),
                Rl(Vec2::new(100.0, ratio * 100.0)) + Ab(Vec2::new(0.0, divider_thickness * 0.5)),
                Rl(Vec2::new(0.0, ratio * 100.0)) + Ab(Vec2::new(0.0, divider_thickness * 0.5)),
                Rl(Vec2::new(100.0, 100.0)),
            ),
        };

        let first_pane = self
            .add_node()
            .boundary(first_pane_min, first_pane_max)
            .with_clip()
            .entity();

        let cursor_icon = match direction {
            SplitDirection::Horizontal => nightshade_platform::CursorIcon::ColResize,
            SplitDirection::Vertical => nightshade_platform::CursorIcon::RowResize,
        };

        let divider_entity = self
            .add_node()
            .boundary(divider_min, divider_max)
            .with_rect(0.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
            .color_raw::<UiBase>(divider_color)
            .color_raw::<UiHover>(divider_hover_color)
            .with_interaction()
            .with_transition::<UiHover>(8.0, 6.0)
            .with_cursor_icon(cursor_icon)
            .entity();

        let second_pane = self
            .add_node()
            .boundary(second_pane_min, second_pane_max)
            .with_clip()
            .entity();

        self.pop_parent();

        self.commands().set(
            container_entity,
            crate::components::UiSplitterData {
                direction,
                ratio,
                changed: false,
                first_pane,
                second_pane,
                divider_entity,
                min_ratio: 0.05,
                max_ratio: 0.95,
            },
        );

        container_entity
    }
}

fn build_scroll_track(
    tree: &mut UiTreeBuilder,
    content_padding: f32,
    content_spacing: f32,
) -> (
    nightshade::ecs::Entity,
    nightshade::ecs::Entity,
    nightshade::ecs::Entity,
) {
    let content_entity = tree
        .add_node()
        .boundary(
            Ab(Vec2::new(0.0, 0.0)),
            Ab(Vec2::new(-12.0, 0.0)) + Rl(Vec2::new(100.0, 0.0)),
        )
        .flow(FlowDirection::Vertical, content_padding, content_spacing)
        .entity();

    let track_entity = tree
        .add_node()
        .boundary(
            Ab(Vec2::new(-10.0, 2.0)) + Rl(Vec2::new(100.0, 0.0)),
            Ab(Vec2::new(-2.0, -2.0)) + Rl(Vec2::new(100.0, 100.0)),
        )
        .with_rect(4.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
        .color_raw::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.2))
        .with_visible(false)
        .entity();

    tree.push_parent(track_entity);

    let thumb_entity = tree
        .add_node()
        .window(
            Ab(Vec2::new(0.0, 0.0)),
            Ab(Vec2::new(8.0, 30.0)),
            Anchor::TopLeft,
        )
        .with_rect(4.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
        .with_theme_color::<UiBase>(ThemeColor::Scrollbar)
        .with_theme_color::<UiHover>(ThemeColor::ScrollbarHover)
        .with_interaction()
        .with_transition::<UiHover>(8.0, 6.0)
        .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
        .entity();

    tree.pop_parent();

    (content_entity, track_entity, thumb_entity)
}
