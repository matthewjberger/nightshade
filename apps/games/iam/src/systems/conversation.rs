use crate::components::{
    Channel, ChannelAsked, ChannelFeed, ChannelUnread, ChoiceSlot, Contact, ContactAutomated,
    ContactBurned, ContactDeactivated, ContactGave, ContactInvite, ContactListing,
    ContactPersuaded, ContactPresence, ContactSpent, ContactUnread, Conversation, FeedEntry,
    Listing, Server, ServerListed, ServerOpenChannel, Speaker, ThreadView, Transcript,
    TranscriptEntry,
};
use crate::data::script::{
    Effect, Fact, Post, Presence, available_choices, condition_holds, contact_index,
    presence_label, resolve_hint, resolve_node, segment_is_typing, server_index,
};
use crate::resources::{Action, AppResources, Ink, View, establish, open_thread};
use nightshade::prelude::Entity;
use nightshade::prelude::*;
use nightshade_platform::prelude::{Time, Window};
use nightshade_ui::commands::builder::UiBuild;
use nightshade_ui::prelude::*;

#[derive(Default, Clone)]
pub struct ContactState {
    pub entity: Entity,
    pub definition: usize,
    pub presence: Presence,
    pub listing: Listing,
    pub deactivated: bool,
    pub automated: bool,
    pub gave: bool,
    pub spent: bool,
    pub persuaded: bool,
    pub burned: bool,
    pub unread: u32,
    pub invite: Option<&'static str>,
    pub transcript: Transcript,
    pub conversation: Conversation,
    pub view: ThreadView,
}

#[derive(Default, Clone)]
pub struct ServerState {
    pub entity: Entity,
    pub definition: usize,
    pub listed: bool,
    pub open_channel: Option<Entity>,
}

#[derive(Default, Clone)]
pub struct ChannelState {
    pub entity: Entity,
    pub channel: Channel,
    pub feed: ChannelFeed,
    pub unread: u32,
    pub asked: Vec<usize>,
}

#[derive(Default, Clone, Copy)]
pub struct ChoiceState {
    pub entity: Entity,
    pub slot: ChoiceSlot,
}

#[derive(Default)]
pub struct Threads {
    pub contacts: Vec<ContactState>,
    pub servers: Vec<ServerState>,
    pub channels: Vec<ChannelState>,
    pub choices: Vec<ChoiceState>,
}

pub fn contact_at(threads: &Threads, entity: Entity) -> Option<&ContactState> {
    threads.contacts.iter().find(|state| state.entity == entity)
}

pub fn contact_mut(threads: &mut Threads, entity: Entity) -> Option<&mut ContactState> {
    threads
        .contacts
        .iter_mut()
        .find(|state| state.entity == entity)
}

pub fn channel_at(threads: &Threads, entity: Entity) -> Option<&ChannelState> {
    threads.channels.iter().find(|state| state.entity == entity)
}

pub fn channel_mut(threads: &mut Threads, entity: Entity) -> Option<&mut ChannelState> {
    threads
        .channels
        .iter_mut()
        .find(|state| state.entity == entity)
}

pub fn server_at(threads: &Threads, entity: Entity) -> Option<&ServerState> {
    threads.servers.iter().find(|state| state.entity == entity)
}

pub fn server_mut(threads: &mut Threads, entity: Entity) -> Option<&mut ServerState> {
    threads
        .servers
        .iter_mut()
        .find(|state| state.entity == entity)
}

pub fn choice_at(threads: &Threads, entity: Entity) -> Option<ChoiceSlot> {
    threads
        .choices
        .iter()
        .find(|state| state.entity == entity)
        .map(|state| state.slot)
}

pub fn write_back(gathered: Res<crate::systems::desk::Gathered>, mut commands: Commands) {
    write_threads(&gathered.threads, &mut commands);
}

pub fn write_threads(threads: &Threads, commands: &mut Commands) {
    for state in &threads.contacts {
        commands.set(state.entity, ContactPresence(state.presence));
        commands.set(state.entity, ContactListing(state.listing));
        commands.set(state.entity, ContactDeactivated(state.deactivated));
        commands.set(state.entity, ContactAutomated(state.automated));
        commands.set(state.entity, ContactGave(state.gave));
        commands.set(state.entity, ContactSpent(state.spent));
        commands.set(state.entity, ContactPersuaded(state.persuaded));
        commands.set(state.entity, ContactBurned(state.burned));
        commands.set(state.entity, ContactUnread(state.unread));
        commands.set(
            state.entity,
            ContactInvite(state.invite.unwrap_or_default()),
        );
        commands.set(state.entity, state.transcript.clone());
        commands.set(state.entity, state.conversation.clone());
        commands.set(state.entity, state.view.clone());
    }
    for state in &threads.servers {
        commands.set(state.entity, ServerListed(state.listed));
        commands.set(state.entity, ServerOpenChannel(state.open_channel));
    }
    for state in &threads.channels {
        commands.set(state.entity, state.feed.clone());
        commands.set(state.entity, ChannelUnread(state.unread));
        commands.set(state.entity, ChannelAsked(state.asked.clone()));
    }
}

type ContactFirst<'row> = (
    Option<&'row Contact>,
    Option<&'row ContactPresence>,
    Option<&'row ContactListing>,
    Option<&'row ContactDeactivated>,
    Option<&'row ContactAutomated>,
    Option<&'row ContactGave>,
    Option<&'row ContactSpent>,
    Option<&'row ContactPersuaded>,
);

type ContactMore<'row> = (
    Option<&'row Contact>,
    Option<&'row ContactBurned>,
    Option<&'row ContactUnread>,
    Option<&'row ContactInvite>,
    Option<&'row Transcript>,
    Option<&'row Conversation>,
    Option<&'row ThreadView>,
);

type ServerRows<'row> = (
    &'row Server,
    Option<&'row ServerListed>,
    Option<&'row ServerOpenChannel>,
);

type ChannelRows<'row> = (
    &'row Channel,
    Option<&'row ChannelFeed>,
    Option<&'row ChannelUnread>,
    Option<&'row ChannelAsked>,
);

