use nalgebra_glm;
use nalgebra_glm::Vec2;

use crate::components::layer::UiLayer;
use crate::components::layout_types::{FlowLayout, UiLayoutType};
use crate::components::{
    DEFAULT_FONT_SIZE, TextOverflow, UiDepthMode, UiLayoutNode, UiLayoutRoot, UiNodeContent,
    UiPanelData, UiPanelKind,
};
use crate::data::units::UiEvalContext;
use crate::queries::layout::compute_layout_rect;
use crate::resources::taffy::UiTaffy;
use crate::resources::{
    DockState, PanelDockConfig, ReservedAreas, RetainedUiDirty, RetainedUiRuntime, WindowDockLayout,
};
use nightshade_paint::data::Rect;
use nightshade_platform::contract::window::Window;
use nightshade_text::contract::resources::TextState;

use super::color_blend::apply_animation_to_rect;

struct StackEntry {
    entity: nightshade::ecs::Entity,
    parent_rect: Rect,
    parent_depth: f32,
    parent_font_size: f32,
    parent_clip_rect: Option<Rect>,
    parent_layer: Option<UiLayer>,
    parent_rotation: f32,
    parent_pivot: Vec2,
    flow_override_rect: Option<Rect>,
}

#[derive(Clone, Copy)]
struct TaffyPass {
    absolute_scale: f32,
    viewport: Vec2,
}

fn viewport_size_of(window: &Window) -> Vec2 {
    window
        .cached_viewport_size
        .map(|(width, height)| Vec2::new(width as f32, height as f32))
        .unwrap_or(FALLBACK_VIEWPORT)
}

const FALLBACK_VIEWPORT: Vec2 = Vec2::new(800.0, 600.0);

struct PanelEntry {
    entity: nightshade::ecs::Entity,
    config: PanelDockConfig,
}

fn floating_rect(
    storage: &nightshade::ecs::Storage,
    entity: nightshade::ecs::Entity,
) -> (Option<[f32; 2]>, Option<[f32; 2]>) {
    let Some(UiLayoutType::Window(window)) = storage
        .get::<UiLayoutNode>(entity)
        .and_then(|node| node.base_layout)
    else {
        return (None, None);
    };
    (
        window.position.absolute.map(|value| [value.x, value.y]),
        window.size.absolute.map(|value| [value.x, value.y]),
    )
}

fn collect_panels(storage: &nightshade::ecs::Storage) -> Vec<PanelEntry> {
    let panel_entities: Vec<nightshade::ecs::Entity> = storage
        .query_entities::<(crate::components::UiPanelData,)>()
        .collect();

    let mut entries: Vec<PanelEntry> = Vec::with_capacity(panel_entities.len());
    for entity in panel_entities {
        let visible = storage
            .get::<UiLayoutNode>(entity)
            .map(|node| node.visible)
            .unwrap_or(true);
        let Some(data) = storage.get::<UiPanelData>(entity) else {
            continue;
        };
        let (floating_position, floating_size) = if data.panel_kind == UiPanelKind::Floating {
            floating_rect(storage, entity)
        } else {
            (None, None)
        };
        entries.push(PanelEntry {
            entity,
            config: PanelDockConfig {
                id: data.id.clone(),
                title: data.title.clone(),
                kind: data.panel_kind,
                focus_order: data.focus_order,
                size: data.default_dock_size,
                visible,
                collapsed: data.collapsed,
                floating_position,
                floating_size,
            },
        });
    }
    entries.sort_by_key(|entry| entry.config.focus_order);
    entries
}

fn dock_against(
    reserved: &mut ReservedAreas,
    kind: UiPanelKind,
    size: f32,
    viewport: Vec2,
) -> Option<(Vec2, Vec2)> {
    let remaining_height = viewport.y - reserved.top - reserved.bottom;
    let remaining_width = viewport.x - reserved.left - reserved.right;
    let rect = match kind {
        UiPanelKind::DockedLeft => (
            Vec2::new(reserved.left, reserved.top),
            Vec2::new(size, remaining_height),
        ),
        UiPanelKind::DockedRight => (
            Vec2::new(viewport.x - reserved.right - size, reserved.top),
            Vec2::new(size, remaining_height),
        ),
        UiPanelKind::DockedTop => (
            Vec2::new(reserved.left, reserved.top),
            Vec2::new(remaining_width, size),
        ),
        UiPanelKind::DockedBottom => (
            Vec2::new(reserved.left, viewport.y - reserved.bottom - size),
            Vec2::new(remaining_width, size),
        ),
        UiPanelKind::Floating => return None,
    };
    match kind {
        UiPanelKind::DockedLeft => reserved.left += size,
        UiPanelKind::DockedRight => reserved.right += size,
        UiPanelKind::DockedTop => reserved.top += size,
        UiPanelKind::DockedBottom => reserved.bottom += size,
        UiPanelKind::Floating => {}
    }
    Some(rect)
}

pub fn ui_docked_panel_layout_system(
    runtime: nightshade::ecs::system_param::Res<RetainedUiRuntime>,
    window: nightshade::ecs::system_param::Res<Window>,
    mut dock: nightshade::ecs::system_param::ResMut<DockState>,
    storage: &mut nightshade::ecs::Storage,
) {
    if !runtime.visible {
        return;
    }

    let rect = layout_rect(&window);
    let origin = rect.min;
    let viewport = rect.max - rect.min;
    let dpi_scale = window.cached_scale_factor;

    *dock = DockState::default();

    let entries = collect_panels(storage);
    let mut reserved = ReservedAreas::default();
    let mut configs: Vec<PanelDockConfig> = Vec::with_capacity(entries.len());

    for entry in entries {
        let kind = entry.config.kind;
        let dock_size = entry.config.size * dpi_scale;
        let visible = entry.config.visible;
        configs.push(entry.config);

        if !visible {
            continue;
        }
        let Some((position, size)) = dock_against(&mut reserved, kind, dock_size, viewport) else {
            continue;
        };

        if let Some(node) = storage.get_mut::<UiLayoutNode>(entry.entity)
            && let Some(UiLayoutType::Window(layout)) = node.base_layout.as_mut()
        {
            let current_absolute = layout.position.absolute.unwrap_or(Vec2::new(0.0, 0.0));
            let parent_min = node.computed_rect.min - current_absolute * dpi_scale;
            layout.position =
                crate::data::units::Ab((origin + position - parent_min) / dpi_scale).into();
            layout.size = crate::data::units::Ab(size / dpi_scale).into();
        }
    }

    dock.primary = WindowDockLayout {
        reserved_areas: reserved,
        panels: configs,
    };
}

