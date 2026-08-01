use crate::components::{BookmarkRow, LinkRow, PageBlock};
use crate::data::ask::Ask;
use crate::data::machine::Screen;
use crate::data::web::{BOOKMARKS, Block, HOME, PAGES, answered, bookmarks, page_at, tidy};
use crate::resources::{AppResources, Ink};
use crate::systems::desk::WindowRow;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::{Input, Time};
use nightshade_ui::commands::builder::UiBuild;
use nightshade_ui::prelude::*;

const SPINNER: [&str; 4] = ["|", "/", "-", "\\"];

pub fn go(resources: &mut AppResources, url: &str, record: bool) {
    let wanted = tidy(url);
    resources.browser.address = wanted.clone();
    resources.browser.address_focused = false;
    resources.browser.page_typed.clear();
    resources.browser.page_answered = None;
    resources.browser.page_refused = false;
    resources.browser.folder_open = false;
    resources.browser.loading = 0.0;

    match page_at(&wanted) {
        Some(index) => {
            resources.browser.not_found = false;
            resources.browser.browsing = Some(index);
            resources.browser.load_wants = PAGES[index].load_seconds;
            if record {
                resources
                    .browser
                    .history
                    .truncate(resources.browser.history_at + 1);
                if resources.browser.history.last() != Some(&index) {
                    resources.browser.history.push(index);
                }
                resources.browser.history_at = resources.browser.history.len().saturating_sub(1);
            }
        }
        None => {
            resources.browser.not_found = true;
            resources.session.cues.push(crate::resources::Cue::Refused);
            resources.browser.browsing = None;
            resources.browser.load_wants = 2.9;
        }
    }
    resources.redraw.machine = true;
}

fn browser_ahead(resources: &AppResources, windows: &[WindowRow]) -> bool {
    let browser = crate::systems::setup::program_index("browser");
    crate::systems::desk::front_program(windows) == Some(browser)
        && resources.desk.screen == Screen::Desktop
}

pub fn advance(
    mut resources: ResMut<AppResources>,
    time: Res<Time>,
    input: Res<Input>,
    actions: Res<Actions<Ask>>,
    mut gathered: ResMut<crate::systems::desk::Gathered>,
    mut build: UiBuild,
    nodes: Query<&mut UiLayoutNode>,
    contents: Query<&mut UiNodeContent>,
) {
    let mut ink = Ink {
        build: build.context(),
        nodes,
        contents,
        gathered: &mut gathered,
    };
    let ink = &mut ink;
    let resources = &mut *resources;
    let program = crate::systems::setup::program_index("browser");
    if resources.desk.screen != Screen::Desktop
        || crate::systems::screensaver::covering(resources)
        || !crate::systems::desk::showing(&ink.gathered.windows, program)
    {
        return;
    }
    if resources.browser.browsing.is_none()
        && !resources.browser.not_found
        && resources.browser.address.is_empty()
    {
        go(resources, HOME, true);
    }

    let delta = time.delta_time;
    if resources.browser.loading < resources.browser.load_wants {
        resources.browser.loading =
            (resources.browser.loading + delta).min(resources.browser.load_wants);
        resources.redraw.machine = true;
        if resources.browser.loading >= resources.browser.load_wants
            && let Some(index) = resources.browser.browsing
        {
            crate::resources::learn(resources, PAGES[index].asserts);
        }
    }

    if !browser_ahead(resources, &ink.gathered.windows) {
        return;
    }
    let typed: Vec<char> = input
        .keyboard
        .frame_chars
        .iter()
        .copied()
        .filter(|character| !character.is_control())
        .collect();
    let backspace = actions.just_pressed(Ask::Erase);
    let submit = actions.just_pressed(Ask::Submit);

    if resources.browser.address_focused {
        if backspace {
            resources.browser.address.pop();
            resources.redraw.machine = true;
        }
        if !typed.is_empty() {
            resources.browser.address.extend(typed);
            resources.redraw.machine = true;
        }
        if submit {
            let wanted = resources.browser.address.clone();
            go(resources, &wanted, true);
        }
        return;
    }

    let has_field = resources
        .browser
        .browsing
        .and_then(|index| PAGES[index].prompt.as_ref())
        .is_some();
    if !has_field || loading(resources) {
        return;
    }
    if backspace {
        resources.browser.page_typed.pop();
        resources.redraw.machine = true;
    }
    if !typed.is_empty() {
        resources.browser.page_typed.extend(typed);
        resources.redraw.machine = true;
    }
    if submit {
        submit_field(resources);
    }
}

