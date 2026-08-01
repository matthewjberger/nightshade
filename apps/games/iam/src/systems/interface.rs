use crate::components::StoreLink;
use crate::components::{
    ChannelAsked, ChoiceSlot, ContactPresence, RowChannel, RowContact, RowServer, RowSlot,
    ServerOpenChannel, Speaker, TranscriptEntry,
};
use crate::data::config::Config;
use crate::data::script::{
    Presence, Tactic, available_questions, avatar_letter, presence_label, resolve_hint,
    speaker_color, tactic_label,
};
use crate::resources::{
    AppResources, Ink, Interface, View, choice_capacity, knows, open_server, open_thread,
};
use crate::systems::conversation::{
    contact_definition_of, contact_status_line, conversation_running, is_deactivated,
    night_can_end, open_choices, typing_now,
};
use crate::systems::feeds::server_unread;
use nightshade::prelude::Entity;
use nightshade::prelude::*;
use nightshade_platform::prelude::{Time, Window};
use nightshade_text::contract::data::text_data::{TextAlignment, VerticalAlignment};
use nightshade_ui::commands::builder::UiBuild;
use nightshade_ui::prelude::*;

struct ContactRowData {
    entity: Entity,
    definition: usize,
    presence: Presence,
    unread: u32,
    deactivated: bool,
    active: bool,
}

pub fn sync(
    mut resources: ResMut<AppResources>,
    settings: Res<Config>,
    time: Res<Time>,
    window: Res<Window>,
    mut gathered: ResMut<crate::systems::desk::Gathered>,
    mut build: UiBuild,
    nodes: Query<&mut UiLayoutNode>,
    contents: Query<&mut UiNodeContent>,
    mut scrolls: Query<&UiScrollAreaData>,
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
    let config = *settings;
    let viewport = crate::systems::desk::viewport_of(&window, &config);
    if resources.redraw.sidebar {
        resources.redraw.sidebar = false;
        refresh_sidebar(resources, ink, &config);
    }
    if resources.redraw.header {
        resources.redraw.header = false;
        refresh_header(resources, ink, &config);
    }
    if resources.redraw.composer {
        resources.redraw.composer = false;
        refresh_composer(resources, ink, &config);
    }
    refresh_marquee(resources, ink, viewport, &config, time.delta_time);
    append_thread_lines(resources, ink, &config);
    append_channel_posts(resources, ink, &config);
    refresh_typing(resources, ink, &config);
    let status = resources
        .script
        .nights
        .get(resources.session.night)
        .map(|night| night.player_status)
        .unwrap_or_default();
    ui_set_text(
        ink.build.commands,
        resources.interface.player_status,
        status,
    );
    let handle = resources.session.handle;
    ui_set_text(
        ink.build.commands,
        resources.interface.player_handle,
        handle,
    );
    pin_scroll(
        resources,
        ink,
        &mut scrolls,
        window.cached_scale_factor,
        &config,
    );
}

pub fn apply_pane_layout(
    interface: &Interface,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    top: f32,
    composer_height: f32,
) {
    set_boundary(
        ink,
        interface.log_holder,
        Ab(vec2(0.0, top)).into(),
        Rl(vec2(100.0, 100.0)) + Ab(vec2(0.0, -composer_height)),
    );
    set_window(
        ink,
        interface.composer,
        Rl(vec2(0.0, 100.0)) + Ab(vec2(0.0, -composer_height)),
        Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, composer_height)),
    );
    set_window(
        ink,
        interface.composer_divider,
        Rl(vec2(0.0, 100.0)) + Ab(vec2(0.0, -composer_height)),
        Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 1.0)),
    );
    ui_mark_layout_dirty(ink.build.dirty);
}

fn set_boundary(
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    entity: Entity,
    position_1: UiValue<Vec2>,
    position_2: UiValue<Vec2>,
) {
    ink.nodes.for_entity(entity, |node| {
        node.base_layout = Some(UiLayoutType::Boundary(BoundaryLayout {
            position_1,
            position_2,
        }));
    });
}

fn set_window(
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    entity: Entity,
    position: UiValue<Vec2>,
    size: UiValue<Vec2>,
) {
    ink.nodes.for_entity(entity, |node| {
        node.base_layout = Some(UiLayoutType::Window(WindowLayout {
            position,
            size,
            anchor: Anchor::TopLeft,
        }));
    });
}

fn presence_color(config: &Config, presence: Presence) -> Vec4 {
    match presence {
        Presence::Online => config.online,
        Presence::Idle => config.idle,
        Presence::Offline => config.offline,
    }
}

fn dim(color: Vec4, factor: f32) -> Vec4 {
    Vec4::new(
        color.x * factor,
        color.y * factor,
        color.z * factor,
        color.w,
    )
}

fn fit_badge(
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
    slot: &RowSlot,
    count: u32,
    middle: f32,
) {
    ui_set_visible(ink.build.dirty, ink.build.commands, slot.badge, count > 0);
    if count == 0 {
        return;
    }
    let text = count.to_string();
    ui_set_text(ink.build.commands, slot.badge_text, &text);
    let measured = measure_ui_text_width(ink.build.text_state, &text, 10.0);
    let width = (measured + 12.0).max(config.badge_min_width);
    set_window(
        ink,
        slot.badge,
        Rl(vec2(100.0, 0.0)) + Ab(vec2(-config.row_gutter - width, middle)),
        Ab(vec2(width, 18.0)).into(),
    );
}

