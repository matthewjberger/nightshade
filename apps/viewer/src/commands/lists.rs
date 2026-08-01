use std::collections::HashMap;

use nightshade::prelude::*;
use nightshade_platform::winit;
use nightshade_ui::commands::builder::UiBuild;
use nightshade_ui::components::{UiTreeNodeData, UiTreeViewData};
use nightshade_ui::prelude::*;

use crate::commands::interface::button;
use crate::components::Deleted;
use crate::data::BrowserItem;
use crate::queries::browsers::{empty_text, page_entries};
use crate::queries::interface::icon_of;
use crate::queries::layout::{layer_row_width, layer_weight_width};
use crate::queries::library::display_name;
use crate::queries::library::roots;
use crate::resources::{BrowserCell, LayerRow, MorphInfo, MorphSlider, TreeRow, ViewerResources};
use crate::theme::*;

pub fn write_render_panel(viewer: &ViewerResources, build: &mut UiBuild<'_>) {
    let handles = &viewer.handles;
    let context = build.context();
    let commands = &mut *context.commands;
    ui_dropdown_set_value(commands, handles.shading_dropdown, viewer.view.shading);
    ui_dropdown_set_value(commands, handles.pbr_dropdown, viewer.view.pbr_debug);
    ui_dropdown_set_value(commands, handles.tonemap_dropdown, viewer.view.tonemap);
    ui_dropdown_set_value(
        commands,
        handles.atmosphere_dropdown,
        viewer.view.atmosphere,
    );
    ui_dropdown_set_value(
        commands,
        handles.anti_aliasing_dropdown,
        viewer.view.anti_aliasing,
    );
    ui_slider_set_value(commands, handles.exposure_slider, viewer.view.exposure);
    ui_slider_set_value(
        commands,
        handles.render_scale_slider,
        viewer.view.render_scale,
    );
    ui_slider_set_value(commands, handles.sun_intensity, viewer.sun.intensity);
    ui_drag_value_set_value(commands, handles.sun_yaw, viewer.sun.yaw);
    ui_drag_value_set_value(commands, handles.sun_pitch, viewer.sun.pitch);
    ui_toggle_set_value(commands, handles.sun_shadows, viewer.sun.shadows);
    ui_toggle_set_value(commands, handles.sky_toggle, viewer.view.show_sky);
    ui_toggle_set_value(commands, handles.normals_toggle, viewer.view.show_normals);
    ui_toggle_set_value(commands, handles.bounds_toggle, viewer.view.show_bounds);
    ui_toggle_set_value(commands, handles.skeleton_toggle, viewer.skeleton.enabled);
    ui_toggle_set_value(commands, handles.ssao_toggle, viewer.view.ssao);
    ui_toggle_set_value(commands, handles.bloom_toggle, viewer.view.bloom);
    ui_toggle_set_value(commands, handles.ssr_toggle, viewer.view.ssr);
    ui_toggle_set_value(commands, handles.ssgi_toggle, viewer.view.ssgi);
    ui_toggle_set_value(commands, handles.fog_toggle, viewer.view.fog);
    ui_drag_value_set_value(commands, handles.fog_start, viewer.view.fog_start);
    ui_drag_value_set_value(commands, handles.fog_end, viewer.view.fog_end);
    ui_toggle_set_value(
        commands,
        handles.depth_of_field_toggle,
        viewer.view.depth_of_field,
    );
    ui_drag_value_set_value(commands, handles.focus_distance, viewer.view.focus_distance);
    ui_drag_value_set_value(commands, handles.focus_range, viewer.view.focus_range);
}