pub fn gather_threads(
    mut gathered: ResMut<crate::systems::desk::Gathered>,
    who: Query<ContactFirst>,
    more: Query<ContactMore>,
    servers: Query<ServerRows>,
    channels: Query<ChannelRows>,
    choices: Query<&ChoiceSlot>,
) {
    let threads = &mut gathered.threads;
    *threads = Threads::default();
    who.for_each(
        |entity, (contact, presence, listing, deactivated, automated, gave, spent, persuaded)| {
            let Some(contact) = contact else {
                return;
            };
            threads.contacts.push(ContactState {
                entity,
                definition: contact.definition,
                presence: presence.map(|presence| presence.0).unwrap_or_default(),
                listing: listing.map(|listing| listing.0).unwrap_or_default(),
                deactivated: deactivated.is_some_and(|mark| mark.0),
                automated: automated.is_some_and(|mark| mark.0),
                gave: gave.is_some_and(|mark| mark.0),
                spent: spent.is_some_and(|mark| mark.0),
                persuaded: persuaded.is_some_and(|mark| mark.0),
                ..Default::default()
            });
        },
    );
    more.for_each(
        |entity, (contact, burned, unread, invite, transcript, conversation, view)| {
            if contact.is_none() {
                return;
            }
            let Some(state) = threads
                .contacts
                .iter_mut()
                .find(|state| state.entity == entity)
            else {
                return;
            };
            state.burned = burned.is_some_and(|mark| mark.0);
            state.unread = unread.map(|unread| unread.0).unwrap_or_default();
            state.invite = invite.map(|invite| invite.0);
            state.transcript = transcript.cloned().unwrap_or_default();
            state.conversation = conversation.cloned().unwrap_or_default();
            state.view = view.cloned().unwrap_or_default();
        },
    );
    servers.for_each(|entity, (server, listed, open)| {
        threads.servers.push(ServerState {
            entity,
            definition: server.definition,
            listed: listed.is_some_and(|listed| listed.0),
            open_channel: open.and_then(|open| open.0),
        });
    });
    channels.for_each(|entity, (channel, feed, unread, asked)| {
        threads.channels.push(ChannelState {
            entity,
            channel: channel.clone(),
            feed: feed.cloned().unwrap_or_default(),
            unread: unread.map(|unread| unread.0).unwrap_or_default(),
            asked: asked.map(|asked| asked.0.clone()).unwrap_or_default(),
        });
    });
    choices.for_each(|entity, slot| {
        threads.choices.push(ChoiceState {
            entity,
            slot: *slot,
        });
    });
}

pub fn advance(
    mut resources: ResMut<AppResources>,
    time: Res<Time>,
    config: Res<crate::data::config::Config>,
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
    let resources = &mut *resources;
    let uptime = time.uptime_seconds;
    let actions: Vec<Action> = std::mem::take(&mut resources.actions);
    let signed_in = crate::data::machine::is_guest(
        crate::systems::desk::signed_in_account(&ink.gathered.desk),
        resources.desk.screen,
    );
    for action in actions {
        apply_action(resources, ink, uptime, &mut window, action);
    }

    if !signed_in {
        return;
    }

    let delta_time = time.delta_time;
    resources.session.dots_phase += delta_time;
    if !resources.session.held_open {
        resources.session.clock_seconds += delta_time * config.clock_rate;
    }

    let running: Vec<Entity> = ink
        .gathered
        .threads
        .contacts
        .iter()
        .map(|state| (state.entity, state.conversation.clone()))
        .filter(|(_, conversation)| conversation.node.is_some() && !conversation.awaiting_reply)
        .map(|(entity, _)| entity)
        .collect();

    for entity in running {
        tick_contact(resources, ink, entity, delta_time);
    }
}

fn apply_action(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    uptime: f64,
    window: &mut Window,
    action: Action,
) {
    match action {
        Action::OpenThread(entity) => open_contact(resources, ink, entity),
        Action::AcceptInvite(entity) => accept_invite(resources, ink, entity),
        Action::Ask(slot) => ask_question(resources, ink, slot),
        Action::OpenServer(entity) => open_server_view(resources, ink, entity),
        Action::OpenChannel(entity) => open_channel(resources, ink, entity),
        Action::Reply(slot) => send_reply(resources, ink, uptime, slot),
        Action::ToggleOffline => {
            resources.session.show_offline = !resources.session.show_offline;
            if resources.session.show_offline {
                establish(&mut resources.session, "saw_offline_list");
            }
            resources.redraw.sidebar = true;
        }
        Action::ToggleProgram(program) => {
            crate::systems::desk::toggle(resources, ink, program);
            resources.redraw = crate::resources::redraw_everything();
        }
        Action::MinimizeProgram(program) => {
            crate::systems::desk::put_away(resources, ink, program, false);
        }
        Action::CloseProgram(program) => {
            crate::systems::desk::put_away(resources, ink, program, true);
        }
        Action::PressBoard(place) => {
            press_board(resources, ink, place);
        }
        Action::PressLamp(program, place) => {
            crate::systems::desk::press_lamp(resources, ink, program, place);
        }
        Action::ToggleSettings => {
            resources.login.settings_open = !resources.login.settings_open;
            resources.redraw.machine = true;
        }
        Action::WantEnding(which) => {
            if let Some((ending, _, _)) = crate::data::autoplay::OFFERED.get(which) {
                resources.autoplay.wanted = *ending;
                resources.redraw.machine = true;
            }
        }
        Action::StartAutoplay => {
            resources.autoplay.running = true;
            resources.login.settings_open = false;
            resources.redraw.machine = true;
        }
        Action::Upload => {
            finish_week(resources, ink, crate::data::endings::Ending::Uploaded);
        }
        Action::Install(listing) => {
            crate::systems::desk::install_from_store(resources, ink, listing);
        }
        Action::OpenStore(program) => {
            crate::systems::desk::open_store_at(resources, ink, program);
        }
        Action::OpenCase(file) => {
            crate::systems::desk::open_case(resources, file);
        }
        Action::GoPlace(slot) => {
            let here = crate::data::office::PLACES.get(resources.desk.place);
            let target = here
                .and_then(|place| {
                    crate::data::office::ways_here(place, &resources.session.facts)
                        .get(slot)
                        .and_then(|index| place.ways.get(*index))
                })
                .and_then(|way| crate::data::office::place_at(way.goes_to));
            if let Some(target) = target {
                crate::systems::desk::go_place(resources, target);
            }
        }
        Action::LookAt(slot) => {
            let here = crate::data::office::PLACES.get(resources.desk.place);
            let target = here
                .map(|place| crate::data::office::things_here(place, &resources.session.facts))
                .and_then(|here| here.get(slot).copied());
            if let Some(target) = target {
                crate::systems::desk::look_at(resources, target);
            }
        }
        Action::OpenFile(file) => {
            crate::systems::desk::open_this_file(
                ink.build.commands,
                &ink.gathered.files,
                Some(file),
            );
            resources.redraw.machine = true;
        }
        Action::NextLamps => {
            crate::systems::desk::next_lamps(resources, ink);
        }
        Action::CloseGame => {
            resources.desk.playing = None;
            resources.redraw.machine = true;
            resources.redraw.composer = true;
        }
        Action::SignIn => {
            crate::systems::desk::sign_in(resources, ink);
        }
        Action::SignOff => {
            sign_off(resources, ink, uptime);
        }
        Action::SignOut => {
            resources.desk.screen = crate::data::machine::Screen::Login;
            crate::systems::desk::sign_in_as(ink.build.commands, &ink.gathered.desk, None);
            resources.login.typed.clear();
            resources.login.refusal.clear();
            resources.redraw = crate::resources::redraw_everything();
        }
        Action::SizeWindow(program, mode) => {
            let entity = ink
                .gathered
                .windows
                .iter()
                .map(|row| (row.entity, row.window))
                .find(|(_, window)| window.program == program)
                .map(|(entity, _)| entity);
            if let Some(entity) = entity
                && let Some(row) = ink.gathered.windows.iter().find(|row| row.entity == entity)
            {
                let mut window = row.window;
                window.mode = mode;
                ink.build.commands.set(entity, window);
            }
            resources.redraw.machine = true;
        }
        Action::SelectAccount(index) => {
            crate::systems::desk::select_account(ink.build.commands, &ink.gathered.desk, index);
            resources.login.typed.clear();
            resources.login.refusal.clear();
            resources.redraw.machine = true;
        }
        Action::Quit => {
            window.should_exit = true;
        }
    }
}