fn gather_contacts(
    resources: &AppResources,
    ink: &Ink<'_, '_, '_, '_, '_, '_>,
) -> (Vec<ContactRowData>, usize, u32) {
    let mut rows: Vec<ContactRowData> = Vec::new();
    let mut hidden = 0usize;
    let mut unread_hidden = 0u32;
    let active = open_thread(&resources.session);

    for state in ink.gathered.threads.contacts.iter() {
        let entity = state.entity;
        if state.listing != crate::components::Listing::Listed {
            continue;
        }
        if state.presence == Presence::Offline && !resources.session.show_offline {
            hidden += 1;
            unread_hidden += state.unread;
            continue;
        }
        rows.push(ContactRowData {
            entity,
            definition: state.definition,
            presence: state.presence,
            unread: state.unread,
            deactivated: false,
            active: active == Some(entity),
        });
    }

    for row in &mut rows {
        row.deactivated = is_deactivated(ink, row.entity);
    }
    rows.sort_by_key(|row| row.definition);
    (rows, hidden, unread_hidden)
}

pub fn calendar(day: &str, date: &str) -> String {
    let mut parts = date.split_whitespace();
    let (Some(number), Some(month), Some(year)) = (parts.next(), parts.next(), parts.next()) else {
        return format!("{day}  {date}");
    };
    const MONTHS: [&str; 12] = [
        "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
    ];
    let Some(index) = MONTHS.iter().position(|name| *name == month) else {
        return format!("{day}  {date}");
    };
    let mut named = day.chars();
    let titled = match named.next() {
        Some(first) => first.to_uppercase().collect::<String>() + named.as_str(),
        None => String::new(),
    };
    let short: String = year
        .chars()
        .skip(year.chars().count().saturating_sub(2))
        .collect();
    format!("{titled}, {:02}/{}/{short}", index + 1, number)
}

fn night_status(resources: &AppResources) -> String {
    if night_can_end(resources) {
        return "iambot is waiting for you to say goodnight".to_string();
    }
    resources
        .script
        .nights
        .get(resources.session.night)
        .map(|night| resolve_hint(night, &resources.session.facts).to_string())
        .unwrap_or_default()
}

fn marquee_width(ink: &Ink<'_, '_, '_, '_, '_, '_>, viewport: Vec2, config: &Config) -> usize {
    let program = crate::systems::setup::program_index("iam");
    let width = ink
        .gathered
        .windows
        .iter()
        .map(|row| (row.entity, row.window))
        .find(|(_, window)| window.program == program)
        .map(|(_, window)| crate::systems::desk::laid_out_size(&window, viewport, config).x)
        .unwrap_or(config.frame_min_size.x);
    (((width - 32.0) / 6.6).floor() as usize).max(8)
}

fn refresh_marquee(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    viewport: Vec2,
    config: &Config,
    delta: f32,
) {
    let status = night_status(resources);
    let width = marquee_width(ink, viewport, config);
    let run: Vec<char> = status.chars().collect();
    if run.len() <= width {
        resources.session.marquee_phase = 0.0;
        ui_set_text(
            ink.build.commands,
            resources.interface.marquee_text,
            &status,
        );
        return;
    }
    let padded: Vec<char> = status.chars().chain("        ".chars()).collect();
    resources.session.marquee_phase += delta * config.marquee_speed;
    if resources.session.marquee_phase >= padded.len() as f32 {
        resources.session.marquee_phase -= padded.len() as f32;
    }
    let at = resources.session.marquee_phase as usize % padded.len();
    let shown: String = padded.iter().cycle().skip(at).take(width).collect();
    ui_set_text(ink.build.commands, resources.interface.marquee_text, &shown);
}

fn refresh_sidebar(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let (rows, hidden, unread_hidden) = gather_contacts(resources, ink);

    let label = if resources.session.show_offline {
        format!("-  HIDE OFFLINE   {} shown", rows.len())
    } else if unread_hidden > 0 {
        format!(
            "+  SHOW OFFLINE   {} hidden, {} unread",
            hidden, unread_hidden
        )
    } else if hidden > 0 {
        format!("+  SHOW OFFLINE   {} hidden", hidden)
    } else {
        "+  SHOW OFFLINE".to_string()
    };
    ui_set_text(
        ink.build.commands,
        resources.interface.offline_label,
        &label,
    );
    ui_set_color::<UiBase>(
        ink.build.commands,
        resources.interface.offline_label,
        if unread_hidden > 0 {
            config.ask_marker
        } else {
            config.text_dim
        },
    );

    let day_number = format!("day {}", resources.session.night + 1);
    ui_set_text(
        ink.build.commands,
        resources.interface.night_label,
        &day_number,
    );

    refresh_pending_rows(resources, ink, config);
    refresh_server_rows(resources, ink, config);
    refresh_channel_rows(resources, ink, config);

    let groups = resources.interface.sidebar_groups.clone();
    for group in &groups {
        let members: Vec<&ContactRowData> = rows
            .iter()
            .filter(|row| row.presence == group.presence)
            .collect();
        let heading = format!(
            "{}  {}",
            presence_label(group.presence).to_uppercase(),
            members.len()
        );
        ui_set_visible(
            ink.build.dirty,
            ink.build.commands,
            group.heading,
            !members.is_empty(),
        );
        ui_set_text(ink.build.commands, group.heading, &heading);

        for (index, slot) in group.rows.iter().enumerate() {
            let Some(row) = members.get(index) else {
                ui_set_visible(ink.build.dirty, ink.build.commands, slot.root, false);
                ink.build.commands.set(slot.root, RowContact(None));
                continue;
            };
            fill_contact_row(resources, ink, config, slot, row);
            ink.build
                .commands
                .set(slot.root, RowContact(Some(row.entity)));
        }
    }
}