pub fn rebuild_recent(viewer: &mut ViewerResources, build: &mut UiBuild<'_>) {
    let mut previous = std::mem::take(&mut viewer.handles.recent_children);
    clear_nodes(build, &mut previous);
    viewer.handles.recent_buttons.clear();

    let container = viewer.handles.recent_container;
    let paths = viewer.recent.clone();
    let mut children = Vec::new();
    let mut buttons = Vec::new();
    let mut tree = UiTreeBuilder::from_parent(build.context(), container);
    if paths.is_empty() {
        children.push(
            tree.add_node()
                .fill_width()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 24.0)))
                .with_text("Nothing was opened from a path yet.", 12.0)
                .text_left()
                .fg(ThemeColor::TextDisabled)
                .entity(),
        );
    }
    for path in paths {
        let name = std::path::Path::new(&path)
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| path.clone());
        let row = tree
            .add_node()
            .fill_width()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 30.0)))
            .flow_horizontal()
            .padding(0.0)
            .gap(8.0)
            .align_cross(FlowAlignment::Center)
            .entity();
        children.push(row);
        let button = tree.in_parent(row, |tree| {
            let button = button(tree, &name, 200.0);
            tree.add_node()
                .flow_child(Ab(vec2(0.0, 16.0)))
                .flex_grow(1.0)
                .with_text(&path, 10.0)
                .text_left()
                .with_text_overflow(TextOverflow::Ellipsis)
                .fg(ThemeColor::TextDisabled);
            button
        });
        buttons.push((button, path));
    }
    tree.finish();

    viewer.handles.recent_children = children;
    viewer.handles.recent_buttons = buttons;
}

pub fn rebuild_library_strip(viewer: &mut ViewerResources, build: &mut UiBuild<'_>) {
    let mut previous = std::mem::take(&mut viewer.handles.library_children);
    clear_nodes(build, &mut previous);
    viewer.handles.library_forget.clear();

    let container = viewer.handles.library_chips;
    let roots = roots(viewer);
    let folders = roots.len();
    let mut paths = roots;
    paths.extend(viewer.library.files.iter().cloned());
    let counts: Vec<usize> = paths
        .iter()
        .enumerate()
        .map(|(index, path)| {
            if index >= folders {
                return 1;
            }
            let prefix = format!("{}{}", path.trim_end_matches(['/', '\\']), separator(path));
            viewer
                .library
                .entries
                .iter()
                .filter(|entry| entry.path.starts_with(&prefix))
                .count()
        })
        .collect();
    let built_in = viewer.library.built_in.clone();

    let mut children = Vec::new();
    let mut forget = Vec::new();
    let mut tree = UiTreeBuilder::from_parent(build.context(), container);
    if paths.is_empty() {
        children.push(
            tree.add_node()
                .flow_child(Ab(vec2(320.0, 20.0)))
                .with_text("No folder is remembered yet.", 11.0)
                .text_left()
                .fg(ThemeColor::TextDisabled)
                .entity(),
        );
    }
    for (path, count) in paths.iter().zip(counts) {
        let permanent = Some(path) == built_in.as_ref();
        let label = format!("{} ({count})", display_name(path));
        let label_width = measured_width(&label);
        let width = if permanent {
            label_width + 18.0
        } else {
            label_width + 34.0
        };
        let chip = tree
            .add_node()
            .flow_child(Ab(vec2(width, 26.0)))
            .flow_horizontal()
            .padding(6.0)
            .gap(6.0)
            .align_cross(FlowAlignment::Center)
            .with_rect(5.0, 1.0, vec4(1.0, 1.0, 1.0, 0.1))
            .with_theme_border_color(ThemeColor::Border)
            .color_raw::<UiBase>(vec4(0.0, 0.0, 0.0, 0.25))
            .entity();
        children.push(chip);
        let remove = tree.in_parent(chip, |tree| {
            tree.add_node()
                .flow_child(Ab(vec2(label_width, 16.0)))
                .with_text(&label, 11.0)
                .text_left()
                .with_text_overflow(TextOverflow::Ellipsis)
                .fg(if permanent {
                    ThemeColor::TextDisabled
                } else {
                    ThemeColor::Text
                });
            (!permanent).then(|| close_button(tree))
        });
        if let Some(remove) = remove {
            forget.push((remove, path.clone()));
        }
    }
    tree.finish();

    viewer.handles.library_children = children;
    viewer.handles.library_forget = forget;
}

fn separator(path: &str) -> char {
    if path.contains('\\') { '\\' } else { '/' }
}

