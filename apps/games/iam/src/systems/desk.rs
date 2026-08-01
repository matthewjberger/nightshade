use crate::components::{
    AccountOpened, AccountRow, BarPressed, BoardCard, BoardColumn, CaseRow, ChecklistRow, Dragging,
    FileRow, FrameSlot, GamePlace, LampRow, Listing, NoteCard, NoticeRow, OpenFile, PairingRow,
    ProgramIcon, ProgramWindow, Resizing, RowSlot, SelectedAccount, SessionRow, SignedIn, StartRow,
    StoreButton, StoreRow, TallyRow, TaskbarButton, ThingRow, ToastRow, WayRow, WindowMode,
};
use crate::data::ask::Ask;
use crate::data::board::{LEADS, named, opens_into, pinned, progress};
use crate::data::config::Config;
use crate::data::cross::PAIRINGS;
use crate::data::files::{LEFTOVERS, readable as readable_file};
use crate::data::games::POSTED;
use crate::data::lights;
use crate::data::machine::{ACCOUNTS, STICKY_NOTE, Screen, opens};
use crate::data::notes::{NOTES, readable};
use crate::data::programs::{PROGRAMS, comes_with_the_desk, index_of};
use crate::data::script::{Fact, condition_holds};
use crate::resources::{AppResources, Ink};
use crate::systems::intro::covered;
use crate::systems::setup::program_index;
use nightshade::prelude::Entity;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::{Input, MouseState, Time, Window};
use nightshade_platform::winit;
use nightshade_ui::commands::builder::UiBuild;
use nightshade_ui::prelude::*;

pub struct WindowRow {
    pub entity: Entity,
    pub window: ProgramWindow,
    pub slot: Option<FrameSlot>,
    pub pressed: Option<f64>,
}

pub fn change_window(
    windows: &mut [WindowRow],
    commands: &mut Commands,
    entity: Entity,
    change: impl FnOnce(&mut ProgramWindow),
) -> bool {
    let Some(row) = windows.iter_mut().find(|row| row.entity == entity) else {
        return false;
    };
    change(&mut row.window);
    commands.set(entity, row.window);
    true
}

pub struct AccountEntry {
    pub entity: Entity,
    pub account: usize,
    pub opened: bool,
    pub slot: RowSlot,
}

#[derive(Default)]
pub struct Gathered {
    pub windows: Vec<WindowRow>,
    pub held: Held,
    pub desk: Desk,
    pub files: Files,
    pub rows: Rows,
    pub threads: crate::systems::conversation::Threads,
    pub roots: Vec<Entity>,
    pub frames: Vec<(usize, FrameSlot)>,
}

#[derive(Default)]
pub struct Held {
    pub carried: Vec<Entity>,
    pub stretched: Vec<Entity>,
}

#[derive(Default)]
pub struct Desk {
    pub accounts: Vec<AccountEntry>,
    pub selected_rows: Vec<Entity>,
    pub signed_in_rows: Vec<Entity>,
    pub selected_account: Option<usize>,
    pub signed_in_account: Option<usize>,
}

#[derive(Default)]
pub struct Files {
    pub rows: Vec<FileEntry>,
    pub open_rows: Vec<Entity>,
    pub open: Option<usize>,
}

pub struct FileEntry {
    pub entity: Entity,
    pub file: usize,
}

#[derive(Default)]
pub struct Rows {
    pub notices: Vec<(Entity, NoticeRow)>,
    pub toasts: Vec<(Entity, ToastRow)>,
    pub tallies: Vec<(Entity, TallyRow)>,
    pub endings: Vec<(Entity, crate::components::EndingRow)>,
    pub store: Vec<(Entity, StoreRow)>,
    pub sessions: Vec<(Entity, usize)>,
    pub pairings: Vec<(Entity, PairingRow)>,
    pub cases: Vec<(Entity, CaseRow)>,
    pub files: Vec<(Entity, FileRow)>,
    pub ways: Vec<(Entity, WayRow)>,
    pub things: Vec<(Entity, ThingRow)>,
    pub lamps: Vec<(Entity, LampRow)>,
    pub starts: Vec<(Entity, StartRow)>,
    pub taskbar: Vec<(Entity, TaskbarButton)>,
    pub icons: Vec<(Entity, usize)>,
    pub cards: Vec<(Entity, BoardCard)>,
    pub columns: Vec<(Entity, BoardColumn)>,
    pub notes: Vec<(Entity, NoteCard)>,
    pub checklist: Vec<(Entity, ChecklistRow)>,
    pub places: Vec<(Entity, GamePlace)>,
    pub bookmarks: Vec<(Entity, crate::components::BookmarkRow)>,
    pub blocks: Vec<(Entity, crate::components::PageBlock)>,
    pub links: Vec<(Entity, crate::components::LinkRow)>,
}

type FirstRows<'row> = (
    Option<&'row NoticeRow>,
    Option<&'row ToastRow>,
    Option<&'row TallyRow>,
    Option<&'row crate::components::EndingRow>,
    Option<&'row StoreRow>,
    Option<&'row SessionRow>,
    Option<&'row PairingRow>,
    Option<&'row CaseRow>,
);

type SecondRows<'row> = (
    Option<&'row FileRow>,
    Option<&'row WayRow>,
    Option<&'row ThingRow>,
    Option<&'row LampRow>,
    Option<&'row StartRow>,
    Option<&'row TaskbarButton>,
    Option<&'row ProgramIcon>,
    Option<&'row BoardCard>,
);

type ThirdRows<'row> = (
    Option<&'row BoardColumn>,
    Option<&'row NoteCard>,
    Option<&'row ChecklistRow>,
    Option<&'row GamePlace>,
    Option<&'row crate::components::BookmarkRow>,
    Option<&'row crate::components::PageBlock>,
    Option<&'row crate::components::LinkRow>,
);

pub fn gather_rows(
    mut gathered: ResMut<Gathered>,
    first: Query<FirstRows>,
    second: Query<SecondRows>,
    third: Query<ThirdRows>,
) {
    let rows = &mut gathered.rows;
    *rows = Rows::default();
    first.for_each(
        |entity, (notice, toast, tally, ending, store, session, pairing, case)| {
            if let Some(row) = notice {
                rows.notices.push((entity, *row));
            }
            if let Some(row) = toast {
                rows.toasts.push((entity, *row));
            }
            if let Some(row) = tally {
                rows.tallies.push((entity, *row));
            }
            if let Some(row) = ending {
                rows.endings.push((entity, *row));
            }
            if let Some(row) = store {
                rows.store.push((entity, *row));
            }
            if let Some(row) = session {
                rows.sessions.push((entity, row.0));
            }
            if let Some(row) = pairing {
                rows.pairings.push((entity, *row));
            }
            if let Some(row) = case {
                rows.cases.push((entity, *row));
            }
        },
    );
    second.for_each(
        |entity, (file, way, thing, lamp, start, taskbar, icon, card)| {
            if let Some(row) = file {
                rows.files.push((entity, *row));
            }
            if let Some(row) = way {
                rows.ways.push((entity, *row));
            }
            if let Some(row) = thing {
                rows.things.push((entity, *row));
            }
            if let Some(row) = lamp {
                rows.lamps.push((entity, *row));
            }
            if let Some(row) = start {
                rows.starts.push((entity, *row));
            }
            if let Some(row) = taskbar {
                rows.taskbar.push((entity, *row));
            }
            if let Some(row) = icon {
                rows.icons.push((entity, row.0));
            }
            if let Some(row) = card {
                rows.cards.push((entity, *row));
            }
        },
    );
    third.for_each(
        |entity, (column, note, checklist, place, bookmark, block, link)| {
            if let Some(row) = column {
                rows.columns.push((entity, *row));
            }
            if let Some(row) = note {
                rows.notes.push((entity, *row));
            }
            if let Some(row) = checklist {
                rows.checklist.push((entity, *row));
            }
            if let Some(row) = place {
                rows.places.push((entity, *row));
            }
            if let Some(row) = bookmark {
                rows.bookmarks.push((entity, *row));
            }
            if let Some(row) = block {
                rows.blocks.push((entity, *row));
            }
            if let Some(row) = link {
                rows.links.push((entity, *row));
            }
        },
    );
}

pub fn gather_windows(
    mut gathered: ResMut<Gathered>,
    windows: Query<(&ProgramWindow, Option<&FrameSlot>, Option<&BarPressed>)>,
) {
    let gathered = &mut gathered.windows;
    gathered.clear();
    windows.for_each(|entity, (window, slot, pressed)| {
        gathered.push(WindowRow {
            entity,
            window: *window,
            slot: slot.copied(),
            pressed: pressed.and_then(|pressed| pressed.0),
        });
    });
}

pub fn gather_held(
    mut gathered: ResMut<Gathered>,
    dragging: Query<&ProgramWindow, WithTag<Dragging>>,
    resizing: Query<&ProgramWindow, WithTag<Resizing>>,
) {
    let held = &mut gathered.held;
    held.carried.clear();
    held.stretched.clear();
    dragging.for_each(|entity, _| held.carried.push(entity));
    resizing.for_each(|entity, _| held.stretched.push(entity));
}

pub fn gather_desk(
    mut gathered: ResMut<Gathered>,
    accounts: Query<(&AccountRow, Option<&AccountOpened>)>,
    selected: Query<&AccountRow, WithTag<SelectedAccount>>,
    signed_in: Query<&AccountRow, WithTag<SignedIn>>,
) {
    let desk = &mut gathered.desk;
    *desk = Desk::default();
    accounts.for_each(|entity, (row, opened)| {
        desk.accounts.push(AccountEntry {
            entity,
            account: row.definition,
            opened: opened.is_some_and(|opened| opened.0),
            slot: row.slot,
        });
    });
    selected.for_each(|entity, row| {
        desk.selected_rows.push(entity);
        desk.selected_account = Some(row.definition);
    });
    signed_in.for_each(|entity, row| {
        desk.signed_in_rows.push(entity);
        desk.signed_in_account = Some(row.definition);
    });
}

