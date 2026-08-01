use nalgebra_glm;
use nalgebra_glm::Vec4;

use crate::components::UiNodeContent;
use crate::components::layer::UiLayer;
use crate::systems::canvas_render::emit_canvas_commands;
use crate::systems::tile_render::emit_tile_containers;
use nightshade_paint::data::DrawText;
use nightshade_paint::data::Rect;
use nightshade_paint::data::{DrawImage, DrawRect};
use nightshade_text::contract::data::text_data::{TextAlignment, TextMesh, VerticalAlignment};

fn emit_top_progress_bar(
    sync: &mut crate::systems::surface::RenderSync<'_>,
    viewport_size: nalgebra_glm::Vec2,
    dpi_scale: f32,
) {
    let progress = sync.progress_bar.clone();
    if !progress.is_active() {
        return;
    }
    let theme = sync.theme.active_theme();
    let accent = progress.color.unwrap_or(theme.accent_color);
    let track = Vec4::new(accent.x, accent.y, accent.z, accent.w * 0.20);
    let height = progress.height * dpi_scale;
    let bar_y = 0.0_f32;

    sync.frame.rects.push(DrawRect {
        position: nalgebra_glm::Vec2::new(0.0, bar_y),
        size: nalgebra_glm::Vec2::new(viewport_size.x, height),
        color: track,
        corner_radius: 0.0,
        border_width: 0.0,
        border_color: Vec4::new(0.0, 0.0, 0.0, 0.0),
        rotation: 0.0,
        clip_rect: None,
        layer: UiLayer::Tooltips as u8,
        z_index: 1000,
        shadow: None,
        effect_kind: 0,
        effect_params: [0.0; 4],
        quad_corners: None,
    });
    sync.frame.rect_entities.push(None);

    let (fill_x, fill_width) = if progress.indeterminate {
        let bar_width = (viewport_size.x * 0.3).max(60.0);
        let total = viewport_size.x + bar_width;
        let phase = (progress.phase * 0.5).rem_euclid(1.0);
        let center = total * phase - bar_width * 0.5;
        (center, bar_width)
    } else {
        let value = progress.value.unwrap_or(0.0).clamp(0.0, 1.0);
        (0.0, viewport_size.x * value)
    };

    if fill_width > 0.0 {
        let highlight = Vec4::new(
            (accent.x + 0.1).min(1.0),
            (accent.y + 0.1).min(1.0),
            (accent.z + 0.1).min(1.0),
            accent.w,
        );
        sync.frame.rects.push(DrawRect {
            position: nalgebra_glm::Vec2::new(fill_x, bar_y),
            size: nalgebra_glm::Vec2::new(fill_width, height),
            color: accent,
            corner_radius: 0.0,
            border_width: 0.0,
            border_color: Vec4::new(0.0, 0.0, 0.0, 0.0),
            rotation: 0.0,
            clip_rect: Some(nightshade_paint::data::Rect::new(
                0.0,
                bar_y,
                viewport_size.x,
                bar_y + height,
            )),
            layer: UiLayer::Tooltips as u8,
            z_index: 1001,
            shadow: None,
            effect_kind: 2,
            effect_params: [highlight.x, highlight.y, highlight.z, highlight.w],
            quad_corners: None,
        });
        sync.frame.rect_entities.push(None);
    }
}

fn emit_rect_entry(
    entry: &RenderEntry,
    z_index: i32,
    dpi_scale: f32,
    corner_radius: f32,
    border_width: f32,
    border_color: Vec4,
) -> DrawRect {
    let shadow = entry.shadow.and_then(|shadow_entry| {
        if shadow_entry.color.w > 0.0 {
            Some(nightshade_paint::data::DrawShadow {
                color: shadow_entry.color,
                offset: shadow_entry.offset * dpi_scale,
                blur: shadow_entry.blur * dpi_scale,
                spread: shadow_entry.spread * dpi_scale,
            })
        } else {
            None
        }
    });
    let base_size = entry.computed_rect.size();
    let scale = entry.blended_transform.scale.max(0.0);
    let scaled_size = base_size * scale;
    let center = entry.computed_rect.center();
    let scaled_min = center - scaled_size * 0.5;
    let translated_min = scaled_min + entry.blended_transform.offset * dpi_scale;
    let effective_corner_radius = entry
        .blended_transform
        .radius_override
        .unwrap_or(corner_radius);
    DrawRect {
        position: spin(
            translated_min + scaled_size * 0.5,
            entry.pivot,
            entry.rotation,
        ) - scaled_size * 0.5,
        size: scaled_size,
        color: entry.computed_color,
        corner_radius: effective_corner_radius * dpi_scale,
        border_width: border_width * dpi_scale,
        border_color,
        rotation: entry.rotation,
        clip_rect: entry.clip_rect,
        layer: entry.layer as u8,
        z_index,
        shadow,
        effect_kind: entry.effect_kind,
        effect_params: entry.effect_params,
        quad_corners: None,
    }
}