fn measured_width(label: &str) -> f32 {
    (label.chars().count() as f32 * 6.0).clamp(60.0, 220.0)
}

fn close_button(tree: &mut UiTreeBuilder) -> Entity {
    use nightshade_ui::data::icons::material;
    let entity = tree
        .add_node()
        .flow_child(Ab(vec2(16.0, 16.0)))
        .with_icon(material::CLOSE, 12.0)
        .text_center()
        .fg(ThemeColor::TextDisabled)
        .fg_hover(ThemeColor::Text)
        .with_interaction()
        .with_cursor_icon(winit::window::CursorIcon::Pointer)
        .entity();
    tree.commands().set(
        entity,
        UiButtonData {
            clicked: false,
            text_slot: None,
        },
    );
    entity
}

pub fn clear_nodes(build: &mut UiBuild<'_>, previous: &mut Vec<Entity>) {
    if previous.is_empty() {
        return;
    }
    let mut context = build.context();
    for entity in previous.drain(..) {
        ui_despawn_node(&mut context, entity);
    }
}

pub fn rebuild_scene_tree(viewer: &mut ViewerResources, build: &mut UiBuild<'_>) {
    let mut previous: Vec<Entity> = viewer
        .handles
        .tree_rows
        .drain(..)
        .map(|row| row.wrapper)
        .collect();
    clear_nodes(build, &mut previous);

    let rows = viewer.model.rows.clone();
    let view = viewer.handles.tree_view;
    let container = viewer.handles.tree_container;
    let empty = viewer.handles.tree_empty;

    {
        let mut context = build.context();
        reset_tree(&mut context, view);
    }

    let mut tree = UiTreeBuilder::from_parent(build.context(), container);
    let mut containers: HashMap<Entity, Entity> = HashMap::new();
    let mut built = Vec::with_capacity(rows.len());
    for row in &rows {
        let parent_container = row
            .parent
            .and_then(|parent| containers.get(&parent).copied())
            .unwrap_or(container);
        let node = tree.add_tree_node_with_icon(
            view,
            parent_container,
            Some(icon_of(row.kind)),
            &row.name,
            row.depth as usize,
            row.entity.id as u64,
        );
        let data = tree.component::<UiTreeNodeData>(node);
        let wrapper = data
            .as_ref()
            .map(|data| data.wrapper_entity)
            .unwrap_or(node);
        if let Some(data) = data {
            containers.insert(row.entity, data.children_container);
        }
        let eye = tree.in_parent(node, |tree| eye_button(tree, row.visible));
        if row.depth == 0 {
            let mut context = tree.context_reborrow();
            ui_tree_node_expand_on_build(&mut context, node, true);
        }
        built.push(TreeRow {
            node,
            wrapper,
            eye,
            target: row.entity,
        });
    }
    tree.finish();

    let has_rows = !built.is_empty();
    viewer.handles.tree_rows = built;
    viewer.view.applied.selected = None;

    let context = build.context();
    ui_set_visible(context.dirty, context.commands, empty, !has_rows);
}

fn eye_button(tree: &mut UiTreeBuilder, visible: bool) -> Entity {
    use nightshade_ui::data::icons::material;
    let glyph = if visible {
        material::VISIBILITY
    } else {
        material::VISIBILITY_OFF
    };
    let entity = tree
        .add_node()
        .flow_child(Ab(vec2(22.0, 18.0)))
        .with_icon(glyph, 13.0)
        .text_center()
        .fg(ThemeColor::TextDisabled)
        .fg_hover(ThemeColor::Text)
        .with_interaction()
        .with_cursor_icon(winit::window::CursorIcon::Pointer)
        .entity();
    tree.commands().set(
        entity,
        UiButtonData {
            clicked: false,
            text_slot: None,
        },
    );
    entity
}

fn reset_tree(
    context: &mut nightshade_ui::commands::builder::UiBuildContext<'_, '_>,
    view: Entity,
) {
    context.commands.queue(move |storage, _| {
        if let Some(data) = storage.get_mut::<UiTreeViewData>(view) {
            data.node_entities.clear();
            data.selected_nodes.clear();
            data.pre_filter_expanded.clear();
            data.changed = false;
        }
    });
}

