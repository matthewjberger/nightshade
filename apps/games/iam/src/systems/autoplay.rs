use crate::components::{Conversation, Listing};
use crate::data::autoplay::{Source, kept};
use crate::data::machine::Screen;
use crate::data::script::{Fact, Presence};
use crate::resources::{Action, AppResources, Ink};
use nightshade::prelude::Entity;
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;
use nightshade_ui::commands::builder::UiBuild;
use nightshade_ui::prelude::*;

const BEAT: f32 = 0.55;

pub fn drive(
    mut resources: ResMut<AppResources>,
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
    if !resources.autoplay.running || resources.desk.screen == Screen::Over {
        return;
    }
    resources.autoplay.waited += time.delta_time;
    if resources.autoplay.waited < BEAT {
        return;
    }
    resources.autoplay.waited = 0.0;

    if resources.desk.screen == Screen::Login {
        work_the_login(resources, ink);
        return;
    }
    if busy(resources, ink) {
        return;
    }
    let did = go_back_to_guest(resources, ink)
        || fetch(resources, ink)
        || finishing_move(resources, ink)
        || talk(resources, ink)
        || finish(resources, ink);
    if did {
        resources.autoplay.idle = 0;
        return;
    }
    resources.autoplay.idle += 1;
    if resources.autoplay.idle >= crate::data::autoplay::GIVE_UP_AFTER {
        resources.autoplay.running = false;
        resources.autoplay.stuck = true;
        crate::systems::desk::raise_notice(
            resources,
            "autoplay",
            "there is nothing left it knows how to do. it has stopped.",
        );
    }
}

fn go_back_to_guest(resources: &mut AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>) -> bool {
    let guest = crate::data::machine::index_of("guest");
    if crate::systems::desk::signed_in_account(&ink.gathered.desk) == Some(guest) {
        return false;
    }
    let wanted = crate::data::autoplay::also_wants(resources.autoplay.wanted);
    let still_wants = owed_now(resources, wanted).iter().any(|fact| {
        matches!(
            kept(fact),
            Some(Source::Account(_)) | Some(Source::Leftover)
        )
    });
    if still_wants {
        return false;
    }
    resources.actions.push(Action::SignOut);
    true
}

fn finishing_move(resources: &mut AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>) -> bool {
    if resources.autoplay.wanted != crate::data::endings::Ending::Uploaded {
        return false;
    }
    if !resources.session.facts.contains(&"was_given_botterfly") {
        return false;
    }
    let botterfly = match crate::data::programs::index_of("botterfly") {
        Some(index) => index,
        None => return false,
    };
    if open_or_install(resources, ink, "botterfly") {
        return true;
    }
    if !crate::systems::desk::showing(&ink.gathered.windows, botterfly) {
        return false;
    }
    resources.actions.push(Action::Upload);
    true
}

fn busy(resources: &AppResources, ink: &Ink<'_, '_, '_, '_, '_, '_>) -> bool {
    if resources.boot.night_waiting
        || !matches!(
            resources.boot.screensaver,
            crate::data::machine::Screensaver::Idle
        )
        || crate::systems::intro::covered(resources)
    {
        return true;
    }
    if crate::systems::browser::loading(resources) {
        return true;
    }
    ink.gathered
        .threads
        .contacts
        .iter()
        .any(|state| state.conversation.node.is_some() && !state.conversation.awaiting_reply)
}

fn work_the_login(resources: &mut AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>) {
    let wanted = crate::data::autoplay::also_wants(resources.autoplay.wanted);
    let borrowed = owed_now(resources, wanted)
        .iter()
        .find_map(|fact| match kept(fact) {
            Some(Source::Account(id)) => Some(*id),
            _ => None,
        });
    let id = borrowed.unwrap_or("guest");
    let Some(index) = crate::data::machine::ACCOUNTS
        .iter()
        .position(|entry| entry.id == id)
    else {
        return;
    };
    let password = crate::data::machine::ACCOUNTS[index].password;

    if crate::systems::desk::selected_account(&ink.gathered.desk) != Some(index) {
        resources.actions.push(Action::SelectAccount(index));
        return;
    }
    if resources.login.typed != password {
        resources.login.typed = password.to_string();
        resources.login.refusal.clear();
        resources.redraw.machine = true;
        return;
    }
    resources.actions.push(Action::SignIn);
}

