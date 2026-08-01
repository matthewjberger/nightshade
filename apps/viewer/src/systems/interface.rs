use nightshade::prelude::*;
use nightshade_platform::prelude::Window;
use nightshade_ui::commands::builder::UiBuild;
use nightshade_ui::prelude::ui_despawn_node;

use crate::commands::interface::build_tree;
use crate::queries::layout::{layout_of, layout_width};
use crate::resources::{AppliedUi, UiHandles, ViewerResources};

pub fn build_ui(mut viewer: ResMut<ViewerResources>, window: Res<Window>, mut build: UiBuild<'_>) {
    let layout = layout_of(layout_width(&window));
    build_tree(&mut viewer, layout, &mut build);
}

pub fn relayout(mut viewer: ResMut<ViewerResources>, window: Res<Window>, mut build: UiBuild<'_>) {
    let layout = layout_of(layout_width(&window));
    if layout.narrow == viewer.layout.narrow {
        return;
    }
    {
        let mut context = build.context();
        ui_despawn_node(&mut context, viewer.handles.interface);
        ui_despawn_node(&mut context, viewer.handles.show_button);
    }
    let viewer = &mut *viewer;
    viewer.handles = UiHandles::default();
    viewer.view.applied = AppliedUi::default();
    viewer.view.search.clear();
    viewer.view.tree_filter.clear();
    viewer.view.browser_page = 0;
    build_tree(viewer, layout, &mut build);
    viewer.view.tree_dirty = true;
    viewer.view.layers_dirty = true;
    viewer.view.detail_dirty = true;
    viewer.view.browser_dirty = true;
    viewer.view.render_panel_dirty = true;
    viewer.view.inspector_dirty = true;
}