pub fn gather_files(
    mut gathered: ResMut<Gathered>,
    rows: Query<&FileRow>,
    open: Query<&FileRow, WithTag<OpenFile>>,
) {
    let files = &mut gathered.files;
    *files = Files::default();
    rows.for_each(|entity, row| {
        files.rows.push(FileEntry {
            entity,
            file: row.file,
        });
    });
    open.for_each(|entity, row| {
        files.open_rows.push(entity);
        files.open = Some(row.file);
    });
}

pub fn take_the_screen(
    _main: MainThread,
    mut resources: ResMut<AppResources>,
    window: Res<Window>,
) {
    if resources.boot.took_the_screen {
        return;
    }
    let Some(handle) = window.handle.clone() else {
        return;
    };
    let Some(monitor) = handle.current_monitor() else {
        return;
    };
    let wanted = monitor.size();
    let mode = monitor
        .video_modes()
        .filter(|mode| mode.size() == wanted)
        .max_by_key(|mode| mode.refresh_rate_millihertz())
        .or_else(|| {
            monitor.video_modes().max_by_key(|mode| {
                (
                    mode.size().width * mode.size().height,
                    mode.refresh_rate_millihertz(),
                )
            })
        });
    match mode {
        Some(mode) => {
            handle.set_fullscreen(Some(winit::window::Fullscreen::Exclusive(mode)));
        }
        None => handle.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None))),
    }
    resources.boot.took_the_screen = true;
}

pub fn move_pointer(
    resources: Res<AppResources>,
    config: Res<Config>,
    input: Res<Input>,
    mut window: ResMut<Window>,
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
    let focused = window.is_focused;
    nightshade_platform::commands::window::set_cursor_visible(&mut window, !focused);
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.pointer,
        focused,
    );
    let position = pointer_at(&input, &config);
    let pointer = resources.interface.pointer;
    if pointer == Entity::default() {
        return;
    }
    set_pointer(ink, pointer, position);
}

pub fn tick_clock(
    resources: Res<AppResources>,
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
    if resources.desk.screen != Screen::Desktop {
        return;
    }
    let tonight = resources.script.nights.get(resources.session.night);
    let day = tonight.map(|night| night.day).unwrap_or_default();
    let date = tonight.map(|night| night.date).unwrap_or_default();
    ui_set_text(
        ink.build.commands,
        resources.interface.taskbar_clock,
        &crate::systems::interface::wall_clock(resources.session.clock_seconds),
    );
    ui_set_text(
        ink.build.commands,
        resources.interface.taskbar_date,
        &crate::systems::interface::calendar(day, date),
    );
}

pub fn run_toasts(mut resources: ResMut<AppResources>, config: Res<Config>, time: Res<Time>) {
    let resources = &mut *resources;
    if resources.corner.toasts.is_empty()
        || crate::systems::screensaver::covering(resources)
        || crate::systems::intro::covered(resources)
    {
        return;
    }
    let seconds = config.toast_seconds;
    let delta = time.delta_time;
    for toast in resources.corner.toasts.iter_mut() {
        toast.age += delta;
    }
    let before = resources.corner.toasts.len();
    resources.corner.toasts.retain(|toast| toast.age < seconds);
    if resources.corner.toasts.len() != before {
        resources.redraw.machine = true;
    }
}

pub fn raise_toast(
    resources: &mut AppResources,
    ink: &Ink<'_, '_, '_, '_, '_, '_>,
    opens: crate::resources::ToastTarget,
    who: &str,
    text: &str,
) {
    if resources.desk.screen != Screen::Desktop
        || showing(&ink.gathered.windows, program_index("iam"))
    {
        return;
    }
    resources.corner.toasts.push(crate::resources::Toast {
        who: who.to_string(),
        text: text.to_string(),
        opens,
        age: 0.0,
    });
    keep_notice(resources, who, text);
    resources.redraw.machine = true;
}

fn refresh_notices(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let open = resources.corner.notices_open;
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.notices_panel,
        open,
    );
    if !open {
        return;
    }
    let total = resources.corner.notices.len();
    ui_set_text(
        ink.build.commands,
        resources.interface.notices_count,
        &match total {
            0 => "nothing yet".to_string(),
            1 => "1 notice".to_string(),
            many => format!("{many} notices"),
        },
    );

    let first = resources.corner.notices_at.min(furthest_notice(resources));
    let rows: Vec<NoticeRow> = ink
        .gathered
        .rows
        .notices
        .iter()
        .map(|(_, row)| *row)
        .collect();
    for row in rows {
        let kept = total
            .checked_sub(first + row.slot + 1)
            .and_then(|index| resources.corner.notices.get(index));
        let Some(notice) = kept else {
            ui_set_visible(ink.build.dirty, ink.build.commands, row.root, false);
            continue;
        };
        ui_set_visible(ink.build.dirty, ink.build.commands, row.root, true);
        ui_set_text(ink.build.commands, row.who, &notice.who);
        ui_set_text(ink.build.commands, row.body, &notice.text);
        ui_set_text(ink.build.commands, row.stamp, &notice.stamp);
    }

    let more = total.saturating_sub(first + crate::systems::setup::NOTICES_ROWS);
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.notices_more,
        more > 0,
    );
    if more > 0 {
        ui_set_text(
            ink.build.commands,
            resources.interface.notices_more,
            &format!("{more} older"),
        );
    }
    slide_notices(resources, ink, config);
}

fn furthest_notice(resources: &AppResources) -> usize {
    resources
        .corner
        .notices
        .len()
        .saturating_sub(crate::systems::setup::NOTICES_ROWS)
}

fn slide_notices(resources: &AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>, config: &Config) {
    let size = config.notices_size;
    let eased = resources.corner.notices_slide
        * resources.corner.notices_slide
        * (3.0 - 2.0 * resources.corner.notices_slide);
    let out = (1.0 - eased) * (size.x + 20.0);
    ink.nodes
        .for_entity(resources.interface.notices_panel, |node| {
            node.base_layout = Some(UiLayoutType::Window(WindowLayout {
                position: Rl(vec2(100.0, 100.0))
                    + Ab(vec2(
                        -size.x - 12.0 + out,
                        -config.taskbar_height - size.y - 10.0,
                    )),
                size: Ab(size).into(),
                anchor: Anchor::TopLeft,
            }));
        });
}

pub fn run_notices(
    mut resources: ResMut<AppResources>,
    config: Res<Config>,
    time: Res<Time>,
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
    let step = time.delta_time * config.notices_slide;
    let want = if resources.corner.notices_open && resources.desk.screen == Screen::Desktop {
        1.0
    } else {
        0.0
    };
    if (resources.corner.notices_slide - want).abs() < 0.001 {
        return;
    }
    resources.corner.notices_slide = if want > resources.corner.notices_slide {
        (resources.corner.notices_slide + step).min(1.0)
    } else {
        (resources.corner.notices_slide - step).max(0.0)
    };
    slide_notices(resources, ink, &config);
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.notices_panel,
        resources.corner.notices_slide > 0.001,
    );
}

pub fn wind_notices(mut resources: ResMut<AppResources>, config: Res<Config>, input: Res<Input>) {
    let resources = &mut *resources;
    if !resources.corner.notices_open || resources.desk.screen != Screen::Desktop {
        return;
    }
    let wheel = input.mouse.wheel_delta.y;
    if wheel == 0.0 {
        return;
    }
    let furthest = furthest_notice(resources);
    let at = resources.corner.notices_at;
    resources.corner.notices_at = if wheel < 0.0 {
        (at + config.notices_wheel).min(furthest)
    } else {
        at.saturating_sub(config.notices_wheel)
    };
    resources.redraw.machine = true;
}

fn refresh_toasts(resources: &mut AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>) {
    let shown: Vec<(String, String)> = resources
        .corner
        .toasts
        .iter()
        .rev()
        .take(3)
        .map(|toast| (toast.who.clone(), toast.text.clone()))
        .collect();
    let rows: Vec<ToastRow> = ink
        .gathered
        .rows
        .toasts
        .iter()
        .map(|(_, row)| *row)
        .collect();
    for row in rows {
        match shown.get(row.slot) {
            Some((who, text)) => {
                ui_set_visible(ink.build.dirty, ink.build.commands, row.root, true);
                ui_set_text(ink.build.commands, row.who, who);
                ui_set_text(ink.build.commands, row.body, text);
            }
            None => ui_set_visible(ink.build.dirty, ink.build.commands, row.root, false),
        }
    }
}

pub fn toast_target(
    resources: &AppResources,
    slot: usize,
) -> Option<crate::resources::ToastTarget> {
    let count = resources.corner.toasts.len();
    count
        .checked_sub(slot + 1)
        .and_then(|index| resources.corner.toasts.get(index))
        .map(|toast| toast.opens)
}

pub fn run_alarm(
    mut resources: ResMut<AppResources>,
    config: Res<Config>,
    time: Res<Time>,
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
    if resources.glass.alarm <= 0.0 {
        return;
    }
    let delta = time.delta_time;
    resources.glass.alarm += delta;
    ui_set_color::<UiBase>(
        ink.build.commands,
        resources.interface.player_dot,
        config.terminal_red,
    );

    if resources.glass.alarm < config.alarm_seconds {
        return;
    }
    resources.glass.shutoff =
        (resources.glass.shutoff + delta / config.shutoff_seconds.max(0.05)).min(1.0);
    if resources.glass.shutoff < 1.0 {
        return;
    }
    resources.session.terminated = true;
    resources.session.finished = true;
    resources.session.ending = crate::data::endings::Ending::Filtered;
    crate::systems::ambience::go_quiet(resources, ink.build.commands);
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.terminated,
        true,
    );
}

