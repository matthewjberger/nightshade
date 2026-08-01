use nalgebra_glm::Vec4;

use crate::components::layer::UiLayer;
use crate::components::{DropZone, SplitDirection, TileId, TileNode};
use nightshade_paint::data::DrawRect;
use nightshade_paint::data::DrawText;
use nightshade_paint::data::Rect;
use nightshade_text::contract::data::text_data::{TextAlignment, TextMesh, VerticalAlignment};

pub(super) fn emit_tile_containers(
    sync: &mut crate::systems::surface::RenderSync<'_>,
    storage: &mut nightshade::ecs::Storage,
    dpi_scale: f32,
) {
    let tile_entities: Vec<nightshade::ecs::Entity> = storage
        .query_entities::<(crate::components::UiTileContainerData,)>()
        .collect();

    struct TileEmit {
        rects: Vec<DrawRect>,
        texts: Vec<(String, nalgebra_glm::Vec2, f32, Vec4)>,
        overlay_texts: Vec<(String, nalgebra_glm::Vec2, f32, Vec4)>,
        container_depth: f32,
        clip_rect: Option<Rect>,
        layer: UiLayer,
    }

    let mut emits = Vec::new();

    for entity in tile_entities {
        if !crate::commands::widgets::world_layout::ui_chain_visible(
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
            entity,
        ) {
            continue;
        }
        let (data, depth, clip_rect, layer) = {
            let node = match storage.get::<crate::components::UiLayoutNode>(entity) {
                Some(node)
                    if node.visible
                        && (node.computed_rect.width() > 0.0
                            || node.computed_rect.height() > 0.0) =>
                {
                    node
                }
                _ => continue,
            };

            let depth = node.computed_depth;
            let clip = node.computed_clip_rect;
            let node_layer = node.computed_layer.unwrap_or(UiLayer::Background);

            let data = match storage.get::<crate::components::UiTileContainerData>(entity) {
                Some(data) => data.clone(),
                None => continue,
            };

            (data, depth, clip, node_layer)
        };

        let theme = sync.theme.active_theme();
        let border_color = theme.border_color;
        let panel_header_color = theme.panel_header_color;
        let accent_color = theme.accent_color;
        let panel_color = theme.panel_color;
        let text_color = theme.text_color;
        let font_size = theme.font_size;

        let z_index = (depth * 100.0) as i32 + 150;
        let mut rects = Vec::new();
        let mut texts = Vec::new();
        let mut overlay_texts = Vec::new();

        let total_panes = data
            .tiles
            .iter()
            .filter(|tile| matches!(tile, Some(TileNode::Pane { .. })))
            .count();

        for (index, tile) in data.tiles.iter().enumerate() {
            if let Some(TileNode::Split {
                direction,
                children,
                ..
            }) = tile
            {
                let child0_rect = match data.rects.get(children[0].0) {
                    Some(r) => *r,
                    None => continue,
                };

                let (splitter_position, splitter_size) = match direction {
                    SplitDirection::Horizontal => (
                        nalgebra_glm::Vec2::new(child0_rect.max.x, child0_rect.min.y),
                        nalgebra_glm::Vec2::new(data.splitter_width, child0_rect.height()),
                    ),
                    SplitDirection::Vertical => (
                        nalgebra_glm::Vec2::new(child0_rect.min.x, child0_rect.max.y),
                        nalgebra_glm::Vec2::new(child0_rect.width(), data.splitter_width),
                    ),
                };

                rects.push(DrawRect {
                    position: splitter_position,
                    size: splitter_size,
                    color: border_color,
                    corner_radius: 0.0,
                    border_width: 0.0,
                    border_color: Vec4::new(0.0, 0.0, 0.0, 0.0),
                    rotation: 0.0,
                    clip_rect,
                    layer: layer as u8,
                    z_index,
                    shadow: None,
                    effect_kind: 0,
                    effect_params: [0.0; 4],
                    quad_corners: None,
                });
            }

            if let Some(TileNode::Tabs { panes, active }) = tile {
                let show_close = total_panes >= 2;

                let tabs_rect = match data.rects.get(index) {
                    Some(r) => *r,
                    None => continue,
                };

                let tab_bar_rect = Rect {
                    min: tabs_rect.min,
                    max: nalgebra_glm::Vec2::new(
                        tabs_rect.max.x,
                        tabs_rect.min.y + data.tab_bar_height,
                    ),
                };

                rects.push(DrawRect {
                    position: tab_bar_rect.min,
                    size: tab_bar_rect.size(),
                    color: panel_header_color,
                    corner_radius: 0.0,
                    border_width: 0.0,
                    border_color: Vec4::new(0.0, 0.0, 0.0, 0.0),
                    rotation: 0.0,
                    clip_rect,
                    layer: layer as u8,
                    z_index,
                    shadow: None,
                    effect_kind: 0,
                    effect_params: [0.0; 4],
                    quad_corners: None,
                });

                if !panes.is_empty() {
                    let tab_width = tab_bar_rect.width() / panes.len() as f32;
                    for (tab_index, &pane_id) in panes.iter().enumerate() {
                        let tab_x = tab_bar_rect.min.x + tab_index as f32 * tab_width;
                        let is_active = tab_index == *active;

                        let tab_bg = if is_active { accent_color } else { panel_color };

                        rects.push(DrawRect {
                            position: nalgebra_glm::Vec2::new(
                                tab_x + 1.0,
                                tab_bar_rect.min.y + 1.0,
                            ),
                            size: nalgebra_glm::Vec2::new(
                                tab_width - 2.0,
                                data.tab_bar_height - 2.0,
                            ),
                            color: tab_bg,
                            corner_radius: 3.0 * dpi_scale,
                            border_width: 0.0,
                            border_color: Vec4::new(0.0, 0.0, 0.0, 0.0),
                            rotation: 0.0,
                            clip_rect,
                            layer: layer as u8,
                            z_index: z_index + 1,
                            shadow: None,
                            effect_kind: 0,
                            effect_params: [0.0; 4],
                            quad_corners: None,
                        });

                        if let Some(TileNode::Pane { title, .. }) = data.get(pane_id) {
                            let close_w = data.tab_close_width();
                            let title_offset = if show_close { close_w * 0.5 } else { 0.0 };
                            let text_x = tab_x + (tab_width - title_offset) * 0.5;
                            let text_y = tab_bar_rect.min.y + data.tab_bar_height * 0.5;
                            texts.push((
                                title.clone(),
                                nalgebra_glm::Vec2::new(text_x, text_y),
                                font_size * 0.8,
                                text_color,
                            ));

                            if show_close {
                                let close_x = tab_x + tab_width - close_w * 0.5 - 1.0;
                                let close_y = tab_bar_rect.min.y + data.tab_bar_height * 0.5;
                                let is_hovered =
                                    if let Some((hover_tabs, hover_index)) = data.hovered_close {
                                        hover_tabs == TileId(index) && hover_index == tab_index
                                    } else {
                                        false
                                    };
                                let alpha = if is_hovered {
                                    text_color.w
                                } else {
                                    text_color.w * 0.5
                                };
                                let close_color =
                                    Vec4::new(text_color.x, text_color.y, text_color.z, alpha);
                                texts.push((
                                    "\u{00D7}".to_string(),
                                    nalgebra_glm::Vec2::new(close_x, close_y),
                                    font_size * 0.75,
                                    close_color,
                                ));
                            }
                        }
                    }
                }

                rects.push(DrawRect {
                    position: nalgebra_glm::Vec2::new(
                        tabs_rect.min.x,
                        tabs_rect.min.y + data.tab_bar_height - 1.0,
                    ),
                    size: nalgebra_glm::Vec2::new(tabs_rect.width(), 1.0),
                    color: border_color,
                    corner_radius: 0.0,
                    border_width: 0.0,
                    border_color: Vec4::new(0.0, 0.0, 0.0, 0.0),
                    rotation: 0.0,
                    clip_rect,
                    layer: layer as u8,
                    z_index: z_index + 2,
                    shadow: None,
                    effect_kind: 0,
                    effect_params: [0.0; 4],
                    quad_corners: None,
                });
            }
        }

        if let Some(ref preview) = data.drop_preview {
            let target_rect = data.rects.get(preview.target_tile.0).copied();
            if let Some(target_rect) = target_rect {
                let (preview_position, preview_size) = match preview.zone {
                    DropZone::Left => (
                        target_rect.min,
                        nalgebra_glm::Vec2::new(target_rect.width() * 0.5, target_rect.height()),
                    ),
                    DropZone::Right => (
                        nalgebra_glm::Vec2::new(
                            target_rect.min.x + target_rect.width() * 0.5,
                            target_rect.min.y,
                        ),
                        nalgebra_glm::Vec2::new(target_rect.width() * 0.5, target_rect.height()),
                    ),
                    DropZone::Top => (
                        target_rect.min,
                        nalgebra_glm::Vec2::new(target_rect.width(), target_rect.height() * 0.5),
                    ),
                    DropZone::Bottom => (
                        nalgebra_glm::Vec2::new(
                            target_rect.min.x,
                            target_rect.min.y + target_rect.height() * 0.5,
                        ),
                        nalgebra_glm::Vec2::new(target_rect.width(), target_rect.height() * 0.5),
                    ),
                    DropZone::Center => (target_rect.min, target_rect.size()),
                };

                let preview_color = Vec4::new(accent_color.x, accent_color.y, accent_color.z, 0.2);
                rects.push(DrawRect {
                    position: preview_position,
                    size: preview_size,
                    color: preview_color,
                    corner_radius: 0.0,
                    border_width: 2.0 * dpi_scale,
                    border_color: Vec4::new(accent_color.x, accent_color.y, accent_color.z, 0.6),
                    rotation: 0.0,
                    clip_rect: None,
                    layer: UiLayer::DockIndicator as u8,
                    z_index: z_index + 10,
                    shadow: None,
                    effect_kind: 0,
                    effect_params: [0.0; 4],
                    quad_corners: None,
                });
            }
        }

        if let Some((source_tabs_id, tab_index, _)) = data.dragging_tab {
            let mouse_position = &sync.input.mouse.position;
            if let Some(TileNode::Tabs { panes, .. }) = data.get(source_tabs_id)
                && tab_index < panes.len()
            {
                let pane_id = panes[tab_index];
                if let Some(TileNode::Pane { title, .. }) = data.get(pane_id) {
                    let ghost_width = 120.0;
                    let ghost_height = data.tab_bar_height;

                    rects.push(DrawRect {
                        position: nalgebra_glm::Vec2::new(
                            mouse_position.x - ghost_width * 0.5,
                            mouse_position.y - ghost_height * 0.5,
                        ),
                        size: nalgebra_glm::Vec2::new(ghost_width, ghost_height),
                        color: Vec4::new(accent_color.x, accent_color.y, accent_color.z, 0.8),
                        corner_radius: 4.0 * dpi_scale,
                        border_width: 1.0 * dpi_scale,
                        border_color,
                        rotation: 0.0,
                        clip_rect: None,
                        layer: UiLayer::DockIndicator as u8,
                        z_index: z_index + 20,
                        shadow: None,
                        effect_kind: 0,
                        effect_params: [0.0; 4],
                        quad_corners: None,
                    });

                    overlay_texts.push((
                        title.clone(),
                        nalgebra_glm::Vec2::new(mouse_position.x, mouse_position.y),
                        font_size * 0.8,
                        text_color,
                    ));
                }
            }
        }

        emits.push(TileEmit {
            rects,
            texts,
            overlay_texts,
            container_depth: depth,
            clip_rect,
            layer,
        });
    }

    for emit in emits {
        let base_z = (emit.container_depth * 100.0) as i32;
        let (target_rects, target_text_meshes) = target_buffers(sync.frame);
        target_rects.extend(emit.rects);
        for (text, position, size, color) in &emit.texts {
            let physical_size = *size * dpi_scale;
            target_text_meshes.push(DrawText {
                entity: None,
                text: text.clone(),
                mesh: TextMesh::default(),
                position: *position,
                color: *color,
                outline_color: Vec4::new(0.0, 0.0, 0.0, 0.0),
                outline_width: 0.0,
                font_size: physical_size,
                alignment: TextAlignment::Center,
                vertical_alignment: VerticalAlignment::Middle,
                monospace: false,
                font_kind: nightshade_text::contract::data::text_data::FontKind::Default,
                wrap_width: None,
                clip_rect: emit.clip_rect,
                layer: emit.layer as u8,
                z_index: base_z + 155,
                rotation: 0.0,
                pivot: nalgebra_glm::Vec2::new(0.0, 0.0),
                character_colors: None,
                character_background_colors: None,
            });
        }
        for (text, position, size, color) in &emit.overlay_texts {
            let physical_size = *size * dpi_scale;
            target_text_meshes.push(DrawText {
                entity: None,
                text: text.clone(),
                mesh: TextMesh::default(),
                position: *position,
                color: *color,
                outline_color: Vec4::new(0.0, 0.0, 0.0, 0.0),
                outline_width: 0.0,
                font_size: physical_size,
                alignment: TextAlignment::Center,
                vertical_alignment: VerticalAlignment::Middle,
                monospace: false,
                font_kind: nightshade_text::contract::data::text_data::FontKind::Default,
                wrap_width: None,
                clip_rect: None,
                layer: UiLayer::DockIndicator as u8,
                z_index: base_z + 175,
                rotation: 0.0,
                pivot: nalgebra_glm::Vec2::new(0.0, 0.0),
                character_colors: None,
                character_background_colors: None,
            });
        }
    }
}

fn target_buffers(
    frame: &mut crate::resources::RetainedUiFrame,
) -> (&mut Vec<DrawRect>, &mut Vec<DrawText>) {
    (&mut frame.rects, &mut frame.text_meshes)
}
