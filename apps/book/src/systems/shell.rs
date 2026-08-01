use nightshade::prelude::*;
use nightshade_text::contract::data::text_data::{TextAlignment, VerticalAlignment};
use nightshade_ui::commands::builder::{UiBuild, UiTreeBuilder};
use nightshade_ui::components::{UiNodeColor, UiTreeNodeData, UiTreeViewData};
use nightshade_ui::prelude::*;

use crate::commands::ink::BookInk;
use crate::data::style::{
    APPEARANCE_BUTTON, APPEARANCE_MARK, APPEARANCE_PANEL_WIDTH, APPEARANCE_ROW, APPEARANCE_SWATCH,
    BACKDROPS, CAPTION_SIZE, DEMO_MARK, DEMO_ROW_HEIGHT, FOOTER_HEIGHT, HAIRLINE, LABEL_LINE,
    LABEL_SIZE, NAV_PART_HEIGHT, NAV_PART_SIZE, PAGE_PADDING, PROGRESS_HEIGHT, RAIL_WIDTH,
    SIDEBAR_WIDTH, SOURCE_HEADER_HEIGHT, SOURCE_PANEL_DEPTH, SPACE_2, SPACE_3, SPACE_4, SPACE_5,
    SPACE_6, TOP_BAR_HEIGHT,
};
use crate::resources::{Appearance, Library, Shell};
use nightshade_ui::data::icons::material;
use nightshade_ui::resources::theme::UiTheme;

const TITLE_SIZE: f32 = 19.0;

pub fn build_shell(
    mut shell: ResMut<Shell>,
    library: Res<Library>,
    appearance: Res<Appearance>,
    mut build: UiBuild,
) {
    let mut tree = UiTreeBuilder::new(build.context());

    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .entity();

    let mut built = Shell::default();

    tree.in_parent(root, |tree| {
        let bar = build_top_bar(tree);
        built.search_button = bar.search;
        built.appearance_button = bar.appearance;

        let sidebar = build_sidebar(tree, &library);
        built.nav_view = sidebar.view;
        built.expand_all = sidebar.expand_all;
        built.collapse_all = sidebar.collapse_all;
        built.nav_nodes = sidebar.nodes;
        built.nav_chapters = sidebar.chapters;
        built.nav_anchors = sidebar.anchors;
        built.demo_rows = sidebar.demo_rows;
        built.demo_targets = sidebar.demo_targets;

        let frame = build_content_frame(tree);
        built.content_area = frame.content_area;
        built.page_host = frame.page_host;
        built.scroll_area = frame.scroll_area;
        built.backdrops = frame.backdrops;

        built.rail_host = build_rail_host(tree);

        let footer = build_footer(tree);
        built.breadcrumb = footer.breadcrumb;
        built.previous_button = footer.previous_button;
        built.next_button = footer.next_button;
        built.progress_fill = footer.progress_fill;

        built.palette = tree.add_command_palette(10);

        let source = build_source_panel(tree);
        built.source_panel = source.panel;
        built.source_host = source.host;
        built.source_title = source.title;
        built.source_close = source.close;

        let panel = build_appearance_panel(tree, &appearance);
        built.appearance_panel = panel.panel;
        built.theme_rows = panel.theme_rows;
        built.theme_marks = panel.theme_marks;
        built.size_rows = panel.size_rows;
        built.size_marks = panel.size_marks;
    });

    name_for_the_hand(&mut tree, &built);
    *shell = built;
}

fn name_for_the_hand(tree: &mut UiTreeBuilder, shell: &Shell) {
    let (accessibility, commands) = tree.accessibility_and_commands();
    ui_set_test_id(
        accessibility,
        commands,
        shell.appearance_button,
        "appearance",
    );
    ui_set_test_id(
        accessibility,
        commands,
        shell.appearance_panel,
        "appearance-panel",
    );
    ui_set_test_id(accessibility, commands, shell.content_area, "content");
    ui_set_test_id(accessibility, commands, shell.source_panel, "source-panel");
    for (index, row) in shell.theme_rows.iter().enumerate() {
        ui_set_test_id(accessibility, commands, *row, &format!("theme-{index}"));
    }
    for (index, row) in shell.size_rows.iter().enumerate() {
        ui_set_test_id(accessibility, commands, *row, &format!("size-{index}"));
    }
}