pub(super) struct LayoutScope<'a> {
    pub storage: &'a mut nightshade::ecs::Storage,
    pub taffy: &'a mut UiTaffy,
    pub text: &'a mut TextState,
    pub hierarchy: &'a nightshade::ecs::HierarchyIndex,
}

impl LayoutScope<'_> {
    fn reborrow(&mut self) -> LayoutScope<'_> {
        LayoutScope {
            storage: self.storage,
            taffy: self.taffy,
            text: self.text,
            hierarchy: self.hierarchy,
        }
    }
}

struct LayoutTrigger {
    rect: Rect,
    dpi_scale: f32,
    text_changed: bool,
}

fn layout_rect(window: &Window) -> Rect {
    if let Some(rect) = window.content_rect
        && rect.width > 0.5
        && rect.height > 0.5
    {
        return Rect::from_min_max(
            Vec2::new(rect.x, rect.y),
            Vec2::new(rect.x + rect.width, rect.y + rect.height),
        );
    }
    let size = viewport_size_of(window);
    Rect::from_min_max(Vec2::new(0.0, 0.0), size)
}

fn take_layout_trigger(
    window: &Window,
    text: &TextState,
    dirty: &mut RetainedUiDirty,
) -> Option<LayoutTrigger> {
    let rect = layout_rect(window);
    let dpi_scale = window.cached_scale_factor;
    let text_generation = text.cache.generation;

    let text_changed = text_generation != dirty.last_compute_text_generation;
    let moved = dirty
        .last_compute_rect
        .map(|last| {
            (last.min - rect.min).magnitude() > 0.5 || (last.max - rect.max).magnitude() > 0.5
        })
        .unwrap_or(true);
    let changed = moved
        || (dpi_scale - dirty.last_compute_dpi).abs() > f32::EPSILON
        || text_changed
        || dirty.layout_dirty;
    if !changed {
        return None;
    }

    dirty.last_compute_rect = Some(rect);
    dirty.last_compute_dpi = dpi_scale;
    dirty.last_compute_text_generation = text_generation;
    dirty.layout_dirty = false;
    dirty.render_dirty = true;
    dirty.global_version += 1;

    Some(LayoutTrigger {
        rect,
        dpi_scale,
        text_changed,
    })
}