fn owed_now(resources: &AppResources, wanted: &[Fact]) -> Vec<Fact> {
    let facts = &resources.session.facts;
    let mut owed: Vec<Fact> = Vec::new();
    if let Some(night) = resources.script.nights.get(resources.session.night) {
        owed.extend(
            night
                .condition
                .requires
                .iter()
                .copied()
                .filter(|fact| !facts.contains(fact)),
        );
    }
    for fact in wanted {
        if !facts.contains(fact) && !owed.contains(fact) {
            owed.push(fact);
        }
    }
    owed
}

fn talk(resources: &mut AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>) -> bool {
    let guest = crate::data::machine::index_of("guest");
    if crate::systems::desk::signed_in_account(&ink.gathered.desk) != Some(guest) {
        return false;
    }
    let messenger = crate::systems::setup::program_index("iam");
    if !crate::systems::desk::showing(&ink.gathered.windows, messenger) {
        resources.actions.push(Action::ToggleProgram(messenger));
        return true;
    }

    if let Some(waiting) = somebody_waiting(resources, ink) {
        if crate::resources::open_thread(&resources.session) != Some(waiting) {
            resources.actions.push(Action::OpenThread(waiting));
            return true;
        }
        if let Some(slot) = best_reply(resources, ink) {
            resources.actions.push(Action::Reply(slot));
            return true;
        }
    }

    let pending: Vec<Entity> = ink
        .gathered
        .threads
        .contacts
        .iter()
        .filter(|state| state.listing == Listing::Pending)
        .map(|state| state.entity)
        .collect();
    if let Some(entity) = pending.first() {
        resources.actions.push(Action::AcceptInvite(*entity));
        return true;
    }
    let unopened = ink
        .gathered
        .threads
        .contacts
        .iter()
        .filter(|state| state.listing == Listing::Listed)
        .filter(|state| state.presence != Presence::Offline)
        .find(|state| !state.conversation.opened)
        .map(|state| state.entity);
    if let Some(entity) = unopened {
        resources.actions.push(Action::OpenThread(entity));
        return true;
    }

    read_the_rooms(resources, ink)
}

fn somebody_waiting(
    resources: &AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
) -> Option<Entity> {
    let owed = owed_now(
        resources,
        crate::data::autoplay::also_wants(resources.autoplay.wanted),
    );
    let mut best: Option<(i32, Entity)> = None;
    let waiting: Vec<(Entity, Conversation)> = ink
        .gathered
        .threads
        .contacts
        .iter()
        .map(|state| (state.entity, state.conversation.clone()))
        .collect();
    for (entity, conversation) in waiting {
        if !conversation.awaiting_reply {
            continue;
        }
        let Some(node) = conversation.node else {
            continue;
        };
        let Some(definition) = crate::systems::conversation::contact_definition_of(ink, entity)
        else {
            continue;
        };
        let Some(beat) = resources.script.contacts[definition].nodes.get(node) else {
            continue;
        };
        let open = crate::data::script::available_choices(beat, &resources.session.facts);
        if open.is_empty() {
            continue;
        }
        let score = open
            .iter()
            .map(|index| {
                let mut seen = vec![node];
                worth(
                    resources,
                    definition,
                    beat.choices[*index].goto,
                    &owed,
                    &mut seen,
                    0,
                )
            })
            .max()
            .unwrap_or(0);
        if best.is_none_or(|(held, _)| score > held) {
            best = Some((score, entity));
        }
    }
    best.map(|(_, entity)| entity)
}

const LOOKAHEAD: usize = 4;