pub fn loading(resources: &AppResources) -> bool {
    resources.browser.loading < resources.browser.load_wants
}

pub fn submit_field(resources: &mut AppResources) {
    let Some(index) = resources.browser.browsing else {
        return;
    };
    let Some(prompt) = PAGES[index].prompt.as_ref() else {
        return;
    };
    if let Some(which) = answered(prompt, &resources.browser.page_typed) {
        resources.browser.page_answered = Some(which);
        resources.browser.page_refused = false;
        crate::resources::learn(resources, prompt.answers[which].asserts);
    } else {
        resources.browser.page_answered = None;
        resources.browser.page_refused = !resources.browser.page_typed.trim().is_empty();
        if resources.browser.page_refused {
            resources.session.cues.push(crate::resources::Cue::Refused);
        }
    }
    resources.redraw.machine = true;
}

pub fn back(resources: &mut AppResources) {
    if resources.browser.history_at == 0 {
        return;
    }
    resources.browser.history_at -= 1;
    let index = resources.browser.history[resources.browser.history_at];
    let url = PAGES[index].url;
    go(resources, url, false);
}

pub fn forward(resources: &mut AppResources) {
    if resources.browser.history_at + 1 >= resources.browser.history.len() {
        return;
    }
    resources.browser.history_at += 1;
    let index = resources.browser.history[resources.browser.history_at];
    let url = PAGES[index].url;
    go(resources, url, false);
}

pub fn reload(resources: &mut AppResources) {
    let url = resources.browser.address.clone();
    go(resources, &url, false);
}

pub fn refresh(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    viewport: Vec2,
    config: &crate::data::config::Config,
) {
    let running = loading(resources);
    let page = resources.browser.browsing.map(|index| &PAGES[index]);

    ui_set_text(
        ink.build.commands,
        resources.interface.browser_tab,
        match (running, page) {
            (true, _) => "Loading\u{2026}",
            (false, Some(page)) => page.title,
            (false, None) => "This site can't be reached",
        },
    );
    let turn = (resources.session.dots_phase * 9.0) as usize % SPINNER.len();
    ui_set_text(
        ink.build.commands,
        resources.interface.browser_spinner,
        if running { SPINNER[turn] } else { "o" },
    );

    let focused = resources.browser.address_focused;
    let caret = if focused && resources.login.caret_lit {
        "|"
    } else {
        ""
    };
    ui_set_text(
        ink.build.commands,
        resources.interface.browser_caret,
        &format!("{}{}", resources.browser.address, caret),
    );
    ui_set_color::<UiBase>(
        ink.build.commands,
        resources.interface.browser_address,
        if focused {
            config.address_focused
        } else {
            config.address_field
        },
    );

    let width = window_width(&ink.gathered.windows, viewport, config);
    let share = if resources.browser.load_wants <= 0.0 {
        1.0
    } else {
        (resources.browser.loading / resources.browser.load_wants).clamp(0.0, 1.0)
    };
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.browser_progress,
        running,
    );
    ink.nodes
        .for_entity(resources.interface.browser_progress, |node| {
            node.base_layout = Some(UiLayoutType::Window(WindowLayout {
                position: Ab(vec2(0.0, 70.0)).into(),
                size: Ab(vec2(width * share, 2.0)).into(),
                anchor: Anchor::TopLeft,
            }));
        });

    refresh_bookmarks(resources, ink, config);
    refresh_page(resources, ink, config, running);
}