pub fn rebuild_cameras(viewer: &mut ViewerResources, build: &mut UiBuild<'_>) {
    let mut previous = std::mem::take(&mut viewer.handles.camera_children);
    clear_nodes(build, &mut previous);
    viewer.handles.camera_dropdown = None;

    if viewer.model.cameras.is_empty() {
        return;
    }

    let container = viewer.handles.camera_container;
    let mut options: Vec<String> = vec!["Viewer camera".to_string()];
    options.extend(viewer.model.cameras.iter().map(|(_, name)| name.clone()));
    let borrowed: Vec<&str> = options.iter().map(String::as_str).collect();

    let mut tree = UiTreeBuilder::from_parent(build.context(), container);
    let header = tree
        .add_node()
        .fill_width()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 18.0)))
        .with_text("Camera", 11.0)
        .text_left()
        .fg(ThemeColor::TextDisabled)
        .entity();
    let holder = tree
        .add_node()
        .fill_width()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 26.0)))
        .entity();
    let dropdown = tree.in_parent(holder, |tree| tree.add_dropdown(&borrowed, 0));
    tree.finish();

    viewer.handles.camera_children = vec![header, holder];
    viewer.handles.camera_dropdown = Some(dropdown);
}

pub fn rebuild_layers(viewer: &mut ViewerResources, build: &mut UiBuild<'_>) {
    let mut previous = std::mem::take(&mut viewer.handles.layer_children);
    clear_nodes(build, &mut previous);
    viewer.handles.layer_rows.clear();

    let layers = viewer.animation.layers.clone();
    if layers.is_empty() {
        return;
    }
    let container = viewer.handles.layer_container;
    let layout = viewer.layout;

    let mut children = Vec::new();
    let mut rows = Vec::new();
    let mut tree = UiTreeBuilder::from_parent(build.context(), container);
    children.push(
        tree.add_node()
            .flow_child(Ab(vec2(layer_row_width(layout), 16.0)))
            .with_text("Layers over the playing clip", 11.0)
            .text_left()
            .fg(ThemeColor::TextDisabled)
            .entity(),
    );
    for (index, layer) in layers.iter().enumerate() {
        let row = tree
            .add_node()
            .flow_child(Ab(vec2(layer_row_width(layout), 24.0)))
            .flow_horizontal()
            .padding(0.0)
            .gap(8.0)
            .align_cross(FlowAlignment::Center)
            .entity();
        children.push(row);
        let (slider, remove) = tree.in_parent(row, |tree| {
            tree.add_node()
                .flow_child(Ab(vec2(LAYER_NAME_WIDTH, 16.0)))
                .with_text(&layer.name, 11.0)
                .text_left()
                .with_text_overflow(TextOverflow::Ellipsis)
                .fg(ThemeColor::Text);
            let holder = tree
                .add_node()
                .flow_child(Ab(vec2(layer_weight_width(layout), 22.0)))
                .entity();
            let slider = tree.in_parent(holder, |tree| {
                tree.add_slider_configured(SliderConfig::new(0.0, 1.0, layer.weight).precision(2))
            });
            let remove = button(tree, "Remove", 82.0);
            (slider, remove)
        });
        rows.push(LayerRow {
            slider,
            remove,
            index,
        });
    }
    tree.finish();

    viewer.handles.layer_children = children;
    viewer.handles.layer_rows = rows;
}