fn worth(
    resources: &AppResources,
    definition: usize,
    id: &str,
    owed: &[Fact],
    seen: &mut Vec<usize>,
    depth: usize,
) -> i32 {
    if id.is_empty() || depth > LOOKAHEAD {
        return 0;
    }
    let contact = &resources.script.contacts[definition];
    let Some(node) = crate::data::script::resolve_node(contact, id, &resources.session.facts)
    else {
        return 0;
    };
    if seen.contains(&node) {
        return 0;
    }
    seen.push(node);
    let beat = &contact.nodes[node];

    let mut score = 1;
    for effect in beat.effects {
        score += match effect {
            crate::data::script::Effect::Assert(fact) if owed.contains(fact) => 400,
            crate::data::script::Effect::Assert(_) => 4,
            crate::data::script::Effect::Reveal(_)
            | crate::data::script::Effect::Invite(_, _)
            | crate::data::script::Effect::RevealServer(_)
            | crate::data::script::Effect::Message(_, _) => 40,
            crate::data::script::Effect::Finish(ending) => {
                if *ending == resources.autoplay.wanted {
                    900
                } else {
                    -900
                }
            }
            crate::data::script::Effect::Ending => {
                if resources.autoplay.wanted == crate::data::endings::Ending::HeldOpen {
                    0
                } else {
                    -900
                }
            }
            _ => 0,
        };
    }

    let under = beat
        .choices
        .iter()
        .filter(|reply| {
            crate::data::script::condition_holds(&reply.condition, &resources.session.facts)
        })
        .map(|reply| worth(resources, definition, reply.goto, owed, seen, depth + 1))
        .max()
        .unwrap_or(0);
    score + under / 2
}

fn best_reply(resources: &AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>) -> Option<Entity> {
    let entity = crate::resources::open_thread(&resources.session)?;
    let definition = crate::systems::conversation::contact_definition_of(ink, entity)?;
    let node = crate::systems::conversation::contact_at(&ink.gathered.threads, entity)?
        .conversation
        .node?;
    let beat = resources.script.contacts[definition].nodes.get(node)?;
    let owed = owed_now(
        resources,
        crate::data::autoplay::also_wants(resources.autoplay.wanted),
    );

    ink.gathered
        .threads
        .choices
        .iter()
        .filter_map(|state| state.slot.choice.map(|choice| (state.entity, choice)))
        .filter(|(_, choice)| *choice != crate::resources::SIGN_OFF)
        .filter_map(|(slot, choice)| beat.choices.get(choice).map(|reply| (slot, reply)))
        .max_by_key(|(_, reply)| {
            let mut seen = vec![node];
            let ahead = worth(resources, definition, reply.goto, &owed, &mut seen, 0);
            let carries: i32 = reply
                .asserts
                .iter()
                .filter(|fact| owed.contains(fact))
                .count() as i32
                * 400;
            ahead + carries
        })
        .map(|(slot, _)| slot)
}

fn read_the_rooms(resources: &mut AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>) -> bool {
    let servers: Vec<Entity> = ink
        .gathered
        .threads
        .servers
        .iter()
        .filter(|state| state.listed)
        .map(|state| state.entity)
        .collect();
    for server in servers {
        let channels: Vec<Entity> = ink
            .gathered
            .threads
            .channels
            .iter()
            .filter(|state| state.channel.server == server)
            .map(|state| state.entity)
            .collect();
        for channel in channels {
            let Some(definition) = channel_definition(resources, ink, channel) else {
                continue;
            };
            let asked = crate::systems::conversation::channel_at(&ink.gathered.threads, channel)
                .map(|state| state.asked.clone())
                .unwrap_or_default();
            let left: Vec<usize> =
                crate::data::script::available_questions(definition, &resources.session.facts)
                    .into_iter()
                    .filter(|index| !asked.contains(index))
                    .collect();
            if left.is_empty() {
                continue;
            }
            let open = crate::systems::conversation::server_at(&ink.gathered.threads, server)
                .and_then(|state| state.open_channel);
            if open != Some(channel) {
                resources.actions.push(Action::OpenServer(server));
                resources.actions.push(Action::OpenChannel(channel));
                return true;
            }
            let slot = ink
                .gathered
                .threads
                .choices
                .iter()
                .filter_map(|state| state.slot.choice.map(|choice| (state.entity, choice)))
                .min_by_key(|(_, choice)| *choice)
                .map(|(entity, _)| entity);
            if let Some(slot) = slot {
                resources.actions.push(Action::Ask(slot));
                return true;
            }
        }
    }
    false
}