pub fn hold_lens(
    mut resources: ResMut<AppResources>,
    config: Res<Config>,
    input: Res<Input>,
    time: Res<Time>,
    window: Res<Window>,
) {
    let resources = &mut *resources;
    let mouse = input.mouse;
    let held = mouse.state.contains(MouseState::RIGHT_CLICKED)
        && !crate::systems::intro::covered(resources)
        && !crate::systems::screensaver::covering(resources);
    let delta = time.delta_time;
    let wanted = if held { 1.0 } else { 0.0 };
    let step = (delta * config.lens_rate).min(1.0);
    resources.glass.lens += (wanted - resources.glass.lens) * step;
    if resources.glass.lens < 0.002 {
        resources.glass.lens = 0.0;
    }
    let across = viewport_of(&window, &config);
    let at = pointer_at(&input, &config);
    if across.x > 0.0 && across.y > 0.0 {
        resources.glass.lens_at = vec2(
            (at.x / across.x).clamp(0.0, 1.0),
            (at.y / across.y).clamp(0.0, 1.0),
        );
    }
}

fn set_pointer(ink: &mut Ink<'_, '_, '_, '_, '_, '_>, pointer: Entity, position: Vec2) {
    ink.nodes.for_entity(pointer, |node| {
        node.base_layout = Some(UiLayoutType::Window(WindowLayout {
            position: Ab(position).into(),
            size: Ab(vec2(14.0, 20.0)).into(),
            anchor: Anchor::TopLeft,
        }));
    });
    ui_mark_layout_dirty(ink.build.dirty);
}

pub fn type_password(
    mut resources: ResMut<AppResources>,
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
    if resources.desk.screen != Screen::Login || covered(resources) {
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

    if backspace {
        resources.login.typed.pop();
        resources.redraw.machine = true;
    }
    if !typed.is_empty() {
        resources.login.typed.extend(typed);
        resources.redraw.machine = true;
    }
    if submit {
        sign_in(resources, ink);
    }
}

fn move_tag<T: 'static>(commands: &mut Commands, wearing: &[Entity], to: Option<Entity>) {
    for entity in wearing {
        commands.remove_tag_type::<T>(*entity);
    }
    if let Some(entity) = to {
        commands.add_tag_type::<T>(entity);
    }
}

fn account_row(desk: &Desk, account: usize) -> Option<Entity> {
    desk.accounts
        .iter()
        .find(|row| row.account == account)
        .map(|row| row.entity)
}

fn file_row(files: &Files, file: usize) -> Option<Entity> {
    files
        .rows
        .iter()
        .find(|row| row.file == file)
        .map(|row| row.entity)
}

pub fn selected_account(desk: &Desk) -> Option<usize> {
    desk.selected_account
}

pub fn select_account(commands: &mut Commands, desk: &Desk, account: usize) {
    let row = account_row(desk, account);
    move_tag::<SelectedAccount>(commands, &desk.selected_rows, row);
}

pub fn signed_in_account(desk: &Desk) -> Option<usize> {
    desk.signed_in_account
}

pub fn sign_in_as(commands: &mut Commands, desk: &Desk, account: Option<usize>) {
    let row = account.and_then(|account| account_row(desk, account));
    move_tag::<SignedIn>(commands, &desk.signed_in_rows, row);
}

pub fn open_file(files: &Files) -> Option<usize> {
    files.open
}

pub fn open_this_file(commands: &mut Commands, files: &Files, file: Option<usize>) {
    let row = file.and_then(|file| file_row(files, file));
    move_tag::<OpenFile>(commands, &files.open_rows, row);
}

pub fn sign_in(resources: &mut AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>) {
    let Some(index) = selected_account(&ink.gathered.desk) else {
        return;
    };
    let Some(account) = ACCOUNTS.get(index) else {
        return;
    };
    let typed = resources.login.typed.clone();
    resources.redraw.machine = true;

    if !opens(account, &typed) {
        resources.login.refusal = if account.understands.is_empty() {
            "that is not the password".to_string()
        } else if condition_holds(
            &crate::data::script::Condition {
                requires: account.understands,
                forbids: &[],
            },
            &resources.session.facts,
        ) {
            account.locked_note.to_string()
        } else {
            "that is not the password".to_string()
        };
        resources.login.typed.clear();
        resources.session.cues.push(crate::resources::Cue::Refused);
        return;
    }

    if let Some(archive) = account.archive {
        resources.script = archive;
    }
    mark_opened(ink, index);
    if index != crate::data::machine::index_of("guest")
        && !resources.session.facts.contains(&"opened_an_account")
    {
        resources.session.facts.push("opened_an_account");
    }
    sign_in_as(ink.build.commands, &ink.gathered.desk, Some(index));
    resources.desk.screen = Screen::Desktop;
    resources.login.typed.clear();
    resources.login.refusal.clear();
    resources.redraw = crate::resources::redraw_everything();
}

fn mark_opened(ink: &mut Ink<'_, '_, '_, '_, '_, '_>, account: usize) {
    let rows: Vec<Entity> = ink
        .gathered
        .desk
        .accounts
        .iter()
        .filter(|row| row.account == account)
        .map(|row| row.entity)
        .collect();
    for row in rows {
        ink.build.commands.set(row, AccountOpened(true));
    }
}

fn opened_accounts(ink: &Ink<'_, '_, '_, '_, '_, '_>) -> Vec<&'static str> {
    ink.gathered
        .desk
        .accounts
        .iter()
        .filter(|row| row.opened)
        .filter_map(|row| ACCOUNTS.get(row.account))
        .map(|account| account.id)
        .collect()
}

pub fn drag_windows(
    mut resources: ResMut<AppResources>,
    config: Res<Config>,
    input: Res<Input>,
    time: Res<Time>,
    window: Res<Window>,
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
    if covered(resources) || crate::systems::screensaver::covering(resources) {
        release_pointer(ink);
        return;
    }
    let mouse = input.mouse;
    let held = mouse.state.contains(MouseState::LEFT_CLICKED);
    let pressed = mouse.state.contains(MouseState::LEFT_JUST_PRESSED);
    let viewport = viewport_of(&window, &config);

    if !held {
        release_pointer(ink);
        return;
    }

    if pressed {
        let at = pointer_at(&input, &config);
        let Some(program) = under_pointer(&ink.gathered.windows, at, viewport, &config) else {
            return;
        };
        raise(resources, ink, program);
        let Some(held_window) = window_of(&ink.gathered.windows, program) else {
            return;
        };
        let (position, size) = laid_out(&held_window, viewport, &config);
        let on_bar = at.y <= position.y + config.frame_bar_height;

        let now = time.uptime_seconds;
        if on_bar {
            let Some(entity) = window_entity(&ink.gathered.windows, program) else {
                return;
            };
            let doubled = ink
                .gathered
                .windows
                .iter()
                .find(|row| row.entity == entity)
                .and_then(|row| row.pressed)
                .is_some_and(|last| now - last <= config.double_click_seconds as f64);
            ink.build.commands.set(entity, BarPressed(Some(now)));
            if doubled {
                release_pointer(ink);
                ink.build.commands.set(entity, BarPressed(None));
                let wanted = match held_window.mode {
                    WindowMode::Floating => WindowMode::Maximized,
                    _ => WindowMode::Floating,
                };
                change_window(
                    &mut ink.gathered.windows,
                    ink.build.commands,
                    entity,
                    |target| {
                        target.mode = wanted;
                    },
                );
                resources.redraw.machine = true;
                return;
            }
        }

        let Some(entity) = window_entity(&ink.gathered.windows, program) else {
            return;
        };
        if held_window.mode != WindowMode::Floating {
            if on_bar {
                ink.build.commands.add_tag_type::<Dragging>(entity);
            }
            return;
        }
        let corner = position + size;
        let on_grip = at.x >= corner.x - 18.0
            && at.x <= corner.x
            && at.y >= corner.y - 18.0
            && at.y <= corner.y;
        if on_grip {
            ink.build.commands.add_tag_type::<Resizing>(entity);
        } else if on_bar {
            ink.build.commands.add_tag_type::<Dragging>(entity);
        }
        return;
    }

    let delta = pointer_moved(&input, &config);
    if delta.x == 0.0 && delta.y == 0.0 {
        return;
    }
    let at = pointer_at(&input, &config);
    let carried = ink.gathered.held.carried.clone();
    for entity in carried {
        let Some(carried) = ink
            .gathered
            .windows
            .iter()
            .find(|row| row.entity == entity)
            .map(|row| row.window)
        else {
            continue;
        };
        if carried.mode != WindowMode::Floating {
            let (position, size) = laid_out(&carried, viewport, &config);
            let along = if size.x > 0.0 {
                ((at.x - position.x) / size.x).clamp(0.0, 1.0)
            } else {
                0.5
            };
            change_window(
                &mut ink.gathered.windows,
                ink.build.commands,
                entity,
                |target| {
                    target.mode = WindowMode::Floating;
                    target.position = vec2(
                        at.x - target.size.x * along,
                        (at.y - config.frame_bar_height * 0.5).max(0.0),
                    );
                },
            );
            resources.redraw.machine = true;
            continue;
        }
        if change_window(
            &mut ink.gathered.windows,
            ink.build.commands,
            entity,
            |target| {
                target.position += delta;
            },
        ) {
            resources.redraw.machine = true;
        }
    }
    let stretched = ink.gathered.held.stretched.clone();
    for entity in stretched {
        if change_window(
            &mut ink.gathered.windows,
            ink.build.commands,
            entity,
            |stretched| {
                stretched.size.x = (stretched.size.x + delta.x).max(config.frame_min_size.x);
                stretched.size.y = (stretched.size.y + delta.y).max(config.frame_min_size.y);
            },
        ) {
            resources.redraw.machine = true;
        }
    }
}