pub fn ui_layout_compute_system(
    window: nightshade::ecs::system_param::Res<Window>,
    hierarchy: nightshade::ecs::system_param::Res<nightshade::ecs::HierarchyIndex>,
    mut runtime: nightshade::ecs::system_param::ResMut<RetainedUiRuntime>,
    mut dirty: nightshade::ecs::system_param::ResMut<RetainedUiDirty>,
    mut taffy: nightshade::ecs::system_param::ResMut<UiTaffy>,
    mut text: nightshade::ecs::system_param::ResMut<TextState>,
    storage: &mut nightshade::ecs::Storage,
) {
    if !runtime.visible {
        return;
    }

    let Some(trigger) = take_layout_trigger(&window, &text, &mut dirty) else {
        return;
    };
    let dpi_scale = trigger.dpi_scale;

    let mut scope = LayoutScope {
        storage,
        taffy: &mut taffy,
        text: &mut text,
        hierarchy: &hierarchy,
    };

    prune_taffy(&mut scope);

    if trigger.text_changed {
        scope.taffy.invalidate_wrap_cache();
    }

    let root_origin = trigger.rect.min;
    let viewport_size = trigger.rect.max - trigger.rect.min;

    let root_entities: Vec<nightshade::ecs::Entity> = scope
        .storage
        .query_entities::<(crate::components::UiLayoutRoot,)>()
        .collect();

    let mut region_rects = std::mem::take(&mut runtime.taffy_region_rects);
    region_rects.clear();

    let mut children_scratch: Vec<nightshade::ecs::Entity> = Vec::new();

    let mut all_sorted_nodes = std::mem::take(&mut runtime.z_sorted_nodes);
    all_sorted_nodes.clear();

    let mut stack: Vec<StackEntry> = Vec::new();

    for root_entity in root_entities {
        let Some((root_intrinsic_scale, default_font_size)) = scope
            .storage
            .get::<UiLayoutRoot>(root_entity)
            .map(|root| (root.absolute_scale, root.default_font_size))
        else {
            continue;
        };

        let root_viewport_size = viewport_size;
        let absolute_scale = root_intrinsic_scale * dpi_scale;

        let root_rect = Rect::new(
            root_origin.x,
            root_origin.y,
            root_viewport_size.x,
            root_viewport_size.y,
        );
        let root_children: &[nightshade::ecs::Entity] = scope.hierarchy.children(root_entity);

        for child in root_children.iter().rev() {
            if scope.storage.get::<UiLayoutNode>(*child).is_some() {
                stack.push(StackEntry {
                    entity: *child,
                    parent_rect: root_rect,
                    parent_depth: 0.0,
                    parent_font_size: default_font_size,
                    parent_clip_rect: None,
                    parent_layer: None,
                    parent_rotation: 0.0,
                    parent_pivot: Vec2::new(0.0, 0.0),
                    flow_override_rect: None,
                });
            }
        }

        while let Some(entry) = stack.pop() {
            let (
                base_layout,
                flow_layout,
                responsive_flow,
                grid_layout,
                depth_mode,
                node_font_size,
                visible,
                clip_content,
                node_layer,
                node_rotation,
                node_min_size,
                node_max_size,
            ) = {
                match scope.storage.get::<UiLayoutNode>(entry.entity) {
                    Some(node) => (
                        node.base_layout,
                        effective_flow(scope.storage, scope.hierarchy, entry.entity, node),
                        node.responsive_flow,
                        node.grid_layout,
                        node.depth,
                        node.font_size,
                        node.visible,
                        node.clip_content,
                        node.layer,
                        node.rotation,
                        node.min_size,
                        node.max_size,
                    ),
                    None => continue,
                }
            };

            if !visible {
                continue;
            }

            let font_size = node_font_size.unwrap_or(entry.parent_font_size);

            let context = UiEvalContext {
                parent_width: entry.parent_rect.width(),
                parent_height: entry.parent_rect.height(),
                viewport_width: root_viewport_size.x,
                viewport_height: root_viewport_size.y,
                font_size,
                absolute_scale,
            };

            let computed_rect = if let Some(flow_rect) = entry.flow_override_rect {
                flow_rect
            } else if let Some(base_layout) = &base_layout {
                compute_layout_rect(base_layout, &context, &entry.parent_rect)
            } else {
                entry.parent_rect
            };

            let taffy_sized = entry.flow_override_rect.is_some();
            let constrained = !taffy_sized && (node_min_size.is_some() || node_max_size.is_some());
            let computed_rect = if constrained {
                let mut width = computed_rect.width();
                let mut height = computed_rect.height();
                if let Some(min) = node_min_size {
                    width = width.max(min.x * absolute_scale);
                    height = height.max(min.y * absolute_scale);
                }
                if let Some(max) = node_max_size {
                    width = width.min(max.x * absolute_scale);
                    height = height.min(max.y * absolute_scale);
                }
                Rect::from_min_max(
                    computed_rect.min,
                    Vec2::new(computed_rect.min.x + width, computed_rect.min.y + height),
                )
            } else {
                computed_rect
            };

            let depth = match depth_mode {
                UiDepthMode::Add(delta) => entry.parent_depth + delta,
                UiDepthMode::Set(absolute) => absolute,
            };

            let effective_layer = node_layer.or(entry.parent_layer);

            let is_popup_layer = matches!(
                effective_layer,
                Some(UiLayer::Popups) | Some(UiLayer::Tooltips)
            );

            let computed_rect =
                if is_popup_layer && matches!(base_layout, Some(UiLayoutType::Window(_))) {
                    keep_popup_inside(computed_rect, entry.parent_rect, root_rect)
                } else {
                    computed_rect
                };

            let (node_clip_rect, child_clip_rect) = if is_popup_layer {
                (None, None)
            } else if clip_content {
                let clipped = match &entry.parent_clip_rect {
                    Some(parent_clip) => {
                        Some(computed_rect.intersect(parent_clip).unwrap_or(*parent_clip))
                    }
                    None => Some(computed_rect),
                };
                (clipped, clipped)
            } else {
                (entry.parent_clip_rect, entry.parent_clip_rect)
            };

            let computed_rect = if let Some(node) = scope.storage.get::<UiLayoutNode>(entry.entity)
                && let Some(animation) = &node.animation
            {
                apply_animation_to_rect(computed_rect, animation)
            } else {
                computed_rect
            };

            let computed_rect = snap_to_pixel_grid(computed_rect);

            let prev_rect_data = scope
                .storage
                .get::<UiLayoutNode>(entry.entity)
                .map(|node| (node.computed_rect.min, node.computed_rect.max));
            let new_rect_data = (computed_rect.min, computed_rect.max);
            let effective_rotation = entry.parent_rotation + node_rotation;
            let effective_pivot = if entry.parent_rotation != 0.0 {
                entry.parent_pivot
            } else if node_rotation != 0.0 {
                computed_rect.center()
            } else {
                Vec2::new(0.0, 0.0)
            };
            if let Some(node) = scope.storage.get_mut::<UiLayoutNode>(entry.entity) {
                node.computed_rotation = effective_rotation;
                node.computed_pivot = effective_pivot;
                node.computed_rect = computed_rect;
                node.computed_depth = depth;
                node.computed_font_scale = root_intrinsic_scale;
                node.computed_clip_rect = node_clip_rect;
                node.computed_layer = effective_layer;
            }
            if prev_rect_data != Some(new_rect_data) {
                let global = dirty.global_version;
                dirty.node_versions.insert(entry.entity, global);
            }

            all_sorted_nodes.push((
                entry.entity,
                layer_rank(effective_layer) + depth * DEPTH_WITHIN_LAYER,
            ));

            children_scratch.clear();
            children_scratch.extend_from_slice(scope.hierarchy.children(entry.entity));

            let scroll_offset = scope
                .storage
                .get::<UiLayoutNode>(entry.entity)
                .map(|node| node.scroll_offset * absolute_scale)
                .unwrap_or(Vec2::new(0.0, 0.0));
            let scroll_shift = scroll_offset != Vec2::new(0.0, 0.0);
            let scrolled_parent_rect = if scroll_shift {
                Rect::from_min_max(
                    computed_rect.min - scroll_offset,
                    computed_rect.max - scroll_offset,
                )
            } else {
                computed_rect
            };

            if flow_layout.is_some() || grid_layout.is_some() {
                let is_region_root = entry.flow_override_rect.is_none();
                let responsive_repass = flow_layout.is_some() && responsive_flow.is_some();
                let child_rects: Vec<(nightshade::ecs::Entity, Option<Rect>)> =
                    if is_region_root || responsive_repass {
                        let parent_style = if let Some(flow) = &flow_layout {
                            let effective_flow = if let Some(resp) = responsive_flow
                                && computed_rect.width() < resp.max_width * absolute_scale
                            {
                                FlowLayout {
                                    direction: resp.direction,
                                    ..*flow
                                }
                            } else {
                                *flow
                            };
                            crate::systems::taffy_layout::flow_to_style(
                                scope.storage.get::<UiLayoutNode>(entry.entity).unwrap(),
                                &effective_flow,
                                scrolled_parent_rect.size(),
                                absolute_scale,
                            )
                        } else {
                            crate::systems::taffy_layout::grid_to_style(
                                scope.storage.get::<UiLayoutNode>(entry.entity).unwrap(),
                                grid_layout.as_ref().unwrap(),
                                scrolled_parent_rect.size(),
                                absolute_scale,
                            )
                        };
                        compute_taffy_children(
                            &mut scope,
                            entry.entity,
                            parent_style,
                            &children_scratch,
                            &scrolled_parent_rect,
                            TaffyPass {
                                absolute_scale,
                                viewport: root_viewport_size,
                            },
                            &mut region_rects,
                        )
                    } else {
                        read_region_children(&scope.reborrow(), &children_scratch, &region_rects)
                    };

                for (child, child_rect) in child_rects.iter().rev() {
                    stack.push(StackEntry {
                        entity: *child,
                        parent_rect: scrolled_parent_rect,
                        parent_depth: depth,
                        parent_font_size: font_size,
                        parent_clip_rect: child_clip_rect,
                        parent_layer: effective_layer,
                        parent_rotation: effective_rotation,
                        parent_pivot: effective_pivot,
                        flow_override_rect: *child_rect,
                    });
                }
            } else {
                for child in children_scratch.iter().rev() {
                    if scope.storage.get::<UiLayoutNode>(*child).is_some() {
                        stack.push(StackEntry {
                            entity: *child,
                            parent_rect: scrolled_parent_rect,
                            parent_depth: depth,
                            parent_font_size: font_size,
                            parent_clip_rect: child_clip_rect,
                            parent_layer: effective_layer,
                            parent_rotation: effective_rotation,
                            parent_pivot: effective_pivot,
                            flow_override_rect: None,
                        });
                    }
                }
            }
        }
    }

    runtime.taffy_region_rects = region_rects;

    all_sorted_nodes.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let primary_viewport = (viewport_size.x, viewport_size.y);
    let primary_grid = crate::systems::picking::build_picking_grid(
        scope.storage,
        &all_sorted_nodes,
        primary_viewport,
    );
    runtime.z_sorted_nodes = all_sorted_nodes;
    runtime.picking_grid = Some(primary_grid);
}