fn channel_definition<'a>(
    resources: &'a AppResources,
    ink: &Ink<'_, '_, '_, '_, '_, '_>,
    channel: Entity,
) -> Option<&'a crate::data::script::ChannelDefinition> {
    let parts = crate::systems::conversation::channel_at(&ink.gathered.threads, channel)?
        .channel
        .clone();
    let server = crate::systems::conversation::server_at(&ink.gathered.threads, parts.server)?;
    resources
        .script
        .servers
        .get(server.definition)?
        .channels
        .get(parts.definition)
}

fn fetch(resources: &mut AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>) -> bool {
    let wanted = crate::data::autoplay::also_wants(resources.autoplay.wanted);
    for fact in owed_now(resources, wanted) {
        let Some(source) = kept(fact) else {
            continue;
        };
        if work_the_source(resources, ink, source) {
            return true;
        }
    }
    false
}

fn work_the_source(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    source: &Source,
) -> bool {
    match source {
        Source::Page(url) => {
            let browser = crate::systems::setup::program_index("browser");
            if !crate::systems::desk::showing(&ink.gathered.windows, browser) {
                resources.actions.push(Action::ToggleProgram(browser));
                return true;
            }
            if crate::data::web::tidy(&resources.browser.address) == crate::data::web::tidy(url) {
                return false;
            }
            crate::systems::browser::go(resources, url, true);
            true
        }
        Source::Answer(url, typed) => {
            let browser = crate::systems::setup::program_index("browser");
            if !crate::systems::desk::showing(&ink.gathered.windows, browser) {
                resources.actions.push(Action::ToggleProgram(browser));
                return true;
            }
            if crate::data::web::tidy(&resources.browser.address) != crate::data::web::tidy(url) {
                crate::systems::browser::go(resources, url, true);
                return true;
            }
            if resources.browser.page_typed != *typed {
                resources.browser.page_typed = typed.to_string();
                resources.redraw.machine = true;
                return true;
            }
            crate::systems::browser::submit_field(resources);
            true
        }
        Source::Program(id) => open_or_install(resources, ink, id),
        Source::MyFile => {
            if open_or_install(resources, ink, "case") {
                return true;
            }
            let mine = ink
                .gathered
                .rows
                .cases
                .iter()
                .map(|(_, row)| row.file)
                .find(|file| {
                    crate::data::case::DOSSIERS
                        .get(*file)
                        .is_some_and(|dossier| dossier.mine)
                });
            match mine {
                Some(file) => {
                    resources.actions.push(Action::OpenCase(file));
                    true
                }
                None => false,
            }
        }
        Source::Board(id) => {
            if open_or_install(resources, ink, id) {
                return true;
            }
            let Some(program) = crate::data::programs::index_of(id) else {
                return false;
            };
            let goal_lit = *id == "lights_on";
            match next_press(&ink.gathered.rows, program, goal_lit) {
                Some(place) => {
                    crate::systems::desk::press_lamp(resources, ink, program, place);
                    true
                }
                None => false,
            }
        }
        Source::Pairing => open_or_install(resources, ink, "cross"),
        Source::Leftover => {
            if open_or_install(resources, ink, "files") {
                return true;
            }
            let reading = crate::systems::desk::open_file(&ink.gathered.files);
            let file = ink
                .gathered
                .rows
                .files
                .iter()
                .map(|(_, row)| row.file)
                .find(|file| reading != Some(*file));
            match file {
                Some(file) => {
                    crate::systems::desk::open_this_file(
                        ink.build.commands,
                        &ink.gathered.files,
                        Some(file),
                    );
                    resources.redraw.machine = true;
                    true
                }
                None => false,
            }
        }
        Source::Account(_) => {
            resources.actions.push(Action::SignOut);
            true
        }
        Source::Walk(place) => walk_to(resources, ink, place),
    }
}