fn release_pointer(ink: &mut Ink<'_, '_, '_, '_, '_, '_>) {
    move_tag::<Dragging>(ink.build.commands, &ink.gathered.held.carried, None);
    move_tag::<Resizing>(ink.build.commands, &ink.gathered.held.stretched, None);
}

pub fn window_entity(windows: &[WindowRow], program: usize) -> Option<Entity> {
    windows
        .iter()
        .find(|row| row.window.program == program)
        .map(|row| row.entity)
}

fn window_of(windows: &[WindowRow], program: usize) -> Option<ProgramWindow> {
    windows
        .iter()
        .find(|row| row.window.program == program)
        .map(|row| row.window)
}

fn under_pointer(
    windows: &[WindowRow],
    at: Vec2,
    viewport: Vec2,
    config: &Config,
) -> Option<usize> {
    let mut found: Option<(u32, usize)> = None;
    for window in windows.iter().map(|row| &row.window) {
        if !window.open || window.minimized {
            continue;
        }
        let (position, size) = laid_out(window, viewport, config);
        let inside = at.x >= position.x
            && at.x <= position.x + size.x
            && at.y >= position.y
            && at.y <= position.y + size.y;
        if inside && found.is_none_or(|(order, _)| window.order > order) {
            found = Some((window.order, window.program));
        }
    }
    found.map(|(_, program)| program)
}

pub fn laid_out_size(window: &ProgramWindow, viewport: Vec2, config: &Config) -> Vec2 {
    laid_out(window, viewport, config).1
}

fn laid_out(window: &ProgramWindow, viewport: Vec2, config: &Config) -> (Vec2, Vec2) {
    match window.mode {
        WindowMode::Floating => (window.position, window.size),
        WindowMode::Maximized => (
            vec2(0.0, 0.0),
            vec2(viewport.x, viewport.y - config.taskbar_height),
        ),
    }
}

pub fn viewport_of(window: &Window, config: &Config) -> Vec2 {
    let pixels = window
        .cached_viewport_size
        .map(|(width, height)| vec2(width as f32, height as f32))
        .unwrap_or_else(|| vec2(1180.0, 760.0));
    pixels / config.interface_scale.max(0.01)
}

fn pointer_at(input: &Input, config: &Config) -> Vec2 {
    input.mouse.position / config.interface_scale.max(0.01)
}

fn pointer_moved(input: &Input, config: &Config) -> Vec2 {
    input.mouse.position_delta / config.interface_scale.max(0.01)
}

pub fn raise(resources: &mut AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>, program: usize) {
    let highest = ink
        .gathered
        .windows
        .iter()
        .map(|row| row.window.order)
        .max()
        .unwrap_or_default();
    if let Some(entity) = window_entity(&ink.gathered.windows, program) {
        let raised = ink
            .gathered
            .windows
            .iter()
            .find(|row| row.entity == entity)
            .is_some_and(|row| row.window.order != highest);
        if raised {
            change_window(
                &mut ink.gathered.windows,
                ink.build.commands,
                entity,
                |window| {
                    window.order = highest + 1;
                },
            );
            resources.redraw.machine = true;
        }
    }
}

pub fn toggle(resources: &mut AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>, program: usize) {
    let highest = ink
        .gathered
        .windows
        .iter()
        .map(|row| row.window.order)
        .max()
        .unwrap_or_default();
    let front = front_program(&ink.gathered.windows);
    let Some(entity) = window_entity(&ink.gathered.windows, program) else {
        return;
    };
    change_window(
        &mut ink.gathered.windows,
        ink.build.commands,
        entity,
        |window| {
            if window.open && !window.minimized && front == Some(program) {
                window.minimized = true;
            } else {
                let first_time = !window.open;
                window.open = true;
                window.minimized = false;
                window.order = highest + 1;
                if first_time && PROGRAMS.get(program).is_some_and(|entry| entry.opens_full) {
                    window.mode = WindowMode::Maximized;
                }
            }
        },
    );
    resources.redraw.machine = true;
}

pub fn put_away(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    program: usize,
    closed: bool,
) {
    if let Some(entity) = window_entity(&ink.gathered.windows, program) {
        change_window(
            &mut ink.gathered.windows,
            ink.build.commands,
            entity,
            |window| {
                window.minimized = true;
                if closed {
                    window.open = false;
                }
            },
        );
    }
    resources.redraw.machine = true;
}

fn place_frames(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    viewport: Vec2,
    config: &Config,
) {
    let on_desktop = resources.desk.screen == Screen::Desktop;
    let mut placed: Vec<(FrameSlot, Vec2, Vec2, u32, bool, bool)> = Vec::new();
    for row in ink.gathered.windows.iter() {
        let Some(slot) = row.slot else {
            continue;
        };
        let window = &row.window;
        let (position, size) = laid_out(window, viewport, config);
        placed.push((
            slot,
            position,
            size,
            window.order,
            window.open && !window.minimized && on_desktop,
            window.mode == WindowMode::Floating,
        ));
    }

    for (slot, position, size, order, showing, floating) in placed {
        ui_set_visible(ink.build.dirty, ink.build.commands, slot.root, showing);
        ui_set_visible(
            ink.build.dirty,
            ink.build.commands,
            slot.grip,
            showing && floating,
        );
        if !showing {
            continue;
        }
        set_frame(ink, slot.root, position, size, order);
    }
}

pub fn front_program(windows: &[WindowRow]) -> Option<usize> {
    windows
        .iter()
        .filter(|row| row.window.open && !row.window.minimized)
        .max_by_key(|row| row.window.order)
        .map(|row| row.window.program)
}

pub fn showing(windows: &[WindowRow], program: usize) -> bool {
    windows
        .iter()
        .any(|row| row.window.program == program && row.window.open && !row.window.minimized)
}

const WINDOW_DEPTH: f32 = 1000.0;

fn set_frame(
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    frame: Entity,
    position: Vec2,
    size: Vec2,
    order: u32,
) {
    ink.nodes.for_entity(frame, |node| {
        node.base_layout = Some(UiLayoutType::Window(WindowLayout {
            position: Ab(position).into(),
            size: Ab(size).into(),
            anchor: Anchor::TopLeft,
        }));
        node.depth = UiDepthMode::Set(1.0 + order as f32 * WINDOW_DEPTH);
    });
}

pub fn blink_caret(mut resources: ResMut<AppResources>, config: Res<Config>, time: Res<Time>) {
    let resources = &mut *resources;
    if resources.desk.screen != Screen::Login {
        return;
    }
    resources.login.caret_phase += time.delta_time;
    let lit = ((resources.login.caret_phase * config.caret_rate) as u32).is_multiple_of(2);
    if lit != resources.login.caret_lit {
        resources.login.caret_lit = lit;
        resources.redraw.machine = true;
    }
}

pub fn refresh(
    mut resources: ResMut<AppResources>,
    settings: Res<Config>,
    window: Res<Window>,
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
    if !resources.redraw.machine {
        return;
    }
    resources.redraw.machine = false;
    let config = *settings;
    let viewport = viewport_of(&window, &config);
    let screen = resources.desk.screen;

    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.login,
        screen == Screen::Login,
    );
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.desktop,
        screen == Screen::Desktop,
    );
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.over,
        screen == Screen::Over,
    );
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.board,
        true,
    );
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.sessions,
        true,
    );
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.cross,
        true,
    );
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.files,
        true,
    );
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.browser,
        true,
    );
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.lights,
        true,
    );
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.notes,
        true,
    );
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.shell,
        true,
    );
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.store,
        true,
    );
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.botterfly,
        true,
    );
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.office,
        true,
    );
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.case,
        true,
    );
    refresh_occupied(resources, ink);
    place_frames(resources, ink, viewport, &config);

    if let Some(index) = signed_in_account(&ink.gathered.desk)
        && let Some(account) = ACCOUNTS.get(index)
    {
        let line = format!("{}  Â·  {}", account.display_name, account.handle);
        ui_set_text(ink.build.commands, resources.interface.desktop_user, &line);
    }

    match screen {
        Screen::Over => refresh_over(resources, ink),
        Screen::Login => {
            refresh_login(resources, ink, &config);
            refresh_settings(resources, ink, &config);
        }
        Screen::Desktop => {
            let up = on_the_desktop(resources, ink);
            refresh_desktop(resources, ink, &config, &up);
            refresh_taskbar(ink, &config, &up);
            refresh_start_menu(resources, ink, &config, &up);
            refresh_game(resources, ink, &config);
            refresh_board(resources, ink, &config);
            refresh_sessions(resources, ink);
            for board in ["lights", "lights_on"] {
                let program = program_index(board);
                if showing(&ink.gathered.windows, program) {
                    refresh_lights(resources, ink, &config, program);
                }
            }
            if showing(&ink.gathered.windows, program_index("case")) {
                refresh_case(resources, ink, &config);
            }
            if showing(&ink.gathered.windows, program_index("office")) {
                refresh_office(resources, ink);
            }
            if showing(&ink.gathered.windows, program_index("botterfly")) {
                ui_set_text(
                    ink.build.commands,
                    resources.interface.botterfly_count,
                    &format!("{}", 8815 - 1),
                );
            }
            if showing(&ink.gathered.windows, program_index("cross")) {
                refresh_cross(resources, ink, &config);
            }
            if showing(&ink.gathered.windows, program_index("files")) {
                refresh_files(resources, ink, &config);
            }
            if showing(&ink.gathered.windows, program_index("browser")) {
                crate::systems::browser::refresh(resources, ink, viewport, &config);
            }
            refresh_notes(resources, ink);
        }
    }
    ui_mark_layout_dirty(ink.build.dirty);
}