fn unasked_count(
    resources: &AppResources,
    ink: &Ink<'_, '_, '_, '_, '_, '_>,
    channel: Entity,
) -> usize {
    let Some(entry) = crate::systems::conversation::channel_at(&ink.gathered.threads, channel)
        .map(|state| state.channel.clone())
    else {
        return 0;
    };
    let definition = &resources.script.servers[entry.server_definition].channels[entry.definition];
    let asked = crate::systems::conversation::channel_at(&ink.gathered.threads, channel)
        .map(|state| ChannelAsked(state.asked.clone()))
        .map(|asked| asked.0.clone())
        .unwrap_or_default();
    available_questions(definition, &resources.session.facts)
        .into_iter()
        .filter(|index| !asked.contains(index))
        .count()
}

fn server_unasked(
    resources: &AppResources,
    ink: &Ink<'_, '_, '_, '_, '_, '_>,
    server: Entity,
) -> usize {
    let rooms: Vec<Entity> = ink
        .gathered
        .threads
        .channels
        .iter()
        .map(|state| (state.entity, state.channel.clone()))
        .filter(|(_, channel)| channel.server == server)
        .map(|(entity, _)| entity)
        .collect();
    rooms
        .into_iter()
        .map(|entity| unasked_count(resources, ink, entity))
        .sum()
}

fn refresh_pending_rows(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let heading = resources.interface.pending_heading;
    let slots = resources.interface.pending_rows.clone();
    let mut waiting: Vec<(Entity, usize, &'static str)> = ink
        .gathered
        .threads
        .contacts
        .iter()
        .filter(|state| state.listing == crate::components::Listing::Pending)
        .filter_map(|state| {
            state
                .invite
                .map(|invite| (state.entity, state.definition, invite))
        })
        .collect();
    waiting.sort_by_key(|(_, definition, _)| *definition);

    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        heading,
        !waiting.is_empty(),
    );
    ui_set_text(
        ink.build.commands,
        heading,
        &format!("PENDING  {}", waiting.len()),
    );

    for (index, slot) in slots.iter().enumerate() {
        let Some((entity, definition_index, note)) = waiting.get(index).copied() else {
            ui_set_visible(ink.build.dirty, ink.build.commands, slot.root, false);
            ink.build.commands.set(slot.root, RowContact(None));
            continue;
        };
        let definition = &resources.script.contacts[definition_index];
        ui_set_visible(ink.build.dirty, ink.build.commands, slot.root, true);
        ui_set_color::<UiBase>(ink.build.commands, slot.root, config.row_base);
        ui_set_color::<UiBase>(
            ink.build.commands,
            slot.avatar,
            dim(definition.accent, 0.35),
        );
        ui_set_text(
            ink.build.commands,
            slot.letter,
            &avatar_letter(definition.display_name),
        );
        ui_set_color::<UiBase>(ink.build.commands, slot.letter, config.text_dim);
        ui_set_visible(
            ink.build.dirty,
            ink.build.commands,
            slot.silhouette_head,
            false,
        );
        ui_set_visible(
            ink.build.dirty,
            ink.build.commands,
            slot.silhouette_body,
            false,
        );
        ui_set_visible(ink.build.dirty, ink.build.commands, slot.dot, false);
        ui_set_text(ink.build.commands, slot.name, definition.display_name);
        ui_set_color::<UiBase>(ink.build.commands, slot.name, config.text_dim);
        ui_set_text(ink.build.commands, slot.status, note);
        ui_set_visible(ink.build.dirty, ink.build.commands, slot.badge, true);
        ui_set_color::<UiBase>(ink.build.commands, slot.badge, config.accept_badge);
        ui_set_text(ink.build.commands, slot.badge_text, "accept");
        let width = measure_ui_text_width(ink.build.text_state, "accept", 10.0) + 14.0;
        set_window(
            ink,
            slot.badge,
            Rl(vec2(100.0, 0.0)) + Ab(vec2(-config.row_gutter - width, 17.0)),
            Ab(vec2(width, 18.0)).into(),
        );
        ink.build.commands.set(slot.root, RowContact(Some(entity)));
    }
}

fn refresh_server_rows(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let script = resources.script;
    let open = open_server(&resources.session);
    let mut listed: Vec<(Entity, usize)> = ink
        .gathered
        .threads
        .servers
        .iter()
        .filter(|state| state.listed)
        .map(|state| (state.entity, state.definition))
        .collect();
    listed.sort_by_key(|(_, definition)| *definition);

    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        resources.interface.server_heading,
        !listed.is_empty(),
    );
    ui_set_text(
        ink.build.commands,
        resources.interface.server_heading,
        &format!("SERVERS  {}", listed.len()),
    );

    let slots = resources.interface.server_rows.clone();
    for (index, slot) in slots.iter().enumerate() {
        let Some((entity, definition_index)) = listed.get(index).copied() else {
            ui_set_visible(ink.build.dirty, ink.build.commands, slot.root, false);
            ink.build.commands.set(slot.root, RowServer(None));
            continue;
        };
        let definition = &script.servers[definition_index];
        let unread = server_unread(ink, entity);
        let active = open == Some(entity);

        ui_set_visible(ink.build.dirty, ink.build.commands, slot.root, true);
        ui_set_color::<UiBase>(
            ink.build.commands,
            slot.root,
            if active {
                config.row_active
            } else {
                config.row_base
            },
        );
        ui_set_color::<UiBase>(
            ink.build.commands,
            slot.avatar,
            dim(definition.accent, 0.55),
        );
        ui_set_text(
            ink.build.commands,
            slot.letter,
            &avatar_letter(definition.name),
        );
        ui_set_color::<UiBase>(ink.build.commands, slot.letter, config.text);
        ui_set_visible(
            ink.build.dirty,
            ink.build.commands,
            slot.silhouette_head,
            false,
        );
        ui_set_visible(
            ink.build.dirty,
            ink.build.commands,
            slot.silhouette_body,
            false,
        );
        ui_set_visible(ink.build.dirty, ink.build.commands, slot.dot, false);
        ui_set_text(ink.build.commands, slot.name, definition.name);
        ui_set_color::<UiBase>(ink.build.commands, slot.name, config.text);
        let unasked = server_unasked(resources, ink, entity);
        let subtitle = if unasked > 0 {
            format!(
                "{} rooms  Â·  {} to ask",
                definition.channels.len(),
                unasked
            )
        } else {
            format!("{} rooms", definition.channels.len())
        };
        ui_set_text(ink.build.commands, slot.status, &subtitle);
        ui_set_color::<UiBase>(
            ink.build.commands,
            slot.status,
            if unasked > 0 {
                config.ask_marker
            } else {
                config.text_faint
            },
        );
        fit_badge(ink, config, slot, unread, 17.0);
        ink.build.commands.set(slot.root, RowServer(Some(entity)));
    }
}

