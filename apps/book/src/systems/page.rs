pub mod blocks;

use nightshade::prelude::*;
use nightshade_ui::commands::builder::{UiBuild, UiTreeBuilder};
use nightshade_ui::components::{UiNodeColor, UiStateWeights, UiTreeNodeData};
use nightshade_ui::data::state::UiSelected;
use nightshade_ui::prelude::*;

use crate::commands::ink::BookInk;
use crate::commands::picture::Pictures;
use crate::data::style::{LABEL_SIZE, NAV_ITEM_HEIGHT, PAGE_MAX_WIDTH, PAGE_PADDING, SPACE_3};
use crate::queries::book::{breadcrumb, next_page, page_title, previous_page};
use crate::queries::screen::to_units;
use crate::resources::{BookState, Library, PagePhase, PageStage, Shell};

pub fn advance(
    mut state: ResMut<BookState>,
    mut page_stage: ResMut<PageStage>,
    mut pictures: ResMut<Pictures>,
    shell: Res<Shell>,
    library: Res<Library>,
    mut build: UiBuild,
) {
    let theme_name = build.theme().current_theme.name.clone();
    let type_scale = build.theme().type_scale_target;
    if state.phase == PagePhase::Idle
        && (state.theme_name != theme_name || state.type_scale != type_scale)
        && !library.chapters.is_empty()
    {
        state.theme_name = theme_name;
        state.type_scale = type_scale;
        if state.requested.is_none() {
            state.requested = Some(state.current);
        }
    }

    match state.phase {
        PagePhase::Idle => {
            let Some(index) = state.requested.take() else {
                return;
            };
            state.pending = Some(index);
            state.phase = PagePhase::Clear;
        }
        PagePhase::Clear => {
            *page_stage = PageStage::default();
            if let Some(root) = state.page_root.take() {
                ui_despawn_node(&mut build.context(), root);
            }
            clear_rail(&mut build, &mut state);
            state.phase = PagePhase::Build;
        }
        PagePhase::Build => {
            let Some(index) = state.pending.take() else {
                state.phase = PagePhase::Idle;
                return;
            };
            let Some(chapter) = library.chapters.get(index) else {
                state.phase = PagePhase::Idle;
                return;
            };

            let mut tree = UiTreeBuilder::from_parent(build.context(), shell.page_host);
            let root = tree
                .add_node()
                .flow_child(Ab(vec2(state.page_width.max(1.0), 0.0)))
                .auto_size(AutoSizeMode::Height)
                .flow(FlowDirection::Vertical, 0.0, 0.0)
                .entity();
            tree.in_parent(root, |tree| {
                blocks::build_blocks(tree, chapter, &mut page_stage, &mut pictures);
            });
            state.page_root = Some(root);

            state.rail_root = build_rail(&mut build, &shell, chapter);
            update_chrome(&mut build, &shell, &library, index);
            state.current = index;
            state.phase = PagePhase::Idle;
        }
    }
}

pub fn fit(
    mut state: ResMut<BookState>,
    page_stage: Res<PageStage>,
    shell: Res<Shell>,
    window: Res<nightshade_platform::prelude::Window>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut nodes: Query<(&mut UiLayoutNode,)>,
) {
    let mut across = 0.0;
    nodes.for_entity(shell.content_area, |(node,)| {
        across = to_units(node.computed_rect.width(), &window);
    });
    if across <= 0.0 {
        return;
    }
    let mut wanted = across - PAGE_PADDING * 2.0;
    if page_stage.expanded.is_none() {
        wanted = wanted.min(PAGE_MAX_WIDTH);
    }
    let wanted = wanted.max(1.0);
    if (wanted - state.page_width).abs() < 0.5 {
        return;
    }
    state.page_width = wanted;
    let Some(root) = state.page_root else {
        return;
    };
    nodes.for_entity(root, |(node,)| {
        node.flow_child_size = Some(Ab(vec2(wanted, 0.0)).into());
    });
    dirty.layout_dirty = true;
}

fn clear_rail(build: &mut UiBuild, state: &mut BookState) {
    let Some(root) = state.rail_root.take() else {
        return;
    };
    ui_despawn_node(&mut build.context(), root);
}