fn refresh_occupied(resources: &mut AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>) {
    let guest = crate::data::machine::index_of("guest");
    let occupied = signed_in_account(&ink.gathered.desk)
        .filter(|index| *index != guest)
        .and_then(|index| ACCOUNTS.get(index));
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.occupied,
        occupied.is_some(),
    );
    let Some(account) = occupied else {
        return;
    };
    ui_set_text(
        ink.build.commands,
        resources.interface.occupied_handle,
        &format!("{} is already connected", account.handle),
    );
    ui_set_text(
        ink.build.commands,
        resources.interface.occupied_body,
        "This account has an open session. The server will not open a second one for the same handle.\n\nThe session opened at 3:07 AM and has not closed. No session on this server has closed.\n\nSign in as Guest to use the messenger. Everything else on this desk is still readable from here.",
    );
}

fn refresh_over(resources: &mut AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>) {
    let facts = resources.session.facts.clone();
    let ending = resources.session.ending;
    ui_set_text(
        ink.build.commands,
        resources.interface.over_heading,
        crate::data::endings::heading(ending),
    );
    ui_set_text(
        ink.build.commands,
        resources.interface.over_closing,
        &crate::data::endings::closing(ending).join(
            "
",
        ),
    );
    let tally = crate::data::endings::tally(&facts);
    let rows: Vec<TallyRow> = ink
        .gathered
        .rows
        .tallies
        .iter()
        .map(|(_, row)| *row)
        .collect();
    for row in rows {
        let Some((title, pinned, of)) = tally.get(row.lead) else {
            continue;
        };
        ui_set_text(
            ink.build.commands,
            row.label,
            &format!("{title:<22}{pinned} of {of}"),
        );
    }
    let (pinned, of) = progress(&facts);
    ui_set_text(
        ink.build.commands,
        resources.interface.over_verdict,
        crate::data::endings::verdict(ending, pinned, of),
    );
    ui_set_text(
        ink.build.commands,
        resources.interface.over_session,
        &format!(
            "{}   connected 3:07 AM   {}",
            resources.session.handle,
            crate::data::endings::footer(ending)
        ),
    );
}

fn refresh_settings(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.settings_panel,
        resources.login.settings_open,
    );
    if !resources.login.settings_open {
        return;
    }
    let wanted = resources.autoplay.wanted;
    let rows: Vec<crate::components::EndingRow> = ink
        .gathered
        .rows
        .endings
        .iter()
        .map(|(_, row)| *row)
        .collect();
    for row in rows {
        let Some((ending, label, note)) = crate::data::autoplay::OFFERED.get(row.which) else {
            ui_set_visible(ink.build.dirty, ink.build.commands, row.root, false);
            continue;
        };
        ui_set_visible(ink.build.dirty, ink.build.commands, row.root, true);
        ui_set_text(ink.build.commands, row.label, label);
        ui_set_text(ink.build.commands, row.note, note);
        ui_set_visible(
            ink.build.dirty,
            ink.build.commands,
            row.dot,
            *ending == wanted,
        );
        ui_set_color::<UiBase>(
            ink.build.commands,
            row.label,
            if *ending == wanted {
                config.text
            } else {
                config.text_dim
            },
        );
    }
}

fn refresh_login(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    ui_set_color::<UiBase>(
        ink.build.commands,
        resources.interface.password_box,
        config.field_focused,
    );
    let rows: Vec<(usize, RowSlot, bool)> = ink
        .gathered
        .desk
        .accounts
        .iter()
        .map(|row| (row.account, row.slot, row.opened))
        .collect();
    let typing_at = selected_account(&ink.gathered.desk);

    for (index, slot, opened) in rows {
        let Some(account) = ACCOUNTS.get(index) else {
            continue;
        };
        let selected = typing_at == Some(index);
        ui_set_text(ink.build.commands, slot.name, account.display_name);
        ui_set_text(ink.build.commands, slot.status, account.note);
        ui_set_text(
            ink.build.commands,
            slot.letter,
            &crate::data::script::avatar_letter(account.display_name),
        );
        ui_set_color::<UiBase>(
            ink.build.commands,
            slot.root,
            if selected {
                config.row_hover
            } else {
                config.transparent
            },
        );
        ui_set_color::<UiBase>(
            ink.build.commands,
            slot.name,
            if opened { config.text } else { config.text_dim },
        );
        ui_set_text(
            ink.build.commands,
            slot.badge_text,
            if opened { "" } else { "locked" },
        );
        ui_set_visible(ink.build.dirty, ink.build.commands, slot.badge, !opened);
    }

    let masked = "*".repeat(resources.login.typed.chars().count());
    let caret = if resources.login.caret_lit { "|" } else { " " };
    ui_set_text(
        ink.build.commands,
        resources.interface.password_field,
        &format!("{masked}{caret}"),
    );
    ui_set_text(
        ink.build.commands,
        resources.interface.refusal,
        &resources.login.refusal,
    );
    ui_set_text(
        ink.build.commands,
        resources.interface.sticky_note,
        STICKY_NOTE,
    );
}

fn on_the_desktop(resources: &AppResources, ink: &Ink<'_, '_, '_, '_, '_, '_>) -> Vec<bool> {
    let facts = &resources.session.facts;
    let mut up: Vec<bool> = PROGRAMS
        .iter()
        .map(|program| comes_with_the_desk(program, facts))
        .collect();
    for window in ink.gathered.windows.iter().map(|row| &row.window) {
        if window.installed
            && let Some(slot) = up.get_mut(window.program)
        {
            *slot = true;
        }
    }
    up
}

fn refresh_desktop(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
    up: &[bool],
) {
    let latest = PROGRAMS
        .iter()
        .enumerate()
        .rfind(|(index, program)| up[*index] && !program.installed_note.is_empty())
        .map(|(_, program)| program.installed_note)
        .unwrap_or_default();
    let icons: Vec<(Entity, usize)> = ink
        .gathered
        .rows
        .icons
        .iter()
        .map(|(entity, index)| (*entity, *index))
        .collect();
    for (icon, index) in icons {
        ui_set_visible(
            ink.build.dirty,
            ink.build.commands,
            icon,
            up.get(index).copied().unwrap_or(false),
        );
    }
    ui_set_text(ink.build.commands, resources.interface.desktop_note, latest);
    announce_offers(resources, up);
    announce_installs(resources, ink, up);
    refresh_notices(resources, ink, config);
    refresh_store(resources, ink, config);
    refresh_tonight(resources, ink);
    refresh_toasts(resources, ink);
}

pub fn watch_tonight(
    mut resources: ResMut<AppResources>,
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
    if !crate::data::machine::is_guest(signed_in_account(&ink.gathered.desk), resources.desk.screen)
    {
        return;
    }
    let lines = crate::data::tonight::left_to_do(resources);

    let learned: Vec<(Fact, &'static str)> = lines
        .iter()
        .filter(|(fact, text, done)| {
            !done
                && *text != crate::data::tonight::UNKNOWN
                && !resources.corner.revealed.contains(fact)
        })
        .map(|(fact, text, _)| (*fact, *text))
        .collect();
    for (fact, text) in learned {
        resources.corner.revealed.push(fact);
        raise_notice(resources, crate::data::tonight::REVEALED, text);
    }

    let ticked: Vec<(Fact, &'static str)> = lines
        .iter()
        .filter(|(fact, _, done)| *done && !resources.corner.ticked.contains(fact))
        .map(|(fact, text, _)| (*fact, *text))
        .collect();
    if ticked.is_empty() {
        return;
    }
    for (fact, text) in ticked {
        if !resources.corner.revealed.contains(&fact) {
            resources.corner.revealed.push(fact);
        }
        resources.corner.ticked.push(fact);
        raise_notice(resources, crate::data::tonight::TICKED, text);
    }
    if !lines.is_empty() && lines.iter().all(|(_, _, done)| *done) {
        raise_notice(
            resources,
            crate::data::tonight::FINISHED,
            crate::data::tonight::NOTHING_LEFT,
        );
    }
}

fn announce_offers(resources: &mut AppResources, up: &[bool]) {
    let waiting: Vec<usize> = crate::data::store::listed(&resources.session.facts)
        .into_iter()
        .filter(|listing| !resources.desk.store_offered.contains(listing))
        .collect();
    for listing in waiting {
        resources.desk.store_offered.push(listing);
        let Some(entry) = crate::data::store::LISTINGS.get(listing) else {
            continue;
        };
        let Some(program) = entry.program else {
            continue;
        };
        if index_of(program).is_some_and(|index| up.get(index).copied().unwrap_or(false)) {
            continue;
        }
        let text = format!("{} is waiting in the software list", entry.title);
        resources.corner.toasts.push(crate::resources::Toast {
            who: "software".to_string(),
            text: text.clone(),
            opens: crate::resources::ToastTarget::Store(program),
            age: 0.0,
        });
        keep_notice(resources, "software", &text);
        resources.redraw.machine = true;
    }
}

fn announce_installs(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    up: &[bool],
) {
    let arrived: Vec<(Entity, usize)> = ink
        .gathered
        .windows
        .iter()
        .map(|row| (row.entity, row.window))
        .filter(|(_, window)| !window.announced)
        .filter(|(_, window)| up.get(window.program).copied().unwrap_or(false))
        .filter(|(_, window)| {
            PROGRAMS
                .get(window.program)
                .is_some_and(|program| !program.installed_note.is_empty())
        })
        .map(|(entity, window)| (entity, window.program))
        .collect();
    for (entity, index) in arrived {
        change_window(
            &mut ink.gathered.windows,
            ink.build.commands,
            entity,
            |window| {
                window.announced = true;
            },
        );
        let Some(program) = PROGRAMS.get(index) else {
            continue;
        };
        let heading = format!("{} is on the desktop", program.label);
        raise_notice(resources, &heading, program.installed_note);
    }
}

pub fn raise_notice(resources: &mut AppResources, who: &str, text: &str) {
    if resources.desk.screen != Screen::Desktop {
        return;
    }
    resources.corner.toasts.push(crate::resources::Toast {
        who: who.to_string(),
        text: text.to_string(),
        opens: crate::resources::ToastTarget::Nothing,
        age: 0.0,
    });
    keep_notice(resources, who, text);
    resources.redraw.machine = true;
}

fn keep_notice(resources: &mut AppResources, who: &str, text: &str) {
    let stamp = crate::systems::interface::day_stamp(resources);
    resources.corner.notices.push(crate::resources::Notice {
        who: who.to_string(),
        text: text.to_string(),
        stamp,
    });
}

fn refresh_tonight(resources: &mut AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>) {
    let mine = crate::data::machine::is_guest(
        signed_in_account(&ink.gathered.desk),
        resources.desk.screen,
    ) && !resources.session.finished;
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.tonight,
        mine,
    );
    if !mine {
        return;
    }
    let heading = crate::data::tonight::heading(resources);
    ui_set_text(
        ink.build.commands,
        resources.interface.tonight_heading,
        heading,
    );

    let lines = crate::data::tonight::left_to_do(resources);
    let rows: Vec<ChecklistRow> = ink
        .gathered
        .rows
        .checklist
        .iter()
        .map(|(_, row)| *row)
        .collect();
    for row in rows {
        match lines.get(row.row) {
            Some((_, text, done)) => {
                ui_set_visible(ink.build.dirty, ink.build.commands, row.root, true);
                ui_set_text(ink.build.commands, row.label, text);
                ui_set_text(ink.build.commands, row.tick, if *done { "x" } else { "" });
            }
            None => ui_set_visible(ink.build.dirty, ink.build.commands, row.root, false),
        }
    }

    let done = lines.is_empty() || lines.iter().all(|(_, _, ticked)| *ticked);
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.tonight_done,
        done,
    );
    if !done {
        return;
    }
    let top = crate::data::tonight::FIRST_ROW
        + lines.len() as f32 * crate::data::tonight::ROW_PITCH
        + 6.0;
    ink.nodes
        .for_entity(resources.interface.tonight_done, |node| {
            node.base_layout = Some(UiLayoutType::Window(WindowLayout {
                position: Ab(vec2(18.0, top)).into(),
                size: Ab(vec2(232.0, 44.0)).into(),
                anchor: Anchor::TopLeft,
            }));
        });
    ui_set_text(
        ink.build.commands,
        resources.interface.tonight_done,
        crate::data::tonight::NOTHING_LEFT,
    );
}

