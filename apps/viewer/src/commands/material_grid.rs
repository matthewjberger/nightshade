use nightshade::prelude::*;
use nightshade_ui::commands::builder::UiBuild;
use nightshade_ui::prelude::*;

use crate::commands::interface::button;
use crate::commands::lists::clear_nodes;
use crate::data::{ALPHA_LABELS, BASE, COAT, MaterialControl, SURFACE, THROUGH};
use crate::resources::{MaterialHandles, MaterialView, TextureRow, ViewerResources};
use crate::theme::TEXTURE_PREVIEW;

pub fn rebuild_material(viewer: &mut ViewerResources, build: &mut UiBuild<'_>) {
    let mut previous = std::mem::take(&mut viewer.handles.material.children);
    clear_nodes(build, &mut previous);
    viewer.handles.material = MaterialHandles::default();

    let Some(name) = viewer.selection.material.clone() else {
        return;
    };
    let values = viewer.selection.material_values.clone();
    let container = viewer.handles.material_container;
    let previews = viewer.model.previews.clone();

    let mut handles = MaterialHandles::default();
    let mut tree = UiTreeBuilder::from_parent(build.context(), container);

    handles.children.push(
        tree.add_node()
            .fill_width()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 16.0)))
            .with_text(&name, 11.0)
            .text_left()
            .with_text_overflow(TextOverflow::Ellipsis)
            .fg(ThemeColor::TextAccent)
            .entity(),
    );
    handles.children.push(
        tree.add_node()
            .fill_width()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 14.0)))
            .with_text("Every mesh that names this material changes with it.", 10.0)
            .text_left()
            .with_text_wrap()
            .fg(ThemeColor::TextDisabled)
            .entity(),
    );

    let grid = tree.add_property_grid(76.0);
    handles.children.push(grid);

    let base = tree.add_property_section(grid, "Base");
    handles.base_color = color_row(&mut tree, grid, base, "Color", vec4_of(values.base_color));
    handles.alpha_mode =
        tree.add_property_dropdown(grid, base, "Alpha", &ALPHA_LABELS, values.alpha_mode);
    for control in BASE {
        handles.sliders.push((
            control_slider(&mut tree, grid, base, control, &values),
            control,
        ));
    }
    handles.unlit = toggle_row(&mut tree, grid, base, "Unlit", values.unlit);
    handles.double_sided = toggle_row(&mut tree, grid, base, "Two sided", values.double_sided);

    let emissive = tree.add_property_section(grid, "Emissive");
    handles.emissive = color_row(
        &mut tree,
        grid,
        emissive,
        "Color",
        vec4_of([
            values.emissive[0],
            values.emissive[1],
            values.emissive[2],
            1.0,
        ]),
    );
    handles.sliders.push((
        control_slider(
            &mut tree,
            grid,
            emissive,
            MaterialControl::EmissiveStrength,
            &values,
        ),
        MaterialControl::EmissiveStrength,
    ));

    let surface = tree.add_property_section(grid, "Surface");
    for control in SURFACE {
        handles.sliders.push((
            control_slider(&mut tree, grid, surface, control, &values),
            control,
        ));
    }

    let through = tree.add_property_section(grid, "Transmission");
    for control in THROUGH {
        handles.sliders.push((
            control_slider(&mut tree, grid, through, control, &values),
            control,
        ));
    }

    let coat = tree.add_property_section(grid, "Coat and sheen");
    for control in COAT {
        handles.sliders.push((
            control_slider(&mut tree, grid, coat, control, &values),
            control,
        ));
    }

    if !values.textures.is_empty() {
        let textures = tree.add_property_section(grid, "Textures");
        for (texture_slot, name) in &values.textures {
            let row = tree.add_property_row(grid, textures, texture_slot.label());
            let slot = previews.get(name).copied();
            let mut nodes = vec![row];
            let mut replace = Entity::default();
            tree.in_parent(row, |tree| {
                let holder = tree
                    .add_node()
                    .flow_child(Ab(vec2(0.0, TEXTURE_PREVIEW + 4.0)))
                    .flex_grow(1.0)
                    .flow_horizontal()
                    .padding(0.0)
                    .gap(6.0)
                    .align_cross(FlowAlignment::Center)
                    .with_interaction()
                    .with_tooltip("Drop a picture here to try it in this slot")
                    .entity();
                nodes.push(holder);
                tree.in_parent(holder, |tree| {
                    let picture = tree
                        .add_node()
                        .flow_child(Ab(vec2(TEXTURE_PREVIEW, TEXTURE_PREVIEW)))
                        .rect(3.0);
                    let picture = match slot {
                        Some(slot) => picture
                            .with_image_uv(slot.layer, slot.uv_min, slot.uv_max)
                            .color_raw::<UiBase>(vec4(1.0, 1.0, 1.0, 1.0)),
                        None => picture.color_raw::<UiBase>(vec4(1.0, 1.0, 1.0, 0.05)),
                    }
                    .entity();
                    nodes.push(picture);
                    let label = tree
                        .add_node()
                        .flow_child(Ab(vec2(0.0, 18.0)))
                        .flex_grow(1.0)
                        .with_text(name, 10.0)
                        .text_left()
                        .with_text_overflow(TextOverflow::Ellipsis)
                        .fg(ThemeColor::TextDisabled)
                        .entity();
                    nodes.push(label);
                    replace = button(tree, "Replace", 66.0);
                    nodes.push(replace);
                });
            });
            handles.texture_rows.push(TextureRow {
                slot: *texture_slot,
                nodes,
                replace,
            });
        }
    }

    let footer = tree
        .add_node()
        .fill_width()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 30.0)))
        .flow_horizontal()
        .padding(0.0)
        .gap(6.0)
        .align_main(FlowAlignment::End)
        .entity();
    handles.children.push(footer);
    handles.reset = tree.in_parent(footer, |tree| button(tree, "As loaded", 92.0));

    tree.finish();
    viewer.handles.material = handles;
}

fn vec4_of(color: [f32; 4]) -> Vec4 {
    vec4(color[0], color[1], color[2], color[3])
}

fn color_row(
    tree: &mut UiTreeBuilder,
    grid: Entity,
    section: Entity,
    label: &str,
    color: Vec4,
) -> Entity {
    let row = tree.add_property_row(grid, section, label);
    tree.in_parent(row, |tree| tree.add_color_picker(color))
}

fn toggle_row(
    tree: &mut UiTreeBuilder,
    grid: Entity,
    section: Entity,
    label: &str,
    value: bool,
) -> Entity {
    let row = tree.add_property_row(grid, section, label);
    tree.in_parent(row, |tree| tree.add_toggle(value))
}

fn control_slider(
    tree: &mut UiTreeBuilder,
    grid: Entity,
    section: Entity,
    control: MaterialControl,
    values: &MaterialView,
) -> Entity {
    let (label, low, high, value) = control.row(values);
    let row = tree.add_property_row(grid, section, label);
    tree.in_parent(row, |tree| {
        let holder = tree
            .add_node()
            .flow_child(Ab(vec2(0.0, 22.0)))
            .flex_grow(1.0)
            .entity();
        tree.in_parent(holder, |tree| {
            tree.add_slider_configured(SliderConfig::new(low, high, value).precision(2))
        })
    })
}
