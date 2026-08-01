use crate::resources::{Behavior, Interface, Scene};
use nightshade_navmesh::resources::PathfindingAlgorithm;
use nightshade_ui::components::layout_types::{FlowAlignment, FlowDirection, FlowLayout};
use nightshade_ui::prelude::*;

fn collapsing<F>(tree: &mut UiTreeBuilder, label: &str, default_open: bool, body: F)
where
    F: FnOnce(&mut UiTreeBuilder),
{
    let header = tree.add_collapsing_header(label, default_open);
    let Some(content) = tree
        .component::<UiCollapsingHeaderData>(header)
        .map(|data| data.content_entity)
    else {
        return;
    };
    tree.in_parent(content, body);
}

pub fn build_ui(scene: &mut Scene<'_, '_>, behavior: &Behavior) -> Interface {
    let mut tree_builder = UiTreeBuilder::new(scene.ui_context());
    let panel = tree_builder.add_docked_panel_right("navmesh_demo", "NavMesh Demo Controls", 320.0);
    let content = tree_builder
        .component::<nightshade_ui::components::UiPanelData>(panel)
        .map(|data| data.content_entity)
        .unwrap_or(panel);
    tree_builder.edit::<nightshade_ui::components::UiLayoutNode>(content, |node| {
        node.flow_layout = Some(FlowLayout {
            direction: FlowDirection::Vertical,
            padding: 0.0,
            spacing: 0.0,
            alignment: FlowAlignment::Start,
            cross_alignment: FlowAlignment::Start,
            wrap: false,
        });
    });

    let mut handles = Interface::default();

    tree_builder.in_parent(content, |tree_builder| {
        let scroll = tree_builder.add_scroll_area_fill(8.0, 6.0);
        let scroll_body = tree_builder
            .component::<UiScrollAreaData>(scroll)
            .map(|data| data.content_entity)
            .expect("scroll area body");
        tree_builder.in_parent(scroll_body, |tree_builder| {
            collapsing(tree_builder, "Visualization", true, |tree| {
                handles.show_navmesh = tree.add_checkbox("Show NavMesh", behavior.show_navmesh);
            });

            collapsing(tree_builder, "Behavior", true, |tree| {
                handles.wander_mode = tree.add_checkbox("Wander Mode", behavior.wander);
                handles.follow_mode = tree.add_checkbox("Follow Mode", behavior.follow);
                handles.following_label = tree.add_label("");
            });

            collapsing(tree_builder, "Pathfinding", false, |tree| {
                let algorithm_options: Vec<&str> = PathfindingAlgorithm::all()
                    .iter()
                    .map(|algorithm| algorithm.name())
                    .collect();
                handles.algorithm = tree.add_dropdown(&algorithm_options, 0);
            });

            collapsing(tree_builder, "Fox Settings", false, |tree| {
                tree.add_label("Speed");
                handles.speed = tree.add_slider(1.0, 15.0, behavior.speed);
            });

            collapsing(tree_builder, "Spawning", true, |tree| {
                handles.spawn_button = tree.add_button("Spawn Fox");
                handles.clear_button = tree.add_button("Clear Foxes");
            });

            collapsing(tree_builder, "Stats", true, |tree| {
                handles.foxes_label = tree.add_label("Foxes: 0");
                handles.trees_label = tree.add_label("Trees: 0");
                handles.triangles_label = tree.add_label("NavMesh Triangles: 0");
                handles.fps_label = tree.add_label("FPS: 0");
            });
        });
    });
    tree_builder.finish();

    handles
}