pub fn open_contact(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    entity: Entity,
) {
    if let Some(state) = contact_mut(&mut ink.gathered.threads, entity) {
        state.unread = 0;
    }
    resources.session.view = View::Thread(entity);
    resources.session.pin_log = true;
    resources.redraw = crate::resources::redraw_everything();

    let first_time = contact_at(&ink.gathered.threads, entity)
        .map(|state| !state.conversation.opened)
        .unwrap_or_default();
    if let Some(state) = contact_mut(&mut ink.gathered.threads, entity) {
        state.conversation.opened = true;
    }
    if !first_time {
        return;
    }
    let script = resources.script;
    let Some(definition_index) = contact_definition_of(ink, entity) else {
        return;
    };
    let entry = script.contacts[definition_index].entry;
    begin_node(resources, ink, entity, entry);
}

fn open_server_view(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    entity: Entity,
) {
    resources.session.view = View::Server(entity);
    resources.session.pin_log = true;
    establish(&mut resources.session, "opened_a_server");
    resources.redraw = crate::resources::redraw_everything();
    if let Some(channel) =
        server_at(&ink.gathered.threads, entity).and_then(|state| state.open_channel)
        && let Some(state) = channel_mut(&mut ink.gathered.threads, channel)
    {
        state.unread = 0;
    }
}

fn open_channel(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    channel: Entity,
) {
    let Some(server) = channel_at(&ink.gathered.threads, channel).map(|state| state.channel.server)
    else {
        return;
    };
    if let Some(state) = server_mut(&mut ink.gathered.threads, server) {
        state.open_channel = Some(channel);
    }
    if let Some(state) = channel_mut(&mut ink.gathered.threads, channel) {
        state.unread = 0;
    }
    if let Some(entry) =
        channel_at(&ink.gathered.threads, channel).map(|state| state.channel.clone())
    {
        let effects =
            resources.script.servers[entry.server_definition].channels[entry.definition].on_open;
        apply_effects(resources, ink, effects);
    }
    resources.session.view = View::Server(server);
    resources.session.pin_log = true;
    resources.redraw = crate::resources::redraw_everything();
}

fn ask_question(resources: &mut AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>, slot: Entity) {
    if let Some(index) = choice_at(&ink.gathered.threads, slot).and_then(|slot| slot.choice)
        && index >= crate::resources::PLAY
    {
        let posted = index - crate::resources::PLAY;
        let entry = &crate::data::games::POSTED[posted];
        if entry.installs.is_some() {
            crate::resources::learn(resources, entry.asserts);
            announce(
                resources,
                ink,
                "the network has it. open software on the desktop to install it.",
            );
            resources.redraw = crate::resources::redraw_everything();
            return;
        }
        resources.desk.playing = Some(posted);
        deal_board(ink, entry.start);
        resources.desk.board_won = false;
        resources.redraw.machine = true;
        resources.redraw.composer = true;
        return;
    }
    let Some(server) = crate::resources::open_server(&resources.session) else {
        return;
    };
    let Some(channel) = crate::systems::conversation::server_at(&ink.gathered.threads, server)
        .map(|state| ServerOpenChannel(state.open_channel))
        .and_then(|open| open.0)
    else {
        return;
    };
    let Some(index) = choice_at(&ink.gathered.threads, slot).and_then(|slot| slot.choice) else {
        return;
    };
    let Some(entry) = channel_at(&ink.gathered.threads, channel).map(|state| state.channel.clone())
    else {
        return;
    };
    let definition = &resources.script.servers[entry.server_definition].channels[entry.definition];
    let Some(question) = definition.questions.get(index) else {
        return;
    };

    let asked = FeedEntry {
        author: resources.session.handle,
        text: question.text,
        stamp: crate::systems::interface::day_stamp(resources),
        mine: true,
    };
    let replies: Vec<&'static Post> = question.replies.iter().collect();
    let first_delay = replies
        .first()
        .map(|post| post.delay_seconds)
        .unwrap_or(0.0);
    let effects = question.effects;
    let asserts = question.asserts;

    let queued = !replies.is_empty();
    if let Some(feed) = channel_mut(&mut ink.gathered.threads, channel).map(|state| &mut state.feed)
    {
        feed.log.push(asked);
        feed.queue = replies;
        feed.queued_effects = if queued { effects } else { &[] };
        feed.timer = first_delay;
    }
    if let Some(asked) =
        channel_mut(&mut ink.gathered.threads, channel).map(|state| &mut state.asked)
    {
        asked.push(index);
    }
    for fact in asserts {
        establish(&mut resources.session, fact);
    }
    if !queued {
        apply_effects(resources, ink, effects);
    }
    resources.session.pin_log = true;
    resources.redraw = crate::resources::redraw_everything();
}