fn truncate_with_ellipsis(
    font_engine: &mut nightshade_text::contract::data::font_engine::FontEngine,
    text: &str,
    font_size: f32,
    max_width: f32,
) -> String {
    if text.is_empty()
        || crate::data::units::measure_text_width(font_engine, text, font_size) <= max_width
    {
        return text.to_string();
    }
    let ellipsis_width = crate::data::units::measure_text_width(font_engine, "\u{2026}", font_size);
    let budget = max_width - ellipsis_width;
    if budget <= 0.0 {
        return "\u{2026}".to_string();
    }
    let chars: Vec<char> = text.chars().collect();
    let mut fit = chars.len();
    while fit > 0 {
        let prefix: String = chars[..fit].iter().collect();
        if crate::data::units::measure_text_width(font_engine, &prefix, font_size) <= budget {
            break;
        }
        fit -= 1;
    }
    let mut result: String = chars[..fit].iter().collect();
    result.push('\u{2026}');
    result
}

fn emit_text_entry(
    sync: &crate::systems::surface::RenderSync<'_>,
    entry: &RenderEntry,
    z_index: i32,
    dpi_scale: f32,
    text_override: Option<String>,
) -> Option<DrawText> {
    let UiNodeContent::Text {
        text_slot,
        font_size_override,
        outline_color,
        outline_width,
        alignment,
        vertical_alignment,
        overflow,
        monospace_width,
        font_kind,
        ..
    } = entry.content
    else {
        return None;
    };
    let text = match text_override {
        Some(text) => text,
        None => sync
            .text_state
            .cache
            .get_text(text_slot)
            .map(|text| text.to_string())?,
    };
    let actual_font_size =
        font_size_override.unwrap_or(entry.font_size) * entry.font_scale * dpi_scale;

    let wrap_width: Option<f32> = if overflow == crate::components::TextOverflow::Wrap {
        Some(entry.computed_rect.width())
    } else {
        None
    };

    let position_x = match alignment {
        TextAlignment::Left => entry.computed_rect.min.x,
        TextAlignment::Center => entry.computed_rect.center().x,
        TextAlignment::Right => entry.computed_rect.max.x,
    };
    let position_y = match vertical_alignment {
        VerticalAlignment::Top => entry.computed_rect.min.y,
        VerticalAlignment::Middle => entry.computed_rect.center().y,
        VerticalAlignment::Bottom => entry.computed_rect.max.y,
        VerticalAlignment::Baseline => entry.computed_rect.center().y,
    };
    let snapped_position = nalgebra_glm::Vec2::new(position_x.round(), position_y.round());

    let effective_clip = if matches!(
        overflow,
        crate::components::TextOverflow::Clip
            | crate::components::TextOverflow::Wrap
            | crate::components::TextOverflow::Ellipsis
    ) {
        match entry.clip_rect {
            Some(existing) => Some(entry.computed_rect.intersect(&existing).unwrap_or(existing)),
            None => Some(entry.computed_rect),
        }
    } else {
        entry.clip_rect
    };

    let character_colors = sync.text_cache.character_colors.get(&text_slot).cloned();
    let character_background_colors = sync
        .text_cache
        .character_background_colors
        .get(&text_slot)
        .cloned();

    Some(DrawText {
        entity: Some(entry.entity),
        text,
        mesh: TextMesh::default(),
        position: snapped_position,
        color: entry.computed_color,
        outline_color,
        outline_width,
        font_size: actual_font_size,
        alignment,
        vertical_alignment,
        monospace: monospace_width.is_some(),
        font_kind,
        wrap_width,
        clip_rect: effective_clip,
        layer: entry.layer as u8,
        z_index,
        rotation: entry.rotation,
        pivot: entry.pivot,
        character_colors,
        character_background_colors,
    })
}

fn spin(point: nalgebra_glm::Vec2, pivot: nalgebra_glm::Vec2, angle: f32) -> nalgebra_glm::Vec2 {
    if angle == 0.0 {
        return point;
    }
    let turn = angle.cos();
    let lean = angle.sin();
    let offset = point - pivot;
    pivot
        + nalgebra_glm::Vec2::new(
            offset.x * turn - offset.y * lean,
            offset.x * lean + offset.y * turn,
        )
}

fn emit_image_entry(
    entry: &RenderEntry,
    z_index: i32,
    texture_index: u32,
    uv_min: nalgebra_glm::Vec2,
    uv_max: nalgebra_glm::Vec2,
) -> DrawImage {
    let size = entry.computed_rect.size();
    DrawImage {
        entity: Some(entry.entity),
        position: spin(entry.computed_rect.center(), entry.pivot, entry.rotation) - size * 0.5,
        size,
        texture_index,
        uv_min,
        uv_max,
        color: entry.computed_color,
        clip_rect: entry.clip_rect,
        layer: entry.layer as u8,
        z_index,
        rotation: entry.rotation,
        nearest: false,
    }
}