fn refresh_taskbar(ink: &mut Ink<'_, '_, '_, '_, '_, '_>, config: &Config, up: &[bool]) {
    let running: Vec<(usize, bool, bool)> = ink
        .gathered
        .windows
        .iter()
        .map(|row| (row.entity, row.window))
        .map(|(_, window)| (window.program, window.open, window.minimized))
        .collect();
    let front = front_program(&ink.gathered.windows);
    let buttons: Vec<(Entity, TaskbarButton)> = ink
        .gathered
        .rows
        .taskbar
        .iter()
        .map(|(entity, parts)| (*entity, *parts))
        .collect();

    for (entity, parts) in buttons {
        let Some((_, open, minimized)) = running
            .iter()
            .copied()
            .find(|(program, _, _)| *program == parts.program)
        else {
            continue;
        };
        let has = up.get(parts.program).copied().unwrap_or_default();
        ui_set_visible(ink.build.dirty, ink.build.commands, entity, open && has);
        let ahead = front == Some(parts.program);
        let dim = if minimized {
            config.text_faint
        } else {
            config.text
        };
        ui_set_color::<UiBase>(ink.build.commands, parts.label, dim);
        ui_set_color::<UiBase>(
            ink.build.commands,
            parts.mark,
            if minimized {
                config.text_faint
            } else {
                config.brand
            },
        );
        ui_set_visible(
            ink.build.dirty,
            ink.build.commands,
            parts.lamp,
            open && has && ahead && !minimized,
        );
        let count = waiting_in(ink, parts.program);
        ui_set_visible(
            ink.build.dirty,
            ink.build.commands,
            parts.badge.root,
            open && has && count > 0,
        );
        ui_set_text(
            ink.build.commands,
            parts.badge.label,
            &if count > 99 {
                "99+".to_string()
            } else {
                count.to_string()
            },
        );
    }
}

fn waiting_in(ink: &Ink<'_, '_, '_, '_, '_, '_>, program: usize) -> u32 {
    if program != program_index("iam") {
        return 0;
    }
    let threads: u32 = ink
        .gathered
        .threads
        .contacts
        .iter()
        .filter(|state| state.listing == Listing::Listed)
        .map(|state| state.unread)
        .sum();
    let rooms: u32 = ink
        .gathered
        .threads
        .channels
        .iter()
        .map(|state| state.unread)
        .sum();
    let asking = ink
        .gathered
        .threads
        .contacts
        .iter()
        .filter(|state| state.listing == Listing::Pending)
        .count() as u32;
    threads + rooms + asking
}

fn refresh_start_menu(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
    up: &[bool],
) {
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.start_menu,
        resources.desk.menu_open,
    );

    let rows: Vec<StartRow> = ink
        .gathered
        .rows
        .starts
        .iter()
        .map(|(_, row)| *row)
        .collect();
    let mut installed_rows = 0.0;
    for row in rows {
        let shown = up.get(row.program).copied().unwrap_or_default();
        ui_set_visible(ink.build.dirty, ink.build.commands, row.slot.root, shown);
        if shown {
            installed_rows += 1.0;
        }
    }

    let below = MENU_TOP + installed_rows * MENU_ROW;
    place_menu_row(ink, resources.interface.start_divider, below + 8.0, 1.0);
    place_menu_row(ink, resources.interface.start_signoff, below + 17.0, 30.0);
    place_menu_row(ink, resources.interface.start_signout, below + 47.0, 30.0);

    let can_end = crate::systems::conversation::night_can_end(resources);
    ui_set_color::<UiBase>(
        ink.build.commands,
        resources.interface.start_signoff_label,
        if can_end {
            config.text_dim
        } else {
            config.text_faint
        },
    );
    let height = below + 84.0;

    ink.nodes
        .for_entity(resources.interface.start_menu, |node| {
            node.base_layout = Some(UiLayoutType::Window(WindowLayout {
                position: Rl(vec2(0.0, 100.0))
                    + Ab(vec2(6.0, -config.taskbar_height - height - 6.0)),
                size: Ab(vec2(232.0, height)).into(),
                anchor: Anchor::TopLeft,
            }));
        });
}

const MENU_TOP: f32 = 30.0;
const MENU_ROW: f32 = 34.0;

fn place_menu_row(ink: &mut Ink<'_, '_, '_, '_, '_, '_>, row: Entity, top: f32, height: f32) {
    let rule = height <= 1.0;
    let left = if rule { 14.0 } else { 6.0 };
    let width = if rule { 204.0 } else { 220.0 };
    ink.nodes.for_entity(row, |node| {
        node.base_layout = Some(UiLayoutType::Window(WindowLayout {
            position: Ab(vec2(left, top)).into(),
            size: Ab(vec2(width, height)).into(),
            anchor: Anchor::TopLeft,
        }));
    });
}

fn refresh_game(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let panel = resources.interface.game_panel;
    let Some(index) = resources.desk.playing else {
        ui_set_visible(ink.build.dirty, ink.build.commands, panel, false);
        return;
    };
    let posted = &POSTED[index];
    ui_set_visible(ink.build.dirty, ink.build.commands, panel, true);
    ui_set_text(
        ink.build.commands,
        resources.interface.game_title,
        &format!(
            "{}  Â·  posted by {}  Â·  {}",
            posted.title, posted.author, posted.id
        ),
    );
    ui_set_text(
        ink.build.commands,
        resources.interface.game_blurb,
        posted.blurb,
    );

    let squares: Vec<(Entity, GamePlace)> = ink
        .gathered
        .rows
        .places
        .iter()
        .map(|(entity, square)| (*entity, *square))
        .collect();
    for (root, square) in squares {
        let up = square.place < posted.start.chars().count();
        ui_set_visible(ink.build.dirty, ink.build.commands, root, up);
        if !up {
            continue;
        }
        ui_set_text(ink.build.commands, square.label, &square.face.to_string());
        ui_set_color::<UiBase>(ink.build.commands, root, config.board_dark);
    }

    let prize = if resources.desk.board_won {
        posted.prize
    } else {
        ""
    };
    ui_set_text(ink.build.commands, resources.interface.game_prize, prize);
}

fn refresh_sessions(resources: &mut AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>) {
    let rows: Vec<(Entity, usize)> = ink.gathered.rows.sessions.clone();
    for (entity, index) in rows {
        let Some(account) = ACCOUNTS.get(index) else {
            continue;
        };
        let line = format!(
            "{:<14} {:<14} opened 3:07 AM     closed  --",
            account.display_name, account.handle
        );
        ui_set_text(ink.build.commands, entity, &line);
    }
    ui_set_text(
        ink.build.commands,
        resources.interface.sessions_total,
        &format!(
            "{} sessions opened on this machine, none closed",
            ACCOUNTS.len()
        ),
    );
}

