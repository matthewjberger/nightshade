use std::collections::HashMap;

use nalgebra_glm::Vec2;
use taffy::{AvailableSpace, NodeId, Size, Style, TaffyTree, TraversePartialTree};

use nightshade_text::contract::data::font_engine::FontEngine;
use nightshade_text::contract::data::text_data::FontKind;

pub const LINE_HEIGHT_RATIO: f32 = 1.2;

#[derive(Clone, Debug, PartialEq)]
pub struct WrapText {
    pub text: String,
    pub font_size: f32,
    pub font_kind: FontKind,
}

#[derive(Clone, Debug, Default)]
pub struct NodeMeasure {
    pub intrinsic: Size<f32>,
    pub wrap: Option<WrapText>,
    cached_wrap: Option<(f32, Size<f32>)>,
}

pub struct UiTaffy {
    pub tree: TaffyTree<NodeMeasure>,
    pub entity_to_node: HashMap<nightshade::ecs::Entity, NodeId>,
}

impl Default for UiTaffy {
    fn default() -> Self {
        Self {
            tree: TaffyTree::new(),
            entity_to_node: HashMap::new(),
        }
    }
}

impl std::fmt::Debug for UiTaffy {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UiTaffy")
            .field("entity_count", &self.entity_to_node.len())
            .finish()
    }
}

impl UiTaffy {
    pub fn release(&mut self, entity: nightshade::ecs::Entity) {
        if let Some(node_id) = self.entity_to_node.remove(&entity) {
            self.tree.remove(node_id).expect("taffy remove");
        }
    }

    pub fn ensure_leaf(&mut self, entity: nightshade::ecs::Entity, style: Style) -> NodeId {
        if let Some(&node_id) = self.entity_to_node.get(&entity) {
            let unchanged = self
                .tree
                .style(node_id)
                .is_ok_and(|current| *current == style);
            if !unchanged {
                self.tree
                    .set_style(node_id, style)
                    .expect("taffy set_style");
            }
            return node_id;
        }
        let node_id = self
            .tree
            .new_leaf_with_context(style, NodeMeasure::default())
            .expect("taffy new_leaf");
        self.entity_to_node.insert(entity, node_id);
        node_id
    }

    pub fn set_children(&mut self, parent: nightshade::ecs::Entity, children: &[NodeId]) {
        let Some(&parent_node) = self.entity_to_node.get(&parent) else {
            return;
        };
        if self.children_match(parent_node, children) {
            return;
        }
        self.tree
            .set_children(parent_node, children)
            .expect("taffy set_children");
    }

    fn children_match(&self, parent: NodeId, children: &[NodeId]) -> bool {
        if self.tree.child_count(parent) != children.len() {
            return false;
        }
        children.iter().enumerate().all(|(index, child)| {
            self.tree
                .child_at_index(parent, index)
                .is_ok_and(|existing| existing == *child)
        })
    }

    fn measure_mut(&mut self, entity: nightshade::ecs::Entity) -> Option<&mut NodeMeasure> {
        let node_id = *self.entity_to_node.get(&entity)?;
        self.tree.get_node_context_mut(node_id)
    }

    pub fn set_intrinsic(&mut self, entity: nightshade::ecs::Entity, size: Size<f32>) {
        if let Some(measure) = self.measure_mut(entity)
            && measure.intrinsic != size
        {
            measure.intrinsic = size;
            measure.cached_wrap = None;
        }
    }

    pub fn set_wrap_text(&mut self, entity: nightshade::ecs::Entity, wrap: Option<WrapText>) {
        if let Some(measure) = self.measure_mut(entity)
            && measure.wrap != wrap
        {
            measure.wrap = wrap;
            measure.cached_wrap = None;
        }
    }

    pub fn invalidate_wrap_cache(&mut self) {
        let Self {
            tree,
            entity_to_node,
        } = self;
        for node_id in entity_to_node.values() {
            if let Some(measure) = tree.get_node_context_mut(*node_id) {
                measure.cached_wrap = None;
            }
        }
    }

    pub fn compute_layout(
        &mut self,
        root: nightshade::ecs::Entity,
        available: Size<AvailableSpace>,
        font_engine: &mut FontEngine,
    ) {
        let Some(&root_node) = self.entity_to_node.get(&root) else {
            return;
        };
        self.tree
            .compute_layout_with_measure(
                root_node,
                available,
                |known, available_in, _node_id, context, _style| {
                    measure_node(known, available_in, context, font_engine)
                },
            )
            .expect("taffy compute_layout");
    }

    pub fn rect_for(&self, entity: nightshade::ecs::Entity) -> Option<(Vec2, Vec2)> {
        let node_id = *self.entity_to_node.get(&entity)?;
        let layout = self.tree.layout(node_id).ok()?;
        let position = Vec2::new(layout.location.x, layout.location.y);
        let size = Vec2::new(layout.size.width, layout.size.height);
        Some((position, size))
    }
}

fn measure_node(
    known: Size<Option<f32>>,
    available: Size<AvailableSpace>,
    context: Option<&mut NodeMeasure>,
    font_engine: &mut FontEngine,
) -> Size<f32> {
    if let (Some(width), Some(height)) = (known.width, known.height) {
        return Size { width, height };
    }
    let Some(NodeMeasure {
        intrinsic,
        wrap,
        cached_wrap,
    }) = context
    else {
        return Size::ZERO;
    };

    if let Some(wrap) = wrap.as_ref() {
        let wrap_width = known.width.or(match available.width {
            AvailableSpace::Definite(value) => Some(value),
            _ => None,
        });
        if let Some(width) = wrap_width
            && width > 0.0
        {
            if let Some((cached_width, cached_size)) = cached_wrap
                && (*cached_width - width).abs() < f32::EPSILON
            {
                return *cached_size;
            }
            let wrapped = nightshade_text::contract::data::text_wrapping::wrap_text(
                font_engine,
                &wrap.text,
                wrap.font_size,
                width,
                wrap.font_kind,
            );
            let lines = wrapped.lines().count().max(1) as f32;
            let size = Size {
                width,
                height: wrap.font_size * LINE_HEIGHT_RATIO * lines,
            };
            *cached_wrap = Some((width, size));
            return size;
        }
    }

    Size {
        width: known.width.unwrap_or(intrinsic.width),
        height: known.height.unwrap_or(intrinsic.height),
    }
}