fn fill_contact_row(
    resources: &AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
    slot: &RowSlot,
    row: &ContactRowData,
) {
    let definition = &resources.script.contacts[row.definition];
    let faded = row.presence == Presence::Offline;
    let accent = if row.deactivated {
        config.silhouette_field
    } else if faded {
        dim(definition.accent, 0.45)
    } else {
        definition.accent
    };
    let letters = if row.deactivated {
        config.deactivated
    } else if faded {
        config.text_dim
    } else {
        config.text
    };

    ui_set_visible(ink.build.dirty, ink.build.commands, slot.root, true);
    ui_set_color::<UiBase>(
        ink.build.commands,
        slot.root,
        if row.active {
            config.row_active
        } else {
            config.row_base
        },
    );
    ui_set_color::<UiBase>(
        ink.build.commands,
        slot.avatar,
        if row.deactivated {
            config.silhouette_field
        } else {
            dim(accent, 0.6)
        },
    );
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        slot.silhouette_head,
        row.deactivated,
    );
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        slot.silhouette_body,
        row.deactivated,
    );
    let letter = if row.deactivated {
        String::new()
    } else {
        avatar_letter(definition.display_name)
    };
    ui_set_text(ink.build.commands, slot.letter, &letter);
    ui_set_color::<UiBase>(ink.build.commands, slot.letter, letters);
    ui_set_visible(ink.build.dirty, ink.build.commands, slot.dot, true);
    ui_set_color::<UiBase>(
        ink.build.commands,
        slot.dot,
        presence_color(config, row.presence),
    );

    let name = if row.deactivated {
        "Deactivated User"
    } else {
        definition.display_name
    };
    ui_set_text(ink.build.commands, slot.name, name);
    ui_set_color::<UiBase>(ink.build.commands, slot.name, letters);

    let status = if row.deactivated {
        String::new()
    } else if definition.status_line.is_empty() {
        presence_label(row.presence).to_string()
    } else {
        format!(
            "{}  Â·  {}",
            presence_label(row.presence),
            definition.status_line
        )
    };
    ui_set_text(ink.build.commands, slot.status, &status);

    fit_badge(ink, config, slot, row.unread, 17.0);
}

fn refresh_header(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let interface_name = resources.interface.header_name;
    let interface_status = resources.interface.header_status;
    let avatar = resources.interface.header_avatar;
    let letter = resources.interface.header_letter;
    let head = resources.interface.header_silhouette_head;
    let body = resources.interface.header_silhouette_body;

    show_only_open_column(resources, ink);

    match resources.session.view {
        View::Thread(entity) => {
            let Some(definition_index) = contact_definition_of(ink, entity) else {
                return;
            };
            let definition = &resources.script.contacts[definition_index];
            let dead = is_deactivated(ink, entity);
            ui_set_visible(ink.build.dirty, ink.build.commands, head, dead);
            ui_set_visible(ink.build.dirty, ink.build.commands, body, dead);
            let glyph = if dead {
                String::new()
            } else {
                avatar_letter(definition.display_name)
            };
            ui_set_text(ink.build.commands, letter, &glyph);
            ui_set_color::<UiBase>(
                ink.build.commands,
                avatar,
                if dead {
                    config.silhouette_field
                } else {
                    dim(definition.accent, 0.6)
                },
            );
            ui_set_text(
                ink.build.commands,
                interface_name,
                if dead {
                    "Deactivated User"
                } else {
                    definition.display_name
                },
            );
            ui_set_color::<UiBase>(
                ink.build.commands,
                interface_name,
                if dead {
                    config.deactivated
                } else {
                    config.text
                },
            );
            let status = contact_status_line(resources, ink, entity, definition_index);
            ui_set_text(ink.build.commands, interface_status, &status);
        }
        View::Server(entity) => {
            let Some(definition_index) =
                crate::systems::conversation::server_at(&ink.gathered.threads, entity)
                    .map(|state| state.definition)
            else {
                return;
            };
            let definition = &resources.script.servers[definition_index];
            let open = crate::systems::conversation::server_at(&ink.gathered.threads, entity)
                .map(|state| ServerOpenChannel(state.open_channel))
                .and_then(|open| open.0)
                .and_then(|channel| {
                    crate::systems::conversation::channel_at(&ink.gathered.threads, channel)
                        .map(|state| state.channel.clone())
                })
                .map(|channel| &definition.channels[channel.definition]);
            ui_set_visible(ink.build.dirty, ink.build.commands, head, false);
            ui_set_visible(ink.build.dirty, ink.build.commands, body, false);
            ui_set_text(ink.build.commands, letter, &avatar_letter(definition.name));
            ui_set_color::<UiBase>(ink.build.commands, avatar, dim(definition.accent, 0.6));
            let title = match open {
                Some(channel) => format!("# {}", channel.name),
                None => definition.name.to_string(),
            };
            ui_set_text(ink.build.commands, interface_name, &title);
            ui_set_color::<UiBase>(ink.build.commands, interface_name, config.text);
            let topic = open
                .map(|channel| channel.topic)
                .unwrap_or(definition.topic);
            ui_set_text(ink.build.commands, interface_status, topic);
        }
        View::Nothing => {
            ui_set_visible(ink.build.dirty, ink.build.commands, head, false);
            ui_set_visible(ink.build.dirty, ink.build.commands, body, false);
            ui_set_text(ink.build.commands, letter, "");
            ui_set_color::<UiBase>(ink.build.commands, avatar, config.brand_dim);
            ui_set_text(ink.build.commands, interface_name, "nothing open");
            ui_set_text(
                ink.build.commands,
                interface_status,
                "pick a name on the left",
            );
        }
    }
}