struct RenderEntry {
    entity: nightshade::ecs::Entity,
    rotation: f32,
    pivot: nalgebra_glm::Vec2,
    computed_rect: Rect,
    computed_depth: f32,
    computed_color: Vec4,
    content: UiNodeContent,
    shadow: Option<crate::components::UiNodeShadow>,
    clip_rect: Option<Rect>,
    font_size: f32,
    font_scale: f32,
    layer: UiLayer,
    z_index_override: Option<i32>,
    effect_kind: u32,
    effect_params: [f32; 4],
    blended_transform: crate::components::UiNodeBlendedTransform,
}

pub fn ui_layout_render_sync_system(
    mut sync: crate::systems::surface::RenderSync,
    storage: &mut nightshade::ecs::Storage,
) {
    if !sync.runtime.visible {
        sync.frame.rects.clear();
        sync.frame.rect_entities.clear();
        sync.frame.ui_images.clear();
        sync.frame.text_meshes.clear();
        return;
    }

    let viewport_size = sync
        .window
        .cached_viewport_size
        .map(|(width, height)| nalgebra_glm::Vec2::new(width as f32, height as f32))
        .unwrap_or(nalgebra_glm::Vec2::new(800.0, 600.0));

    let dpi_scale = sync.window.cached_scale_factor;
    let render_dirty = {
        if sync.dirty.render_dirty {
            sync.dirty.rebuilds_owed = 2;
        }
        sync.dirty.render_dirty || sync.dirty.rebuilds_owed > 0
    };

    if render_dirty {
        sync_render_slots(&mut sync, storage);
        sync.frame.rects.clear();
        sync.frame.rect_entities.clear();
        sync.frame.ui_images.clear();
        sync.frame.text_meshes.clear();

        if let Some(bg_color) = sync.runtime.background_color {
            sync.frame.rects.push(DrawRect {
                position: nalgebra_glm::Vec2::new(0.0, 0.0),
                size: viewport_size,
                color: bg_color,
                corner_radius: 0.0,
                border_width: 0.0,
                border_color: Vec4::new(0.0, 0.0, 0.0, 0.0),
                rotation: 0.0,
                clip_rect: None,
                layer: UiLayer::Background as u8,
                z_index: 0,
                shadow: None,
                effect_kind: 0,
                effect_params: [0.0; 4],
                quad_corners: None,
            });
            sync.frame.rect_entities.push(None);
        }

        emit_top_progress_bar(&mut sync, viewport_size, dpi_scale);

        let sorted_nodes = std::mem::take(&mut sync.runtime.z_sorted_nodes);

        let dpi_changed = (dpi_scale - sync.text_cache.mesh_dpi).abs() > f32::EPSILON;
        if dpi_changed {
            sync.text_cache.mesh.clear();
            sync.text_cache.mesh_dpi = dpi_scale;
        }

        let mut render_entries: Vec<RenderEntry> = Vec::new();

        for &(entity, _) in &sorted_nodes {
            let node = match storage.get::<crate::components::UiLayoutNode>(entity) {
                Some(node) => node,
                None => continue,
            };

            let computed_rect = node.computed_rect;

            if computed_rect.max.x < 0.0
                || computed_rect.min.x > viewport_size.x
                || computed_rect.max.y < 0.0
                || computed_rect.min.y > viewport_size.y
            {
                continue;
            }

            let computed_depth = node.computed_depth;
            let node_rotation = node.computed_rotation;
            let node_pivot = node.computed_pivot;
            let font_size = node.font_size.unwrap_or(16.0);
            let font_scale = node.computed_font_scale;
            let node_layer = node.computed_layer.unwrap_or(UiLayer::Background);
            let z_index_override = node.z_index;

            let computed_color = storage
                .get::<crate::components::UiNodeColor>(entity)
                .map(|color| color.computed_color)
                .unwrap_or(Vec4::new(1.0, 1.0, 1.0, 1.0));

            let content = match storage.get::<crate::components::UiNodeContent>(entity) {
                Some(content) => *content,
                None => continue,
            };

            let shadow = storage
                .get::<crate::components::UiNodeShadow>(entity)
                .copied();

            let (effect_kind, effect_params) = storage
                .get::<crate::components::UiNodeEffect>(entity)
                .map(|effect| (effect.kind, effect.params))
                .unwrap_or((0, [0.0; 4]));

            let clip_rect = node.computed_clip_rect;

            let blended_transform = storage
                .get::<crate::components::UiNodeBlendedTransform>(entity)
                .copied()
                .unwrap_or_default();

            render_entries.push(RenderEntry {
                entity,
                rotation: node_rotation,
                pivot: node_pivot,
                computed_rect,
                computed_depth,
                computed_color,
                content,
                shadow,
                clip_rect,
                font_size,
                font_scale,
                layer: node_layer,
                z_index_override,
                effect_kind,
                effect_params,
                blended_transform,
            });
        }

        let mut rects = std::mem::take(&mut sync.frame.rects);
        let mut rect_entities = std::mem::take(&mut sync.frame.rect_entities);
        let mut ui_images = std::mem::take(&mut sync.frame.ui_images);
        let mut text_meshes = Vec::new();

        for entry in &render_entries {
            let z_index = entry
                .z_index_override
                .unwrap_or((entry.computed_depth * 100.0) as i32);

            match &entry.content {
                UiNodeContent::None => {}
                UiNodeContent::Rect {
                    corner_radius,
                    border_width,
                    border_color,
                } => {
                    rects.push(emit_rect_entry(
                        entry,
                        z_index,
                        dpi_scale,
                        *corner_radius,
                        *border_width,
                        *border_color,
                    ));
                    rect_entities.push(Some(entry.entity));
                }
                UiNodeContent::Text {
                    text_slot,
                    font_size_override,
                    overflow,
                    ..
                } => {
                    let text_override = if *overflow == crate::components::TextOverflow::Ellipsis {
                        let font_size = font_size_override.unwrap_or(entry.font_size)
                            * entry.font_scale
                            * dpi_scale;
                        sync.text_state
                            .cache
                            .get_text(*text_slot)
                            .map(|text| text.to_string())
                            .map(|text| {
                                truncate_with_ellipsis(
                                    &mut sync.text_state.font_engine,
                                    &text,
                                    font_size,
                                    entry.computed_rect.width(),
                                )
                            })
                    } else {
                        None
                    };
                    if let Some(instance) =
                        emit_text_entry(&sync, entry, z_index, dpi_scale, text_override)
                    {
                        text_meshes.push(instance);
                    }
                }
                UiNodeContent::Image {
                    texture_index,
                    uv_min,
                    uv_max,
                } => {
                    ui_images.push(emit_image_entry(
                        entry,
                        z_index,
                        *texture_index,
                        *uv_min,
                        *uv_max,
                    ));
                }
            }
        }

        sync.runtime.z_sorted_nodes = sorted_nodes;
        sync.frame.rects = rects;
        sync.frame.rect_entities = rect_entities;
        sync.frame.ui_images = ui_images;
        sync.frame.text_meshes = text_meshes;

        let rect_count = sync.frame.rects.len();
        let image_count = sync.frame.ui_images.len();
        let text_count = sync.frame.text_meshes.len();
        sync.dirty.cached_node_rect_count = rect_count;
        sync.dirty.cached_node_image_count = image_count;
        sync.dirty.cached_node_text_count = text_count;

        sync.dirty.render_dirty = false;
        sync.dirty.rebuilds_owed = sync.dirty.rebuilds_owed.saturating_sub(1);
    } else {
        let rect_count = sync.dirty.cached_node_rect_count;
        let image_count = sync.dirty.cached_node_image_count;
        let text_count = sync.dirty.cached_node_text_count;
        sync.frame.rects.truncate(rect_count);
        sync.frame.ui_images.truncate(image_count);
        sync.frame.text_meshes.truncate(text_count);
    }

    emit_canvas_commands(&mut sync, storage, dpi_scale);

    if let Some(tooltip) = &sync.overlays.tooltip_state {
        let delay = 0.5;
        if sync.timing.current_time - tooltip.hover_start >= delay {
            let mouse_position = &sync.input.mouse.position;
            let target_viewport_size = viewport_size;
            let tooltip_entity = tooltip.tooltip_entity;
            let tooltip_text = tooltip.text.clone();

            if let Some(tooltip_entity) = tooltip_entity {
                let mut tooltip_x = mouse_position.x + 12.0;
                let mut tooltip_y = mouse_position.y + 16.0;
                if let Some(node) = storage.get::<crate::components::UiLayoutNode>(tooltip_entity) {
                    let tooltip_w = node.computed_rect.width();
                    let tooltip_h = node.computed_rect.height();
                    if tooltip_x + tooltip_w > target_viewport_size.x {
                        tooltip_x = target_viewport_size.x - tooltip_w;
                    }
                    if tooltip_y + tooltip_h > target_viewport_size.y {
                        tooltip_y = mouse_position.y - tooltip_h - 4.0;
                    }
                }
                if let Some(node) =
                    storage.get_mut::<crate::components::UiLayoutNode>(tooltip_entity)
                {
                    node.visible = true;
                    if let Some(crate::components::layout_types::UiLayoutType::Window(window)) =
                        node.base_layout.as_mut()
                    {
                        window.position =
                            crate::data::units::Ab(nalgebra_glm::Vec2::new(tooltip_x, tooltip_y))
                                .into();
                    }
                }
            } else {
                let theme = sync.theme.active_theme();
                let tooltip_bg = Vec4::new(
                    theme.panel_color.x,
                    theme.panel_color.y,
                    theme.panel_color.z,
                    0.95,
                );
                let tooltip_border = theme.border_color;
                let tooltip_text_color = theme.text_color;
                let tooltip_corner = theme.corner_radius;

                let font_size = 13.0 * dpi_scale;
                let padding = 8.0 * dpi_scale;
                let max_tooltip_width = 300.0 * dpi_scale;
                let max_text_width = max_tooltip_width - padding * 2.0;

                let measured_width = crate::data::units::measure_text_width(
                    &mut sync.text_state.font_engine,
                    &tooltip_text,
                    font_size,
                );
                let needs_wrap = measured_width > max_text_width;
                let line_count = if needs_wrap {
                    let wrapped = nightshade_text::contract::data::text_wrapping::wrap_text(
                        &mut sync.text_state.font_engine,
                        &tooltip_text,
                        font_size,
                        max_text_width,
                        nightshade_text::contract::data::text_data::FontKind::Default,
                    );
                    wrapped.lines().count().max(1)
                } else {
                    1
                };
                let text_width = if needs_wrap {
                    max_text_width
                } else {
                    measured_width
                };
                let wrap_width = if needs_wrap {
                    Some(max_text_width)
                } else {
                    None
                };

                let line_height = font_size * 1.3;
                let tooltip_height = if line_count > 1 {
                    line_height * line_count as f32 + padding * 2.0
                } else {
                    28.0 * dpi_scale
                };
                let tooltip_width = text_width + padding * 2.0;

                let mut tooltip_x = mouse_position.x + 12.0;
                let mut tooltip_y = mouse_position.y + 16.0;

                if tooltip_x + tooltip_width > target_viewport_size.x {
                    tooltip_x = target_viewport_size.x - tooltip_width;
                }
                if tooltip_y + tooltip_height > target_viewport_size.y {
                    tooltip_y = mouse_position.y - tooltip_height - 4.0;
                }

                let tooltip_rect = DrawRect {
                    position: nalgebra_glm::Vec2::new(tooltip_x, tooltip_y),
                    size: nalgebra_glm::Vec2::new(tooltip_width, tooltip_height),
                    color: tooltip_bg,
                    corner_radius: tooltip_corner * dpi_scale,
                    border_width: 1.0 * dpi_scale,
                    border_color: tooltip_border,
                    rotation: 0.0,
                    clip_rect: None,
                    layer: UiLayer::Tooltips as u8,
                    z_index: 0,
                    shadow: None,
                    effect_kind: 0,
                    effect_params: [0.0; 4],
                    quad_corners: None,
                };
                let tooltip_text_instance = DrawText {
                    entity: None,
                    text: tooltip_text,
                    mesh: TextMesh::default(),
                    position: nalgebra_glm::Vec2::new(
                        tooltip_x + tooltip_width * 0.5,
                        tooltip_y + tooltip_height * 0.5,
                    ),
                    color: tooltip_text_color,
                    outline_color: Vec4::new(0.0, 0.0, 0.0, 0.0),
                    outline_width: 0.0,
                    font_size,
                    alignment: TextAlignment::Center,
                    vertical_alignment: VerticalAlignment::Middle,
                    monospace: false,
                    font_kind: nightshade_text::contract::data::text_data::FontKind::Default,
                    wrap_width,
                    clip_rect: None,
                    layer: UiLayer::Tooltips as u8,
                    rotation: 0.0,
                    pivot: nalgebra_glm::Vec2::new(0.0, 0.0),
                    character_colors: None,
                    character_background_colors: None,
                    z_index: 1,
                };

                sync.frame.rects.push(tooltip_rect);
                sync.frame.text_meshes.push(tooltip_text_instance);
            }
        }
    }

    let focused = if sync.overlays.focus_ring_visible {
        sync.interaction.focused_entity
    } else {
        None
    };
    if let Some(focused) = focused
        && crate::commands::widgets::world_layout::ui_chain_visible(
            &mut |entity| {
                storage
                    .get::<crate::components::UiLayoutNode>(entity)
                    .map(|node| node.visible)
            },
            &mut |entity| {
                storage
                    .get::<nightshade::ecs::ChildOf>(entity)
                    .map(|child_of| child_of.0)
            },
            focused,
        )
        && let Some(node) = storage.get::<crate::components::UiLayoutNode>(focused)
    {
        let rect = node.computed_rect;
        let accent_color = sync.theme.active_theme().accent_color;
        let inset = 3.0 * dpi_scale;
        let halo_inset = 7.0 * dpi_scale;
        let ring_color = Vec4::new(accent_color.x, accent_color.y, accent_color.z, 1.0);
        let halo_color = Vec4::new(accent_color.x, accent_color.y, accent_color.z, 0.35);

        sync.frame.rects.push(DrawRect {
            position: nalgebra_glm::Vec2::new(rect.min.x - halo_inset, rect.min.y - halo_inset),
            size: nalgebra_glm::Vec2::new(
                rect.width() + halo_inset * 2.0,
                rect.height() + halo_inset * 2.0,
            ),
            color: Vec4::new(0.0, 0.0, 0.0, 0.0),
            corner_radius: 10.0 * dpi_scale,
            border_width: 2.0 * dpi_scale,
            border_color: halo_color,
            rotation: 0.0,
            clip_rect: None,
            layer: UiLayer::Tooltips as u8,
            z_index: 0,
            shadow: None,
            effect_kind: 0,
            effect_params: [0.0; 4],
            quad_corners: None,
        });

        sync.frame.rects.push(DrawRect {
            position: nalgebra_glm::Vec2::new(rect.min.x - inset, rect.min.y - inset),
            size: nalgebra_glm::Vec2::new(rect.width() + inset * 2.0, rect.height() + inset * 2.0),
            color: Vec4::new(0.0, 0.0, 0.0, 0.0),
            corner_radius: 6.0 * dpi_scale,
            border_width: 3.0 * dpi_scale,
            border_color: ring_color,
            rotation: 0.0,
            clip_rect: None,
            layer: UiLayer::Tooltips as u8,
            z_index: 0,
            shadow: None,
            effect_kind: 0,
            effect_params: [0.0; 4],
            quad_corners: None,
        });
    }

    if sync.overlays.dock_indicator_active {
        let theme = sync.theme.active_theme();
        let theme_panel_color = theme.panel_color;
        let theme_accent = theme.accent_color;
        let theme_bg_hovered = theme.background_color_hovered;
        let theme_border = theme.border_color;
        let theme_text_color = theme.text_color;

        let indicator_bg = Vec4::new(
            theme_panel_color.x,
            theme_panel_color.y,
            theme_panel_color.z,
            0.95,
        );

        let indicator_size = 40.0;
        let indicator_spacing = 4.0;
        let center = viewport_size * 0.5;
        let total_size = indicator_size * 3.0 + indicator_spacing * 2.0;

        let background_rect_position =
            center - nalgebra_glm::Vec2::new(total_size + 16.0, total_size + 16.0) * 0.5;
        sync.frame.rects.push(DrawRect {
            position: background_rect_position,
            size: nalgebra_glm::Vec2::new(total_size + 16.0, total_size + 16.0),
            color: indicator_bg,
            corner_radius: 8.0,
            border_width: 1.0,
            border_color: theme_border,
            rotation: 0.0,
            clip_rect: None,
            layer: UiLayer::DockIndicator as u8,
            z_index: 0,
            shadow: None,
            effect_kind: 0,
            effect_params: [0.0; 4],
            quad_corners: None,
        });

        let mouse_position = &sync.input.mouse.position;
        let buttons = [
            (
                nalgebra_glm::Vec2::new(center.x, center.y - indicator_size - indicator_spacing),
                "T",
            ),
            (
                nalgebra_glm::Vec2::new(center.x, center.y + indicator_size + indicator_spacing),
                "B",
            ),
            (
                nalgebra_glm::Vec2::new(center.x - indicator_size - indicator_spacing, center.y),
                "L",
            ),
            (
                nalgebra_glm::Vec2::new(center.x + indicator_size + indicator_spacing, center.y),
                "R",
            ),
            (center, "+"),
        ];

        let mut hovered_index: Option<usize> = None;
        for (index, (btn_center, symbol)) in buttons.iter().enumerate() {
            let half = indicator_size * 0.5;
            let btn_min = *btn_center - nalgebra_glm::Vec2::new(half, half);
            let btn_max = *btn_center + nalgebra_glm::Vec2::new(half, half);
            let is_hovered = mouse_position.x >= btn_min.x
                && mouse_position.x <= btn_max.x
                && mouse_position.y >= btn_min.y
                && mouse_position.y <= btn_max.y;

            if is_hovered {
                hovered_index = Some(index);
            }

            let btn_color = if is_hovered {
                theme_accent
            } else {
                theme_bg_hovered
            };

            sync.frame.rects.push(DrawRect {
                position: btn_min,
                size: nalgebra_glm::Vec2::new(indicator_size, indicator_size),
                color: btn_color,
                corner_radius: 4.0,
                border_width: 1.0,
                border_color: theme_border,
                rotation: 0.0,
                clip_rect: None,
                layer: UiLayer::DockIndicator as u8,
                z_index: 1,
                shadow: None,
                effect_kind: 0,
                effect_params: [0.0; 4],
                quad_corners: None,
            });

            let symbol_font_size = indicator_size * 0.4 * dpi_scale;
            let text_position =
                nalgebra_glm::Vec2::new(btn_center.x, btn_center.y + symbol_font_size * 0.15);
            sync.frame.text_meshes.push(DrawText {
                entity: None,
                text: symbol.to_string(),
                mesh: TextMesh::default(),
                position: text_position,
                color: theme_text_color,
                outline_color: Vec4::new(0.0, 0.0, 0.0, 0.0),
                outline_width: 0.0,
                font_size: symbol_font_size,
                alignment: TextAlignment::Center,
                vertical_alignment: VerticalAlignment::Middle,
                monospace: false,
                font_kind: nightshade_text::contract::data::text_data::FontKind::Default,
                wrap_width: None,
                clip_rect: None,
                layer: UiLayer::DockIndicator as u8,
                rotation: 0.0,
                pivot: nalgebra_glm::Vec2::new(0.0, 0.0),
                character_colors: None,
                character_background_colors: None,
                z_index: 2,
            });
        }

        let edge_proximity_index = if hovered_index.is_none() {
            let edge_threshold = 80.0;
            if mouse_position.y < edge_threshold {
                Some(0_usize)
            } else if mouse_position.y > viewport_size.y - edge_threshold {
                Some(1_usize)
            } else if mouse_position.x < edge_threshold {
                Some(2_usize)
            } else if mouse_position.x > viewport_size.x - edge_threshold {
                Some(3_usize)
            } else {
                None
            }
        } else {
            None
        };
        let preview_index = hovered_index.or(edge_proximity_index);

        if let Some(index) = preview_index
            && index < 4
        {
            let reserved = sync.dock.primary.reserved_areas.clone();
            let reserved = &reserved;
            let dock_size = 300.0;
            let (preview_position, preview_size) = match index {
                0 => {
                    let width = viewport_size.x - reserved.left - reserved.right;
                    (
                        nalgebra_glm::Vec2::new(reserved.left, reserved.top),
                        nalgebra_glm::Vec2::new(width, dock_size),
                    )
                }
                1 => {
                    let width = viewport_size.x - reserved.left - reserved.right;
                    (
                        nalgebra_glm::Vec2::new(
                            reserved.left,
                            viewport_size.y - reserved.bottom - dock_size,
                        ),
                        nalgebra_glm::Vec2::new(width, dock_size),
                    )
                }
                2 => {
                    let height = viewport_size.y - reserved.top - reserved.bottom;
                    (
                        nalgebra_glm::Vec2::new(reserved.left, reserved.top),
                        nalgebra_glm::Vec2::new(dock_size, height),
                    )
                }
                3 => {
                    let height = viewport_size.y - reserved.top - reserved.bottom;
                    (
                        nalgebra_glm::Vec2::new(
                            viewport_size.x - reserved.right - dock_size,
                            reserved.top,
                        ),
                        nalgebra_glm::Vec2::new(dock_size, height),
                    )
                }
                _ => (
                    nalgebra_glm::Vec2::new(0.0, 0.0),
                    nalgebra_glm::Vec2::new(0.0, 0.0),
                ),
            };

            let preview_alpha = if hovered_index.is_some() { 0.3 } else { 0.18 };
            let preview_border_alpha = if hovered_index.is_some() { 0.8 } else { 0.5 };
            let preview_fill = Vec4::new(
                theme_accent.x,
                theme_accent.y,
                theme_accent.z,
                preview_alpha,
            );
            sync.frame.rects.push(DrawRect {
                position: preview_position,
                size: preview_size,
                color: preview_fill,
                corner_radius: 0.0,
                border_width: 2.0,
                border_color: Vec4::new(
                    theme_accent.x,
                    theme_accent.y,
                    theme_accent.z,
                    preview_border_alpha,
                ),
                rotation: 0.0,
                clip_rect: None,
                layer: UiLayer::DockIndicator as u8,
                z_index: 0,
                shadow: None,
                effect_kind: 0,
                effect_params: [0.0; 4],
                quad_corners: None,
            });
        }
    }

    emit_tile_containers(&mut sync, storage, dpi_scale);

    flush_overlays(&mut sync);
}