fn window_width(
    windows: &[WindowRow],
    viewport: Vec2,
    config: &crate::data::config::Config,
) -> f32 {
    let browser = crate::systems::setup::program_index("browser");
    windows
        .iter()
        .find(|row| row.window.program == browser)
        .map(|row| crate::systems::desk::laid_out_size(&row.window, viewport, config).x)
        .unwrap_or(config.frame_min_size.x)
}

fn refresh_bookmarks(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &crate::data::config::Config,
) {
    let facts = resources.session.facts.clone();
    let on_bar = bookmarks(&facts, false);
    let in_folder = bookmarks(&facts, true);

    let marks: Vec<BookmarkRow> = ink
        .gathered
        .rows
        .bookmarks
        .iter()
        .map(|(_, mark)| *mark)
        .collect();
    for row in marks {
        let Some(mark) = BOOKMARKS.get(row.bookmark) else {
            continue;
        };
        let up = if row.filed {
            resources.browser.folder_open && in_folder.contains(&row.bookmark)
        } else {
            on_bar.contains(&row.bookmark)
        };
        ui_set_visible(ink.build.dirty, ink.build.commands, row.slot.root, up);
        if !up {
            continue;
        }
        ui_set_text(ink.build.commands, row.slot.label, mark.label);
        ui_set_color::<UiBase>(
            ink.build.commands,
            row.slot.label,
            if !row.filed && resources.browser.address == mark.url {
                config.text
            } else {
                config.text_dim
            },
        );
    }

    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.browser_folder,
        !in_folder.is_empty(),
    );
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.browser_folder_panel,
        resources.browser.folder_open && !in_folder.is_empty(),
    );
}

fn refresh_page(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &crate::data::config::Config,
    running: bool,
) {
    ui_set_text(
        ink.build.commands,
        resources.interface.browser_band,
        &crate::data::web::host_of(&resources.browser.address),
    );

    let mut blocks: Vec<&'static Block> = Vec::new();
    if !running {
        match resources.browser.browsing {
            Some(index) => {
                let page = &PAGES[index];
                blocks.extend(page.body.iter());
                if let Some(which) = resources.browser.page_answered
                    && let Some(prompt) = page.prompt.as_ref()
                    && let Some(answer) = prompt.answers.get(which)
                {
                    blocks.extend(answer.reply.iter());
                }
            }
            None => blocks.extend(crate::data::web::NOT_FOUND.iter()),
        }
    }

    let rows: Vec<PageBlock> = ink
        .gathered
        .rows
        .blocks
        .iter()
        .map(|(_, row)| *row)
        .collect();
    for row in rows {
        match blocks.get(row.block) {
            Some(block) => {
                ui_set_visible(ink.build.dirty, ink.build.commands, row.slot.root, true);
                draw_block(ink, config, row, block);
            }
            None => {
                ui_set_visible(ink.build.dirty, ink.build.commands, row.slot.root, false);
                ui_set_visible(ink.build.dirty, ink.build.commands, row.right, false);
                ui_set_visible(ink.build.dirty, ink.build.commands, row.mark, false);
                ui_set_visible(ink.build.dirty, ink.build.commands, row.rule, false);
                ui_set_visible(ink.build.dirty, ink.build.commands, row.stripe, false);
                ui_set_text(ink.build.commands, row.slot.label, "");
                ui_set_text(ink.build.commands, row.right, "");
            }
        }
    }

    refresh_field(resources, ink, config, running);

    let links: Vec<&'static str> = match (running, resources.browser.browsing) {
        (false, Some(index)) => PAGES[index].links.to_vec(),
        _ => Vec::new(),
    };
    let across = fit_page(resources, ink, config);
    let each = link_width(across, links.len());
    let rows: Vec<LinkRow> = ink
        .gathered
        .rows
        .links
        .iter()
        .map(|(_, row)| *row)
        .collect();
    for row in rows {
        match links.get(row.link) {
            Some(url) => {
                ui_set_visible(ink.build.dirty, ink.build.commands, row.slot.root, true);
                ui_set_text(
                    ink.build.commands,
                    row.slot.label,
                    &crate::data::web::name_of(url),
                );
                ink.nodes.for_entity(row.slot.root, |node| {
                    node.flow_child_size = Some(Ab(vec2(each, 24.0)).into());
                });
            }
            None => ui_set_visible(ink.build.dirty, ink.build.commands, row.slot.root, false),
        }
    }

    let status = if running {
        format!("Waiting for {}\u{2026}", resources.browser.address)
    } else if resources.browser.browsing.is_some() {
        format!("Done. {}", resources.browser.address)
    } else {
        format!("{} could not be reached", resources.browser.address)
    };
    ui_set_text(
        ink.build.commands,
        resources.interface.browser_status,
        &status,
    );
}