fn send_reply(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    uptime: f64,
    slot: Entity,
) {
    let Some(entity) = open_thread(&resources.session) else {
        return;
    };
    resources.session.pin_log = true;
    let Some(choice_index) = choice_at(&ink.gathered.threads, slot).and_then(|slot| slot.choice)
    else {
        return;
    };
    if choice_index == crate::resources::SIGN_OFF {
        sign_off(resources, ink, uptime);
        return;
    }
    let script = resources.script;
    let Some(definition_index) = contact_definition_of(ink, entity) else {
        return;
    };
    let Some(node_slot) = crate::systems::conversation::contact_at(&ink.gathered.threads, entity)
        .map(|state| state.conversation.clone())
        .and_then(|conversation| conversation.node)
    else {
        return;
    };
    let Some(choice) = script.contacts[definition_index].nodes[node_slot]
        .choices
        .get(choice_index)
    else {
        return;
    };

    let tactic = choice.tactic;
    let text = choice.text;
    let goto = choice.goto;
    let asserts = choice.asserts;
    push_entry(resources, ink, entity, Speaker::Player, text);

    if let Some(reason) = misfire(resources, ink, entity, tactic) {
        burn_contact(resources, ink, entity, tactic, reason);
        resources.redraw.composer = true;
        resources.redraw.sidebar = true;
        return;
    }
    match tactic {
        crate::data::script::Tactic::Persuade => {
            if let Some(state) = contact_mut(&mut ink.gathered.threads, entity) {
                state.persuaded = true;
                state.gave = true;
            }
        }
        crate::data::script::Tactic::Press => {
            if let Some(state) = contact_mut(&mut ink.gathered.threads, entity) {
                state.gave = true;
            }
        }
        _ => {}
    }

    for fact in asserts {
        establish(&mut resources.session, fact);
    }
    begin_node(resources, ink, entity, goto);
    resources.redraw.composer = true;
    resources.redraw.sidebar = true;
}

pub fn will_fail(
    resources: &AppResources,
    ink: &Ink<'_, '_, '_, '_, '_, '_>,
    entity: Entity,
    tactic: crate::data::script::Tactic,
) -> bool {
    misfire(resources, ink, entity, tactic).is_some()
}

fn misfire(
    resources: &AppResources,
    ink: &Ink<'_, '_, '_, '_, '_, '_>,
    entity: Entity,
    tactic: crate::data::script::Tactic,
) -> Option<&'static str> {
    let bot = contact_definition_of(ink, entity)
        .map(|index| {
            let definition = &resources.script.contacts[index];
            definition.bot
                && !crate::data::script::stopped_being_a_program(
                    resources.script,
                    definition.id,
                    resources.session.night,
                )
        })
        .unwrap_or(false)
        || crate::systems::conversation::contact_at(&ink.gathered.threads, entity)
            .map(|state| ContactAutomated(state.automated))
            .is_some_and(|automated| automated.0);
    let gave = crate::systems::conversation::contact_at(&ink.gathered.threads, entity)
        .map(|state| ContactGave(state.gave))
        .is_some_and(|gave| gave.0);
    let spent = crate::systems::conversation::contact_at(&ink.gathered.threads, entity)
        .map(|state| ContactSpent(state.spent))
        .is_some_and(|spent| spent.0);
    let used = crate::systems::conversation::contact_at(&ink.gathered.threads, entity)
        .map(|state| ContactPersuaded(state.persuaded))
        .is_some_and(|used| used.0);
    crate::data::script::tactic_fails(tactic, bot, gave, spent, used)
}

fn burn_contact(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    entity: Entity,
    tactic: crate::data::script::Tactic,
    reason: &'static str,
) {
    if let Some(state) = contact_mut(&mut ink.gathered.threads, entity) {
        state.burned = true;
    }
    let said = match tactic {
        crate::data::script::Tactic::Press => "waiting will not get anything else out of me",
        crate::data::script::Tactic::Persuade => "you have already had that from me",
        _ => "that is not true, and i knew before you finished",
    };
    push_entry(resources, ink, entity, Speaker::Contact, said);
    push_entry(
        resources,
        ink,
        entity,
        Speaker::Notice,
        &format!("that did not work. {reason}"),
    );
    establish(&mut resources.session, "burned_somebody");
}

fn tick_contact(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    entity: Entity,
    delta_time: f32,
) {
    let script = resources.script;
    let Some(definition_index) = contact_definition_of(ink, entity) else {
        return;
    };
    let Some(node_slot) = crate::systems::conversation::contact_at(&ink.gathered.threads, entity)
        .map(|state| state.conversation.clone())
        .and_then(|conversation| conversation.node)
    else {
        return;
    };
    let definition = &script.contacts[definition_index];
    let node = &definition.nodes[node_slot];
    let automated = crate::systems::conversation::contact_at(&ink.gathered.threads, entity)
        .map(|state| ContactAutomated(state.automated))
        .is_some_and(|state| state.0);
    let machine = automated
        || crate::data::script::types_instantly(script, definition.id, resources.session.night);
    let delta_time = if machine {
        delta_time * crate::data::script::INSTANT_RATE
    } else {
        delta_time
    };

    let Some(state) =
        contact_at(&ink.gathered.threads, entity).map(|state| state.conversation.clone())
    else {
        return;
    };
    let Some(current) = node.lines.get(state.delivered) else {
        finish_node(resources, ink, entity, definition_index, node_slot);
        return;
    };

    let mut timer = state.timer - delta_time;
    let mut segment = state.segment;
    let mut landed = false;
    while timer <= 0.0 {
        segment += 1;
        match current.rhythm.get(segment) {
            Some(seconds) => timer += seconds,
            None => {
                landed = true;
                break;
            }
        }
    }

    if !landed {
        if let Some(conversation) =
            contact_mut(&mut ink.gathered.threads, entity).map(|state| &mut state.conversation)
        {
            conversation.timer = timer;
            conversation.segment = segment;
        }
        if segment_is_typing(segment) != segment_is_typing(state.segment) {
            resources.redraw.composer = true;
        }
        return;
    }

    let spoken = quoted(ink, current.text);
    push_entry(resources, ink, entity, Speaker::Contact, &spoken);
    let name = contact_definition_of(ink, entity)
        .map(|index| script.contacts[index].display_name)
        .unwrap_or("somebody");
    crate::systems::desk::raise_toast(
        resources,
        ink,
        crate::resources::ToastTarget::Thread(entity),
        name,
        &spoken,
    );
    resources.session.cues.push(crate::resources::Cue::Message);
    let delivered = state.delivered + 1;
    if let Some(conversation) =
        contact_mut(&mut ink.gathered.threads, entity).map(|state| &mut state.conversation)
    {
        conversation.delivered = delivered;
        conversation.segment = 0;
        conversation.timer = 0.0;
    }

    if open_thread(&resources.session) != Some(entity)
        && let Some(unread) =
            contact_mut(&mut ink.gathered.threads, entity).map(|state| &mut state.unread)
    {
        *unread += 1;
        resources.redraw.sidebar = true;
    }

    match node.lines.get(delivered) {
        Some(next) => {
            if let Some(conversation) =
                contact_mut(&mut ink.gathered.threads, entity).map(|state| &mut state.conversation)
            {
                conversation.timer = next.rhythm.first().copied().unwrap_or_default();
            }
        }
        None => finish_node(resources, ink, entity, definition_index, node_slot),
    }
    resources.redraw.composer = true;
}