pub fn reclaim_frame(
    mut frame: nightshade::ecs::system_param::ResMut<crate::resources::RetainedUiFrame>,
    mut paint: nightshade::ecs::system_param::ResMut<nightshade_paint::resources::PaintFrame>,
) {
    if !frame.handed_off {
        return;
    }
    let rects = std::mem::take(&mut paint.rects);
    let rect_entities = std::mem::take(&mut paint.rect_entities);
    let images = std::mem::take(&mut paint.images);
    let text_meshes = std::mem::take(&mut paint.text_meshes);

    frame.rects = rects;
    frame.rect_entities = rect_entities;
    frame.ui_images = images;
    frame.text_meshes = text_meshes;
    frame.handed_off = false;
}

pub fn publish_frame(
    mut sync: crate::systems::surface::RenderSync,
    mut paint: nightshade::ecs::system_param::ResMut<nightshade_paint::resources::PaintFrame>,
    mut render_settings: nightshade::ecs::system_param::ResMut<
        nightshade_render::data::config::RenderSettings,
    >,
) {
    flush_overlays(&mut sync);

    let background_color = sync.runtime.background_color;
    render_settings.swapchain_clear_color = background_color.map(|color| color.into());
    let image_uploads = std::mem::take(&mut sync.runtime.pending_image_uploads);

    let rects = std::mem::take(&mut sync.frame.rects);
    let rect_entities = std::mem::take(&mut sync.frame.rect_entities);
    let images = std::mem::take(&mut sync.frame.ui_images);
    let text_meshes = std::mem::take(&mut sync.frame.text_meshes);
    sync.frame.handed_off = true;

    paint.rects = rects;
    paint.rect_entities = rect_entities;
    paint.images = images;
    paint.text_meshes = text_meshes;
    paint.image_uploads = image_uploads;
}