pub fn rebuild_variants(viewer: &mut ViewerResources, build: &mut UiBuild<'_>) {
    let mut previous = std::mem::take(&mut viewer.handles.variant_children);
    clear_nodes(build, &mut previous);
    viewer.handles.variant_dropdown = None;

    if viewer.model.variants.is_empty() {
        return;
    }

    let container = viewer.handles.variant_container;
    let mut options: Vec<String> = vec!["Default".to_string()];
    options.extend(viewer.model.variants.iter().cloned());
    let borrowed: Vec<&str> = options.iter().map(String::as_str).collect();

    let mut tree = UiTreeBuilder::from_parent(build.context(), container);
    let header = tree
        .add_node()
        .fill_width()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 18.0)))
        .with_text("Material variant", 11.0)
        .text_left()
        .fg(ThemeColor::TextDisabled)
        .entity();
    let holder = tree
        .add_node()
        .fill_width()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 26.0)))
        .entity();
    let dropdown = tree.in_parent(holder, |tree| tree.add_dropdown(&borrowed, 0));
    tree.finish();

    viewer.handles.variant_children = vec![header, holder];
    viewer.handles.variant_dropdown = Some(dropdown);
}

pub fn rebuild_morphs(
    viewer: &mut ViewerResources,
    build: &mut UiBuild<'_>,
    storage: &nightshade::ecs::Storage,
) {
    let mut previous = std::mem::take(&mut viewer.handles.morph_children);
    clear_nodes(build, &mut previous);
    viewer.handles.morph_sliders.clear();

    let morphs: Vec<MorphInfo> = viewer
        .model
        .morphs
        .iter()
        .filter(|morph| !storage.has::<Deleted>(morph.entity))
        .cloned()
        .collect();
    if morphs.is_empty() {
        return;
    }

    let container = viewer.handles.morph_container;

    let mut children = Vec::new();
    let mut sliders = Vec::new();
    let mut tree = UiTreeBuilder::from_parent(build.context(), container);
    children.push(
        tree.add_node()
            .fill_width()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 18.0)))
            .with_text("Morph targets", 11.0)
            .text_left()
            .fg(ThemeColor::TextDisabled)
            .entity(),
    );
    for morph in &morphs {
        let group = tree
            .add_node()
            .fill_width()
            .auto_height()
            .flow_vertical()
            .padding(0.0)
            .gap(2.0)
            .entity();
        children.push(group);
        tree.in_parent(group, |tree| {
            tree.add_node()
                .fill_width()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 16.0)))
                .with_text(&morph.name, 11.0)
                .text_left()
                .with_text_overflow(TextOverflow::Ellipsis)
                .fg(ThemeColor::Text);
            for (index, weight) in morph.weights.iter().enumerate() {
                let row = tree
                    .add_node()
                    .fill_width()
                    .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 22.0)))
                    .flow_horizontal()
                    .padding(0.0)
                    .gap(6.0)
                    .align_cross(FlowAlignment::Center)
                    .entity();
                tree.in_parent(row, |tree| {
                    tree.add_node()
                        .flow_child(Ab(vec2(58.0, 16.0)))
                        .with_text(&format!("Target {index}"), 11.0)
                        .text_left()
                        .fg(ThemeColor::TextDisabled);
                    let holder = tree
                        .add_node()
                        .flow_child(Ab(vec2(0.0, 22.0)))
                        .flex_grow(1.0)
                        .entity();
                    let slider = tree.in_parent(holder, |tree| {
                        tree.add_slider_configured(
                            SliderConfig::new(0.0, 1.0, *weight).precision(2),
                        )
                    });
                    sliders.push(MorphSlider {
                        slider,
                        target: morph.entity,
                        index,
                    });
                });
            }
        });
    }
    tree.finish();

    viewer.handles.morph_children = children;
    viewer.handles.morph_sliders = sliders;
}

pub fn rebuild_clips(viewer: &mut ViewerResources, build: &mut UiBuild<'_>) {
    let mut previous = std::mem::take(&mut viewer.handles.clip_children);
    clear_nodes(build, &mut previous);
    viewer.handles.clip_dropdown = None;

    if viewer.model.clips.is_empty() {
        return;
    }

    let container = viewer.handles.clip_container;
    let names: Vec<String> = viewer
        .model
        .clips
        .iter()
        .enumerate()
        .map(|(index, clip)| {
            if clip.name.is_empty() {
                format!("Clip {index}")
            } else {
                clip.name.clone()
            }
        })
        .collect();
    let borrowed: Vec<&str> = names.iter().map(String::as_str).collect();
    let initial = viewer.animation.clip.unwrap_or(0).min(borrowed.len() - 1);

    let mut tree = UiTreeBuilder::from_parent(build.context(), container);
    let dropdown = tree.add_dropdown(&borrowed, initial);
    tree.finish();

    viewer.handles.clip_children = vec![dropdown];
    viewer.handles.clip_dropdown = Some(dropdown);
}