fn refresh_lights(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
    program: usize,
) {
    let goal = goal_of(program);
    let out = lamps_finished(&ink.gathered.rows, program, goal);
    let rows: Vec<LampRow> = ink
        .gathered
        .rows
        .lamps
        .iter()
        .filter(|(_, row)| row.program == program)
        .map(|(_, row)| *row)
        .collect();
    for row in rows {
        ui_set_color::<UiBase>(
            ink.build.commands,
            row.slot.root,
            if row.lit {
                config.board_lit
            } else {
                config.board_dark
            },
        );
        ui_set_color::<UiBase>(
            ink.build.commands,
            row.slot.label,
            if row.lit {
                config.text
            } else {
                config.text_faint
            },
        );
    }

    if goal == lights::Goal::On {
        ui_set_text(
            ink.build.commands,
            resources.interface.lights_on_says,
            if out { "there you are." } else { "" },
        );
        return;
    }
    ui_set_text(
        ink.build.commands,
        resources.interface.lights_moves,
        &format!("{} presses", resources.desk.lamp_moves),
    );
    ui_set_text(
        ink.build.commands,
        resources.interface.lights_says,
        if out {
            lights::remark(resources.desk.lamps_done)
        } else {
            ""
        },
    );
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.lights_again,
        out,
    );
    ui_set_text(
        ink.build.commands,
        resources.interface.lights_count,
        &if out {
            format!(
                "{} people have finished this. i keep count",
                lights::FINISHERS
            )
        } else {
            String::new()
        },
    );
}

fn lamps_finished(rows: &Rows, program: usize, goal: lights::Goal) -> bool {
    let wanted = goal == lights::Goal::On;
    let mut any = false;
    for (_, row) in rows.lamps.iter() {
        if row.program != program {
            continue;
        }
        if row.lit != wanted {
            return false;
        }
        any = true;
    }
    any
}

fn goal_of(program: usize) -> lights::Goal {
    if program == program_index("lights_on") {
        lights::Goal::On
    } else {
        lights::Goal::Out
    }
}

fn deal_lamps(ink: &mut Ink<'_, '_, '_, '_, '_, '_>, program: usize, board: &str) {
    let lit: Vec<bool> = board.chars().map(|face| face == '1').collect();
    for (entity, row) in ink.gathered.rows.lamps.iter_mut() {
        if row.program != program {
            continue;
        }
        row.lit = lit.get(row.place).copied().unwrap_or_default();
        ink.build.commands.set(*entity, *row);
    }
}

pub fn press_lamp(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    program: usize,
    place: usize,
) {
    let goal = goal_of(program);
    if lamps_finished(&ink.gathered.rows, program, goal) {
        return;
    }
    let turns = [place.wrapping_sub(1), place, place + 1];
    for (entity, row) in ink.gathered.rows.lamps.iter_mut() {
        if row.program == program && turns.contains(&row.place) {
            row.lit = !row.lit;
            ink.build.commands.set(*entity, *row);
        }
    }
    if goal == lights::Goal::Out {
        resources.desk.lamp_moves += 1;
    }
    if !lamps_finished(&ink.gathered.rows, program, goal) {
        resources.redraw.machine = true;
        return;
    }
    match goal {
        lights::Goal::Out => {
            resources.desk.lamps_done += 1;
            crate::resources::learn(resources, &[lights::PLAYED, lights::COUNTED]);
        }
        lights::Goal::On => {
            crate::resources::learn(resources, &[lights::TURNED_ON]);
            resources.desk.case_file = 0;
            toggle(resources, ink, program_index("case"));
            raise(resources, ink, program_index("case"));
        }
    }
    resources.redraw = crate::resources::redraw_everything();
}

pub fn open_case(resources: &mut AppResources, file: usize) {
    let Some(dossier) = crate::data::case::DOSSIERS.get(file) else {
        return;
    };
    resources.desk.case_file = file;
    crate::resources::learn(resources, dossier.asserts);
    resources.redraw = crate::resources::redraw_everything();
}

pub fn next_lamps(resources: &mut AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>) {
    let program = program_index("lights");
    deal_lamps(ink, program, lights::next_board(resources.desk.lamps_done));
    resources.desk.lamp_moves = 0;
    resources.redraw.machine = true;
}

pub fn go_place(resources: &mut AppResources, place: usize) {
    let Some(entry) = crate::data::office::PLACES.get(place) else {
        return;
    };
    resources.desk.place = place;
    resources.desk.looked_at = None;
    crate::resources::learn(resources, entry.asserts);
    resources.redraw.machine = true;
}

pub fn look_at(resources: &mut AppResources, thing: usize) {
    let Some(entry) = crate::data::office::THINGS.get(thing) else {
        return;
    };
    resources.desk.looked_at = Some(thing);
    crate::resources::learn(resources, entry.asserts);
    if let Some(ending) = entry.finishes {
        resources.session.ending = ending;
        resources.session.finished = true;
        resources.session.held_open = true;
        resources.desk.screen = Screen::Over;
        resources.redraw = crate::resources::redraw_everything();
        return;
    }
    resources.redraw.machine = true;
}

fn refresh_office(resources: &mut AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>) {
    let Some(place) = crate::data::office::PLACES.get(resources.desk.place) else {
        return;
    };
    ui_set_text(
        ink.build.commands,
        resources.interface.office_name,
        place.name,
    );
    ui_set_text(
        ink.build.commands,
        resources.interface.office_body,
        place.body,
    );

    let looking = resources
        .desk
        .looked_at
        .and_then(|thing| crate::data::office::THINGS.get(thing));
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.office_look,
        looking.is_some(),
    );
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.office_body,
        looking.is_none(),
    );
    if let Some(thing) = looking {
        ui_set_text(
            ink.build.commands,
            resources.interface.office_look,
            thing.body,
        );
    }

    let open = crate::data::office::ways_here(place, &resources.session.facts);
    let ways: Vec<WayRow> = ink.gathered.rows.ways.iter().map(|(_, row)| *row).collect();
    for row in ways {
        match open.get(row.way) {
            Some(index) => {
                ui_set_visible(ink.build.dirty, ink.build.commands, row.slot.root, true);
                ui_set_text(ink.build.commands, row.slot.label, place.ways[*index].label);
            }
            None => ui_set_visible(ink.build.dirty, ink.build.commands, row.slot.root, false),
        }
    }

    let here = crate::data::office::things_here(place, &resources.session.facts);
    let things: Vec<ThingRow> = ink
        .gathered
        .rows
        .things
        .iter()
        .map(|(_, row)| *row)
        .collect();
    for row in things {
        match here.get(row.thing) {
            Some(index) => {
                ui_set_visible(ink.build.dirty, ink.build.commands, row.slot.root, true);
                ui_set_text(
                    ink.build.commands,
                    row.slot.label,
                    crate::data::office::THINGS[*index].label,
                );
            }
            None => ui_set_visible(ink.build.dirty, ink.build.commands, row.slot.root, false),
        }
    }
}

fn refresh_case(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let Some(dossier) = crate::data::case::DOSSIERS.get(resources.desk.case_file) else {
        return;
    };
    ui_set_text(
        ink.build.commands,
        resources.interface.case_title,
        dossier.summary,
    );
    ui_set_text(
        ink.build.commands,
        resources.interface.case_body,
        dossier.body,
    );

    let rows: Vec<CaseRow> = ink
        .gathered
        .rows
        .cases
        .iter()
        .map(|(_, row)| *row)
        .collect();
    for row in rows {
        let open = row.file == resources.desk.case_file;
        ui_set_color::<UiBase>(
            ink.build.commands,
            row.slot.root,
            if open {
                config.row_pressed
            } else {
                config.pane_background
            },
        );
    }
}

pub fn type_into_store(
    mut resources: ResMut<AppResources>,
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
    if !resources.desk.store_focused || resources.desk.screen != Screen::Desktop {
        return;
    }
    if front_program(&ink.gathered.windows) != Some(program_index("store")) {
        resources.desk.store_focused = false;
        return;
    }
    let keyboard = &input.keyboard;
    let typed: Vec<char> = keyboard
        .frame_chars
        .iter()
        .copied()
        .filter(|character| !character.is_control())
        .collect();
    let backspace = actions.just_pressed(Ask::Erase);
    let clear = actions.just_pressed(Ask::Clear);
    if clear {
        resources.desk.store_typed.clear();
        resources.desk.store_focused = false;
        resources.redraw.machine = true;
        return;
    }
    if backspace {
        resources.desk.store_typed.pop();
        resources.redraw.machine = true;
    }
    if !typed.is_empty() {
        resources.desk.store_typed.extend(typed);
        resources.redraw.machine = true;
    }
}

const UNAVAILABLE_MARK: char = '\u{e3ec}';