fn build_rail(
    build: &mut UiBuild,
    shell: &Shell,
    chapter: &crate::data::book::Chapter,
) -> Option<Entity> {
    if chapter.outline.is_empty() {
        return None;
    }

    let mut tree = UiTreeBuilder::from_parent(build.context(), shell.rail_host);
    let root = tree
        .add_node()
        .flow_child(Rl(vec2(100.0, 0.0)))
        .auto_size(AutoSizeMode::Height)
        .flow(FlowDirection::Vertical, 0.0, 2.0)
        .entity();
    tree.push_parent(root);
    add_rail_row(&mut tree, "On this page", 0.0, None);
    for heading in &chapter.outline {
        let indent = if heading.level >= 3 { SPACE_3 } else { 0.0 };
        add_rail_row(
            &mut tree,
            &heading.title,
            indent,
            Some(heading.anchor.clone()),
        );
    }
    tree.pop_parent();
    Some(root)
}

fn add_rail_row(tree: &mut UiTreeBuilder, title: &str, indent: f32, anchor: Option<String>) {
    let row = tree
        .add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, NAV_ITEM_HEIGHT - 4.0)))
        .rect(4.0)
        .color_raw::<UiBase>(vec4(0.0, 0.0, 0.0, 0.0))
        .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
        .flow(FlowDirection::Horizontal, 0.0, 0.0)
        .with_interaction()
        .with_transition::<UiHover>(10.0, 8.0)
        .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
        .entity();
    tree.in_parent(row, |tree| {
        if indent > 0.0 {
            tree.add_node().flow_child(Ab(vec2(indent, 0.0)));
        }
        tree.add_node()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(-(indent + SPACE_3), NAV_ITEM_HEIGHT - 4.0)))
            .with_text(title, LABEL_SIZE)
            .with_text_overflow(nightshade_ui::components::TextOverflow::Ellipsis)
            .text_left()
            .ink_dim();
    });
    tree.commands()
        .set(row, crate::components::RailLink { anchor });
}

fn update_chrome(build: &mut UiBuild, shell: &Shell, library: &Library, index: usize) {
    let previous = previous_page(library, index);
    let next = next_page(library, index);
    let crumb = breadcrumb(library, index);
    let previous_label = match previous {
        Some(page) => format!("\u{2190}  {}", page_title(library, page)),
        None => "\u{2190}  Start".to_string(),
    };
    let next_label = match next {
        Some(page) => format!("{}  \u{2192}", page_title(library, page)),
        None => "End  \u{2192}".to_string(),
    };
    let chosen = shell
        .nav_nodes
        .iter()
        .zip(shell.nav_chapters.iter())
        .find(|(_, chapter)| **chapter == index)
        .map(|(node, _)| *node);
    let nodes = shell.nav_nodes.clone();

    let mut context = build.context();
    ui_set_text(context.commands, shell.breadcrumb, &crumb);
    ui_set_text(context.commands, shell.previous_button, &previous_label);
    ui_set_text(context.commands, shell.next_button, &next_label);
    ui_set_disabled(context.commands, shell.previous_button, previous.is_none());
    ui_set_disabled(context.commands, shell.next_button, next.is_none());
    mark_chapter(&mut context, nodes, chosen);
}

fn mark_chapter(
    context: &mut nightshade_ui::commands::builder::UiBuildContext<'_, '_>,
    nodes: Vec<Entity>,
    chosen: Option<Entity>,
) {
    let accent = context.theme.active_theme().accent_color;
    let fill = vec4(accent.x, accent.y, accent.z, 0.22);
    context.commands.queue(move |storage, _| {
        for node in &nodes {
            let on = Some(*node) == chosen;
            let row = storage.get_mut::<UiTreeNodeData>(*node).map(|data| {
                data.selected = on;
                data.row_entity
            });
            let Some(row) = row else {
                continue;
            };
            if let Some(colors) = storage.get_mut::<UiNodeColor>(row) {
                colors.colors[UiSelected::INDEX] = on.then_some(fill);
            }
            if let Some(weights) = storage.get_mut::<UiStateWeights>(row) {
                weights.weights[UiSelected::INDEX] = if on { 1.0 } else { 0.0 };
            }
        }
    });
}