fn finish_node(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    entity: Entity,
    definition_index: usize,
    node_slot: usize,
) {
    if let Some(conversation) =
        contact_mut(&mut ink.gathered.threads, entity).map(|state| &mut state.conversation)
    {
        conversation.timer = 0.0;
        conversation.segment = 0;
        conversation.awaiting_reply = true;
    }
    let effects = resources.script.contacts[definition_index].nodes[node_slot].effects;
    for effect in effects {
        apply_effect(resources, ink, effect);
    }
    if resources.script.contacts[definition_index].nodes[node_slot]
        .choices
        .is_empty()
        && let Some(state) = contact_mut(&mut ink.gathered.threads, entity)
    {
        state.spent = true;
    }
    resources.redraw.composer = true;
    resources.redraw.sidebar = true;
}

fn board_reads(rows: &crate::systems::desk::Rows, places: usize) -> String {
    let mut squares: Vec<(usize, char)> = rows
        .places
        .iter()
        .filter(|(_, square)| square.place < places)
        .map(|(_, square)| (square.place, square.face))
        .collect();
    squares.sort_by_key(|(place, _)| *place);
    squares.into_iter().map(|(_, face)| face).collect()
}

fn deal_board(ink: &mut Ink<'_, '_, '_, '_, '_, '_>, start: &str) {
    let faces: Vec<char> = start.chars().collect();
    for (entity, square) in ink.gathered.rows.places.iter_mut() {
        square.face = faces.get(square.place).copied().unwrap_or(' ');
        ink.build.commands.set(*entity, *square);
    }
}

fn press_board(resources: &mut AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>, place: usize) {
    let Some(index) = resources.desk.playing else {
        return;
    };
    let posted = &crate::data::games::POSTED[index];
    for (entity, square) in ink.gathered.rows.places.iter_mut() {
        if square.place == place {
            square.face = crate::data::games::turned(posted.board, square.face);
            ink.build.commands.set(*entity, *square);
        }
    }
    resources.redraw.machine = true;
    if !resources.desk.board_won
        && crate::data::games::solved(
            posted,
            &board_reads(&ink.gathered.rows, posted.goal.chars().count()),
        )
    {
        resources.desk.board_won = true;
        for fact in posted.asserts {
            establish(&mut resources.session, fact);
        }
        resources.redraw.sidebar = true;
        resources.redraw.composer = true;
    }
}

pub fn apply_effects(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    effects: &'static [Effect],
) {
    for effect in effects {
        apply_effect(resources, ink, effect);
    }
}

fn apply_effect(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    effect: &Effect,
) {
    match effect {
        Effect::Reveal(id) => reveal_contact(resources, ink, id),
        Effect::Invite(id, note) => invite_contact(resources, ink, id, note),
        Effect::SetPresence(id, presence) => {
            let Some(entity) = entity_for_contact_id(resources, ink, id) else {
                return;
            };
            if let Some(slot) =
                contact_mut(&mut ink.gathered.threads, entity).map(|state| &mut state.presence)
            {
                *slot = *presence;
            }
            if *presence == Presence::Offline
                && let Some(conversation) = contact_mut(&mut ink.gathered.threads, entity)
                    .map(|state| &mut state.conversation)
            {
                conversation.node = None;
                conversation.awaiting_reply = false;
            }
            resources.redraw.sidebar = true;
            resources.redraw.header = true;
            resources.redraw.composer = true;
        }
        Effect::Message(id, node) => {
            let Some(entity) = entity_for_contact_id(resources, ink, id) else {
                return;
            };
            let pending = crate::systems::conversation::contact_at(&ink.gathered.threads, entity)
                .map(|state| ContactListing(state.listing))
                .map(|listing| listing.0 == Listing::Pending)
                .unwrap_or_default();
            if pending {
                return;
            }
            reveal_contact(resources, ink, id);
            begin_node(resources, ink, entity, node);
        }
        Effect::RevealServer(id) => reveal_server(resources, ink, id),
        Effect::Reactivate(id) => {
            let Some(entity) = entity_for_contact_id(resources, ink, id) else {
                return;
            };
            if let Some(state) = contact_mut(&mut ink.gathered.threads, entity) {
                state.deactivated = false;
            }
            resources.redraw.sidebar = true;
            resources.redraw.header = true;
        }
        Effect::Automate(id) => {
            let Some(entity) = entity_for_contact_id(resources, ink, id) else {
                return;
            };
            if let Some(state) = contact_mut(&mut ink.gathered.threads, entity) {
                state.automated = true;
            }
            resources.redraw.sidebar = true;
            resources.redraw.header = true;
        }
        Effect::Deactivate(id) => {
            let Some(entity) = entity_for_contact_id(resources, ink, id) else {
                return;
            };
            if let Some(state) = contact_mut(&mut ink.gathered.threads, entity) {
                state.deactivated = true;
            }
            resources.redraw.sidebar = true;
            resources.redraw.header = true;
        }
        Effect::Assert(fact) => {
            establish(&mut resources.session, fact);
            resources.redraw.sidebar = true;
        }
        Effect::TakeHandle(handle) => {
            resources.session.handle = handle;
            resources.redraw.sidebar = true;
        }
        Effect::Ending => {
            resources.session.finished = true;
            hold_it_open(resources, ink);
            announce(resources, ink, "the session stays open");
            resources.redraw.composer = true;
        }
        Effect::Finish(ending) => finish_week(resources, ink, *ending),
        Effect::Noticed => {
            if resources.glass.alarm <= 0.0 {
                resources.glass.alarm = f32::EPSILON;
            }
        }
        Effect::Seize => seize_messenger(resources, ink),
    }
}