fn compute_taffy_children(
    scope: &mut LayoutScope<'_>,
    parent_entity: nightshade::ecs::Entity,
    parent_style: taffy::Style,
    children: &[nightshade::ecs::Entity],
    parent_rect: &Rect,
    pass: TaffyPass,
    region_rects: &mut std::collections::HashMap<nightshade::ecs::Entity, Rect>,
) -> Vec<(nightshade::ecs::Entity, Option<Rect>)> {
    use crate::systems::taffy_layout::available_size_for;

    let parent_size = parent_rect.size();

    scope.taffy.ensure_leaf(parent_entity, parent_style);

    let mut child_node_ids: Vec<taffy::NodeId> = Vec::with_capacity(children.len());
    let mut flowed_children: Vec<nightshade::ecs::Entity> = Vec::with_capacity(children.len());
    let mut absolute_children: Vec<nightshade::ecs::Entity> = Vec::new();
    for child in children {
        let Some(child_node) = scope.storage.get::<UiLayoutNode>(*child) else {
            continue;
        };
        if !child_node.visible {
            continue;
        }
        if child_node.base_layout.is_some() {
            absolute_children.push(*child);
            continue;
        }
        let child_node_id = build_taffy_subtree(scope, *child, parent_size, pass);
        child_node_ids.push(child_node_id);
        flowed_children.push(*child);
    }
    scope.taffy.set_children(parent_entity, &child_node_ids);

    let mut text = std::mem::take(scope.text);
    scope.taffy.compute_layout(
        parent_entity,
        available_size_for(parent_size),
        &mut text.font_engine,
    );
    *scope.text = text;

    for child in &flowed_children {
        collect_region_rects(
            scope,
            *child,
            parent_rect.min,
            pass.absolute_scale,
            region_rects,
        );
    }

    let mut rects = Vec::with_capacity(flowed_children.len() + absolute_children.len());
    for child in absolute_children {
        rects.push((child, None));
    }
    for child in flowed_children {
        rects.push((child, region_rects.get(&child).copied()));
    }
    rects
}

fn collect_region_rects(
    scope: &LayoutScope<'_>,
    entity: nightshade::ecs::Entity,
    parent_abs_min: Vec2,
    absolute_scale: f32,
    region_rects: &mut std::collections::HashMap<nightshade::ecs::Entity, Rect>,
) {
    let Some((location, size)) = scope.taffy.rect_for(entity) else {
        return;
    };
    let abs_min = parent_abs_min + location;
    region_rects.insert(entity, Rect::from_min_max(abs_min, abs_min + size));

    let Some(node) = scope.storage.get::<UiLayoutNode>(entity) else {
        return;
    };
    if effective_flow(scope.storage, scope.hierarchy, entity, node).is_none()
        && node.grid_layout.is_none()
    {
        return;
    }
    let child_origin = abs_min - node.scroll_offset * absolute_scale;

    for &child in scope.hierarchy.children(entity) {
        let Some(child_node) = scope.storage.get::<UiLayoutNode>(child) else {
            continue;
        };
        if !child_node.visible {
            continue;
        }
        if child_node.base_layout.is_some() {
            continue;
        }
        collect_region_rects(scope, child, child_origin, absolute_scale, region_rects);
    }
}

fn read_region_children(
    scope: &LayoutScope<'_>,
    children: &[nightshade::ecs::Entity],
    region_rects: &std::collections::HashMap<nightshade::ecs::Entity, Rect>,
) -> Vec<(nightshade::ecs::Entity, Option<Rect>)> {
    let mut rects = Vec::with_capacity(children.len());
    for child in children {
        let Some(child_node) = scope.storage.get::<UiLayoutNode>(*child) else {
            continue;
        };
        if !child_node.visible {
            continue;
        }
        if child_node.base_layout.is_some() {
            rects.push((*child, None));
        } else {
            rects.push((*child, region_rects.get(child).copied()));
        }
    }
    rects
}