struct Bar {
    search: Entity,
    appearance: Entity,
}

fn build_top_bar(tree: &mut UiTreeBuilder) -> Bar {
    let bar = tree
        .add_node()
        .boundary(
            Rl(vec2(0.0, 0.0)),
            Ab(vec2(0.0, TOP_BAR_HEIGHT)) + Rl(vec2(100.0, 0.0)),
        )
        .rect(0.0)
        .fg(ThemeColor::Panel)
        .with_depth(UiDepthMode::Set(10.0))
        .entity();

    let mut search = Entity::default();
    let mut appearance = Entity::default();

    tree.in_parent(bar, |tree| {
        tree.add_node()
            .boundary(
                Rl(vec2(0.0, 100.0)) + Ab(vec2(0.0, -HAIRLINE)),
                Rl(vec2(100.0, 100.0)),
            )
            .rect(0.0)
            .fg(ThemeColor::Border);

        tree.add_node()
            .window(
                Ab(vec2(SPACE_6, TOP_BAR_HEIGHT * 0.5)),
                Ab(vec2(360.0, 26.0)),
                Anchor::CenterLeft,
            )
            .with_text("The Nightshade Book", TITLE_SIZE)
            .text_left()
            .ink();

        search = build_search_button(tree);

        appearance = build_appearance_button(tree);
    });

    Bar { search, appearance }
}

fn build_appearance_button(tree: &mut UiTreeBuilder) -> Entity {
    add_bar_button(tree, material::FORMAT_PAINT, SPACE_6 + APPEARANCE_BUTTON)
}

fn add_bar_button(
    tree: &mut UiTreeBuilder,
    icon: nightshade_ui::data::icons::IconGlyph,
    inset: f32,
) -> Entity {
    let theme = tree.active_theme().clone();
    let button = tree
        .add_node()
        .window(
            Rl(vec2(100.0, 0.0)) + Ab(vec2(-inset, TOP_BAR_HEIGHT * 0.5)),
            Ab(vec2(APPEARANCE_BUTTON, APPEARANCE_BUTTON)),
            Anchor::CenterLeft,
        )
        .with_rect(6.0, theme.border_width, theme.border_color)
        .with_theme_border_color(ThemeColor::Border)
        .with_theme_color::<UiBase>(ThemeColor::InputBackground)
        .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
        .with_interaction()
        .with_transition::<UiHover>(10.0, 8.0)
        .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
        .entity();

    tree.in_parent(button, |tree| {
        tree.add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .with_icon(icon, 18.0)
            .text_center()
            .ink();
    });

    button
}

fn build_search_button(tree: &mut UiTreeBuilder) -> Entity {
    let theme = tree.active_theme().clone();
    let holder = tree
        .add_node()
        .window(
            Rl(vec2(100.0, 0.0)) + Ab(vec2(-460.0, TOP_BAR_HEIGHT * 0.5)),
            Ab(vec2(220.0, 30.0)),
            Anchor::CenterLeft,
        )
        .with_rect(6.0, theme.border_width, theme.border_color)
        .with_theme_border_color(ThemeColor::Border)
        .with_theme_color::<UiBase>(ThemeColor::InputBackground)
        .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
        .with_interaction()
        .with_transition::<UiHover>(10.0, 8.0)
        .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
        .flow(FlowDirection::Horizontal, SPACE_3, SPACE_2)
        .align_cross(FlowAlignment::Center)
        .entity();

    tree.in_parent(holder, |tree| {
        tree.add_node()
            .flow_child(Ab(vec2(0.0, LABEL_LINE)))
            .flex_grow(1.0)
            .with_text("Search the book", LABEL_SIZE)
            .text_left()
            .ink_dim();
        tree.add_node()
            .flow_child(Ab(vec2(12.0, LABEL_LINE)))
            .with_text("/", LABEL_SIZE)
            .text_center()
            .ink_dim();
    });

    holder
}

struct Sidebar {
    view: Entity,
    expand_all: Entity,
    collapse_all: Entity,
    nodes: Vec<Entity>,
    chapters: Vec<usize>,
    anchors: Vec<Option<String>>,
    demo_rows: Vec<Entity>,
    demo_targets: Vec<(usize, Option<String>)>,
}

