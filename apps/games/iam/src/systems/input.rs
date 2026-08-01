use crate::components::{
    AccountRow, BookmarkRow, CaseRow, ChoiceSlot, EndingRow, FileRow, FrameButton, FrameButtonKind,
    GamePlace, LampRow, LinkRow, PageBlock, ProgramIcon, RowChannel, RowContact, RowServer,
    StartRow, StoreButton, StoreLink, TaskbarButton, ThingRow, ToastRow, WayRow,
};
use crate::components::{ContactListing, Listing, WindowMode};
use crate::resources::{Action, AppResources, View};
use crate::systems::desk::WindowRow;
use nightshade::prelude::Entity;
use nightshade::prelude::*;
use nightshade_ui::prelude::*;

enum FrameAction {
    Minimize,
    Maximize,
    Restore,
    Close,
}

#[derive(Default)]
struct Pressed {
    lamp: Option<LampRow>,
    ending: Option<EndingRow>,
    store_button: Option<StoreButton>,
    store_link: Option<StoreLink>,
    case: Option<CaseRow>,
    toast: Option<ToastRow>,
    way: Option<WayRow>,
    thing: Option<ThingRow>,
    start: Option<StartRow>,
    bookmark: Option<BookmarkRow>,
    block: Option<PageBlock>,
    link: Option<LinkRow>,
    file: Option<FileRow>,
    frame: Option<FrameButton>,
    icon: Option<ProgramIcon>,
    taskbar: Option<TaskbarButton>,
    place: Option<GamePlace>,
    account: Option<AccountRow>,
    contact: Option<RowContact>,
    server: Option<RowServer>,
    channel: Option<RowChannel>,
    choice: bool,
}

fn browser_click(resources: &mut AppResources, clicked: Entity, pressed: &Pressed) -> bool {
    let interface = &resources.interface;
    if clicked == interface.browser_back {
        crate::systems::browser::back(resources);
        return true;
    }
    if clicked == interface.browser_forward {
        crate::systems::browser::forward(resources);
        return true;
    }
    if clicked == interface.browser_reload {
        crate::systems::browser::reload(resources);
        return true;
    }
    if clicked == interface.browser_address {
        resources.browser.address_focused = true;
        resources.browser.folder_open = false;
        resources.redraw.machine = true;
        return true;
    }
    if clicked == interface.browser_folder {
        resources.browser.folder_open = !resources.browser.folder_open;
        resources.redraw.machine = true;
        return true;
    }
    if clicked == interface.browser_prompt_field {
        resources.browser.address_focused = false;
        resources.redraw.machine = true;
        return true;
    }
    if clicked == interface.browser_prompt_go {
        crate::systems::browser::submit_field(resources);
        return true;
    }
    if let Some(mark) = pressed.bookmark.map(|row| row.bookmark)
        && let Some(bookmark) = crate::data::web::BOOKMARKS.get(mark)
    {
        crate::systems::browser::go(resources, bookmark.url, true);
        return true;
    }
    if let Some(block) = pressed.block.map(|row| row.block)
        && let Some(index) = resources.browser.browsing
        && let Some(crate::data::web::Block::Row { goes_to, .. }) =
            crate::data::web::PAGES[index].body.get(block)
    {
        crate::systems::browser::go(resources, goes_to, true);
        return true;
    }
    if let Some(link) = pressed.link.map(|row| row.link)
        && let Some(index) = resources.browser.browsing
        && let Some(url) = crate::data::web::PAGES[index].links.get(link)
    {
        crate::systems::browser::go(resources, url, true);
        return true;
    }
    false
}

fn frame_button(pressed: &Pressed, windows: &[WindowRow]) -> Option<(usize, FrameAction)> {
    let button = pressed.frame?;
    let filling = windows
        .iter()
        .find(|row| row.window.program == button.program)
        .map(|row| row.window.mode != WindowMode::Floating)
        .unwrap_or_default();
    let action = match button.kind {
        FrameButtonKind::Minimize => FrameAction::Minimize,
        FrameButtonKind::Close => FrameAction::Close,
        FrameButtonKind::Size if filling => FrameAction::Restore,
        FrameButtonKind::Size => FrameAction::Maximize,
    };
    Some((button.program, action))
}

