use crate::commands::{build_a_preset, stand_the_pictures};
use crate::data::{FAMILIES, Kind, PRESETS, title_of};
use crate::resources::{PaletteRows, Pictures};
use crate::theme;
use nalgebra_glm::vec2;
use nightshade::ecs::system_param::{Commands, Res, ResMut};
use nightshade::prelude::Entity;
use nightshade_camera::commands::scene::spawn_ortho_camera;
use nightshade_camera::resources::ActiveCamera;
use nightshade_render::prelude::RenderSettings;
use nightshade_ui::commands::builder::UiBuild;
use nightshade_ui::prelude::{
    Ab, FlowAlignment, FlowDirection, FlowLayout, RetainedUiRuntime, Rl, UiCollapsingHeaderData,
    UiLayoutNode, UiScrollAreaData, UiTreeBuilder,
};
use nightshade_ui_nodegraph::prelude::{GraphView, spread_the_canvas};

pub fn stand_the_board(mut pictures: ResMut<Pictures>) {
    stand_the_pictures(&mut pictures);
}

pub fn stand_it_up(
    pictures: Res<Pictures>,
    mut settings: ResMut<RenderSettings>,
    mut runtime: ResMut<RetainedUiRuntime>,
    mut active_camera: ResMut<ActiveCamera>,
    mut commands: Commands,
) {
    settings.clear_color = theme::CLEAR;
    runtime.background_color = None;
    active_camera.0 = Some(spawn_ortho_camera(&mut commands, vec2(0.0, 0.0)));
    build_a_preset(&mut commands, &PRESETS[0], &pictures);
}

pub fn spread_the_board(
    mut view: ResMut<GraphView>,
    mut rows: ResMut<PaletteRows>,
    mut build: UiBuild,
) {
    let mut tree = UiTreeBuilder::new(build.context());

    let palette = tree.add_docked_panel_left(
        "shadergraph_nodes",
        theme::PALETTE_TITLE,
        theme::PALETTE_WIDTH,
    );
    let graph =
        tree.add_docked_panel_bottom("shadergraph_graph", theme::GRAPH_TITLE, theme::GRAPH_HEIGHT);

    let Some(palette_content) = tree.panel_content(palette) else {
        tree.finish();
        return;
    };
    let Some(graph_content) = tree.panel_content(graph) else {
        tree.finish();
        return;
    };

    stack_the_content(&mut tree, palette_content);
    tree.in_parent(graph_content, |tree| {
        spread_the_canvas(tree, &mut view);
    });

    let names: Vec<&str> = PRESETS.iter().map(|preset| preset.name).collect();
    let mut buttons = Vec::new();
    let (presets, pluck, write) = tree.in_parent(palette_content, |tree| {
        tree.add_heading(theme::PRESET_HEADING);
        let presets = tree.add_dropdown(&names, 0);
        tree.add_separator();

        tree.add_heading(theme::PICTURE_HEADING);
        let pluck = tree.add_button(theme::PLUCK_LABEL);
        tree.add_separator();

        tree.add_heading(theme::PLACE_HEADING);
        shelve_the_kinds(tree, &mut buttons);
        tree.add_separator();

        let write = tree.add_button(theme::WRITE_LABEL);
        (presets, pluck, write)
    });
    tree.finish();

    rows.buttons = buttons;
    rows.presets = Some(presets);
    rows.pluck = Some(pluck);
    rows.write = Some(write);
}

fn shelve_the_kinds(tree: &mut UiTreeBuilder<'_, '_>, buttons: &mut Vec<(Entity, Kind)>) {
    let shelf = tree.add_scroll_area(vec2(0.0, theme::SHELF_HEIGHT));
    let Some(content) = tree
        .component::<UiScrollAreaData>(shelf)
        .map(|data| data.content_entity)
    else {
        return;
    };
    tree.in_parent(content, |tree| {
        for (order, family) in FAMILIES.iter().enumerate() {
            let header = tree.add_collapsing_header(family.name, order < theme::OPEN_FAMILIES);
            let Some(inside) = tree
                .component::<UiCollapsingHeaderData>(header)
                .map(|data| data.content_entity)
            else {
                continue;
            };
            tree.in_parent(inside, |tree| {
                let row = wrap_a_row(tree);
                tree.in_parent(row, |tree| {
                    for kind in family.kinds {
                        let button = tree.add_button(title_of(*kind));
                        narrow_the_button(tree, button);
                        buttons.push((button, *kind));
                    }
                });
            });
        }
    });
}

fn narrow_the_button(tree: &mut UiTreeBuilder<'_, '_>, button: Entity) {
    let height = tree.active_theme().button_height;
    tree.edit::<UiLayoutNode>(button, move |node| {
        node.flow_child_size = Some(Ab(vec2(theme::KIND_WIDTH, height)).into());
    });
}

fn wrap_a_row(tree: &mut UiTreeBuilder<'_, '_>) -> Entity {
    let row = tree
        .add_node()
        .flow_child(Rl(vec2(100.0, 0.0)))
        .flow(FlowDirection::Horizontal, 0.0, theme::BUTTON_SPACING)
        .auto_height()
        .entity();
    tree.edit::<UiLayoutNode>(row, |node| {
        if let Some(flow) = node.flow_layout.as_mut() {
            flow.wrap = true;
        }
    });
    row
}

fn stack_the_content(tree: &mut UiTreeBuilder<'_, '_>, content: Entity) {
    tree.edit::<UiLayoutNode>(content, |node| {
        node.flow_layout = Some(FlowLayout {
            direction: FlowDirection::Vertical,
            padding: theme::PANEL_PADDING,
            spacing: theme::PANEL_SPACING,
            alignment: FlowAlignment::Start,
            cross_alignment: FlowAlignment::Start,
            wrap: false,
        });
    });
}