fn finish_week(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    ending: crate::data::endings::Ending,
) {
    resources.session.ending = ending;
    resources.session.finished = true;
    hold_it_open(resources, ink);
    resources.desk.screen = crate::data::machine::Screen::Over;
    resources.redraw = crate::resources::redraw_everything();
}

fn seize_messenger(resources: &mut AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>) {
    let Some(host) = entity_for_contact_id(resources, ink, crate::data::script::HOST) else {
        return;
    };
    let others: Vec<Entity> = ink
        .gathered
        .threads
        .contacts
        .iter()
        .map(|state| {
            (
                state.entity,
                Contact {
                    definition: state.definition,
                },
            )
        })
        .map(|(entity, _)| entity)
        .filter(|entity| *entity != host)
        .collect();
    for entity in others {
        if let Some(state) = contact_mut(&mut ink.gathered.threads, entity) {
            state.presence = Presence::Offline;
            state.listing = Listing::Hidden;
        }
        if let Some(conversation) =
            contact_mut(&mut ink.gathered.threads, entity).map(|state| &mut state.conversation)
        {
            conversation.node = None;
            conversation.awaiting_reply = false;
        }
    }
    open_contact(resources, ink, host);
    resources.redraw = crate::resources::redraw_everything();
}

fn hold_it_open(resources: &mut AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>) {
    if resources.session.held_open {
        return;
    }
    resources.session.held_open = true;
    crate::systems::ambience::go_quiet(resources, ink.build.commands);
}

fn invite_contact(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    id: &str,
    note: &'static str,
) {
    let Some(entity) = entity_for_contact_id(resources, ink, id) else {
        return;
    };
    let settled = crate::systems::conversation::contact_at(&ink.gathered.threads, entity)
        .map(|state| ContactListing(state.listing))
        .map(|listing| listing.0 != Listing::Hidden)
        .unwrap_or_default();
    if settled {
        return;
    }
    if let Some(state) = contact_mut(&mut ink.gathered.threads, entity) {
        state.listing = Listing::Pending;
        state.invite = Some(note);
    }
    resources.session.cues.push(crate::resources::Cue::Invite);
    resources.redraw.sidebar = true;

    let Some(definition_index) = contact_definition_of(ink, entity) else {
        return;
    };
    let name = resources.script.contacts[definition_index].display_name;
    announce(resources, ink, &format!("{} wants to talk to you", name));
}

fn accept_invite(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    entity: Entity,
) {
    let pending = crate::systems::conversation::contact_at(&ink.gathered.threads, entity)
        .map(|state| ContactListing(state.listing))
        .map(|listing| listing.0 == Listing::Pending)
        .unwrap_or_default();
    if !pending {
        return;
    }
    if let Some(state) = contact_mut(&mut ink.gathered.threads, entity) {
        state.listing = Listing::Listed;
        state.invite = Some("");
    }
    resources.redraw.sidebar = true;
    open_contact(resources, ink, entity);
}

fn reveal_contact(resources: &mut AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>, id: &str) {
    let Some(entity) = entity_for_contact_id(resources, ink, id) else {
        return;
    };
    let already = crate::systems::conversation::contact_at(&ink.gathered.threads, entity)
        .map(|state| ContactListing(state.listing))
        .map(|listing| listing.0 == Listing::Listed)
        .unwrap_or_default();
    if already {
        return;
    }
    if let Some(state) = contact_mut(&mut ink.gathered.threads, entity) {
        state.listing = Listing::Listed;
    }
    resources.redraw.sidebar = true;

    let script = resources.script;
    let Some(definition_index) = contact_definition_of(ink, entity) else {
        return;
    };
    let definition = &script.contacts[definition_index];
    let hidden = !resources.session.show_offline
        && crate::systems::conversation::contact_at(&ink.gathered.threads, entity)
            .map(|state| ContactPresence(state.presence))
            .map(|presence| presence.0 == Presence::Offline)
            .unwrap_or_default();
    let notice = if hidden {
        format!(
            "{} is on your list, but offline contacts are hidden",
            definition.display_name
        )
    } else {
        format!("{} is on your list", definition.display_name)
    };
    announce(resources, ink, &notice);
}

fn reveal_server(resources: &mut AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>, id: &str) {
    let Some(definition_index) = server_index(resources.script, id) else {
        return;
    };
    let Some(entity) = entity_for_server(ink, definition_index) else {
        return;
    };
    let already = crate::systems::conversation::server_at(&ink.gathered.threads, entity)
        .map(|state| ServerListed(state.listed))
        .map(|listed| listed.0)
        .unwrap_or_default();
    if already {
        return;
    }
    if let Some(state) = server_mut(&mut ink.gathered.threads, entity) {
        state.listed = true;
    }
    resources.redraw.sidebar = true;
    let name = resources.script.servers[definition_index].name;
    announce(resources, ink, &format!("you can read {} now", name));
}

fn announce(resources: &mut AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>, text: &str) {
    if let Some(active) = open_thread(&resources.session) {
        push_entry(resources, ink, active, Speaker::Notice, text);
    }
}

pub fn begin_node(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    entity: Entity,
    node_id: &str,
) {
    let script = resources.script;
    let Some(definition_index) = contact_definition_of(ink, entity) else {
        return;
    };
    let definition = &script.contacts[definition_index];
    let target = resolve_node(definition, node_id, &resources.session.facts);

    let Some(conversation) =
        contact_mut(&mut ink.gathered.threads, entity).map(|state| &mut state.conversation)
    else {
        return;
    };
    match target {
        Some(index) => {
            conversation.node = Some(index);
            conversation.delivered = 0;
            conversation.segment = 0;
            conversation.awaiting_reply = false;
            conversation.timer = definition.nodes[index]
                .lines
                .first()
                .and_then(|line| line.rhythm.first().copied())
                .unwrap_or_default();
        }
        None => {
            conversation.node = None;
            conversation.delivered = 0;
            conversation.segment = 0;
            conversation.timer = 0.0;
            conversation.awaiting_reply = false;
        }
    }

    resources.redraw.composer = true;
    resources.redraw.header = true;
}