fn build_sidebar(tree: &mut UiTreeBuilder, library: &Library) -> Sidebar {
    let sidebar = tree
        .add_node()
        .boundary(
            Ab(vec2(0.0, TOP_BAR_HEIGHT)),
            Ab(vec2(SIDEBAR_WIDTH, -FOOTER_HEIGHT)) + Rl(vec2(0.0, 100.0)),
        )
        .rect(0.0)
        .fg(ThemeColor::Panel)
        .with_depth(UiDepthMode::Set(10.0))
        .entity();

    let mut built = Sidebar {
        view: Entity::default(),
        expand_all: Entity::default(),
        collapse_all: Entity::default(),
        nodes: Vec::new(),
        chapters: Vec::new(),
        anchors: Vec::new(),
        demo_rows: Vec::new(),
        demo_targets: Vec::new(),
    };

    tree.in_parent(sidebar, |tree| {
        tree.add_node()
            .boundary(
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-HAIRLINE, 0.0)),
                Rl(vec2(100.0, 100.0)),
            )
            .rect(0.0)
            .fg(ThemeColor::Border);

        let list = tree
            .add_node()
            .boundary(
                Ab(vec2(SPACE_3, SPACE_3)),
                Rl(vec2(100.0, 100.0)) + Ab(vec2(-(SPACE_3 + HAIRLINE), -SPACE_3)),
            )
            .flow(FlowDirection::Vertical, 0.0, SPACE_2)
            .entity();

        tree.in_parent(list, |tree| {
            let folds = add_fold_controls(tree);
            built.expand_all = folds.expand;
            built.collapse_all = folds.collapse;
            let host = tree
                .add_node()
                .flow_child(Rl(vec2(100.0, 0.0)))
                .flex_grow(1.0)
                .entity();
            tree.in_parent(host, |tree| {
                built.view = tree.add_tree_view(false);
            });
            build_nav(tree, library, &mut built);
        });
    });

    built
}

struct FoldControls {
    expand: Entity,
    collapse: Entity,
}

fn add_fold_controls(tree: &mut UiTreeBuilder) -> FoldControls {
    let row = tree
        .add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, NAV_PART_HEIGHT)))
        .flow(FlowDirection::Horizontal, 0.0, SPACE_3)
        .align_cross(FlowAlignment::Center)
        .entity();

    let mut expand = Entity::default();
    let mut collapse = Entity::default();

    tree.in_parent(row, |tree| {
        tree.add_node()
            .flow_child(Ab(vec2(0.0, LABEL_LINE)))
            .flex_grow(1.0);
        expand = add_fold_word(tree, "Expand all");
        collapse = add_fold_word(tree, "Collapse all");
    });

    FoldControls { expand, collapse }
}

fn add_fold_word(tree: &mut UiTreeBuilder, label: &str) -> Entity {
    tree.add_node()
        .flow_child(Ab(vec2(0.0, LABEL_LINE)))
        .auto_size(AutoSizeMode::Width)
        .with_text(label, NAV_PART_SIZE)
        .text_left()
        .ink_dim()
        .fg_hover(ThemeColor::Accent)
        .with_interaction()
        .with_transition::<UiHover>(12.0, 8.0)
        .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
        .entity()
}

fn build_nav(tree: &mut UiTreeBuilder, library: &Library, built: &mut Sidebar) {
    let view = built.view;
    let root = tree
        .component::<UiTreeViewData>(view)
        .map(|data| data.content_entity)
        .unwrap_or(view);

    let mut containers: Vec<Entity> = vec![root];
    let mut last_part: Option<&str> = None;

    for (index, chapter) in library.chapters.iter().enumerate() {
        if let Some(part) = chapter.part
            && last_part != Some(part)
        {
            let first = last_part.is_none();
            tree.in_parent(root, |tree| add_part_heading(tree, part, first));
            last_part = Some(part);
        }

        let depth = chapter.depth.min(containers.len() - 1);
        let parent = containers[depth];
        let label = format!("{}  {}", chapter.number, chapter.title);
        let node = tree.add_tree_node(view, parent, &label, depth, index as u64);

        let children = tree
            .component::<UiTreeNodeData>(node)
            .map(|data| data.children_container);
        if let Some(children) = children {
            containers.truncate(depth + 1);
            containers.push(children);
        }

        built.nodes.push(node);
        built.chapters.push(index);
        built.anchors.push(None);

        if let Some(demo) = &chapter.demo {
            add_demo_mark(tree, node, demo, built);
            built.demo_targets.push((index, None));
        }

        {
            let mut context = tree.context_reborrow();
            ui_tree_node_expand_on_build(&mut context, node, true);
        }

        if let Some(children) = children {
            add_heading_nodes(tree, view, children, chapter, index, depth, built);
        }
    }

    hide_leaf_arrows(tree, library, built);
}