fn wanted_width(
    resources: &AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &crate::data::config::Config,
) -> f32 {
    let scale = config.interface_scale.max(0.01);
    let mut body = 0.0;
    ink.nodes
        .for_entity(resources.interface.browser_body, |node| {
            body = (node.computed_rect.max.x - node.computed_rect.min.x) / scale;
        });
    if body <= 1.0 {
        return config.page_width;
    }
    config
        .page_width
        .min((body - 2.0 * config.page_margin).max(320.0))
}

fn fit_page(
    resources: &AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &crate::data::config::Config,
) -> f32 {
    let want = wanted_width(resources, ink, config);
    let scale = config.interface_scale.max(0.01);
    let mut across = 0.0;
    ink.nodes
        .for_entity(resources.interface.browser_page, |node| {
            across = (node.computed_rect.max.x - node.computed_rect.min.x) / scale;
        });
    if (want - across).abs() > 0.5 {
        ink.nodes
            .for_entity(resources.interface.browser_page, |node| {
                node.base_layout = Some(UiLayoutType::Window(WindowLayout {
                    position: Rl(vec2(50.0, 0.0)).into(),
                    size: Rl(vec2(0.0, 100.0)) + Ab(vec2(want, 0.0)),
                    anchor: Anchor::TopCenter,
                }));
            });
    }
    want
}

pub fn watch_width(
    mut resources: ResMut<AppResources>,
    config: Res<crate::data::config::Config>,
    mut gathered: ResMut<crate::systems::desk::Gathered>,
    mut build: UiBuild,
    nodes: Query<&mut UiLayoutNode>,
    contents: Query<&mut UiNodeContent>,
) {
    let mut ink = Ink {
        build: build.context(),
        nodes,
        contents,
        gathered: &mut gathered,
    };
    let ink = &mut ink;
    let resources = &mut *resources;
    if resources.desk.screen != crate::data::machine::Screen::Desktop {
        return;
    }
    let want = wanted_width(resources, ink, &config);
    if (want - resources.browser.page_across).abs() <= 0.5 {
        return;
    }
    resources.browser.page_across = want;
    resources.redraw.machine = true;
}

fn link_width(across: f32, count: usize) -> f32 {
    if count == 0 {
        return crate::systems::setup::NAV_WIDEST;
    }
    let strip = (across - crate::systems::setup::NAV_LEFT - 18.0).max(0.0);
    let shared = (strip - crate::systems::setup::NAV_GAP * (count as f32 - 1.0)) / count as f32;
    shared.clamp(58.0, crate::systems::setup::NAV_WIDEST)
}