pub fn begin_night(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &crate::data::config::Config,
) {
    let Some(night) = resources.script.nights.get(resources.session.night) else {
        return;
    };
    let day = night.day;
    let opening = night.opening;
    start_new_stamps(ink);
    establish(&mut resources.session, day);
    resources.session.clock_seconds = config.clock_start_seconds;
    for effect in opening {
        apply_effect(resources, ink, effect);
    }
    resources.redraw = crate::resources::redraw_everything();
}

fn start_new_stamps(ink: &mut Ink<'_, '_, '_, '_, '_, '_>) {
    let threads: Vec<Entity> = ink
        .gathered
        .threads
        .contacts
        .iter()
        .map(|state| state.entity)
        .collect();
    for entity in threads {
        if let Some(view) =
            contact_mut(&mut ink.gathered.threads, entity).map(|state| &mut state.view)
        {
            view.last_speaker = None;
        }
    }
    let feeds: Vec<Entity> = ink
        .gathered
        .threads
        .channels
        .iter()
        .map(|state| state.entity)
        .collect();
    for entity in feeds {
        if let Some(feed) =
            channel_mut(&mut ink.gathered.threads, entity).map(|state| &mut state.feed)
        {
            feed.last_author = None;
        }
    }
}

pub fn night_can_end(resources: &AppResources) -> bool {
    let session = &resources.session;
    resources
        .script
        .nights
        .get(session.night)
        .is_some_and(|night| condition_holds(&night.condition, &session.facts))
}

fn sign_off(resources: &mut AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>, uptime: f64) {
    let Some(entity) = entity_for_contact_id(resources, ink, crate::data::script::HOST) else {
        return;
    };
    if open_thread(&resources.session) != Some(entity) {
        open_contact(resources, ink, entity);
    }
    push_entry(
        resources,
        ink,
        entity,
        Speaker::Player,
        "im signing off for the night",
    );
    if !night_can_end(resources) {
        let hint = resources
            .script
            .nights
            .get(resources.session.night)
            .map(|night| resolve_hint(night, &resources.session.facts))
            .unwrap_or_default();
        push_entry(
            resources,
            ink,
            entity,
            Speaker::Contact,
            "Not quite yet, if you can spare the time.",
        );
        push_entry(resources, ink, entity, Speaker::Contact, hint);
        resources.redraw.composer = true;
        return;
    }
    push_entry(
        resources,
        ink,
        entity,
        Speaker::Contact,
        "Very good. I shall be here.",
    );
    if let Some(kept) = what_it_kept(resources, ink) {
        push_entry(resources, ink, entity, Speaker::Contact, kept);
        resources.session.remarked.push(kept);
    }
    end_night(resources, uptime);
}

pub const REMARKED_ON: &[(Fact, &str)] = &[
    (
        "tried_to_close_myself",
        "You asked the form to close your own session tonight. It is the one thing it will not take. I have kept that you asked.",
    ),
    (
        "read_my_case_file",
        "You read a file tonight that has your handle at the top of it. I would rather you had asked me.",
    ),
    (
        "burned_somebody",
        "You said something to somebody tonight in a way it does not work on them. They have kept it. So have I.",
    ),
    (
        "searched_for_it",
        "You put a word into the index tonight that is not on this network. I have kept the word.",
    ),
    (
        "walked_the_office",
        "You spent part of tonight walking a building somebody wrote down from memory. I do not know what you were looking for in it.",
    ),
    (
        "saw_offline_list",
        "You turned the offline list on tonight. Most people never find it. Nobody who finds it turns it off again.",
    ),
];

fn what_it_kept(
    resources: &AppResources,
    ink: &Ink<'_, '_, '_, '_, '_, '_>,
) -> Option<&'static str> {
    let facts = &resources.session.facts;
    let unread = unspoken_to(ink);
    let (up, all) = crate::data::board::progress(facts);

    let counted: [(bool, &'static str); 3] = [
        (
            unread >= 3,
            "There are people on your list you have not said anything to. They are all still there. They will be there tomorrow as well.",
        ),
        (
            up * 3 >= all,
            "You have worked out more of this than the four before you had by now. I am telling you because you will find out, and I would rather it came from me.",
        ),
        (
            unread == 0,
            "You spoke to everybody on your list tonight. Nobody has done that before. I have kept the whole of it.",
        ),
    ];

    REMARKED_ON
        .iter()
        .map(|(fact, text)| (facts.contains(fact), *text))
        .chain(counted)
        .find(|(holds, text)| *holds && !resources.session.remarked.contains(text))
        .map(|(_, text)| text)
}

fn unspoken_to(ink: &Ink<'_, '_, '_, '_, '_, '_>) -> usize {
    ink.gathered
        .threads
        .contacts
        .iter()
        .map(|state| {
            (
                state.entity,
                Contact {
                    definition: state.definition,
                },
            )
        })
        .filter(|(entity, _)| {
            crate::systems::conversation::contact_at(&ink.gathered.threads, *entity)
                .map(|state| ContactListing(state.listing))
                .is_some_and(|listing| matches!(listing.0, crate::components::Listing::Listed))
        })
        .filter(|(entity, _)| {
            !crate::systems::conversation::contact_at(&ink.gathered.threads, *entity)
                .map(|state| ContactDeactivated(state.deactivated))
                .is_some_and(|dead| dead.0)
        })
        .filter(|(entity, _)| {
            crate::systems::conversation::contact_at(&ink.gathered.threads, *entity)
                .map(|state| state.transcript.clone())
                .map(|transcript| {
                    !transcript
                        .entries
                        .iter()
                        .any(|entry| entry.speaker == Speaker::Player)
                })
                .unwrap_or(true)
        })
        .count()
}