fn refresh_channel_rows(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let heading = resources.interface.channel_heading;
    let slots = resources.interface.channel_rows.clone();

    let Some(server) = open_server(&resources.session) else {
        ui_set_visible(ink.build.dirty, ink.build.commands, heading, false);
        for slot in &slots {
            ui_set_visible(ink.build.dirty, ink.build.commands, slot.root, false);
            ink.build.commands.set(slot.root, RowChannel(None));
        }
        return;
    };
    let Some(definition_index) =
        crate::systems::conversation::server_at(&ink.gathered.threads, server)
            .map(|state| state.definition)
    else {
        return;
    };

    let open = crate::systems::conversation::server_at(&ink.gathered.threads, server)
        .map(|state| ServerOpenChannel(state.open_channel))
        .and_then(|open| open.0);
    let mut channels: Vec<(Entity, usize, u32)> = ink
        .gathered
        .threads
        .channels
        .iter()
        .filter(|state| state.channel.server == server)
        .map(|state| (state.entity, state.channel.definition, state.unread))
        .collect();
    channels.sort_by_key(|(_, definition, _)| *definition);

    ui_set_visible(ink.build.dirty, ink.build.commands, heading, true);
    ui_set_text(
        ink.build.commands,
        heading,
        &format!(
            "{}  {}",
            resources.script.servers[definition_index]
                .name
                .to_uppercase(),
            channels.len()
        ),
    );

    let middle = (config.channel_row_height - 18.0) * 0.5;
    for (index, slot) in slots.iter().enumerate() {
        let Some((entity, channel_index, unread)) = channels.get(index).copied() else {
            ui_set_visible(ink.build.dirty, ink.build.commands, slot.root, false);
            ink.build.commands.set(slot.root, RowChannel(None));
            continue;
        };
        let name = resources.script.servers[definition_index].channels[channel_index].name;
        let active = open == Some(entity);
        ui_set_visible(ink.build.dirty, ink.build.commands, slot.root, true);
        ui_set_text(ink.build.commands, slot.name, name);
        ui_set_color::<UiBase>(
            ink.build.commands,
            slot.name,
            if active || unread > 0 {
                config.text
            } else {
                config.text_dim
            },
        );
        let unasked = unasked_count(resources, ink, entity) > 0;
        ui_set_text(
            ink.build.commands,
            slot.letter,
            if unasked { "?" } else { "#" },
        );
        ui_set_color::<UiBase>(
            ink.build.commands,
            slot.letter,
            if unasked {
                config.ask_marker
            } else if active {
                config.text_dim
            } else {
                config.text_faint
            },
        );
        ui_set_color::<UiBase>(
            ink.build.commands,
            slot.root,
            if active {
                config.row_active
            } else {
                config.row_base
            },
        );
        fit_badge(ink, config, slot, unread, middle);
        ink.build.commands.set(slot.root, RowChannel(Some(entity)));
    }
}

fn show_only_open_column(resources: &AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>) {
    let open_channel = open_server(&resources.session).and_then(|server| {
        crate::systems::conversation::server_at(&ink.gathered.threads, server)
            .map(|state| ServerOpenChannel(state.open_channel))
            .and_then(|open| open.0)
    });
    let thread = open_thread(&resources.session);

    let threads: Vec<(Entity, Entity)> = ink
        .gathered
        .threads
        .contacts
        .iter()
        .map(|state| (state.entity, state.view.clone()))
        .map(|(entity, view)| (entity, view.column))
        .collect();
    for (entity, column) in threads {
        ui_set_visible(
            ink.build.dirty,
            ink.build.commands,
            column,
            thread == Some(entity),
        );
    }

    let feeds: Vec<(Entity, Entity)> = ink
        .gathered
        .threads
        .channels
        .iter()
        .map(|state| (state.entity, state.feed.clone()))
        .map(|(entity, feed)| (entity, feed.column))
        .collect();
    for (entity, column) in feeds {
        ui_set_visible(
            ink.build.dirty,
            ink.build.commands,
            column,
            open_channel == Some(entity),
        );
    }
}

