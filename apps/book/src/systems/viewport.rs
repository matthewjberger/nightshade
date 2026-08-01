use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_render::data::config::{RenderSettings, ViewportRect};
use nightshade_render::resources::viewport::Viewport;
use nightshade_ui::components::UiLayoutNode;
use nightshade_ui::components::layout_types::{BoundaryLayout, UiLayoutType};
use nightshade_ui::prelude::*;

use crate::queries::screen::{point_to_units, to_pixels};
use crate::resources::{DemoRuntime, PageStage, Shell, SourceView};

const MINIMUM_STAGE_SIZE: f32 = 48.0;

pub fn track_stage(
    mut viewport: ResMut<Viewport>,
    mut settings: ResMut<RenderSettings>,
    mut dirty: ResMut<RetainedUiDirty>,
    runtime: Res<DemoRuntime>,
    view: Res<SourceView>,
    shell: Res<Shell>,
    window: Res<Window>,
    theme: Res<ThemeState>,
    page_stage: Res<PageStage>,
    mut nodes: Query<(&mut UiLayoutNode,)>,
) {
    let background = theme.active_theme().background_color;
    settings.clear_color = [
        to_linear(background.x),
        to_linear(background.y),
        to_linear(background.z),
        background.w,
    ];
    settings.render_world_to_swapchain = false;
    viewport.active_viewport_rect = None;
    viewport.required_cameras.clear();
    viewport.camera_tile_rects.clear();

    let mut content = Rect::default();
    nodes.for_entity(shell.content_area, |(node,)| {
        content = node.computed_rect;
    });

    if view.demo.is_some() {
        cover_all(&mut nodes, &mut dirty, &shell, &window, content);
        return;
    }

    let least = to_pixels(MINIMUM_STAGE_SIZE, &window);
    let mut holes: Vec<Rect> = Vec::new();

    for figure in &page_stage.figures {
        let mut placed = None;
        nodes.for_entity(figure.node, |(node,)| {
            if node.visible {
                placed = Some(node.computed_rect);
            }
        });
        let Some(rect) = placed
            .map(|rect| clamp_rect(rect, content))
            .filter(|rect| rect.width() >= least && rect.height() >= least)
        else {
            continue;
        };
        let Some(camera) = runtime.cameras.get(figure.demo).copied() else {
            continue;
        };

        viewport.required_cameras.push(camera);
        viewport.camera_tile_rects.insert(
            camera,
            ViewportRect {
                x: rect.min.x,
                y: rect.min.y,
                width: rect.width(),
                height: rect.height(),
            },
        );
        holes.push(rect);
    }

    if holes.is_empty() {
        cover_all(&mut nodes, &mut dirty, &shell, &window, content);
        return;
    }

    cover_around(&mut nodes, &mut dirty, &shell, &window, content, &mut holes);
}

fn cover_around(
    nodes: &mut Query<(&mut UiLayoutNode,)>,
    dirty: &mut RetainedUiDirty,
    shell: &Shell,
    window: &Window,
    content: Rect,
    holes: &mut [Rect],
) {
    holes.sort_by(|left, right| {
        left.min
            .y
            .partial_cmp(&right.min.y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let width = content.width();
    let height = content.height();
    let page = Rect::from_min_max(vec2(0.0, 0.0), vec2(width, height));

    let local: Vec<Rect> = holes
        .iter()
        .map(|rect| {
            let left = (rect.min.x - content.min.x).clamp(0.0, width);
            let right = (rect.max.x - content.min.x).clamp(left, width);
            let top = (rect.min.y - content.min.y).clamp(0.0, height);
            let bottom = (rect.max.y - content.min.y).clamp(top, height);
            Rect::from_min_max(vec2(left, top), vec2(right, bottom))
        })
        .collect();

    let mut pieces: Vec<Rect> = Vec::with_capacity(local.len() * 3 + 1);
    let mut edge = 0.0_f32;
    for hole in &local {
        pieces.push(Rect::from_min_max(
            vec2(0.0, edge),
            vec2(page.max.x, hole.min.y.max(edge)),
        ));
        pieces.push(beside_left(*hole));
        pieces.push(beside_right(*hole, page));
        edge = hole.max.y.max(edge);
    }
    pieces.push(Rect::from_min_max(vec2(0.0, edge), page.max));

    let closed = Rect::from_min_max(page.max, page.max);
    let mut moved = false;
    for (index, backdrop) in shell.backdrops.iter().enumerate() {
        let rect = pieces.get(index).copied().unwrap_or(closed);
        moved |= set_backdrop(nodes, window, *backdrop, content.min, rect);
    }
    if moved {
        dirty.layout_dirty = true;
    }
}

fn beside_left(hole: Rect) -> Rect {
    Rect::from_min_max(vec2(0.0, hole.min.y), vec2(hole.min.x, hole.max.y))
}

fn beside_right(hole: Rect, page: Rect) -> Rect {
    Rect::from_min_max(vec2(hole.max.x, hole.min.y), vec2(page.max.x, hole.max.y))
}

fn cover_all(
    nodes: &mut Query<(&mut UiLayoutNode,)>,
    dirty: &mut RetainedUiDirty,
    shell: &Shell,
    window: &Window,
    content: Rect,
) {
    let page = Rect::from_min_max(vec2(0.0, 0.0), vec2(content.width(), content.height()));
    let closed = Rect::from_min_max(page.max, page.max);
    let mut moved = false;
    for (index, backdrop) in shell.backdrops.iter().enumerate() {
        let rect = if index == 0 { page } else { closed };
        moved |= set_backdrop(nodes, window, *backdrop, content.min, rect);
    }
    if moved {
        dirty.layout_dirty = true;
    }
}

fn set_backdrop(
    nodes: &mut Query<(&mut UiLayoutNode,)>,
    window: &Window,
    entity: Entity,
    origin: Vec2,
    rect: Rect,
) -> bool {
    let layout = UiLayoutType::Boundary(BoundaryLayout {
        position_1: Ab(point_to_units(rect.min, window)).into(),
        position_2: Ab(point_to_units(rect.max, window)).into(),
    });
    let wanted = Rect::from_min_max(origin + rect.min, origin + rect.max);

    let mut moved = false;
    nodes.for_entity(entity, |(node,)| {
        let placed = node.computed_rect;
        let same = (placed.min.x - wanted.min.x).abs() < 0.5
            && (placed.min.y - wanted.min.y).abs() < 0.5
            && (placed.max.x - wanted.max.x).abs() < 0.5
            && (placed.max.y - wanted.max.y).abs() < 0.5;
        node.base_layout = Some(layout);
        moved = !same;
    });
    moved
}

fn clamp_rect(rect: Rect, bounds: Rect) -> Rect {
    if bounds.width() <= 1.0 || bounds.height() <= 1.0 {
        return rect;
    }
    let min = vec2(rect.min.x.max(bounds.min.x), rect.min.y.max(bounds.min.y));
    let max = vec2(rect.max.x.min(bounds.max.x), rect.max.y.min(bounds.max.y));
    Rect::from_min_max(min, vec2(max.x.max(min.x), max.y.max(min.y)))
}

fn to_linear(channel: f32) -> f32 {
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}