type PressedFirst<'row> = (
    Option<&'row LampRow>,
    Option<&'row EndingRow>,
    Option<&'row StoreButton>,
    Option<&'row StoreLink>,
    Option<&'row CaseRow>,
    Option<&'row ToastRow>,
    Option<&'row WayRow>,
    Option<&'row ThingRow>,
);

type PressedSecond<'row> = (
    Option<&'row StartRow>,
    Option<&'row BookmarkRow>,
    Option<&'row PageBlock>,
    Option<&'row LinkRow>,
    Option<&'row FileRow>,
    Option<&'row FrameButton>,
    Option<&'row ProgramIcon>,
    Option<&'row TaskbarButton>,
);

type PressedThird<'row> = (
    Option<&'row GamePlace>,
    Option<&'row AccountRow>,
    Option<&'row RowContact>,
    Option<&'row RowServer>,
    Option<&'row RowChannel>,
    Option<&'row ChoiceSlot>,
    Option<&'row ContactListing>,
);

pub fn poll(
    mut resources: ResMut<AppResources>,
    frame: Res<RetainedUiFrame>,
    gathered: Res<crate::systems::desk::Gathered>,
    mut first: Query<PressedFirst>,
    mut second: Query<PressedSecond>,
    mut third: Query<PressedThird>,
) {
    let resources = &mut *resources;
    if crate::systems::intro::covered(resources)
        || crate::systems::screensaver::covering(resources)
        || resources.session.terminated
    {
        return;
    }
    let clicks: Vec<Entity> = ui_events(&frame)
        .iter()
        .filter_map(|event| match event {
            UiEvent::ButtonClicked(entity) => Some(*entity),
            _ => None,
        })
        .collect();

    let mut queued: Vec<Action> = Vec::new();
    for clicked in clicks {
        let mut pressed = Pressed::default();
        first.for_entity(
            clicked,
            |(lamp, ending, button, link, case, toast, way, thing)| {
                pressed.lamp = lamp.copied();
                pressed.ending = ending.copied();
                pressed.store_button = button.copied();
                pressed.store_link = link.copied();
                pressed.case = case.copied();
                pressed.toast = toast.copied();
                pressed.way = way.copied();
                pressed.thing = thing.copied();
            },
        );
        second.for_entity(
            clicked,
            |(start, bookmark, block, link, file, frame, icon, taskbar)| {
                pressed.start = start.copied();
                pressed.bookmark = bookmark.copied();
                pressed.block = block.copied();
                pressed.link = link.copied();
                pressed.file = file.copied();
                pressed.frame = frame.copied();
                pressed.icon = icon.copied();
                pressed.taskbar = taskbar.copied();
            },
        );
        third.for_entity(
            clicked,
            |(place, account, contact, server, channel, choice, _)| {
                pressed.place = place.copied();
                pressed.account = account.copied();
                pressed.contact = contact.copied();
                pressed.server = server.copied();
                pressed.channel = channel.copied();
                pressed.choice = choice.is_some();
            },
        );

        if let Some(lamp) = pressed.lamp {
            queued.push(Action::PressLamp(lamp.program, lamp.place));
            continue;
        }
        if clicked == resources.interface.settings_cog {
            queued.push(Action::ToggleSettings);
            continue;
        }
        if clicked == resources.interface.settings_start {
            queued.push(Action::StartAutoplay);
            continue;
        }
        if let Some(row) = pressed.ending.map(|row| row.which) {
            queued.push(Action::WantEnding(row));
            continue;
        }
        if clicked == resources.interface.store_field {
            resources.desk.store_focused = true;
            resources.redraw.machine = true;
            continue;
        }
        if clicked == resources.interface.botterfly_go {
            queued.push(Action::Upload);
            continue;
        }
        if let Some(button) = pressed.store_button {
            queued.push(Action::Install(button.0));
            continue;
        }
        if let Some(link) = pressed.store_link {
            queued.push(Action::OpenStore(link.0));
            continue;
        }
        if let Some(file) = pressed.case.map(|row| row.file) {
            queued.push(Action::OpenCase(file));
            continue;
        }
        if let Some(slot) = pressed.toast.map(|row| row.slot)
            && let Some(target) = crate::systems::desk::toast_target(resources, slot)
        {
            resources.corner.toasts.clear();
            if matches!(
                target,
                crate::resources::ToastTarget::Thread(_)
                    | crate::resources::ToastTarget::Room { .. }
            ) {
                queued.push(Action::ToggleProgram(crate::systems::setup::program_index(
                    "iam",
                )));
            }
            match target {
                crate::resources::ToastTarget::Thread(thread) => {
                    queued.push(Action::OpenThread(thread));
                }
                crate::resources::ToastTarget::Room { server, channel } => {
                    queued.push(Action::OpenServer(server));
                    queued.push(Action::OpenChannel(channel));
                }
                crate::resources::ToastTarget::Store(program) => {
                    queued.push(Action::OpenStore(program));
                }
                crate::resources::ToastTarget::Nothing => {}
            }
            continue;
        }
        if let Some(way) = pressed.way.map(|row| row.way) {
            queued.push(Action::GoPlace(way));
            continue;
        }
        if let Some(thing) = pressed.thing.map(|row| row.thing) {
            queued.push(Action::LookAt(thing));
            continue;
        }
        if clicked == resources.interface.lights_again {
            queued.push(Action::NextLamps);
            continue;
        }
        if clicked == resources.interface.sign_in {
            queued.push(Action::SignIn);
            continue;
        }
        if clicked == resources.interface.taskbar_calendar {
            resources.corner.notices_open = !resources.corner.notices_open;
            resources.corner.notices_at = 0;
            resources.desk.menu_open = false;
            resources.redraw.machine = true;
            continue;
        }
        if clicked == resources.interface.taskbar_start {
            resources.desk.menu_open = !resources.desk.menu_open;
            resources.corner.notices_open = false;
            resources.redraw.machine = true;
            continue;
        }
        if clicked == resources.interface.start_signout {
            resources.desk.menu_open = false;
            queued.push(Action::SignOut);
            continue;
        }
        if clicked == resources.interface.start_signoff {
            if !crate::systems::conversation::night_can_end(resources) {
                continue;
            }
            resources.desk.menu_open = false;
            queued.push(Action::SignOff);
            continue;
        }
        if let Some(program) = pressed.start.map(|row| row.program) {
            resources.desk.menu_open = false;
            queued.push(Action::ToggleProgram(program));
            continue;
        }
        if browser_click(resources, clicked, &pressed) {
            continue;
        }
        if let Some(file) = pressed.file.map(|row| row.file) {
            queued.push(Action::OpenFile(file));
            continue;
        }
        if let Some((index, action)) = frame_button(&pressed, &gathered.windows) {
            queued.push(match action {
                FrameAction::Minimize => Action::MinimizeProgram(index),
                FrameAction::Maximize => Action::SizeWindow(index, WindowMode::Maximized),
                FrameAction::Restore => Action::SizeWindow(index, WindowMode::Floating),
                FrameAction::Close => Action::CloseProgram(index),
            });
        } else if let Some(icon) = pressed.icon {
            queued.push(Action::ToggleProgram(icon.0));
        } else if let Some(button) = pressed.taskbar {
            queued.push(Action::ToggleProgram(button.program));
        } else if clicked == resources.interface.quit_button {
            queued.push(Action::Quit);
        } else if clicked == resources.interface.game_close {
            queued.push(Action::CloseGame);
        } else if let Some(place) = pressed.place {
            queued.push(Action::PressBoard(place.place));
        } else if let Some(account) = pressed.account {
            queued.push(Action::SelectAccount(account.definition));
        } else if clicked == resources.interface.offline_button {
            queued.push(Action::ToggleOffline);
        } else if let Some(entity) = pressed.contact.and_then(|row| row.0) {
            let mut pending = false;
            third.for_entity(entity, |(_, _, _, _, _, _, listing)| {
                pending = listing.is_some_and(|listing| listing.0 == Listing::Pending);
            });
            queued.push(if pending {
                Action::AcceptInvite(entity)
            } else {
                Action::OpenThread(entity)
            });
        } else if let Some(entity) = pressed.server.and_then(|row| row.0) {
            queued.push(Action::OpenServer(entity));
        } else if let Some(entity) = pressed.channel.and_then(|row| row.0) {
            queued.push(Action::OpenChannel(entity));
        } else if pressed.choice {
            queued.push(match resources.session.view {
                View::Server(_) => Action::Ask(clicked),
                _ => Action::Reply(clicked),
            });
        }
    }

    resources.actions.extend(queued);
}