fn add_heading_nodes(
    tree: &mut UiTreeBuilder,
    view: Entity,
    parent: Entity,
    chapter: &crate::data::book::Chapter,
    index: usize,
    depth: usize,
    built: &mut Sidebar,
) {
    let mut containers: Vec<Entity> = vec![parent];
    let mut levels: Vec<u8> = vec![1];

    for heading in &chapter.outline {
        while levels.len() > 1 && *levels.last().unwrap_or(&1) >= heading.level {
            levels.pop();
            containers.pop();
        }
        let parent = *containers.last().unwrap_or(&parent);
        let row_depth = depth + levels.len();
        let node = tree.add_tree_node(view, parent, &heading.title, row_depth, u64::MAX);

        built.nodes.push(node);
        built.chapters.push(index);
        built.anchors.push(Some(heading.anchor.clone()));

        if let Some(demo) = &heading.demo {
            add_demo_mark(tree, node, demo, built);
            built
                .demo_targets
                .push((index, Some(heading.anchor.clone())));
        }

        {
            let mut context = tree.context_reborrow();
            ui_tree_node_expand_on_build(&mut context, node, true);
        }

        if let Some(children) = tree
            .component::<UiTreeNodeData>(node)
            .map(|data| data.children_container)
        {
            containers.push(children);
            levels.push(heading.level);
        }
        hide_arrow(tree, node);
    }
}

fn add_demo_mark(tree: &mut UiTreeBuilder, node: Entity, demo: &str, built: &mut Sidebar) {
    let entry = crate::systems::demos::demo_entry(demo);
    let title = entry.map(|entry| entry.title).unwrap_or(demo);
    let blurb = entry.map(|entry| entry.blurb).unwrap_or("");

    let Some(container) = tree
        .component::<UiTreeNodeData>(node)
        .map(|data| data.children_container)
    else {
        return;
    };

    let mut row = Entity::default();
    tree.in_parent(container, |tree| {
        row = tree
            .add_node()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, DEMO_ROW_HEIGHT)))
            .rect(4.0)
            .color_raw::<UiBase>(vec4(0.0, 0.0, 0.0, 0.0))
            .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
            .with_interaction()
            .with_transition::<UiHover>(10.0, 8.0)
            .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
            .flow(FlowDirection::Horizontal, SPACE_2, SPACE_2)
            .align_cross(FlowAlignment::Center)
            .entity();

        tree.in_parent(row, |tree| {
            tree.add_node()
                .flow_child(Ab(vec2(DEMO_MARK, DEMO_MARK)))
                .with_icon(material::PLAY_ARROW, DEMO_MARK)
                .text_center()
                .ink();

            let column = tree
                .add_node()
                .flow_child(Ab(vec2(0.0, DEMO_ROW_HEIGHT - SPACE_2)))
                .flex_grow(1.0)
                .flow(FlowDirection::Vertical, 0.0, 0.0)
                .entity();

            tree.in_parent(column, |tree| {
                tree.add_node()
                    .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, LABEL_LINE)))
                    .with_text(title, NAV_PART_SIZE)
                    .text_left()
                    .with_text_overflow(TextOverflow::Ellipsis)
                    .ink();

                tree.add_node()
                    .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, LABEL_LINE)))
                    .with_text(blurb, NAV_PART_SIZE - 1.0)
                    .text_left()
                    .with_text_overflow(TextOverflow::Ellipsis)
                    .ink_dim();
            });

            tree.add_node()
                .flow_child(Ab(vec2(DEMO_MARK, DEMO_MARK)))
                .with_icon(material::ARROW_FORWARD, DEMO_MARK)
                .text_center()
                .ink_dim();
        });
    });

    built.demo_rows.push(row);
}

