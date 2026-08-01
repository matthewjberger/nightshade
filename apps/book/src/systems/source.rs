use nightshade::prelude::*;
use nightshade_ui::commands::builder::{UiBuild, UiTreeBuilder};
use nightshade_ui::components::UiLayoutNode;
use nightshade_ui::prelude::*;

use crate::commands::source::build_source;
use crate::data::source::files_of;
use crate::data::style::{
    CARD_PADDING, HAIRLINE, SOURCE_GUTTER, SOURCE_ROOM_STEP, SOURCE_TREE_WIDTH, SPACE_3,
};
use crate::resources::{PageStage, Shell, SourceView};

pub fn poll(
    mut view: ResMut<SourceView>,
    shell: Res<Shell>,
    page_stage: Res<PageStage>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut interactions: Query<(&UiNodeInteraction,)>,
    mut nodes: Query<(&mut UiLayoutNode,)>,
) {
    let was_open = view.demo;

    for figure in &page_stage.figures {
        if clicked(&mut interactions, figure.source_button) {
            view.demo = if view.demo == Some(figure.demo) {
                None
            } else {
                view.file = first_file(figure.demo);
                Some(figure.demo)
            };
        }
    }

    if view.demo.is_some() && clicked(&mut interactions, shell.source_close) {
        view.demo = None;
    }

    if let Some(open) = view.demo
        && !page_stage.figures.iter().any(|figure| figure.demo == open)
    {
        view.demo = None;
    }

    if view.demo.is_some()
        && let Some(row) = view
            .rows
            .iter()
            .position(|row| clicked(&mut interactions, *row))
        && let Some(file) = view.row_files.get(row).copied()
    {
        view.file = file;
    }

    if view.demo == was_open {
        return;
    }

    nodes.for_entity(shell.source_panel, |(node,)| {
        node.visible = view.demo.is_some();
    });
    dirty.layout_dirty = true;
}

pub fn refresh(
    mut view: ResMut<SourceView>,
    shell: Res<Shell>,
    window: Res<nightshade_platform::prelude::Window>,
    mut build: UiBuild,
) {
    let steps = room_steps(&window);
    let wanted = view.demo.map(|demo| (demo, view.file, steps));
    if wanted == view.shown {
        return;
    }

    if let Some(root) = view.root.take() {
        ui_despawn_node(&mut build.context(), root);
    }
    view.rows.clear();
    view.row_files.clear();
    view.shown = wanted;

    let Some((demo, file, steps)) = wanted else {
        return;
    };

    let mut tree = UiTreeBuilder::from_parent(build.context(), shell.source_host);
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();
    let mut built = None;
    tree.in_parent(root, |tree| {
        built = Some(build_source(
            tree,
            demo,
            file,
            steps as f32 * SOURCE_ROOM_STEP,
        ));
    });

    if let Some(built) = built {
        view.rows = built.rows;
        view.row_files = built.row_files;
    }
    view.root = Some(root);

    let title = crate::systems::demos::demo_entry(demo)
        .map(|entry| entry.title.to_string())
        .unwrap_or_else(|| demo.to_string());
    ui_set_text(build.context().commands, shell.source_title, &title);
}

fn first_file(demo: &str) -> usize {
    files_of(demo)
        .iter()
        .position(|file| file.own)
        .unwrap_or_default()
}

fn room_steps(window: &nightshade_platform::prelude::Window) -> i32 {
    let across = window
        .cached_viewport_size
        .map(|(width, _)| crate::queries::screen::to_units(width as f32, window))
        .unwrap_or(0.0);
    let room = across - SOURCE_TREE_WIDTH - HAIRLINE - CARD_PADDING * 2.0 - SOURCE_GUTTER - SPACE_3;
    (room / SOURCE_ROOM_STEP).floor().max(1.0) as i32
}

fn clicked(interactions: &mut Query<(&UiNodeInteraction,)>, entity: Entity) -> bool {
    let mut hit = false;
    interactions.for_entity(entity, |(interaction,)| hit = interaction.clicked);
    hit
}