fn build_taffy_subtree(
    scope: &mut LayoutScope<'_>,
    entity: nightshade::ecs::Entity,
    parent_size: Vec2,
    pass: TaffyPass,
) -> taffy::NodeId {
    use crate::systems::taffy_layout::child_to_style;

    let (mut style, is_container, font_size) = match scope.storage.get::<UiLayoutNode>(entity) {
        Some(node) => {
            let flow = effective_flow(scope.storage, scope.hierarchy, entity, node);
            let mut style = child_to_style(node, parent_size, pass.absolute_scale);
            if node.flow_layout.is_none()
                && let Some(flow) = flow.as_ref()
            {
                crate::systems::taffy_layout::apply_flow_container(
                    &mut style,
                    flow,
                    pass.absolute_scale,
                );
            }
            (
                style,
                flow.is_some() || node.grid_layout.is_some(),
                node.font_size.unwrap_or(DEFAULT_FONT_SIZE),
            )
        }
        None => (taffy::Style::default(), false, DEFAULT_FONT_SIZE),
    };

    if is_container {
        apply_absolute_child_floor(&mut style, scope, entity, parent_size, font_size, pass);
    }

    let node_id = scope.taffy.ensure_leaf(entity, style);

    if !is_container {
        let measured = measure_leaf(scope, entity, pass.absolute_scale);
        scope.taffy.set_intrinsic(entity, measured);
        let wrap = wrap_text_request(scope, entity, pass.absolute_scale);
        scope.taffy.set_wrap_text(entity, wrap);
        return node_id;
    }

    let child_count = scope.hierarchy.children(entity).len();

    let entity_size = scope
        .taffy
        .rect_for(entity)
        .map(|(_, size)| size)
        .unwrap_or(parent_size);

    let mut child_node_ids: Vec<taffy::NodeId> = Vec::new();
    for index in 0..child_count {
        let Some(child) = scope.hierarchy.children(entity).get(index).copied() else {
            continue;
        };
        let Some(child_node) = scope.storage.get::<UiLayoutNode>(child) else {
            continue;
        };
        if !child_node.visible {
            continue;
        }
        if child_node.base_layout.is_some() {
            continue;
        }
        let child_node_id = build_taffy_subtree(scope, child, entity_size, pass);
        child_node_ids.push(child_node_id);
    }
    scope.taffy.set_children(entity, &child_node_ids);

    node_id
}

fn apply_absolute_child_floor(
    style: &mut taffy::Style,
    scope: &LayoutScope<'_>,
    container: nightshade::ecs::Entity,
    parent_size: Vec2,
    font_size: f32,
    pass: TaffyPass,
) {
    use taffy::Dimension;

    let context = UiEvalContext {
        parent_width: parent_size.x,
        parent_height: parent_size.y,
        viewport_width: pass.viewport.x,
        viewport_height: pass.viewport.y,
        font_size,
        absolute_scale: pass.absolute_scale,
    };
    let available_parent = Rect::new(0.0, 0.0, parent_size.x, parent_size.y);

    let mut extent = Vec2::new(0.0, 0.0);
    for &child in scope.hierarchy.children(container) {
        let Some(node) = scope.storage.get::<UiLayoutNode>(child) else {
            continue;
        };
        if !node.visible {
            continue;
        }
        let Some(base_layout) = node.base_layout.as_ref() else {
            continue;
        };
        let rect = compute_layout_rect(base_layout, &context, &available_parent);
        extent.x = extent.x.max(rect.max.x);
        extent.y = extent.y.max(rect.max.y);
    }

    if extent.x > 0.0 && matches!(style.size.width, Dimension::Auto) {
        style.min_size.width = dimension_floor(style.min_size.width, extent.x);
    }
    if extent.y > 0.0 && matches!(style.size.height, Dimension::Auto) {
        style.min_size.height = dimension_floor(style.min_size.height, extent.y);
    }
}

fn dimension_floor(dimension: taffy::Dimension, floor: f32) -> taffy::Dimension {
    use taffy::prelude::FromLength;

    match dimension {
        taffy::Dimension::Length(value) => taffy::Dimension::from_length(value.max(floor)),
        _ => taffy::Dimension::from_length(floor),
    }
}

fn text_content_of(
    scope: &LayoutScope<'_>,
    entity: nightshade::ecs::Entity,
    absolute_scale: f32,
) -> Option<TextContent> {
    let content = scope.storage.get::<UiNodeContent>(entity)?;
    let layout_node = scope.storage.get::<UiLayoutNode>(entity)?;
    let UiNodeContent::Text {
        text_slot,
        font_size_override,
        overflow,
        font_kind,
        ..
    } = content
    else {
        return None;
    };
    let base_font = layout_node.font_size.unwrap_or(DEFAULT_FONT_SIZE);
    let font_size = font_size_override.unwrap_or(base_font) * absolute_scale;
    let text = scope.text.cache.get_text(*text_slot)?.to_string();
    Some(TextContent {
        text,
        font_size,
        font_kind: *font_kind,
        overflow: *overflow,
    })
}

struct TextContent {
    text: String,
    font_size: f32,
    font_kind: nightshade_text::contract::data::text_data::FontKind,
    overflow: TextOverflow,
}

fn wrap_text_request(
    scope: &LayoutScope<'_>,
    entity: nightshade::ecs::Entity,
    absolute_scale: f32,
) -> Option<crate::resources::taffy::WrapText> {
    let content = text_content_of(scope, entity, absolute_scale)?;
    matches!(content.overflow, TextOverflow::Wrap).then_some(crate::resources::taffy::WrapText {
        text: content.text,
        font_size: content.font_size,
        font_kind: content.font_kind,
    })
}

fn effective_flow(
    storage: &nightshade::ecs::Storage,
    hierarchy: &nightshade::ecs::HierarchyIndex,
    entity: nightshade::ecs::Entity,
    node: &UiLayoutNode,
) -> Option<FlowLayout> {
    if node.flow_layout.is_some() || node.grid_layout.is_some() {
        return node.flow_layout;
    }
    if matches!(node.auto_size, crate::components::AutoSizeMode::None) {
        return None;
    }
    let has_child = hierarchy.children(entity).iter().any(|child| {
        storage
            .get::<UiLayoutNode>(*child)
            .is_some_and(|child_node| child_node.visible && child_node.base_layout.is_none())
    });
    has_child.then(FlowLayout::stack)
}

fn measure_leaf(
    scope: &mut LayoutScope<'_>,
    entity: nightshade::ecs::Entity,
    absolute_scale: f32,
) -> taffy::Size<f32> {
    let Some(content) = text_content_of(scope, entity, absolute_scale) else {
        return taffy::Size::ZERO;
    };
    let width = crate::data::units::measure_text_width_with_kind(
        &mut scope.text.font_engine,
        &content.text,
        content.font_size,
        content.font_kind,
    );
    taffy::Size {
        width,
        height: content.font_size * crate::resources::taffy::LINE_HEIGHT_RATIO,
    }
}

fn layer_rank(layer: Option<UiLayer>) -> f32 {
    layer.unwrap_or(UiLayer::Background) as i32 as f32 * LAYER_BAND
}