pub fn flush_overlays(sync: &mut crate::systems::surface::RenderSync<'_>) {
    let overlay_rects: Vec<_> = std::mem::take(&mut sync.frame.overlay_rects);
    sync.frame.rects.extend(overlay_rects);

    let overlay_text: Vec<_> = std::mem::take(&mut sync.frame.overlay_text);
    for entry in overlay_text {
        sync.frame.text_meshes.push(DrawText {
            entity: None,
            text: entry.text,
            mesh: TextMesh::default(),
            position: entry.position,
            color: entry.properties.color,
            outline_color: entry.properties.outline_color,
            outline_width: entry.properties.outline_width,
            font_size: entry.properties.font_size,
            alignment: entry.properties.alignment,
            vertical_alignment: entry.properties.vertical_alignment,
            monospace: entry.properties.monospace_width.is_some(),
            font_kind: entry.properties.font_kind,
            wrap_width: None,
            clip_rect: entry.clip_rect,
            layer: entry.layer as u8,
            z_index: entry.z_index,
            rotation: 0.0,
            pivot: nalgebra_glm::Vec2::new(0.0, 0.0),
            character_colors: None,
            character_background_colors: None,
        });
    }
}

fn sync_render_slots(
    sync: &mut crate::systems::surface::RenderSync<'_>,
    storage: &nightshade::ecs::Storage,
) {
    let sorted_nodes = sync.runtime.z_sorted_nodes.clone();
    let mut current_rect_entities: std::collections::HashSet<nightshade::ecs::Entity> =
        std::collections::HashSet::new();
    let mut current_image_entities: std::collections::HashSet<nightshade::ecs::Entity> =
        std::collections::HashSet::new();
    let mut current_text_entities: std::collections::HashSet<nightshade::ecs::Entity> =
        std::collections::HashSet::new();

    for &(entity, _) in &sorted_nodes {
        if let Some(content) = storage.get::<crate::components::UiNodeContent>(entity) {
            match content {
                UiNodeContent::Rect { .. } => {
                    current_rect_entities.insert(entity);
                    crate::commands::slots::allocate_rect_slot(sync.slots, entity);
                }
                UiNodeContent::Image { .. } => {
                    current_image_entities.insert(entity);
                    crate::commands::slots::allocate_image_slot(sync.slots, entity);
                }
                UiNodeContent::Text { .. } => {
                    current_text_entities.insert(entity);
                    crate::commands::slots::allocate_text_slot(sync.slots, entity);
                }
                UiNodeContent::None => {}
            }
        }
    }

    let stale_rects: Vec<nightshade::ecs::Entity> = sync
        .slots
        .rect_slots
        .keys()
        .copied()
        .filter(|entity| !current_rect_entities.contains(entity))
        .collect();
    for entity in stale_rects {
        crate::commands::slots::release_rect_slot(sync.slots, entity);
    }

    let stale_images: Vec<nightshade::ecs::Entity> = sync
        .slots
        .image_slots
        .keys()
        .copied()
        .filter(|entity| !current_image_entities.contains(entity))
        .collect();
    for entity in stale_images {
        crate::commands::slots::release_image_slot(sync.slots, entity);
    }

    let stale_text: Vec<nightshade::ecs::Entity> = sync
        .slots
        .text_slots
        .keys()
        .copied()
        .filter(|entity| !current_text_entities.contains(entity))
        .collect();
    for entity in stale_text {
        crate::commands::slots::release_text_slot(sync.slots, entity);
    }
}