fn append_thread_lines(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let script = resources.script;
    let player = resources.session.handle;

    let pending: Vec<Entity> = ink
        .gathered
        .threads
        .contacts
        .iter()
        .filter(|state| state.view.drawn < state.transcript.entries.len())
        .map(|state| state.entity)
        .collect();

    for entity in pending {
        let Some(definition_index) = contact_definition_of(ink, entity) else {
            continue;
        };
        let definition = &script.contacts[definition_index];
        let dead = is_deactivated(ink, entity);
        let name = if dead {
            "Deactivated User"
        } else {
            definition.display_name
        };
        let accent = if dead {
            config.deactivated
        } else {
            definition.accent
        };
        let Some(view) = crate::systems::conversation::contact_at(&ink.gathered.threads, entity)
            .map(|state| state.view.clone())
        else {
            continue;
        };
        let entries: Vec<TranscriptEntry> =
            crate::systems::conversation::contact_at(&ink.gathered.threads, entity)
                .map(|state| state.transcript.clone())
                .map(|transcript| transcript.entries[view.drawn..].to_vec())
                .unwrap_or_default();

        let mut last_speaker = view.last_speaker;
        let mut tree = UiTreeBuilder::from_parent(ink.build.reborrow(), view.column);
        for entry in &entries {
            match entry.speaker {
                Speaker::Notice => {
                    last_speaker = None;
                    build_notice(&mut tree, config, &entry.text);
                }
                Speaker::Contact => {
                    if last_speaker != Some(Speaker::Contact) {
                        build_author(&mut tree, config, name, &entry.stamp, accent, false);
                    }
                    last_speaker = Some(Speaker::Contact);
                    match crate::data::script::sent_program(&entry.text) {
                        Some(program) => build_program_card(&mut tree, config, program),
                        None => {
                            build_bubble(&mut tree, config, &entry.text, config.their_bubble, false)
                        }
                    }
                }
                Speaker::Player => {
                    if last_speaker != Some(Speaker::Player) {
                        build_author(
                            &mut tree,
                            config,
                            player,
                            &entry.stamp,
                            config.text_dim,
                            true,
                        );
                    }
                    last_speaker = Some(Speaker::Player);
                    build_bubble(&mut tree, config, &entry.text, config.your_bubble, true);
                }
            }
        }
        tree.finish_subtree();

        let appended = entries.len();
        if let Some(view) =
            crate::systems::conversation::contact_mut(&mut ink.gathered.threads, entity)
                .map(|state| &mut state.view)
        {
            view.drawn += appended;
            view.last_speaker = last_speaker;
        }
    }
}

fn append_channel_posts(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let script = resources.script;
    let player = resources.session.handle;
    let pending: Vec<Entity> = ink
        .gathered
        .threads
        .channels
        .iter()
        .map(|state| (state.entity, state.feed.clone()))
        .filter(|(_, feed)| feed.drawn < feed.log.len())
        .map(|(entity, _)| entity)
        .collect();

    for entity in pending {
        let Some(feed) = crate::systems::conversation::channel_at(&ink.gathered.threads, entity)
            .map(|state| state.feed.clone())
        else {
            continue;
        };
        let mut last_author = feed.last_author;
        let mut tree = UiTreeBuilder::from_parent(ink.build.reborrow(), feed.column);
        for line in &feed.log[feed.drawn..] {
            if last_author != Some(line.author) {
                let color = if line.mine {
                    config.text_dim
                } else {
                    speaker_color(script, line.author)
                };
                let name = if line.mine { player } else { line.author };
                build_author(&mut tree, config, name, &line.stamp, color, line.mine);
            }
            last_author = Some(line.author);
            let fill = if line.mine {
                config.your_bubble
            } else {
                config.their_bubble
            };
            build_bubble(&mut tree, config, line.text, fill, line.mine);
        }
        tree.finish_subtree();

        let drawn = feed.log.len();
        if let Some(feed) =
            crate::systems::conversation::channel_mut(&mut ink.gathered.threads, entity)
                .map(|state| &mut state.feed)
        {
            feed.drawn = drawn;
            feed.last_author = last_author;
        }
    }
}

pub fn day_stamp(resources: &AppResources) -> String {
    let day = resources
        .script
        .nights
        .get(resources.session.night)
        .map(|night| night.day)
        .unwrap_or_default();
    format!("{} {}", day, short_clock(resources.session.clock_seconds))
}

fn twelve_hour(seconds: f32) -> (u32, u32, u32, &'static str) {
    let total = seconds.max(0.0) as u32 % 86_400;
    let hour = total / 3600;
    let shown = match hour % 12 {
        0 => 12,
        other => other,
    };
    let suffix = if hour < 12 { "AM" } else { "PM" };
    (shown, (total % 3600) / 60, total % 60, suffix)
}

fn short_clock(seconds: f32) -> String {
    let (hour, minute, _, suffix) = twelve_hour(seconds);
    format!("{hour}:{minute:02} {suffix}")
}

pub fn wall_clock(seconds: f32) -> String {
    let (hour, minute, second, suffix) = twelve_hour(seconds);
    format!("{hour}:{minute:02}:{second:02} {suffix}")
}

fn build_author(
    tree: &mut UiTreeBuilder,
    config: &Config,
    name: &str,
    stamp: &str,
    color: Vec4,
    mine: bool,
) {
    let text = format!("{}   {}", name, stamp);
    let alignment = if mine {
        TextAlignment::Right
    } else {
        TextAlignment::Left
    };
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(-config.log_track_width, 16.0)))
        .with_text(&text, 12.2)
        .with_text_alignment(alignment, VerticalAlignment::Bottom)
        .color_raw::<UiBase>(color)
        .entity();
}

fn build_notice(tree: &mut UiTreeBuilder, config: &Config, text: &str) {
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(-config.log_track_width, 20.0)))
        .with_text(text, 12.8)
        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
        .color_raw::<UiBase>(config.notice_text)
        .entity();
}