fn refresh_field(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &crate::data::config::Config,
    running: bool,
) {
    let field = resources
        .browser
        .browsing
        .filter(|_| !running)
        .and_then(|index| PAGES[index].prompt.as_ref());
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.browser_prompt,
        field.is_some(),
    );
    let Some(prompt) = field else {
        return;
    };
    let label = if resources.browser.page_refused {
        prompt.refusal
    } else {
        prompt.label
    };
    ui_set_text(
        ink.build.commands,
        resources.interface.browser_prompt_label,
        label,
    );
    ui_set_color::<UiBase>(
        ink.build.commands,
        resources.interface.browser_prompt_label,
        if resources.browser.page_refused {
            config.accent_warn
        } else {
            config.page_faint
        },
    );

    let typing = !resources.browser.address_focused;
    let caret = if typing && resources.login.caret_lit {
        "|"
    } else {
        ""
    };
    let shown = if resources.browser.page_typed.is_empty() && !typing {
        prompt.hint.to_string()
    } else {
        format!("{}{}", resources.browser.page_typed, caret)
    };
    ui_set_text(
        ink.build.commands,
        resources.interface.browser_prompt_caret,
        &shown,
    );
    ui_set_color::<UiBase>(
        ink.build.commands,
        resources.interface.browser_prompt_caret,
        if resources.browser.page_typed.is_empty() && !typing {
            config.page_faint
        } else {
            config.page_text
        },
    );
}

struct Setting {
    size: f32,
    mono: bool,
    color: Vec4,
    fill: Vec4,
    inset: f32,
    above: f32,
    below: f32,
    gap: f32,
}

fn draw_block(
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &crate::data::config::Config,
    row: crate::components::PageBlock,
    block: &Block,
) {
    let slot = row.slot;
    let setting = match block {
        Block::Heading(_) => Setting {
            size: 21.4,
            mono: false,
            color: config.page_text,
            fill: config.transparent,
            inset: 0.0,
            above: 22.0,
            below: 12.0,
            gap: 6.0,
        },
        Block::Lead(_) => Setting {
            size: 13.0,
            mono: false,
            color: config.page_faint,
            fill: config.transparent,
            inset: 0.0,
            above: 2.0,
            below: 10.0,
            gap: 0.0,
        },
        Block::Text(_) => Setting {
            size: 13.4,
            mono: false,
            color: config.page_text,
            fill: config.transparent,
            inset: 0.0,
            above: 6.0,
            below: 8.0,
            gap: 0.0,
        },
        Block::Rule => Setting {
            size: 1.0,
            mono: false,
            color: config.transparent,
            fill: config.transparent,
            inset: 0.0,
            above: 0.0,
            below: 0.0,
            gap: 0.0,
        },
        Block::Pre(_) => Setting {
            size: 12.2,
            mono: true,
            color: config.page_text,
            fill: config.page_panel,
            inset: 16.0,
            above: 10.0,
            below: 10.0,
            gap: 8.0,
        },
        Block::Note(_) => Setting {
            size: 10.8,
            mono: false,
            color: config.page_faint,
            fill: config.transparent,
            inset: 0.0,
            above: 12.0,
            below: 8.0,
            gap: 0.0,
        },
        Block::Row { .. } => Setting {
            size: 12.8,
            mono: false,
            color: config.page_link,
            fill: config.transparent,
            inset: 16.0,
            above: 7.0,
            below: 7.0,
            gap: 0.0,
        },
        Block::By { .. } => Setting {
            size: 11.6,
            mono: true,
            color: config.page_faint,
            fill: config.page_panel,
            inset: 14.0,
            above: 8.0,
            below: 4.0,
            gap: 0.0,
        },
        Block::Says(_) => Setting {
            size: 13.4,
            mono: false,
            color: config.page_text,
            fill: config.page_panel,
            inset: 14.0,
            above: 2.0,
            below: 10.0,
            gap: 0.0,
        },
    };

    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        row.rule,
        matches!(block, Block::Heading(_)),
    );
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        row.stripe,
        matches!(block, Block::Pre(_)),
    );
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        row.mark,
        matches!(block, Block::Row { .. }),
    );
    ui_set_color::<UiHover>(
        ink.build.commands,
        slot.root,
        match block {
            Block::Row { .. } => config.page_row_hover,
            _ => config.transparent,
        },
    );

    if matches!(block, Block::Rule) {
        ui_set_text(ink.build.commands, slot.label, "");
        ui_set_color::<UiBase>(ink.build.commands, slot.root, config.page_rule);
        set_type(&mut ink.contents, slot.label, 1.0, false);
        set_row_rule(ink, slot.root);
        ui_set_visible(ink.build.dirty, ink.build.commands, row.right, false);
        ui_set_text(ink.build.commands, row.right, "");
        set_label_width(ink, slot.label, 0.0);
        set_spacer(ink, row.above, 0.0);
        set_spacer(ink, row.below, 0.0);
        return;
    }

    let (left, right) = match block {
        Block::Row { text, .. } => split_columns(text),
        Block::By { author, when } => (author.to_string(), when.to_string()),
        Block::Heading(text)
        | Block::Lead(text)
        | Block::Text(text)
        | Block::Pre(text)
        | Block::Note(text)
        | Block::Says(text) => (text.to_string(), String::new()),
        Block::Rule => (String::new(), String::new()),
    };
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        row.right,
        !right.is_empty(),
    );
    ui_set_text(ink.build.commands, row.right, &right);
    if !right.is_empty() {
        ui_set_color::<UiBase>(ink.build.commands, row.right, config.page_faint);
    }

    ui_set_text(ink.build.commands, slot.label, &left);
    ui_set_color::<UiBase>(ink.build.commands, slot.label, setting.color);
    ui_set_color::<UiBase>(ink.build.commands, slot.root, setting.fill);
    set_type(&mut ink.contents, slot.label, setting.size, setting.mono);
    clear_row_rule(ink, slot.root, setting.inset, setting.gap);
    set_spacer(ink, row.above, setting.above);
    set_spacer(ink, row.below, setting.below);
    set_label_width(
        ink,
        slot.label,
        if right.is_empty() { 0.0 } else { RIGHT_COLUMN },
    );
}