const LAYER_BAND: f32 = 1000.0;
const DEPTH_WITHIN_LAYER: f32 = 0.1;

fn keep_popup_inside(popup: Rect, anchor: Rect, viewport: Rect) -> Rect {
    let size = popup.size();
    let mut top = popup.min.y;
    if top + size.y > viewport.max.y {
        let above = anchor.min.y - size.y;
        top = if above >= viewport.min.y {
            above
        } else {
            (viewport.max.y - size.y).max(viewport.min.y)
        };
    }
    let mut left = popup.min.x;
    if left + size.x > viewport.max.x {
        left = viewport.max.x - size.x;
    }
    left = left.max(viewport.min.x);
    Rect::from_min_max(Vec2::new(left, top), Vec2::new(left + size.x, top + size.y))
}

fn snap_to_pixel_grid(rect: Rect) -> Rect {
    Rect::from_min_max(
        Vec2::new(rect.min.x.round(), rect.min.y.round()),
        Vec2::new(rect.max.x.round(), rect.max.y.round()),
    )
}

fn prune_taffy(scope: &mut LayoutScope<'_>) {
    let mut live: std::collections::HashSet<nightshade::ecs::Entity> =
        std::collections::HashSet::new();
    let mut stack: Vec<nightshade::ecs::Entity> = scope
        .storage
        .query_entities::<(crate::components::UiLayoutRoot,)>()
        .collect();
    while let Some(entity) = stack.pop() {
        if !live.insert(entity) {
            continue;
        }
        stack.extend(scope.hierarchy.children(entity).iter().copied());
    }
    let stale: Vec<nightshade::ecs::Entity> = scope
        .taffy
        .entity_to_node
        .keys()
        .copied()
        .filter(|entity| !live.contains(entity))
        .collect();
    for entity in stale {
        scope.taffy.release(entity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::components::layout_types::FlowDirection;
    use crate::components::{UiPropertyGridData, UiScrollAreaData};
    use crate::data::units::{Ab, Rl};
    use nightshade_paint::data::Anchor;

    fn setup_world(width: u32, height: u32) -> nightshade::ecs::World {
        let mut world = nightshade::ecs::World::default();
        crate::insert_ui_resources(&mut world.resources);
        let window = world.res_mut::<Window>();
        window.cached_viewport_size = Some((width, height));
        world.res_mut::<Window>().cached_scale_factor = 1.0;
        world.res_mut::<RetainedUiRuntime>().visible = true;
        world
    }

    fn compute_layout(world: &mut nightshade::ecs::World) {
        world
            .resources
            .res_mut::<nightshade::ecs::HierarchyIndex>()
            .sync(&mut world.storage);
        world.res_mut::<RetainedUiDirty>().layout_dirty = true;
        nightshade::ecs::system_param::run_system(world, ui_layout_compute_system);
    }

    fn rect_of(storage: &nightshade::ecs::Storage, entity: nightshade::ecs::Entity) -> Rect {
        storage
            .get::<UiLayoutNode>(entity)
            .expect("layout node present")
            .computed_rect
    }

    fn assert_rect(actual: Rect, min: Vec2, max: Vec2) {
        let epsilon = 0.05;
        let close = (actual.min.x - min.x).abs() < epsilon
            && (actual.min.y - min.y).abs() < epsilon
            && (actual.max.x - max.x).abs() < epsilon
            && (actual.max.y - max.y).abs() < epsilon;
        assert!(
            close,
            "expected min {:?} max {:?}, got min {:?} max {:?}",
            min, max, actual.min, actual.max
        );
    }

    #[test]
    fn vertical_flow_stacks_children_with_gap() {
        let mut world = setup_world(1000, 800);
        let (container, first, second) =
            crate::commands::builder::tree_over(&mut world.storage, &mut world.resources, |tree| {
                let container = tree
                    .add_node()
                    .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
                    .flow(FlowDirection::Vertical, 0.0, 10.0)
                    .entity();
                let (first, second) = tree.in_parent(container, |tree| {
                    let first = tree
                        .add_node()
                        .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 100.0)))
                        .entity();
                    let second = tree
                        .add_node()
                        .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 100.0)))
                        .entity();
                    (first, second)
                });
                (container, first, second)
            });

        compute_layout(&mut world);

        assert_rect(
            rect_of(&world.storage, container),
            Vec2::new(0.0, 0.0),
            Vec2::new(1000.0, 800.0),
        );
        assert_rect(
            rect_of(&world.storage, first),
            Vec2::new(0.0, 0.0),
            Vec2::new(1000.0, 100.0),
        );
        assert_rect(
            rect_of(&world.storage, second),
            Vec2::new(0.0, 110.0),
            Vec2::new(1000.0, 210.0),
        );
    }

    #[test]
    fn absolute_scale_shrinks_flow_children_and_gap() {
        let mut world = setup_world(1000, 800);
        let (left, right) =
            crate::commands::builder::tree_over(&mut world.storage, &mut world.resources, |tree| {
                tree.set_absolute_scale(0.5);
                let container = tree
                    .add_node()
                    .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
                    .flow(FlowDirection::Horizontal, 0.0, 20.0)
                    .entity();
                let (left, right) = tree.in_parent(container, |tree| {
                    let left = tree
                        .add_node()
                        .flow_child(Ab(Vec2::new(200.0, 0.0)) + Rl(Vec2::new(0.0, 100.0)))
                        .entity();
                    let right = tree
                        .add_node()
                        .flow_child(Ab(Vec2::new(200.0, 0.0)) + Rl(Vec2::new(0.0, 100.0)))
                        .entity();
                    (left, right)
                });
                (left, right)
            });

        compute_layout(&mut world);

        assert_rect(
            rect_of(&world.storage, left),
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 800.0),
        );
        assert_rect(
            rect_of(&world.storage, right),
            Vec2::new(110.0, 0.0),
            Vec2::new(210.0, 800.0),
        );
    }

    #[test]
    fn nested_flow_region_resolves_grandchildren() {
        let mut world = setup_world(1000, 800);
        let (_outer, inner, left, right) =
            crate::commands::builder::tree_over(&mut world.storage, &mut world.resources, |tree| {
                let outer = tree
                    .add_node()
                    .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
                    .flow(FlowDirection::Vertical, 0.0, 0.0)
                    .entity();
                let (inner, left, right) = tree.in_parent(outer, |tree| {
                    let inner = tree
                        .add_node()
                        .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 200.0)))
                        .flow(FlowDirection::Horizontal, 0.0, 20.0)
                        .entity();
                    let (left, right) = tree.in_parent(inner, |tree| {
                        let left = tree
                            .add_node()
                            .flow_child(Ab(Vec2::new(200.0, 0.0)) + Rl(Vec2::new(0.0, 100.0)))
                            .entity();
                        let right = tree
                            .add_node()
                            .flow_child(Ab(Vec2::new(200.0, 0.0)) + Rl(Vec2::new(0.0, 100.0)))
                            .entity();
                        (left, right)
                    });
                    (inner, left, right)
                });
                (outer, inner, left, right)
            });

        compute_layout(&mut world);

        assert_rect(
            rect_of(&world.storage, inner),
            Vec2::new(0.0, 0.0),
            Vec2::new(1000.0, 200.0),
        );
        assert_rect(
            rect_of(&world.storage, left),
            Vec2::new(0.0, 0.0),
            Vec2::new(200.0, 200.0),
        );
        assert_rect(
            rect_of(&world.storage, right),
            Vec2::new(220.0, 0.0),
            Vec2::new(420.0, 200.0),
        );
    }

    #[test]
    fn scroll_offset_shifts_flow_children() {
        let mut world = setup_world(1000, 800);
        let (container, first, second) =
            crate::commands::builder::tree_over(&mut world.storage, &mut world.resources, |tree| {
                let container = tree
                    .add_node()
                    .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
                    .flow(FlowDirection::Vertical, 0.0, 10.0)
                    .entity();
                let (first, second) = tree.in_parent(container, |tree| {
                    let first = tree
                        .add_node()
                        .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 100.0)))
                        .entity();
                    let second = tree
                        .add_node()
                        .flow_child(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, 100.0)))
                        .entity();
                    (first, second)
                });
                (container, first, second)
            });

        if let Some(node) = world.storage.get_mut::<UiLayoutNode>(container) {
            node.scroll_offset = Vec2::new(0.0, 50.0);
        }

        compute_layout(&mut world);

        assert_rect(
            rect_of(&world.storage, first),
            Vec2::new(0.0, -50.0),
            Vec2::new(1000.0, 50.0),
        );
        assert_rect(
            rect_of(&world.storage, second),
            Vec2::new(0.0, 60.0),
            Vec2::new(1000.0, 160.0),
        );
    }

    #[test]
    fn auto_container_grows_to_fit_absolute_child() {
        let mut world = setup_world(1000, 800);
        let inner =
            crate::commands::builder::tree_over(&mut world.storage, &mut world.resources, |tree| {
                let outer = tree
                    .add_node()
                    .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
                    .flow(FlowDirection::Vertical, 0.0, 0.0)
                    .entity();

                tree.in_parent(outer, |tree| {
                    let inner = tree
                        .add_node()
                        .flow(FlowDirection::Vertical, 0.0, 0.0)
                        .entity();
                    tree.in_parent(inner, |tree| {
                        tree.add_node()
                            .window(
                                Ab(Vec2::new(0.0, 0.0)),
                                Ab(Vec2::new(300.0, 150.0)),
                                Anchor::TopLeft,
                            )
                            .entity();
                    });
                    inner
                })
            });

        compute_layout(&mut world);

        assert_rect(
            rect_of(&world.storage, inner),
            Vec2::new(0.0, 0.0),
            Vec2::new(300.0, 150.0),
        );
    }

    #[test]
    fn property_grid_rows_stack_without_overlap() {
        let mut world = setup_world(1000, 800);
        let grid =
            crate::commands::builder::tree_over(&mut world.storage, &mut world.resources, |tree| {
                let container = tree
                    .add_node()
                    .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
                    .flow(FlowDirection::Vertical, 0.0, 0.0)
                    .entity();

                tree.in_parent(container, |tree| {
                    let grid = tree.add_property_grid(96.0);
                    let section = tree.add_property_section(grid, "Transform");
                    tree.add_property_drag_value(grid, section, "X", -100.0, 100.0, 1.0);
                    tree.add_property_drag_value(grid, section, "Y", -100.0, 100.0, 2.0);
                    tree.add_property_drag_value(grid, section, "Z", -100.0, 100.0, 3.0);
                    grid
                })
            });

        compute_layout(&mut world);

        let rows = world
            .storage
            .get::<UiPropertyGridData>(grid)
            .expect("property grid data")
            .row_entities
            .clone();
        assert_eq!(rows.len(), 3, "three drag-value rows expected");

        let mut rects: Vec<Rect> = rows
            .iter()
            .map(|row| rect_of(&world.storage, *row))
            .collect();
        for rect in &rects {
            let height = rect.height();
            assert!(
                height > 1.0 && height < 80.0,
                "a property row should be a single line tall, got {height}"
            );
        }

        rects.sort_by(|a, b| {
            a.min
                .y
                .partial_cmp(&b.min.y)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for pair in rects.windows(2) {
            assert!(
                pair[1].min.y >= pair[0].max.y - 0.5,
                "rows overlap: {:?} then {:?}",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn docked_panel_scroll_grid_diagnostic() {
        let mut world = setup_world(1920, 1080);
        let (grid, panel) =
            crate::commands::builder::tree_over(&mut world.storage, &mut world.resources, |tree| {
                let panel = tree.add_docked_panel_right("inspector", "Inspector", 320.0);
                let content = tree
                    .component::<UiPanelData>(panel)
                    .map(|data| data.content_entity)
                    .expect("panel content");
                let grid = tree.in_parent(content, |tree| {
                    let scroll = tree.add_scroll_area_fill(0.0, 0.0);
                    let body = tree
                        .component::<UiScrollAreaData>(scroll)
                        .map(|data| data.content_entity)
                        .unwrap_or(scroll);
                    tree.in_parent(body, |tree| {
                        let grid = tree.add_property_grid(96.0);
                        let section = tree.add_property_section(grid, "Transform");
                        tree.add_property_drag_value(grid, section, "X", -100.0, 100.0, 1.0);
                        tree.add_property_drag_value(grid, section, "Y", -100.0, 100.0, 2.0);
                        tree.add_property_drag_value(grid, section, "Z", -100.0, 100.0, 3.0);
                        grid
                    })
                });
                (grid, panel)
            });

        for _ in 0..2 {
            world
                .resources
                .res_mut::<nightshade::ecs::HierarchyIndex>()
                .sync(&mut world.storage);
            for mut system in crate::default_ui_systems() {
                system(&mut world);
            }
        }

        let panel_rect = rect_of(&world.storage, panel);
        assert!(
            (panel_rect.min.x - 1600.0).abs() < 1.0,
            "right-docked panel should start at viewport width minus its size, got {}",
            panel_rect.min.x
        );

        let rows = world
            .storage
            .get::<UiPropertyGridData>(grid)
            .expect("property grid data")
            .row_entities
            .clone();
        let mut rects: Vec<Rect> = rows
            .iter()
            .map(|row| rect_of(&world.storage, *row))
            .collect();
        for rect in &rects {
            assert!(
                rect.height() > 1.0 && rect.height() < 80.0,
                "row inside the docked panel should be one line tall, got {}",
                rect.height()
            );
            assert!(
                rect.min.x >= panel_rect.min.x && rect.max.x <= panel_rect.max.x + 0.5,
                "row should sit within the panel bounds"
            );
        }
        rects.sort_by(|a, b| {
            a.min
                .y
                .partial_cmp(&b.min.y)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for pair in rects.windows(2) {
            assert!(
                pair[1].min.y >= pair[0].max.y - 0.5,
                "rows overlap inside the docked panel: {:?} then {:?}",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn property_grid_rows_stack_at_high_dpi() {
        let mut world = setup_world(1280, 1600);
        world.res_mut::<Window>().cached_scale_factor = 2.0;
        let grid =
            crate::commands::builder::tree_over(&mut world.storage, &mut world.resources, |tree| {
                let container = tree
                    .add_node()
                    .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
                    .flow(FlowDirection::Vertical, 0.0, 0.0)
                    .entity();

                tree.in_parent(container, |tree| {
                    let grid = tree.add_property_grid(96.0);
                    let section = tree.add_property_section(grid, "Transform");
                    tree.add_property_drag_value(grid, section, "X", -100.0, 100.0, 1.0);
                    tree.add_property_drag_value(grid, section, "Y", -100.0, 100.0, 2.0);
                    tree.add_property_drag_value(grid, section, "Z", -100.0, 100.0, 3.0);
                    grid
                })
            });

        compute_layout(&mut world);

        let rows = world
            .storage
            .get::<UiPropertyGridData>(grid)
            .expect("property grid data")
            .row_entities
            .clone();
        let mut rects: Vec<Rect> = rows
            .iter()
            .map(|row| rect_of(&world.storage, *row))
            .collect();
        for rect in &rects {
            let height = rect.height();
            assert!(
                height > 1.0 && height < 160.0,
                "at dpi 2 a property row should be one scaled line, got {height}"
            );
        }
        rects.sort_by(|a, b| {
            a.min
                .y
                .partial_cmp(&b.min.y)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for pair in rects.windows(2) {
            assert!(
                pair[1].min.y >= pair[0].max.y - 0.5,
                "rows overlap at dpi 2: {:?} then {:?}",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn property_grid_mixed_widgets_stack_without_overlap() {
        let mut world = setup_world(1000, 800);
        let grid =
            crate::commands::builder::tree_over(&mut world.storage, &mut world.resources, |tree| {
                let container = tree
                    .add_node()
                    .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
                    .flow(FlowDirection::Vertical, 0.0, 0.0)
                    .entity();

                tree.in_parent(container, |tree| {
                    let grid = tree.add_property_grid(96.0);
                    let section = tree.add_property_section(grid, "Mixed");
                    tree.add_property_drag_value(grid, section, "Drag", -100.0, 100.0, 1.0);
                    tree.add_property_checkbox(grid, section, "Check", true);
                    tree.add_property_dropdown(grid, section, "Pick", &["A", "B", "C"], 0);
                    tree.add_property_text_input(grid, section, "Text", "value");
                    grid
                })
            });

        compute_layout(&mut world);

        let rows = world
            .storage
            .get::<UiPropertyGridData>(grid)
            .expect("property grid data")
            .row_entities
            .clone();
        assert_eq!(rows.len(), 4, "four widget rows expected");

        let mut rects: Vec<Rect> = rows
            .iter()
            .map(|row| rect_of(&world.storage, *row))
            .collect();
        for rect in &rects {
            let height = rect.height();
            assert!(
                height > 1.0 && height < 80.0,
                "a property row should be a single line tall, got {height}"
            );
            assert!(
                rect.width() > 200.0,
                "a property row should span the grid width, got {}",
                rect.width()
            );
        }

        rects.sort_by(|a, b| {
            a.min
                .y
                .partial_cmp(&b.min.y)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for pair in rects.windows(2) {
            assert!(
                pair[1].min.y >= pair[0].max.y - 0.5,
                "rows overlap: {:?} then {:?}",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn explicit_size_container_ignores_absolute_child() {
        let mut world = setup_world(1000, 800);
        let inner =
            crate::commands::builder::tree_over(&mut world.storage, &mut world.resources, |tree| {
                let outer = tree
                    .add_node()
                    .boundary(Rl(Vec2::new(0.0, 0.0)), Rl(Vec2::new(100.0, 100.0)))
                    .flow(FlowDirection::Vertical, 0.0, 0.0)
                    .entity();

                tree.in_parent(outer, |tree| {
                    let inner = tree
                        .add_node()
                        .flow_child(Ab(Vec2::new(120.0, 40.0)))
                        .flow(FlowDirection::Vertical, 0.0, 0.0)
                        .entity();
                    tree.in_parent(inner, |tree| {
                        tree.add_node()
                            .window(
                                Ab(Vec2::new(0.0, 0.0)),
                                Ab(Vec2::new(300.0, 150.0)),
                                Anchor::TopLeft,
                            )
                            .entity();
                    });
                    inner
                })
            });

        compute_layout(&mut world);

        assert_rect(
            rect_of(&world.storage, inner),
            Vec2::new(0.0, 0.0),
            Vec2::new(120.0, 40.0),
        );
    }
}
