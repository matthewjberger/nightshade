use nightshade::prelude::*;
use nightshade_ui::components::layout_types::{BoundaryLayout, UiLayoutType};
use nightshade_ui::components::{UiLayoutNode, UiScrollAreaData};
use nightshade_ui::prelude::*;

use crate::data::style::{PAGE_PADDING, PROGRESS_HEIGHT};
use crate::queries::screen::to_units;
use crate::resources::{BookState, PageStage, Reading, Shell};

pub fn track(
    mut reading: ResMut<Reading>,
    mut state: ResMut<BookState>,
    mut dirty: ResMut<RetainedUiDirty>,
    shell: Res<Shell>,
    page_stage: Res<PageStage>,
    window: Res<nightshade_platform::prelude::Window>,
    mut areas: Query<(&mut UiScrollAreaData,)>,
    mut nodes: Query<(&mut UiLayoutNode,)>,
) {
    let settled = !dirty.layout_dirty;
    let mut offset = 0.0;
    let mut reported_height = 0.0;
    areas.for_entity(shell.scroll_area, |(area,)| {
        offset = area.scroll_offset;
        reported_height = to_units(area.content_height, &window);
    });

    let mut area = Rect::default();
    nodes.for_entity(shell.content_area, |(node,)| {
        area = node.computed_rect;
    });
    let visible_height = to_units(area.height(), &window);
    let mut root_height = 0.0;
    if let Some(root) = state.page_root {
        nodes.for_entity(root, |(node,)| {
            root_height = to_units(node.computed_rect.height(), &window);
        });
    }

    let measured = root_height > 1.0 && visible_height > 0.0;
    let content_height = if measured {
        (root_height + PAGE_PADDING * 2.0).max(reported_height)
    } else {
        0.0
    };
    let reach = (content_height - visible_height).max(0.0);

    if measured
        && settled
        && let Some(wanted) = state.scroll_to.take()
    {
        offset = wanted.clamp(0.0, reach);
        areas.for_entity(shell.scroll_area, |(area,)| {
            area.scroll_offset = offset;
        });
        dirty.layout_dirty = true;
    }

    if settled && let Some(anchor) = state.jump_to.take() {
        offset = heading_offset(
            &page_stage,
            &mut nodes,
            &window,
            &anchor,
            shell.page_host,
            offset,
        );
        if measured {
            offset = offset.clamp(0.0, reach);
        }
        areas.for_entity(shell.scroll_area, |(area,)| {
            area.scroll_offset = offset;
        });
        dirty.layout_dirty = true;
    }

    if measured && offset > reach {
        offset = reach;
        areas.for_entity(shell.scroll_area, |(area,)| {
            area.scroll_offset = offset;
        });
        dirty.layout_dirty = true;
    }

    reading.height = content_height;
    reading.visible = visible_height;
    reading.offset = offset;
    reading.progress = progress(offset, content_height, visible_height);

    nodes.for_entity(shell.progress_fill, |(node,)| {
        node.base_layout = Some(UiLayoutType::Boundary(BoundaryLayout {
            position_1: Rl(vec2(0.0, 0.0)).into(),
            position_2: Rl(vec2(reading.progress * 100.0, 0.0)) + Ab(vec2(0.0, PROGRESS_HEIGHT)),
        }));
    });
}

fn progress(offset: f32, height: f32, visible: f32) -> f32 {
    if height <= 0.0 || visible <= 0.0 {
        return 0.0;
    }
    let reach = height - visible;
    if reach <= 1.0 {
        return 0.0;
    }
    (offset / reach).clamp(0.0, 1.0)
}

fn heading_offset(
    page_stage: &PageStage,
    nodes: &mut Query<(&mut UiLayoutNode,)>,
    window: &nightshade_platform::prelude::Window,
    anchor: &str,
    host: Entity,
    fallback: f32,
) -> f32 {
    let Some((_, entity)) = page_stage.headings.iter().find(|(name, _)| name == anchor) else {
        return fallback;
    };

    let mut heading_top = None;
    nodes.for_entity(*entity, |(node,)| {
        heading_top = Some(node.computed_rect.min.y);
    });
    let mut host_top = None;
    nodes.for_entity(host, |(node,)| {
        host_top = Some(node.computed_rect.min.y);
    });

    match (heading_top, host_top) {
        (Some(heading), Some(top)) => to_units((heading - top).max(0.0), window),
        _ => fallback,
    }
}