fn build_program_card(tree: &mut UiTreeBuilder, config: &Config, program: &'static str) {
    let listing = crate::data::store::for_program(program)
        .and_then(|index| crate::data::store::LISTINGS.get(index));
    let (title, blurb) = match listing {
        Some(entry) => (entry.title, entry.blurb),
        None => ("unknown program", "this network has never heard of it"),
    };
    let glyph = crate::data::programs::PROGRAMS
        .iter()
        .find(|entry| entry.id == program)
        .map(|entry| entry.glyph)
        .unwrap_or(nightshade_ui::data::icons::material::GRID_ON);

    let row = tree
        .add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(-config.log_track_width, 0.0)))
        .flow_with_alignment(
            FlowDirection::Horizontal,
            0.0,
            0.0,
            FlowAlignment::Start,
            FlowAlignment::Start,
        )
        .entity();

    tree.in_parent(row, |tree| {
        let card = tree
            .add_node()
            .flow_child(Ab(vec2(config.bubble_max_width.min(330.0), 66.0)))
            .with_rect(10.0, 1.0, config.brand)
            .color_raw::<UiBase>(config.their_bubble)
            .color_raw::<UiHover>(config.row_hover)
            .with_transition::<UiHover>(16.0, 8.0)
            .with_interaction()
            .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
            .entity();
        tree.context_reborrow()
            .commands
            .set(card, StoreLink(program));
        tree.in_parent(card, |tree| {
            let mark = tree
                .add_node()
                .window(Ab(vec2(12.0, 12.0)), Ab(vec2(42.0, 42.0)), Anchor::TopLeft)
                .with_rect(6.0, 1.0, config.divider)
                .color_raw::<UiBase>(config.pane_background)
                .entity();
            tree.in_parent(mark, |tree| {
                tree.add_node()
                    .absolute_fill(0.0)
                    .with_icon(glyph, 18.0)
                    .color_raw::<UiBase>(config.brand)
                    .entity();
            });
            tree.add_node()
                .window(Ab(vec2(64.0, 11.0)), Ab(vec2(240.0, 17.0)), Anchor::TopLeft)
                .with_text(title, 13.4)
                .text_left()
                .with_text_overflow(TextOverflow::Ellipsis)
                .color_raw::<UiBase>(config.text)
                .entity();
            tree.add_node()
                .window(Ab(vec2(64.0, 28.0)), Ab(vec2(244.0, 14.0)), Anchor::TopLeft)
                .with_text(blurb, 11.0)
                .text_left()
                .with_text_overflow(TextOverflow::Ellipsis)
                .color_raw::<UiBase>(config.text_faint)
                .entity();
            tree.add_node()
                .window(Ab(vec2(64.0, 44.0)), Ab(vec2(240.0, 14.0)), Anchor::TopLeft)
                .with_text("OPEN IN SOFTWARE", 10.4)
                .text_left()
                .color_raw::<UiBase>(config.brand)
                .entity();
        });
    });
}

fn is_host(resources: &AppResources, ink: &Ink<'_, '_, '_, '_, '_, '_>, entity: Entity) -> bool {
    contact_definition_of(ink, entity)
        .map(|index| resources.script.contacts[index].id == crate::data::script::HOST)
        .unwrap_or_default()
}

fn build_bubble(tree: &mut UiTreeBuilder, config: &Config, text: &str, fill: Vec4, mine: bool) {
    let shade = if mine {
        config.your_bubble_shade
    } else {
        config.bubble_shade
    };
    let measured = measure_ui_text_width(
        tree.context_reborrow().text_state,
        text,
        config.message_font_size,
    );
    let width = (measured + config.bubble_padding * 2.0 + 3.0).clamp(48.0, config.bubble_max_width);
    let alignment = if mine {
        FlowAlignment::End
    } else {
        FlowAlignment::Start
    };

    let row = tree
        .add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(-config.log_track_width, 0.0)))
        .flow_with_alignment(
            FlowDirection::Horizontal,
            0.0,
            0.0,
            alignment,
            FlowAlignment::Start,
        )
        .entity();

    tree.in_parent(row, |tree| {
        let bubble = tree
            .add_node()
            .flow_child(Ab(vec2(width, 0.0)))
            .flow(FlowDirection::Vertical, config.bubble_padding, 0.0)
            .with_rect(12.0, 0.0, config.transparent)
            .color_raw::<UiBase>(fill)
            .with_effect(RectEffect::VerticalGradient { secondary: shade })
            .entity();
        tree.in_parent(bubble, |tree| {
            tree.add_node()
                .fill_width()
                .auto_size(AutoSizeMode::Height)
                .with_text(text, config.message_font_size)
                .with_text_wrap()
                .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
                .color_raw::<UiBase>(config.text)
                .entity();
        });
    });
}