fn refresh_store(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let shown = crate::data::store::matching(&resources.session.facts, &resources.desk.store_typed);
    let typing = resources.desk.store_focused;
    let caret = if typing && resources.login.caret_lit {
        "|"
    } else {
        ""
    };
    let field = if resources.desk.store_typed.is_empty() && !typing {
        "search the list".to_string()
    } else {
        format!("{}{}", resources.desk.store_typed, caret)
    };
    ui_set_text(ink.build.commands, resources.interface.store_caret, &field);
    ui_set_color::<UiBase>(
        ink.build.commands,
        resources.interface.store_caret,
        if resources.desk.store_typed.is_empty() && !typing {
            config.text_faint
        } else {
            config.text
        },
    );
    ui_set_color::<UiBase>(
        ink.build.commands,
        resources.interface.store_field,
        if typing {
            config.address_focused
        } else {
            config.address_field
        },
    );

    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.store_empty,
        shown.is_empty(),
    );
    ui_set_text(
        ink.build.commands,
        resources.interface.store_empty,
        if resources.desk.store_typed.trim().is_empty() {
            "the list is empty"
        } else {
            "nothing on the list matches that"
        },
    );

    let rows: Vec<StoreRow> = ink
        .gathered
        .rows
        .store
        .iter()
        .map(|(_, row)| *row)
        .collect();
    for mut row in rows {
        let Some(listing) = shown.get(row.slot).and_then(|index| {
            crate::data::store::LISTINGS
                .get(*index)
                .map(|entry| (*index, entry))
        }) else {
            ui_set_visible(ink.build.dirty, ink.build.commands, row.root, false);
            continue;
        };
        let (index, listing) = listing;
        if row.listing != index {
            row.listing = index;
            ink.build.commands.set(row.root, row);
            ink.build.commands.set(row.button, StoreButton(index));
        }
        ui_set_visible(ink.build.dirty, ink.build.commands, row.root, true);

        let got = listing
            .program
            .and_then(index_of)
            .and_then(|program| window_entity(&ink.gathered.windows, program))
            .and_then(|entity| ink.gathered.windows.iter().find(|row| row.entity == entity))
            .is_some_and(|row| row.window.installed);
        let available = listing.program.is_some();
        let mut mark = [0u8; 4];
        let face = listing
            .program
            .and_then(index_of)
            .and_then(|index| PROGRAMS.get(index))
            .map(|program| program.glyph.codepoint)
            .unwrap_or(UNAVAILABLE_MARK);
        ui_set_text(ink.build.commands, row.glyph, face.encode_utf8(&mut mark));
        ui_set_text(ink.build.commands, row.title, listing.title);
        ui_set_text(
            ink.build.commands,
            row.by,
            &format!("{}    {}", listing.by, listing.size),
        );
        ui_set_text(ink.build.commands, row.blurb, listing.blurb);
        ui_set_text(ink.build.commands, row.note, listing.note);
        ui_set_text(
            ink.build.commands,
            row.button_label,
            match (available, got) {
                (false, _) => "UNAVAILABLE",
                (true, true) => "INSTALLED",
                (true, false) => "INSTALL",
            },
        );
        ui_set_color::<UiBase>(
            ink.build.commands,
            row.button_label,
            if available && !got {
                config.brand
            } else {
                config.text_faint
            },
        );
        ui_set_color::<UiBase>(
            ink.build.commands,
            row.button,
            if available && !got {
                config.composer_background
            } else {
                config.pane_background
            },
        );
        ui_set_color::<UiBase>(
            ink.build.commands,
            row.root,
            if resources.desk.store_at == Some(index) && !got {
                config.row_hover
            } else {
                config.composer_background
            },
        );
    }
}

pub fn install_from_store(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    listing: usize,
) {
    let Some(entry) = crate::data::store::LISTINGS.get(listing) else {
        return;
    };
    if !crate::data::store::listed(&resources.session.facts).contains(&listing) {
        return;
    }
    let Some(window) = entry
        .program
        .and_then(index_of)
        .and_then(|index| window_entity(&ink.gathered.windows, index))
    else {
        return;
    };
    if ink
        .gathered
        .windows
        .iter()
        .any(|row| row.entity == window && row.window.installed)
    {
        return;
    }
    change_window(
        &mut ink.gathered.windows,
        ink.build.commands,
        window,
        |open| {
            open.installed = true;
        },
    );
    resources.redraw.machine = true;
}

pub fn open_store_at(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    program: &str,
) {
    resources.desk.store_at = crate::data::store::for_program(program);
    let Some(entity) = window_entity(&ink.gathered.windows, program_index("store")) else {
        return;
    };
    if ink
        .gathered
        .windows
        .iter()
        .any(|row| row.entity == entity && row.window.open && !row.window.minimized)
    {
        resources.redraw.machine = true;
        return;
    }
    toggle(resources, ink, program_index("store"));
}

fn refresh_cross(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let made_pairs = crate::data::cross::readable(&resources.session.facts);
    let mut gained: Vec<Fact> = Vec::new();
    let mut apart = 0usize;

    let rows: Vec<PairingRow> = ink
        .gathered
        .rows
        .pairings
        .iter()
        .map(|(_, row)| *row)
        .collect();
    for row in rows {
        let index = row.pairing;
        let slot = row.slot;
        let Some(pairing) = PAIRINGS.get(index) else {
            continue;
        };
        let paired = made_pairs.contains(&index);
        ui_set_visible(ink.build.dirty, ink.build.commands, slot.root, paired);
        if !paired {
            continue;
        }
        if !pairing.agrees {
            apart += 1;
        }
        ui_set_text(ink.build.commands, slot.source, pairing.source);
        ui_set_text(
            ink.build.commands,
            slot.claim,
            &format!("said     {}", pairing.claim),
        );
        ui_set_text(
            ink.build.commands,
            slot.record,
            &format!("held     {}", pairing.record),
        );
        ui_set_text(ink.build.commands, slot.reading, pairing.reading);
        let (verdict, tint) = if pairing.agrees {
            ("agrees", config.text_faint)
        } else {
            ("differs", config.brand)
        };
        ui_set_text(ink.build.commands, slot.verdict, verdict);
        ui_set_color::<UiBase>(ink.build.commands, slot.verdict, tint);
        gained.extend(pairing.asserts);
    }

    let summary = if made_pairs.is_empty() {
        "nothing to set beside anything yet".to_string()
    } else {
        format!("{} pairs made, {apart} of them apart", made_pairs.len())
    };
    ui_set_text(
        ink.build.commands,
        resources.interface.cross_summary,
        &summary,
    );

    crate::resources::learn(resources, &gained);
}

fn refresh_files(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let opened = opened_accounts(ink);
    let on_disk = readable_file(&opened, &resources.session.facts);

    if open_file(&ink.gathered.files).is_none_or(|index| !on_disk.contains(&index)) {
        open_this_file(
            ink.build.commands,
            &ink.gathered.files,
            on_disk.first().copied(),
        );
    }
    let reading = open_file(&ink.gathered.files);

    let rows: Vec<FileRow> = ink
        .gathered
        .rows
        .files
        .iter()
        .map(|(_, row)| *row)
        .collect();
    for row in rows {
        let index = row.file;
        let slot = row.slot;
        let Some(file) = LEFTOVERS.get(index) else {
            continue;
        };
        let present = on_disk.contains(&index);
        ui_set_visible(ink.build.dirty, ink.build.commands, slot.root, present);
        if !present {
            continue;
        }
        let open = reading == Some(index);
        ui_set_color::<UiBase>(
            ink.build.commands,
            slot.root,
            if open {
                config.icon_selected
            } else {
                config.transparent
            },
        );
        ui_set_text(ink.build.commands, slot.name, file.name);
        ui_set_color::<UiBase>(
            ink.build.commands,
            slot.name,
            if open { config.text } else { config.text_dim },
        );
        ui_set_text(
            ink.build.commands,
            slot.owner,
            &file.owner.replace('_', "."),
        );
    }

    match reading.and_then(|index| LEFTOVERS.get(index)) {
        Some(file) => {
            ui_set_text(ink.build.commands, resources.interface.file_name, file.name);
            ui_set_text(
                ink.build.commands,
                resources.interface.file_stamp,
                file.stamp,
            );
            ui_set_text(ink.build.commands, resources.interface.file_body, file.body);
            crate::resources::learn(resources, file.asserts);
        }
        None => {
            ui_set_text(ink.build.commands, resources.interface.file_name, "");
            ui_set_text(ink.build.commands, resources.interface.file_stamp, "");
            ui_set_text(
                ink.build.commands,
                resources.interface.file_body,
                "nothing here reads yet. the disk holds what the people before you left in their own accounts, and an account has to be opened once before the rest of the machine can see into it.",
            );
        }
    }

    ui_set_text(
        ink.build.commands,
        resources.interface.files_count,
        &format!(
            "{} files, across {} of the accounts on this machine",
            on_disk.len(),
            opened.len()
        ),
    );
}

fn refresh_notes(resources: &mut AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>) {
    let facts = &resources.session.facts;
    let mut shown = 0;
    let cards: Vec<NoteCard> = ink
        .gathered
        .rows
        .notes
        .iter()
        .map(|(_, card)| *card)
        .collect();
    for card in cards {
        let Some(note) = NOTES.get(card.note) else {
            continue;
        };
        let up = readable(note, facts);
        ui_set_visible(ink.build.dirty, ink.build.commands, card.root, up);
        if up {
            shown += 1;
            ui_set_text(ink.build.commands, card.text, note.text);
            ui_set_text(
                ink.build.commands,
                card.author,
                &format!("- {}", note.author),
            );
        }
    }
    ui_set_text(
        ink.build.commands,
        resources.interface.notes_count,
        &format!("{shown} left on this desk"),
    );
}

fn refresh_board(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let facts = &resources.session.facts;
    let cards: Vec<(Entity, BoardCard)> = ink
        .gathered
        .rows
        .cards
        .iter()
        .map(|(entity, card)| (*entity, *card))
        .collect();

    let columns: Vec<BoardColumn> = ink
        .gathered
        .rows
        .columns
        .iter()
        .map(|(_, column)| *column)
        .collect();
    for column in columns {
        let Some(lead) = LEADS.get(column.lead) else {
            continue;
        };
        let up = pinned(lead, facts);
        let title = if named(lead, facts) {
            format!("{}  {}/{}", lead.title, up, lead.notes.len())
        } else {
            format!("{}  ?", lead.unknown)
        };
        ui_set_text(ink.build.commands, column.heading, &title);
        ui_set_color::<UiBase>(
            ink.build.commands,
            column.heading,
            if up == lead.notes.len() {
                config.text
            } else {
                config.text_dim
            },
        );
        let footer = if up == lead.notes.len() {
            opens_into(lead, facts)
        } else {
            String::new()
        };
        ui_set_text(ink.build.commands, column.footer, &footer);
    }

    for (entity, card) in cards {
        let note = LEADS
            .get(card.lead)
            .and_then(|lead| lead.notes.get(card.note));
        let known = note
            .map(|note| facts.contains(&note.fact))
            .unwrap_or_default();
        ui_set_visible(ink.build.dirty, ink.build.commands, entity, known);
        if known && let Some(note) = note {
            ui_set_text(ink.build.commands, card.label, note.text);
        }
    }

    let (up, total) = progress(facts);
    ui_set_text(
        ink.build.commands,
        resources.interface.board_progress,
        &format!("{up} of {total} pinned"),
    );
}