pub fn rebuild_browser(viewer: &mut ViewerResources, build: &mut UiBuild<'_>) {
    let mut previous = std::mem::take(&mut viewer.handles.browser_children);
    clear_nodes(build, &mut previous);
    viewer.handles.browser_cells.clear();

    let Some(tab) = viewer.view.browser else {
        return;
    };
    let query = viewer.view.search.to_lowercase();
    let entries = page_entries(viewer, tab, &query, viewer.view.browser_page);
    viewer.browsers.wanted = entries
        .iter()
        .map(|entry| entry.key.clone())
        .filter(|key| !key.is_empty())
        .collect();
    viewer.library.sources = entries
        .iter()
        .filter_map(|entry| match &entry.item {
            BrowserItem::File(path) => Some((entry.key.clone(), path.clone())),
            _ => None,
        })
        .collect();
    let container = viewer.handles.browser_grid;
    let slots = viewer.browsers.slots.clone();

    let mut children = Vec::new();
    let mut cells = Vec::new();
    let mut tree = UiTreeBuilder::from_parent(build.context(), container);

    if entries.is_empty() {
        children.push(
            tree.add_node()
                .fill_width()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 40.0)))
                .with_text(empty_text(viewer, tab), 13.0)
                .fg(ThemeColor::TextDisabled)
                .entity(),
        );
    }

    for entry in &entries {
        let text_slot = tree.text_state_mut().cache.add_text(&entry.label);
        let slot = slots.get(&entry.key).copied();
        let cell = tree
            .add_node()
            .flow_child(Ab(vec2(BROWSER_CELL, BROWSER_CELL + 22.0)))
            .with_rect(6.0, 1.0, vec4(1.0, 1.0, 1.0, 0.1))
            .with_theme_border_color(ThemeColor::Border)
            .color_raw::<UiBase>(vec4(0.0, 0.0, 0.0, 0.25))
            .fg_hover(ThemeColor::BackgroundHover)
            .with_transition::<UiHover>(14.0, 8.0)
            .with_interaction()
            .with_cursor_icon(winit::window::CursorIcon::Pointer)
            .with_children(|inner| {
                match slot {
                    Some(slot) => {
                        inner
                            .add_node()
                            .window(
                                Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 4.0)),
                                Ab(vec2(BROWSER_CELL - 8.0, BROWSER_CELL - 8.0)),
                                Anchor::TopCenter,
                            )
                            .with_image_uv(slot.layer, slot.uv_min, slot.uv_max)
                            .color_raw::<UiBase>(vec4(1.0, 1.0, 1.0, 1.0));
                    }
                    None => {
                        inner
                            .add_node()
                            .window(
                                Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 4.0)),
                                Ab(vec2(BROWSER_CELL - 8.0, BROWSER_CELL - 8.0)),
                                Anchor::TopCenter,
                            )
                            .rect(4.0)
                            .color_raw::<UiBase>(vec4(1.0, 1.0, 1.0, 0.05));
                    }
                }
                inner
                    .add_node()
                    .window(
                        Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -4.0)),
                        Ab(vec2(BROWSER_CELL - 8.0, 16.0)),
                        Anchor::BottomCenter,
                    )
                    .with_text_slot(text_slot, 11.0)
                    .text_center()
                    .with_text_overflow(TextOverflow::Ellipsis)
                    .fg(ThemeColor::Text);
            })
            .entity();
        tree.commands().set(
            cell,
            UiButtonData {
                clicked: false,
                text_slot: Some(text_slot),
            },
        );
        children.push(cell);
        cells.push(BrowserCell {
            button: cell,
            item: entry.item.clone(),
        });
    }
    tree.finish();

    viewer.handles.browser_children = children;
    viewer.handles.browser_cells = cells;
}