fn refresh_composer(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let script = resources.script;
    let mut choices: Vec<(&'static str, Tactic, bool)> = Vec::new();
    let mut targets: Vec<usize> = Vec::new();
    let mut placeholder = String::new();

    match resources.session.view {
        View::Thread(entity) => {
            let name = contact_definition_of(ink, entity)
                .map(|index| {
                    if is_deactivated(ink, entity) {
                        "that account"
                    } else {
                        script.contacts[index].display_name
                    }
                })
                .unwrap_or("they");
            targets = open_choices(resources, ink, entity);
            if targets.is_empty() {
                let offline =
                    crate::systems::conversation::contact_at(&ink.gathered.threads, entity)
                        .map(|state| ContactPresence(state.presence))
                        .is_some_and(|presence| presence.0 == Presence::Offline);
                placeholder = if conversation_running(ink, entity) {
                    format!("{} is typing", name)
                } else if offline {
                    format!("{} is offline", name)
                } else {
                    format!("{} has nothing more to say", name)
                };
            } else if let Some(definition_index) = contact_definition_of(ink, entity)
                && let Some(node_slot) =
                    crate::systems::conversation::contact_at(&ink.gathered.threads, entity)
                        .map(|state| state.conversation.clone())
                        .and_then(|conversation| conversation.node)
            {
                let node = &script.contacts[definition_index].nodes[node_slot];
                choices = targets
                    .iter()
                    .map(|index| {
                        let choice = &node.choices[*index];
                        let warned = crate::data::script::tactic_lesson(choice.tactic)
                            .is_some_and(|lesson| knows(&resources.session, lesson))
                            && crate::systems::conversation::will_fail(
                                resources,
                                ink,
                                entity,
                                choice.tactic,
                            );
                        (choice.text, choice.tactic, warned)
                    })
                    .collect();
            }
            if is_host(resources, ink, entity) && !conversation_running(ink, entity) {
                targets.push(crate::resources::SIGN_OFF);
                choices.push(("im signing off for the night", Tactic::Plain, false));
                placeholder.clear();
            }
        }
        View::Server(server) => {
            let open_entity =
                crate::systems::conversation::server_at(&ink.gathered.threads, server)
                    .map(|state| ServerOpenChannel(state.open_channel))
                    .and_then(|open| open.0);
            let open = open_entity.and_then(|channel| {
                crate::systems::conversation::channel_at(&ink.gathered.threads, channel)
                    .map(|state| state.channel.clone())
            });
            match (open, open_entity) {
                (Some(channel), Some(open_entity)) => {
                    let definition =
                        &script.servers[channel.server_definition].channels[channel.definition];
                    let asked = crate::systems::conversation::channel_at(
                        &ink.gathered.threads,
                        open_entity,
                    )
                    .map(|state| ChannelAsked(state.asked.clone()))
                    .map(|asked| asked.0.clone())
                    .unwrap_or_default();
                    targets = available_questions(definition, &resources.session.facts)
                        .into_iter()
                        .filter(|index| !asked.contains(index))
                        .collect();
                    choices = targets
                        .iter()
                        .map(|index| {
                            (
                                definition.questions[*index].text,
                                definition.questions[*index].tactic,
                                false,
                            )
                        })
                        .collect();
                    for posted in
                        crate::data::games::in_room(definition.name, &resources.session.facts)
                    {
                        let entry = &crate::data::games::POSTED[posted];
                        targets.push(crate::resources::PLAY + posted);
                        choices.push((entry.title, Tactic::Plain, false));
                    }
                    if choices.is_empty() {
                        placeholder = "nothing to ask here yet".to_string();
                    }
                }
                _ => placeholder = "pick a room".to_string(),
            }
        }
        View::Nothing => {
            placeholder = "pick a name on the left".to_string();
        }
    }

    let slots: Vec<(Entity, ChoiceSlot)> = ink
        .gathered
        .threads
        .choices
        .iter()
        .map(|state| (state.entity, state.slot))
        .collect();
    for (root, mut slot) in slots {
        match choices.get(slot.slot) {
            Some((text, tactic, warned)) => {
                ui_set_visible(ink.build.dirty, ink.build.commands, root, true);
                let mark = if *warned {
                    crate::data::script::WILL_NOT_WORK
                } else {
                    tactic_label(*tactic)
                };
                ui_set_text(ink.build.commands, slot.label, &format!("{mark}{text}"));
                let color = match (warned, tactic) {
                    (true, _) => config.accent_warn,
                    (false, Tactic::Plain) => config.text,
                    (false, _) => config.tactic_text,
                };
                ui_set_color::<UiBase>(ink.build.commands, slot.label, color);
            }
            None => ui_set_visible(ink.build.dirty, ink.build.commands, root, false),
        }
        slot.choice = targets.get(slot.slot).copied();
        ink.build.commands.set(root, slot);
    }
    if resources.session.finished && choices.is_empty() {
        placeholder = if knows(&resources.session, "closed_the_window") {
            format!(
                "session held open  Â·  {}  Â·  it is drawing you anyway",
                day_stamp(resources)
            )
        } else {
            format!(
                "session held open  Â·  {}  Â·  two handles, one chair",
                day_stamp(resources)
            )
        };
    }

    let placeholder_row = resources.interface.placeholder_row;
    ui_set_visible(
        ink.build.dirty,
        ink.build.commands,
        placeholder_row,
        choices.is_empty(),
    );
    if choices.is_empty() {
        let label = resources.interface.placeholder_label;
        ui_set_text(ink.build.commands, label, &placeholder);
    }

    let rows = choices.len().clamp(1, choice_capacity(script)) as f32;
    let composer_height = config.composer_padding * 2.0
        + rows * config.choice_height
        + (rows - 1.0) * config.composer_spacing;
    let top = config.header_height;
    apply_pane_layout(&resources.interface, ink, top, composer_height);
}

fn refresh_typing(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let row = resources.interface.typing_bubble;
    let typing = match open_thread(&resources.session) {
        Some(entity) => typing_now(ink, entity),
        None => false,
    };
    if resources.session.typing_shown != typing {
        resources.session.typing_shown = typing;
        ui_set_visible(ink.build.dirty, ink.build.commands, row, typing);
        ui_mark_layout_dirty(ink.build.dirty);
    }
    if !typing {
        return;
    }
    let rate = config.typing_dots_per_second;
    let dots = ((resources.session.dots_phase * rate) as usize % 3) + 1;
    ui_set_text(
        ink.build.commands,
        resources.interface.typing_dots,
        &".".repeat(dots),
    );
}

fn pin_scroll(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    scrolls: &mut Query<&UiScrollAreaData>,
    scale_factor: f32,
    config: &Config,
) {
    let list = resources.interface.log_scroll;
    let mut found: Option<UiScrollAreaData> = None;
    scrolls.for_entity(list, |data| found = Some(data.clone()));
    let Some(data) = found else {
        return;
    };

    let scale = scale_factor.max(0.001);
    let bottom = (data.content_height - data.visible_height).max(0.0) / scale;
    let previous = resources.session.last_bottom;
    resources.session.last_bottom = bottom;

    let opened = resources.session.pin_log;
    resources.session.pin_log = false;
    if !opened {
        if (bottom - previous).abs() < 0.5 {
            return;
        }
        if data.scroll_offset < previous - config.scroll_follow_slack {
            return;
        }
    }

    ui_scroll_area_set_offset(ink.build.commands, list, bottom);
}