fn next_press(rows: &crate::systems::desk::Rows, program: usize, goal_lit: bool) -> Option<usize> {
    let mut lamps: Vec<(usize, bool)> = rows
        .lamps
        .iter()
        .filter(|(_, row)| row.program == program)
        .map(|(_, row)| (row.place, row.lit))
        .collect();
    if lamps.is_empty() {
        return None;
    }
    lamps.sort_by_key(|(place, _)| *place);
    let count = lamps.len();
    let state: Vec<bool> = lamps.iter().map(|(_, lit)| *lit).collect();

    for presses in 0u32..(1 << count) {
        let mut board = state.clone();
        for place in 0..count {
            if presses & (1 << place) == 0 {
                continue;
            }
            let first = place.saturating_sub(1);
            let last = (place + 1).min(count - 1);
            for lit in board.iter_mut().take(last + 1).skip(first) {
                *lit = !*lit;
            }
        }
        if board.iter().all(|lit| *lit == goal_lit) {
            let first = (0..count).find(|place| presses & (1 << place) != 0)?;
            return lamps.get(first).map(|(place, _)| *place);
        }
    }
    None
}

fn open_or_install(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    id: &str,
) -> bool {
    let Some(index) = crate::data::programs::index_of(id) else {
        return false;
    };
    let up = crate::systems::desk::window_entity(&ink.gathered.windows, index)
        .and_then(|entity| ink.gathered.windows.iter().find(|row| row.entity == entity))
        .is_some_and(|row| row.window.installed)
        || crate::data::programs::PROGRAMS
            .get(index)
            .is_some_and(|program| {
                crate::data::programs::comes_with_the_desk(program, &resources.session.facts)
            });
    if !up {
        let Some(listing) = crate::data::store::for_program(id)
            .filter(|index| crate::data::store::listed(&resources.session.facts).contains(index))
        else {
            return false;
        };
        let store = crate::systems::setup::program_index("store");
        if !crate::systems::desk::showing(&ink.gathered.windows, store) {
            resources.actions.push(Action::ToggleProgram(store));
            return true;
        }
        resources.actions.push(Action::Install(listing));
        return true;
    }
    if crate::systems::desk::showing(&ink.gathered.windows, index) {
        return false;
    }
    resources.actions.push(Action::ToggleProgram(index));
    true
}

fn walk_to(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    place: &str,
) -> bool {
    if open_or_install(resources, ink, "office") {
        return true;
    }
    let Some(standing) = crate::data::office::PLACES.get(resources.desk.place) else {
        return false;
    };

    let here = crate::data::office::things_here(standing, &resources.session.facts);
    let unread = here
        .iter()
        .position(|thing| !resources.autoplay.read.contains(thing));
    if let Some(row) = unread {
        resources.autoplay.read.push(here[row]);
        resources.actions.push(Action::LookAt(row));
        return true;
    }
    if standing.id == place {
        return false;
    }
    let step =
        crate::data::office::step_towards(resources.desk.place, place, &resources.session.facts);
    match step {
        Some(way) => {
            resources.actions.push(Action::GoPlace(way));
            true
        }
        None => false,
    }
}

fn finish(resources: &mut AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>) -> bool {
    if !crate::systems::conversation::night_can_end(resources) {
        return false;
    }
    let sign_off = ink
        .gathered
        .threads
        .choices
        .iter()
        .find(|state| state.slot.choice == Some(crate::resources::SIGN_OFF))
        .map(|state| state.entity);
    match sign_off {
        Some(slot) => {
            resources.actions.push(Action::Reply(slot));
            true
        }
        None => false,
    }
}