fn hide_leaf_arrows(tree: &mut UiTreeBuilder, library: &Library, built: &Sidebar) {
    for (position, node) in built.nodes.iter().enumerate() {
        if built.anchors.get(position).is_none_or(Option::is_some) {
            continue;
        }
        let Some(index) = built.chapters.get(position).copied() else {
            continue;
        };
        let Some(chapter) = library.chapters.get(index) else {
            continue;
        };
        if !chapter.outline.is_empty() {
            continue;
        }
        let deeper = library
            .chapters
            .get(index + 1)
            .is_some_and(|next| next.depth > chapter.depth);
        if deeper {
            continue;
        }
        hide_arrow(tree, *node);
    }
}

fn hide_arrow(tree: &mut UiTreeBuilder, node: Entity) {
    let Some(arrow) = tree
        .component::<UiTreeNodeData>(node)
        .map(|data| data.arrow_entity)
    else {
        return;
    };
    tree.edit::<UiThemeBinding>(arrow, |binding| {
        binding.color_roles[UiBase::INDEX] = None;
    });
    tree.edit::<UiNodeColor>(arrow, |color| {
        color.colors[UiBase::INDEX] = Some(vec4(0.0, 0.0, 0.0, 0.0));
    });
}

fn add_part_heading(tree: &mut UiTreeBuilder, part: &str, first: bool) {
    if !first {
        tree.add_spacing(SPACE_4);
    }
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, NAV_PART_HEIGHT)))
        .with_text(&part.to_uppercase(), NAV_PART_SIZE)
        .text_left()
        .ink_dim();
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, HAIRLINE)))
        .rect(0.0)
        .fg(ThemeColor::Border);
    tree.add_spacing(SPACE_2);
}

struct ContentFrame {
    content_area: Entity,
    page_host: Entity,
    scroll_area: Entity,
    backdrops: Vec<Entity>,
}

fn build_content_frame(tree: &mut UiTreeBuilder) -> ContentFrame {
    let content_area = tree
        .add_node()
        .boundary(
            Ab(vec2(SIDEBAR_WIDTH, TOP_BAR_HEIGHT)),
            Rl(vec2(100.0, 100.0)) + Ab(vec2(-RAIL_WIDTH, -FOOTER_HEIGHT)),
        )
        .entity();

    let mut page_host = Entity::default();
    let mut scroll_area = Entity::default();
    let mut backdrops = Vec::with_capacity(BACKDROPS);

    tree.in_parent(content_area, |tree| {
        for _ in 0..BACKDROPS {
            backdrops.push(add_backdrop(tree));
        }

        scroll_area = tree.add_scroll_area_fill(PAGE_PADDING, 0.0);
        page_host = scroll_content(tree, scroll_area).unwrap_or_default();
        tree.edit::<UiLayoutNode>(page_host, |node| {
            if let Some(flow) = node.flow_layout.as_mut() {
                flow.cross_alignment = FlowAlignment::Center;
            }
        });
    });

    ContentFrame {
        content_area,
        page_host,
        scroll_area,
        backdrops,
    }
}

fn add_backdrop(tree: &mut UiTreeBuilder) -> Entity {
    tree.add_node()
        .boundary(Rl(vec2(0.0, 100.0)), Rl(vec2(100.0, 100.0)))
        .rect(0.0)
        .fg(ThemeColor::Background)
        .entity()
}

fn build_rail_host(tree: &mut UiTreeBuilder) -> Entity {
    let rail = tree
        .add_node()
        .boundary(
            Rl(vec2(100.0, 0.0)) + Ab(vec2(-RAIL_WIDTH, TOP_BAR_HEIGHT)),
            Rl(vec2(100.0, 100.0)) + Ab(vec2(0.0, -FOOTER_HEIGHT)),
        )
        .rect(0.0)
        .fg(ThemeColor::Background)
        .entity();

    let mut host = Entity::default();
    tree.in_parent(rail, |tree| {
        host = tree
            .add_node()
            .boundary(
                Ab(vec2(0.0, SPACE_5)),
                Rl(vec2(100.0, 100.0)) + Ab(vec2(-SPACE_5, 0.0)),
            )
            .flow(FlowDirection::Vertical, 0.0, 2.0)
            .entity();
    });
    host
}

