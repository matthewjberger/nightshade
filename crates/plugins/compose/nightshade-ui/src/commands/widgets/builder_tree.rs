use nalgebra_glm;
use nalgebra_glm::{Vec2, Vec4};

use crate::commands::builder::UiTreeBuilder;
use crate::components::layout_types::FlowDirection;
use crate::data::state::{UiBase, UiHover};
use crate::data::units::{Ab, Rl};
use nightshade_text::contract::data::text_data::{TextAlignment, VerticalAlignment};

use crate::prelude::*;
impl<'a, 'c> UiTreeBuilder<'a, 'c> {
    pub fn add_tree_view(&mut self, multi_select: bool) -> nightshade::ecs::Entity {
        let scroll_entity = self.add_scroll_area_fill(2.0, 0.0);
        let (content_entity, thumb_entity, track_entity) =
            widget::<UiScrollAreaData>(self.commands(), scroll_entity)
                .map(|data| (data.content_entity, data.thumb_entity, data.track_entity))
                .unwrap_or((scroll_entity, scroll_entity, scroll_entity));

        self.commands().set(
            scroll_entity,
            UiTreeViewData {
                selected_nodes: Vec::new(),
                multi_select,
                node_entities: Vec::new(),
                changed: false,
                content_entity,
                context_menu_node: None,
                filter_text: String::new(),
                filter_active: false,
                pre_filter_expanded: std::collections::HashMap::new(),
                scroll_offset: 0.0,
                thumb_entity,
                track_entity,
                thumb_dragging: false,
                thumb_drag_start_offset: 0.0,
            },
        );
        self.edit::<crate::components::UiNodeInteraction>(scroll_entity, |interaction| {
            interaction.accessible_role = Some(AccessibleRole::Tree);
        });

        scroll_entity
    }

    pub fn add_tree_node(
        &mut self,
        tree: nightshade::ecs::Entity,
        parent_container: nightshade::ecs::Entity,
        label: &str,
        depth: usize,
        user_data: u64,
    ) -> nightshade::ecs::Entity {
        self.add_tree_node_with_icon(tree, parent_container, None, label, depth, user_data)
    }

    pub fn add_tree_node_with_icon(
        &mut self,
        tree: nightshade::ecs::Entity,
        parent_container: nightshade::ecs::Entity,
        icon: Option<crate::data::icons::IconGlyph>,
        label: &str,
        depth: usize,
        user_data: u64,
    ) -> nightshade::ecs::Entity {
        let theme = self.active_theme();
        let font_size = theme.font_size;
        let row_height = font_size * 1.4;
        let indent = depth as f32 * 16.0;

        let arrow_slot = self.text_state_mut().cache.add_text("\u{25B6}");
        let label_slot = self.text_state_mut().cache.add_text(label);

        let wrapper_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 0.0)))
            .flow(FlowDirection::Vertical, 0.0, 0.0)
            .entity();

        self.edit::<nightshade::ecs::ChildOf>(wrapper_entity, move |parent| {
            *parent = nightshade::ecs::ChildOf(parent_container);
        });

        self.push_parent(wrapper_entity);

        let row_entity = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, row_height)))
            .with_rect(2.0, 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))
            .color_raw::<UiBase>(Vec4::new(0.0, 0.0, 0.0, 0.0))
            .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
            .with_interaction()
            .with_transition::<UiHover>(8.0, 6.0)
            .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
            .flow(FlowDirection::Horizontal, 4.0, 2.0)
            .entity();

        self.push_parent(row_entity);

        if indent > 0.0 {
            self.add_node()
                .flow_child(Ab(Vec2::new(indent, row_height)))
                .entity();
        }

        let arrow_entity = self
            .add_node()
            .flow_child(Ab(Vec2::new(16.0, row_height)))
            .with_text_slot(arrow_slot, font_size * 0.8)
            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
            .with_theme_color::<UiBase>(ThemeColor::Text)
            .entity();

        if let Some(icon) = icon {
            self.add_node()
                .flow_child(Ab(Vec2::new(18.0, row_height)))
                .with_icon(icon, font_size)
                .with_theme_color::<UiBase>(ThemeColor::Text)
                .entity();
        }

        self.add_node()
            .flow_child(Rl(Vec2::new(0.0, 100.0)))
            .flex_grow(1.0)
            .with_text_slot(label_slot, font_size)
            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
            .with_theme_color::<UiBase>(ThemeColor::Text)
            .entity();

        self.pop_parent();

        let children_container = self
            .add_node()
            .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 0.0)))
            .flow(FlowDirection::Vertical, 0.0, 0.0)
            .with_visible(false)
            .entity();

        self.pop_parent();

        let node_entity = row_entity;

        let existing_nodes =
            if let Some(tree_data) = self.component::<crate::components::UiTreeViewData>(tree) {
                tree_data.node_entities.clone()
            } else {
                Vec::new()
            };

        let parent_node = existing_nodes.iter().find_map(|&candidate| {
            if let Some(nd) = self.component::<crate::components::UiTreeNodeData>(candidate)
                && nd.children_container == parent_container
            {
                return Some(candidate);
            }
            None
        });

        self.commands().set(
            node_entity,
            UiTreeNodeData {
                label: label.to_string(),
                text_slot: label_slot,
                depth,
                expanded: false,
                selected: false,
                row_entity,
                arrow_entity,
                arrow_text_slot: arrow_slot,
                children_container,
                user_data,
                parent_node,
                wrapper_entity,
                lazy: false,
                lazy_loaded: false,
            },
        );
        self.edit::<crate::components::UiNodeInteraction>(node_entity, |interaction| {
            interaction.accessible_role = Some(AccessibleRole::TreeItem);
        });

        self.edit::<crate::components::UiTreeViewData>(tree, move |tree_data| {
            tree_data.node_entities.push(node_entity);
        });

        node_entity
    }
}