const RIGHT_COLUMN: f32 = 246.0;

fn set_spacer(ink: &mut Ink<'_, '_, '_, '_, '_, '_>, entity: Entity, height: f32) {
    ui_set_visible(ink.build.dirty, ink.build.commands, entity, height > 0.0);
    ink.nodes.for_entity(entity, |node| {
        node.flow_child_size = Some(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, height)));
    });
}

fn set_label_width(ink: &mut Ink<'_, '_, '_, '_, '_, '_>, entity: Entity, reserved: f32) {
    ink.nodes.for_entity(entity, |node| {
        node.flow_child_size = Some(Rl(vec2(100.0, 0.0)) + Ab(vec2(-reserved, 0.0)));
    });
}

fn split_columns(text: &str) -> (String, String) {
    match text.split_once("  ") {
        Some((left, right)) => (left.trim_end().to_string(), right.trim_start().to_string()),
        None => (text.to_string(), String::new()),
    }
}

fn set_row_rule(ink: &mut Ink<'_, '_, '_, '_, '_, '_>, entity: Entity) {
    ink.nodes.for_entity(entity, |node| {
        node.auto_size = AutoSizeMode::None;
        node.flow_child_size = Some(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 1.0)));
        if let Some(flow) = node.flow_layout.as_mut() {
            flow.padding = 0.0;
            flow.spacing = 0.0;
        }
    });
}

fn clear_row_rule(ink: &mut Ink<'_, '_, '_, '_, '_, '_>, entity: Entity, inset: f32, gap: f32) {
    ink.nodes.for_entity(entity, |node| {
        node.auto_size = AutoSizeMode::Height;
        node.flow_child_size = Some(Rl(vec2(100.0, 0.0)).into());
        if let Some(flow) = node.flow_layout.as_mut() {
            flow.padding = inset;
            flow.spacing = gap;
        }
    });
}

fn set_type(contents: &mut Query<&mut UiNodeContent>, entity: Entity, size: f32, mono: bool) {
    contents.for_entity(entity, |content| {
        if let UiNodeContent::Text {
            font_size_override,
            font_kind,
            ..
        } = content
        {
            *font_size_override = Some(size);
            *font_kind = if mono {
                nightshade_text::contract::data::text_data::FontKind::Mono
            } else {
                nightshade_text::contract::data::text_data::FontKind::Default
            };
        }
    });
}
