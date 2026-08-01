use nightshade::prelude::*;
use nightshade_ui::commands::builder::UiBuild;
use nightshade_ui::prelude::*;

use crate::commands::lists::clear_nodes;
use crate::data::LIGHT_KIND_LABELS;
use crate::resources::{CameraHandles, LightHandles, ViewerResources};

pub fn rebuild_light(viewer: &mut ViewerResources, build: &mut UiBuild<'_>) {
    let mut previous = std::mem::take(&mut viewer.handles.light.children);
    clear_nodes(build, &mut previous);
    viewer.handles.light = LightHandles::default();

    let Some(light) = viewer.selection.light.clone() else {
        return;
    };
    let container = viewer.handles.light_container;

    let mut handles = LightHandles::default();
    let mut tree = UiTreeBuilder::from_parent(build.context(), container);

    let grid = tree.add_property_grid(64.0);
    handles.children.push(grid);
    let section = tree.add_property_section(grid, "Light");
    let kind = tree.add_property_row(grid, section, "Kind");
    tree.in_parent(kind, |tree| {
        tree.add_node()
            .flow_child(Ab(vec2(0.0, 18.0)))
            .flex_grow(1.0)
            .with_text(LIGHT_KIND_LABELS[light.kind.min(2)], 11.0)
            .text_left()
            .fg(ThemeColor::TextDisabled);
    });
    handles.color = {
        let row = tree.add_property_row(grid, section, "Color");
        tree.in_parent(row, |tree| {
            tree.add_color_picker(vec4(light.color[0], light.color[1], light.color[2], 1.0))
        })
    };
    handles.intensity = drag_row(&mut tree, grid, section, "Intensity", 0.05, light.intensity);
    if light.kind != 0 {
        handles.range = drag_row(&mut tree, grid, section, "Range", 0.05, light.range);
    }
    if light.kind == 2 {
        handles.inner_cone = slider_row(
            &mut tree,
            grid,
            section,
            "Inner cone",
            0.0,
            90.0,
            light.inner_cone,
        );
        handles.outer_cone = slider_row(
            &mut tree,
            grid,
            section,
            "Outer cone",
            0.0,
            90.0,
            light.outer_cone,
        );
    }
    handles.cast_shadows = {
        let row = tree.add_property_row(grid, section, "Shadows");
        tree.in_parent(row, |tree| tree.add_toggle(light.cast_shadows))
    };

    tree.finish();
    viewer.handles.light = handles;
}

pub fn rebuild_camera(viewer: &mut ViewerResources, build: &mut UiBuild<'_>) {
    let mut previous = std::mem::take(&mut viewer.handles.camera_fields.children);
    clear_nodes(build, &mut previous);
    viewer.handles.camera_fields = CameraHandles::default();

    let Some(camera) = viewer.selection.camera.clone() else {
        return;
    };
    let container = viewer.handles.camera_section;

    let mut handles = CameraHandles::default();
    let mut tree = UiTreeBuilder::from_parent(build.context(), container);

    let grid = tree.add_property_grid(64.0);
    handles.children.push(grid);
    let section = tree.add_property_section(grid, "Camera");
    let (label, speed) = if camera.perspective {
        ("Field of view", 0.25)
    } else {
        ("Height", 0.01)
    };
    handles.field_of_view = drag_row(&mut tree, grid, section, label, speed, camera.field_of_view);
    handles.near = drag_row(&mut tree, grid, section, "Near", 0.001, camera.near);
    handles.far = drag_row(&mut tree, grid, section, "Far", 0.5, camera.far);

    tree.finish();
    viewer.handles.camera_fields = handles;
}

fn slider_row(
    tree: &mut UiTreeBuilder,
    grid: Entity,
    section: Entity,
    label: &str,
    low: f32,
    high: f32,
    value: f32,
) -> Entity {
    let row = tree.add_property_row(grid, section, label);
    tree.in_parent(row, |tree| {
        let holder = tree
            .add_node()
            .flow_child(Ab(vec2(0.0, 22.0)))
            .flex_grow(1.0)
            .entity();
        tree.in_parent(holder, |tree| {
            tree.add_slider_configured(
                SliderConfig::new(low, high, value.clamp(low, high)).precision(2),
            )
        })
    })
}

fn drag_row(
    tree: &mut UiTreeBuilder,
    grid: Entity,
    section: Entity,
    label: &str,
    speed: f32,
    value: f32,
) -> Entity {
    let row = tree.add_property_row(grid, section, label);
    tree.in_parent(row, |tree| {
        let holder = tree
            .add_node()
            .flow_child(Ab(vec2(0.0, 22.0)))
            .flex_grow(1.0)
            .entity();
        tree.in_parent(holder, |tree| {
            tree.add_drag_value_configured(
                DragValueConfig::new(0.0, 1.0e6, value)
                    .speed(speed)
                    .precision(3),
            )
        })
    })
}