struct Footer {
    breadcrumb: Entity,
    previous_button: Entity,
    next_button: Entity,
    progress_fill: Entity,
}

fn build_footer(tree: &mut UiTreeBuilder) -> Footer {
    let footer = tree
        .add_node()
        .boundary(
            Rl(vec2(0.0, 100.0)) + Ab(vec2(0.0, -FOOTER_HEIGHT)),
            Rl(vec2(100.0, 100.0)),
        )
        .rect(0.0)
        .fg(ThemeColor::Panel)
        .with_depth(UiDepthMode::Set(10.0))
        .entity();

    let mut breadcrumb = Entity::default();
    let mut previous_button = Entity::default();
    let mut next_button = Entity::default();
    let mut progress_fill = Entity::default();

    tree.in_parent(footer, |tree| {
        tree.add_node()
            .boundary(
                Rl(vec2(0.0, 0.0)),
                Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, HAIRLINE)),
            )
            .rect(0.0)
            .fg(ThemeColor::Border);

        progress_fill = tree
            .add_node()
            .boundary(
                Rl(vec2(0.0, 0.0)),
                Rl(vec2(0.0, 0.0)) + Ab(vec2(0.0, PROGRESS_HEIGHT)),
            )
            .rect(0.0)
            .fg(ThemeColor::Accent)
            .entity();

        previous_button = add_page_link(tree, Anchor::CenterLeft, Ab(vec2(SPACE_6, 0.0)).into());
        next_button = add_page_link(
            tree,
            Anchor::CenterRight,
            Rl(vec2(100.0, 0.0)) + Ab(vec2(-SPACE_6, 0.0)),
        );

        breadcrumb = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, FOOTER_HEIGHT * 0.5)),
                Ab(vec2(420.0, 20.0)),
                Anchor::Center,
            )
            .with_text("", CAPTION_SIZE)
            .text_center()
            .ink_dim()
            .entity();
    });

    Footer {
        breadcrumb,
        previous_button,
        next_button,
        progress_fill,
    }
}

fn add_page_link(tree: &mut UiTreeBuilder, anchor: Anchor, position: UiValue<Vec2>) -> Entity {
    let position: UiValue<Vec2> = position + Ab(vec2(0.0, FOOTER_HEIGHT * 0.5));
    tree.add_node()
        .window(position, Ab(vec2(240.0, 28.0)), anchor)
        .with_text("", LABEL_SIZE + 1.0)
        .ink_dim()
        .fg_hover(ThemeColor::Accent)
        .with_interaction()
        .with_transition::<UiHover>(10.0, 8.0)
        .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
        .with_text_alignment(
            match anchor {
                Anchor::CenterRight => TextAlignment::Right,
                _ => TextAlignment::Left,
            },
            VerticalAlignment::Middle,
        )
        .entity()
}

fn scroll_content(tree: &mut UiTreeBuilder, scroll_area: Entity) -> Option<Entity> {
    tree.commands()
        .staged::<UiScrollAreaData>(scroll_area)
        .map(|data| data.content_entity)
}

pub fn fold_tree(
    mut state: ResMut<crate::resources::BookState>,
    shell: Res<Shell>,
    mut build: UiBuild,
    storage: &mut nightshade::ecs::Storage,
) {
    if let Some(expanded) = state.fold_all.take() {
        let nodes = shell.nav_nodes.clone();
        let mut context = build.context();
        for node in nodes {
            ui_tree_node_set_expanded(&mut context, storage, node, expanded);
        }
    }
}

struct AppearancePanel {
    panel: Entity,
    theme_rows: Vec<Entity>,
    theme_marks: Vec<Entity>,
    size_rows: Vec<Entity>,
    size_marks: Vec<Entity>,
}