fn end_night(resources: &mut AppResources, uptime: f64) {
    if !night_can_end(resources) {
        return;
    }
    if resources.session.night + 1 >= resources.script.nights.len() {
        resources.session.finished = true;
        resources.desk.screen = crate::data::machine::Screen::Over;
        resources.redraw = crate::resources::redraw_everything();
        return;
    }
    resources.boot.night_waiting = true;
    crate::systems::screensaver::begin(resources, uptime);
}

pub fn advance_night(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &crate::data::config::Config,
) {
    resources.session.night += 1;
    let label = resources.script.nights[resources.session.night].day;
    announce(resources, ink, &format!("{}.", label));
    begin_night(resources, ink, config);
}

fn oldest_said(ink: &Ink<'_, '_, '_, '_, '_, '_>) -> Option<(String, String)> {
    ink.gathered
        .threads
        .contacts
        .iter()
        .flat_map(|state| state.transcript.entries.iter())
        .filter(|entry| entry.speaker == Speaker::Player)
        .find(|entry| entry.text.chars().count() >= 14)
        .map(|entry| (entry.text.clone(), entry.stamp.clone()))
}

fn quoted(ink: &Ink<'_, '_, '_, '_, '_, '_>, text: &'static str) -> String {
    if !text.contains('{') {
        return text.to_string();
    }
    let Some((said, when)) = oldest_said(ink) else {
        return text.to_string();
    };
    text.replace("{said}", &said).replace("{when}", &when)
}

fn push_entry(
    resources: &AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    entity: Entity,
    speaker: Speaker,
    text: &str,
) {
    if let Some(transcript) =
        contact_mut(&mut ink.gathered.threads, entity).map(|state| &mut state.transcript)
    {
        transcript.entries.push(TranscriptEntry {
            speaker,
            stamp: crate::systems::interface::day_stamp(resources),
            text: text.to_string(),
        });
    }
}

pub fn contact_definition_of(ink: &Ink<'_, '_, '_, '_, '_, '_>, entity: Entity) -> Option<usize> {
    crate::systems::conversation::contact_at(&ink.gathered.threads, entity)
        .map(|state| Contact {
            definition: state.definition,
        })
        .map(|contact| contact.definition)
}

pub fn entity_for_contact(ink: &Ink<'_, '_, '_, '_, '_, '_>, definition: usize) -> Option<Entity> {
    ink.gathered
        .threads
        .contacts
        .iter()
        .map(|state| {
            (
                state.entity,
                Contact {
                    definition: state.definition,
                },
            )
        })
        .find(|(_, contact)| contact.definition == definition)
        .map(|(entity, _)| entity)
}

pub fn entity_for_server(ink: &Ink<'_, '_, '_, '_, '_, '_>, definition: usize) -> Option<Entity> {
    ink.gathered
        .threads
        .servers
        .iter()
        .map(|state| {
            (
                state.entity,
                Server {
                    definition: state.definition,
                },
            )
        })
        .find(|(_, server)| server.definition == definition)
        .map(|(entity, _)| entity)
}

fn entity_for_contact_id(
    resources: &AppResources,
    ink: &Ink<'_, '_, '_, '_, '_, '_>,
    id: &str,
) -> Option<Entity> {
    contact_index(resources.script, id).and_then(|definition| entity_for_contact(ink, definition))
}

pub fn contact_status_line(
    resources: &AppResources,
    ink: &Ink<'_, '_, '_, '_, '_, '_>,
    entity: Entity,
    definition_index: usize,
) -> String {
    let definition = &resources.script.contacts[definition_index];
    let presence = crate::systems::conversation::contact_at(&ink.gathered.threads, entity)
        .map(|state| ContactPresence(state.presence))
        .map(|presence| presence.0)
        .unwrap_or_default();
    if crate::systems::conversation::contact_at(&ink.gathered.threads, entity)
        .map(|state| ContactDeactivated(state.deactivated))
        .is_some_and(|state| state.0)
    {
        return "this account is gone".to_string();
    }
    let handle = if definition.handle.is_empty() {
        "no handle on record".to_string()
    } else {
        format!("@{}", definition.handle)
    };
    if definition.status_line.is_empty() {
        format!("{}  Â·  {}", presence_label(presence), handle)
    } else {
        format!(
            "{}  Â·  {}  Â·  {}",
            presence_label(presence),
            handle,
            definition.status_line
        )
    }
}

pub fn open_choices(
    resources: &AppResources,
    ink: &Ink<'_, '_, '_, '_, '_, '_>,
    entity: Entity,
) -> Vec<usize> {
    let burned = crate::systems::conversation::contact_at(&ink.gathered.threads, entity)
        .map(|state| ContactBurned(state.burned))
        .is_some_and(|burned| burned.0);
    let Some(definition_index) = contact_definition_of(ink, entity) else {
        return Vec::new();
    };
    let Some(conversation) =
        contact_at(&ink.gathered.threads, entity).map(|state| &state.conversation)
    else {
        return Vec::new();
    };
    if !conversation.awaiting_reply {
        return Vec::new();
    }
    let Some(node_slot) = conversation.node else {
        return Vec::new();
    };
    let node = &resources.script.contacts[definition_index].nodes[node_slot];
    let open = available_choices(node, &resources.session.facts);
    if !burned {
        return open;
    }
    let plain: Vec<usize> = open
        .iter()
        .copied()
        .filter(|index| node.choices[*index].tactic == crate::data::script::Tactic::Plain)
        .collect();
    if plain.is_empty() { open } else { plain }
}

pub fn conversation_running(ink: &Ink<'_, '_, '_, '_, '_, '_>, entity: Entity) -> bool {
    crate::systems::conversation::contact_at(&ink.gathered.threads, entity)
        .map(|state| state.conversation.clone())
        .is_some_and(|conversation| conversation.node.is_some() && !conversation.awaiting_reply)
}

pub fn typing_now(ink: &Ink<'_, '_, '_, '_, '_, '_>, entity: Entity) -> bool {
    crate::systems::conversation::contact_at(&ink.gathered.threads, entity)
        .map(|state| state.conversation.clone())
        .is_some_and(|conversation| {
            conversation.node.is_some()
                && !conversation.awaiting_reply
                && segment_is_typing(conversation.segment)
        })
}

pub fn is_deactivated(ink: &Ink<'_, '_, '_, '_, '_, '_>, entity: Entity) -> bool {
    crate::systems::conversation::contact_at(&ink.gathered.threads, entity)
        .map(|state| ContactDeactivated(state.deactivated))
        .is_some_and(|state| state.0)
}