fn build_appearance_panel(tree: &mut UiTreeBuilder, appearance: &Appearance) -> AppearancePanel {
    let theme = tree.active_theme().clone();
    let presets: Vec<UiTheme> = tree.theme_state().presets.clone();
    let selected = tree.theme_state().selected_preset_index.unwrap_or(0);

    let mut parts = vec![NAV_PART_HEIGHT];
    parts.extend(presets.iter().map(|_| APPEARANCE_ROW));
    parts.push(HAIRLINE);
    parts.push(NAV_PART_HEIGHT);
    parts.extend(
        appearance
            .sizes
            .iter()
            .map(|(_, scale)| (APPEARANCE_ROW * scale).max(APPEARANCE_ROW)),
    );
    let height = SPACE_4 * 2.0 + parts.iter().sum::<f32>() + SPACE_2 * (parts.len() as f32 - 1.0);

    let panel = tree
        .add_node()
        .window(
            Rl(vec2(100.0, 0.0)) + Ab(vec2(-SPACE_6, TOP_BAR_HEIGHT + SPACE_2)),
            Ab(vec2(APPEARANCE_PANEL_WIDTH, height)),
            Anchor::TopRight,
        )
        .with_rect(10.0, theme.border_width, theme.border_color)
        .with_theme_border_color(ThemeColor::Border)
        .with_theme_color::<UiBase>(ThemeColor::Panel)
        .with_shadow(
            theme.panel_shadow.color,
            theme.panel_shadow.offset,
            theme.panel_shadow.blur,
            theme.panel_shadow.spread,
        )
        .with_layer(UiLayer::Popups)
        .with_depth(UiDepthMode::Set(60.0))
        .with_interaction()
        .flow(FlowDirection::Vertical, SPACE_4, SPACE_2)
        .with_visible(false)
        .entity();

    let mut built = AppearancePanel {
        panel,
        theme_rows: Vec::new(),
        theme_marks: Vec::new(),
        size_rows: Vec::new(),
        size_marks: Vec::new(),
    };

    tree.in_parent(panel, |tree| {
        add_panel_heading(tree, "Theme");
        for (index, preset) in presets.iter().enumerate() {
            let (row, mark) = add_theme_row(tree, preset, index == selected);
            built.theme_rows.push(row);
            built.theme_marks.push(mark);
        }

        add_panel_rule(tree);
        add_panel_heading(tree, "Text size");
        for (index, (name, scale)) in appearance.sizes.iter().enumerate() {
            let (row, mark) = add_size_row(tree, name, *scale, (*scale - 1.0).abs() < 0.001);
            built.size_rows.push(row);
            built.size_marks.push(mark);
            let _ = index;
        }
    });

    built
}

fn add_panel_heading(tree: &mut UiTreeBuilder, text: &str) {
    tree.add_node()
        .flow_child(Ab(vec2(0.0, NAV_PART_HEIGHT)))
        .with_text(text, NAV_PART_SIZE)
        .text_left()
        .ink_dim();
}

fn add_panel_rule(tree: &mut UiTreeBuilder) {
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, HAIRLINE)))
        .rect(0.0)
        .fg(ThemeColor::Border);
}

fn add_theme_row(tree: &mut UiTreeBuilder, preset: &UiTheme, selected: bool) -> (Entity, Entity) {
    let row = tree
        .add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, APPEARANCE_ROW)))
        .with_rect(6.0, 1.0, preset.border_color)
        .color_raw::<UiBase>(preset.panel_color)
        .color_raw::<UiHover>(preset.background_color_hovered)
        .with_interaction()
        .with_transition::<UiHover>(10.0, 8.0)
        .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
        .flow(FlowDirection::Horizontal, SPACE_3, SPACE_2)
        .align_cross(FlowAlignment::Center)
        .entity();

    let mut mark = Entity::default();
    tree.in_parent(row, |tree| {
        tree.add_node()
            .flow_child(Ab(vec2(APPEARANCE_SWATCH, APPEARANCE_SWATCH)))
            .with_rect(3.0, 0.0, preset.accent_color)
            .color_raw::<UiBase>(preset.accent_color);

        tree.add_node()
            .flow_child(Ab(vec2(0.0, LABEL_LINE)))
            .flex_grow(1.0)
            .with_text(&preset.name, LABEL_SIZE)
            .text_left()
            .ink();

        mark = tree
            .add_node()
            .flow_child(Ab(vec2(APPEARANCE_MARK, APPEARANCE_MARK)))
            .with_icon(material::CHECK, APPEARANCE_MARK)
            .text_center()
            .ink()
            .with_visible(selected)
            .entity();
    });

    (row, mark)
}

fn add_size_row(
    tree: &mut UiTreeBuilder,
    name: &str,
    scale: f32,
    selected: bool,
) -> (Entity, Entity) {
    let height = (APPEARANCE_ROW * scale).max(APPEARANCE_ROW);
    let row = tree
        .add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, height)))
        .rect(6.0)
        .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
        .color_raw::<UiBase>(vec4(0.0, 0.0, 0.0, 0.0))
        .with_interaction()
        .with_transition::<UiHover>(10.0, 8.0)
        .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
        .flow(FlowDirection::Horizontal, SPACE_3, SPACE_2)
        .align_cross(FlowAlignment::Center)
        .entity();

    let mut mark = Entity::default();
    tree.in_parent(row, |tree| {
        tree.add_node()
            .flow_child(Ab(vec2(0.0, LABEL_LINE * scale)))
            .flex_grow(1.0)
            .with_text(name, LABEL_SIZE * scale)
            .text_left()
            .ink();

        mark = tree
            .add_node()
            .flow_child(Ab(vec2(APPEARANCE_MARK, APPEARANCE_MARK)))
            .with_icon(material::CHECK, APPEARANCE_MARK)
            .text_center()
            .ink()
            .with_visible(selected)
            .entity();
    });

    (row, mark)
}

struct SourcePanel {
    panel: Entity,
    host: Entity,
    title: Entity,
    close: Entity,
}

fn build_source_panel(tree: &mut UiTreeBuilder) -> SourcePanel {
    let theme = tree.active_theme().clone();

    let panel = tree
        .add_node()
        .boundary(
            Ab(vec2(0.0, TOP_BAR_HEIGHT)),
            Rl(vec2(100.0, 100.0)) + Ab(vec2(0.0, -FOOTER_HEIGHT)),
        )
        .rect(0.0)
        .with_theme_color::<UiBase>(ThemeColor::Background)
        .with_depth(UiDepthMode::Set(SOURCE_PANEL_DEPTH))
        .with_interaction()
        .with_visible(false)
        .flow(FlowDirection::Vertical, 0.0, 0.0)
        .entity();

    let mut title = Entity::default();
    let mut close = Entity::default();
    let mut host = Entity::default();

    tree.in_parent(panel, |tree| {
        let header = tree
            .add_node()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, SOURCE_HEADER_HEIGHT)))
            .rect(0.0)
            .fg(ThemeColor::PanelHeader)
            .flow(FlowDirection::Horizontal, SPACE_5, SPACE_3)
            .align_cross(FlowAlignment::Center)
            .entity();

        tree.in_parent(header, |tree| {
            tree.add_node()
                .flow_child(Ab(vec2(16.0, LABEL_LINE)))
                .with_icon(material::CODE, 15.0)
                .text_center()
                .ink();

            title = tree
                .add_node()
                .flow_child(Ab(vec2(0.0, LABEL_LINE)))
                .flex_grow(1.0)
                .with_text("Source", TITLE_SIZE - 4.0)
                .text_left()
                .ink()
                .entity();

            close = tree
                .add_node()
                .flow_child(Ab(vec2(APPEARANCE_BUTTON, APPEARANCE_BUTTON)))
                .with_rect(6.0, theme.border_width, theme.border_color)
                .with_theme_border_color(ThemeColor::Border)
                .with_theme_color::<UiBase>(ThemeColor::InputBackground)
                .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
                .with_interaction()
                .with_transition::<UiHover>(10.0, 8.0)
                .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
                .with_children(|tree| {
                    tree.add_node()
                        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
                        .with_icon(material::CLOSE, 15.0)
                        .text_center()
                        .ink();
                })
                .entity();
        });

        tree.add_node()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, HAIRLINE)))
            .rect(0.0)
            .fg(ThemeColor::Border);

        host = tree
            .add_node()
            .flow_child(Rl(vec2(100.0, 0.0)))
            .flex_grow(1.0)
            .with_clip()
            .entity();
    });

    SourcePanel {
        panel,
        host,
        title,
        close,
    }
}
