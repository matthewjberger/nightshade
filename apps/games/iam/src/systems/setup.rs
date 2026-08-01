use crate::components::Listing;
use crate::components::{
    AccountOpened, AccountRow, BoardCard, BoardColumn, BookmarkRow, ChoiceSlot, FileRow, FileSlot,
    FrameButton, FrameButtonKind, FrameSlot, GamePlace, LabeledSlot, LampRow, LinkRow, NoteCard,
    PageBlock, PairingRow, PairingSlot, ProgramIcon, ProgramWindow, RowSlot, SelectedAccount,
    SessionRow, StartRow, TallyRow, TaskbarButton,
};
use crate::components::{
    CaseRow, ChecklistRow, EndingRow, NoticeRow, StoreButton, StoreRow, ThingRow, ToastRow, WayRow,
};
use crate::components::{
    Channel, ChannelAsked, ChannelFeed, ChannelUnread, Contact, ContactDeactivated, ContactInvite,
    ContactListing, ContactPresence, ContactUnread, Conversation, Server, ServerListed,
    ServerOpenChannel, Speaker, ThreadView, Transcript, TranscriptEntry,
};
use crate::data::board::LEADS;
use crate::data::config::{Config, row_inner_width, sidebar_list_bounds};
use crate::data::cross::PAIRINGS;
use crate::data::files::LEFTOVERS;
use crate::data::lights::{BOARDS, BYLINE};
use crate::data::machine::ACCOUNTS;
use crate::data::notes::NOTES;
use crate::data::programs::PROGRAMS;
use crate::data::script::{Presence, Script, contact_index, validate};
use crate::data::web::{BOOKMARKS, FOLDER};
use crate::resources::{
    AppResources, Ink, Interface, SidebarGroup, channel_row_capacity, choice_capacity,
    group_capacity, pending_capacity, server_capacity,
};
use crate::systems::conversation::{begin_night, entity_for_contact};
use nightshade::prelude::Entity;
use nightshade::prelude::*;
use nightshade_platform::CursorIcon;
use nightshade_render::prelude::{AntiAliasing, RenderSettings};
use nightshade_text::contract::data::text_data::{TextAlignment, VerticalAlignment};
use nightshade_ui::commands::builder::UiBuild;
use nightshade_ui::data::icons::{IconGlyph, material};
use nightshade_ui::prelude::*;

const PRESENCE_GROUPS: [Presence; 3] = [Presence::Online, Presence::Idle, Presence::Offline];

pub fn dress_the_runtime(
    config: Res<Config>,
    mut retained: ResMut<RetainedUiRuntime>,
    mut render: ResMut<RenderSettings>,
) {
    retained.visible = true;
    retained.background_color = Some(config.window_background);
    render.anti_aliasing = AntiAliasing::Off;
}

pub fn initialize(
    mut resources: ResMut<AppResources>,
    settings: Res<Config>,
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
    validate(resources.script);
    resources.session.handle = resources.script.player_handle;

    let config = *settings;

    spawn_contacts(resources, ink);
    build_desktop(resources, ink, &config);
    build_frames(ink, &config);
    build_window(resources, ink, &config);
    build_board(resources, ink, &config);
    build_sessions(resources, ink, &config);
    build_cross(resources, ink, &config);
    build_store(resources, ink, &config);
    build_botterfly(resources, ink, &config);
    build_files(resources, ink, &config);
    build_browser(resources, ink, &config);
    build_lights(
        resources,
        ink,
        &config,
        "lights",
        crate::data::lights::Goal::Out,
    );
    build_lights(
        resources,
        ink,
        &config,
        "lights_on",
        crate::data::lights::Goal::On,
    );
    build_case(resources, ink, &config);
    build_office(resources, ink, &config);
    build_terminated(resources, ink, &config);
    build_over(resources, ink, &config);
    build_boot(resources, ink, &config);
    build_waiting(resources, ink, &config);
    build_start_menu(resources, ink, &config);
    build_notes(resources, ink, &config);
    build_login(resources, ink, &config);
    build_pointer(resources, ink, &config);
    build_thread_columns(resources, ink);
    spawn_servers(resources, ink);
    build_typing_bubble(resources, ink, &config);
    build_game_panel(resources, ink, &config);
    resources.desk.place =
        crate::data::office::place_at(crate::data::office::START).unwrap_or_default();
    let roots = ink.gathered.roots.clone();
    set_scale(&mut ink.build, &roots, config.interface_scale);
    begin_night(resources, ink, &config);
    open_first_thread(resources, ink);
    crate::systems::conversation::write_threads(&ink.gathered.threads, ink.build.commands);
}

fn set_scale(context: &mut UiBuildContext<'_, '_>, roots: &[Entity], scale: f32) {
    for root in roots {
        context.commands.set(
            *root,
            UiLayoutRoot {
                absolute_scale: scale,
                ..UiLayoutRoot::default()
            },
        );
    }
    ui_mark_layout_dirty(context.dirty);
}

fn spawn_contacts(resources: &AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>) {
    for (definition_index, definition) in resources.script.contacts.iter().enumerate() {
        let entity = ink.build.commands.spawn((
            Contact {
                definition: definition_index,
            },
            ContactPresence(definition.presence),
            ContactListing(if definition.listed {
                Listing::Listed
            } else {
                Listing::Hidden
            }),
            ContactInvite(""),
            ContactDeactivated(false),
            ContactUnread(0),
            Transcript {
                entries: definition
                    .history
                    .iter()
                    .map(|line| TranscriptEntry {
                        speaker: if line.mine {
                            Speaker::Player
                        } else {
                            Speaker::Contact
                        },
                        stamp: line.stamp.to_string(),
                        text: line.text.to_string(),
                    })
                    .collect(),
            },
            Conversation::default(),
            ThreadView::default(),
        ));
        ink.gathered
            .threads
            .contacts
            .push(crate::systems::conversation::ContactState {
                entity,
                definition: definition_index,
                presence: definition.presence,
                listing: if definition.listed {
                    Listing::Listed
                } else {
                    Listing::Hidden
                },
                transcript: Transcript {
                    entries: definition
                        .history
                        .iter()
                        .map(|line| TranscriptEntry {
                            speaker: if line.mine {
                                Speaker::Player
                            } else {
                                Speaker::Contact
                            },
                            stamp: line.stamp.to_string(),
                            text: line.text.to_string(),
                        })
                        .collect(),
                },
                ..Default::default()
            });
    }
}

fn build_thread_columns(resources: &mut AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>) {
    let content = resources.interface.log_content;
    for definition_index in 0..resources.script.contacts.len() {
        let Some(entity) = entity_for_contact(ink, definition_index) else {
            continue;
        };
        let column = build_log_column(ink, content);
        let view = ThreadView {
            column,
            drawn: 0,
            last_speaker: None,
        };
        ink.build.commands.set(entity, view.clone());
        if let Some(state) =
            crate::systems::conversation::contact_mut(&mut ink.gathered.threads, entity)
        {
            state.view = view;
        }
    }
}

fn spawn_servers(resources: &mut AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>) {
    let content = resources.interface.log_content;
    for (definition_index, definition) in resources.script.servers.iter().enumerate() {
        let server = ink.build.commands.spawn((
            Server {
                definition: definition_index,
            },
            ServerListed(definition.listed),
            ServerOpenChannel::default(),
        ));
        ink.gathered
            .threads
            .servers
            .push(crate::systems::conversation::ServerState {
                entity: server,
                definition: definition_index,
                listed: definition.listed,
                open_channel: None,
            });
        let mut first: Option<Entity> = None;
        for channel_index in 0..definition.channels.len() {
            let column = build_log_column(ink, content);
            let parts = Channel {
                server,
                server_definition: definition_index,
                definition: channel_index,
            };
            let channel = ink.build.commands.spawn((
                parts.clone(),
                ChannelFeed {
                    column,
                    log: Vec::new(),
                    held: Vec::new(),
                    queue: Vec::new(),
                    queued_effects: &[],
                    delivered: 0,
                    drawn: 0,
                    timer: definition.channels[channel_index]
                        .posts
                        .first()
                        .map(|post| post.delay_seconds)
                        .unwrap_or_default(),
                    last_author: None,
                },
                ChannelUnread(0),
                ChannelAsked::default(),
            ));
            ink.gathered
                .threads
                .channels
                .push(crate::systems::conversation::ChannelState {
                    entity: channel,
                    channel: parts,
                    feed: ChannelFeed {
                        column,
                        timer: definition.channels[channel_index]
                            .posts
                            .first()
                            .map(|post| post.delay_seconds)
                            .unwrap_or_default(),
                        ..Default::default()
                    },
                    unread: 0,
                    asked: Vec::new(),
                });
            first.get_or_insert(channel);
        }
        ink.build.commands.set(server, ServerOpenChannel(first));
        if let Some(state) =
            crate::systems::conversation::server_mut(&mut ink.gathered.threads, server)
        {
            state.open_channel = first;
        }
    }
}

fn build_typing_bubble(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let interface = &mut resources.interface;
    let mut tree = UiTreeBuilder::from_parent(ink.build.reborrow(), interface.log_content);
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
        .with_visible(false)
        .entity();
    interface.typing_bubble = row;
    interface.typing_dots = tree.in_parent(row, |tree| {
        let bubble = tree
            .add_node()
            .flow_child(Ab(vec2(52.0, 26.0)))
            .flow(FlowDirection::Vertical, config.bubble_padding, 0.0)
            .with_rect(12.0, 0.0, config.transparent)
            .color_raw::<UiBase>(config.their_bubble)
            .entity();
        tree.in_parent(bubble, |tree| {
            tree.add_node()
                .absolute_fill(0.0)
                .with_text("", config.message_font_size)
                .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                .color_raw::<UiBase>(config.text_dim)
                .entity()
        })
    });
    tree.finish_subtree();
}

fn build_log_column(ink: &mut Ink<'_, '_, '_, '_, '_, '_>, content: Entity) -> Entity {
    let mut tree = UiTreeBuilder::from_parent(ink.build.reborrow(), content);
    let column = tree
        .add_node()
        .flow_child(Rl(vec2(100.0, 0.0)))
        .flow(FlowDirection::Vertical, 0.0, 7.0)
        .with_visible(false)
        .entity();
    tree.finish_subtree();
    column
}

fn open_first_thread(resources: &mut AppResources, ink: &mut Ink<'_, '_, '_, '_, '_, '_>) {
    let Some(first) = contact_index(resources.script, "iambot") else {
        return;
    };
    let Some(entity) = entity_for_contact(ink, first) else {
        return;
    };
    crate::systems::conversation::open_contact(resources, ink, entity);
}

fn build_window(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let mut replies: Vec<(Entity, ChoiceSlot)> = Vec::new();
    let content = frame_content(&ink.gathered.frames, program_index("iam"));
    let mut tree = UiTreeBuilder::from_parent(ink.build.reborrow(), content);

    let shell = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.window_background)
        .with_visible(false)
        .entity();
    resources.interface.shell = shell;

    resources.interface.occupied = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.window_background)
        .with_effect(RectEffect::VerticalGradient {
            secondary: config.login_shade,
        })
        .with_layer(UiLayer::Popups)
        .with_visible(false)
        .entity();
    let occupied = resources.interface.occupied;
    tree.in_parent(occupied, |tree| {
        let panel = tree
            .add_node()
            .window(Rl(vec2(50.0, 42.0)), Ab(vec2(520.0, 208.0)), Anchor::Center)
            .with_rect(7.0, 1.0, config.divider)
            .color_raw::<UiBase>(config.pane_background)
            .with_effect(RectEffect::VerticalGradient {
                secondary: config.pane_shade,
            })
            .entity();
        tree.in_parent(panel, |tree| {
            tree.add_node()
                .window(Ab(vec2(28.0, 26.0)), Ab(vec2(460.0, 20.0)), Anchor::TopLeft)
                .with_text("iam", 17.1)
                .text_left()
                .color_raw::<UiBase>(config.brand)
                .entity();
            resources.interface.occupied_handle = tree
                .add_node()
                .window(Ab(vec2(28.0, 54.0)), Ab(vec2(460.0, 20.0)), Anchor::TopLeft)
                .with_text("", 14.6)
                .text_left()
                .color_raw::<UiBase>(config.text)
                .entity();
            resources.interface.occupied_body = tree
                .add_node()
                .window(
                    Ab(vec2(28.0, 84.0)),
                    Ab(vec2(464.0, 108.0)),
                    Anchor::TopLeft,
                )
                .with_text("", 13.4)
                .with_text_wrap()
                .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
                .color_raw::<UiBase>(config.text_dim)
                .entity();
        });
    });

    tree.in_parent(shell, |tree| {
        build_marquee(&mut resources.interface, config, tree);

        let band = config.marquee_height;
        let sidebar = tree
            .add_node()
            .boundary(
                Ab(vec2(0.0, band)),
                Ab(vec2(config.sidebar_width, 0.0)) + Rl(vec2(0.0, 100.0)),
            )
            .with_rect(0.0, 0.0, config.transparent)
            .color_raw::<UiBase>(config.sidebar_background)
            .with_effect(RectEffect::VerticalGradient {
                secondary: config.sidebar_shade,
            })
            .entity();
        tree.in_parent(sidebar, |tree| {
            build_sidebar(&mut resources.interface, resources.script, config, tree);
        });

        let pane = tree
            .add_node()
            .boundary(Ab(vec2(config.sidebar_width, band)), Rl(vec2(100.0, 100.0)))
            .with_rect(0.0, 0.0, config.transparent)
            .color_raw::<UiBase>(config.pane_background)
            .with_effect(RectEffect::VerticalGradient {
                secondary: config.pane_shade,
            })
            .entity();
        replies = tree.in_parent(pane, |tree| {
            build_pane(&mut resources.interface, resources.script, config, tree)
        });
    });

    tree.finish_subtree();

    for (root, slot) in replies {
        ink.build.commands.set(root, slot);
    }
}

pub fn program_index(id: &str) -> usize {
    debug_assert!(
        crate::data::programs::index_of(id).is_some(),
        "nothing on the desktop is called {id}",
    );
    crate::data::programs::index_of(id).unwrap_or_default()
}

fn build_game_panel(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let mut places: Vec<(Entity, GamePlace)> = Vec::new();
    let content = frame_content(&ink.gathered.frames, program_index("iam"));
    let mut tree = UiTreeBuilder::from_parent(ink.build.reborrow(), content);
    let panel = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 50.0)) + Ab(vec2(-260.0, -180.0)),
            Ab(vec2(520.0, 360.0)),
            Anchor::TopLeft,
        )
        .with_rect(10.0, 1.0, config.divider)
        .color_raw::<UiBase>(config.pane_background)
        .with_effect(RectEffect::VerticalGradient {
            secondary: config.pane_shade,
        })
        .with_visible(false)
        .entity();
    resources.interface.game_panel = panel;

    tree.in_parent(panel, |tree| {
        resources.interface.game_title = tree
            .add_node()
            .window(Ab(vec2(22.0, 20.0)), Ab(vec2(400.0, 20.0)), Anchor::TopLeft)
            .with_text("", 17.1)
            .text_left()
            .color_raw::<UiBase>(config.text)
            .entity();
        resources.interface.game_blurb = tree
            .add_node()
            .window(Ab(vec2(22.0, 42.0)), Ab(vec2(476.0, 32.0)), Anchor::TopLeft)
            .with_text("", 12.8)
            .with_text_wrap()
            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
            .color_raw::<UiBase>(config.text_faint)
            .entity();

        let row = tree
            .add_node()
            .window(
                Ab(vec2(22.0, 130.0)),
                Ab(vec2(476.0, 62.0)),
                Anchor::TopLeft,
            )
            .flow_with_alignment(
                FlowDirection::Horizontal,
                0.0,
                6.0,
                FlowAlignment::Center,
                FlowAlignment::Center,
            )
            .entity();
        tree.in_parent(row, |tree| {
            for place in 0..BOARD_PLACES {
                let mut slot = LabeledSlot {
                    root: tree
                        .add_node()
                        .flow_child(Ab(vec2(config.board_place, 54.0)))
                        .with_rect(6.0, 1.0, config.divider)
                        .color_raw::<UiBase>(config.board_dark)
                        .color_raw::<UiHover>(config.row_hover)
                        .with_transition::<UiHover>(18.0, 9.0)
                        .with_interaction()
                        .with_cursor_icon(CursorIcon::Pointer)
                        .with_visible(false)
                        .entity(),
                    ..LabeledSlot::default()
                };
                let root = slot.root;
                slot.label = tree.in_parent(root, |tree| {
                    tree.add_node()
                        .absolute_fill(0.0)
                        .with_text("", 20.7)
                        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                        .color_raw::<UiBase>(config.text)
                        .entity()
                });
                places.push((
                    root,
                    GamePlace {
                        place,
                        face: ' ',
                        label: slot.label,
                    },
                ));
            }
        });

        resources.interface.game_prize = tree
            .add_node()
            .window(
                Ab(vec2(22.0, 224.0)),
                Ab(vec2(476.0, 46.0)),
                Anchor::TopLeft,
            )
            .with_text("", 13.4)
            .with_text_wrap()
            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
            .color_raw::<UiBase>(config.online)
            .entity();

        resources.interface.game_close = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 100.0)) + Ab(vec2(-58.0, -52.0)),
                Ab(vec2(116.0, 30.0)),
                Anchor::TopLeft,
            )
            .with_rect(5.0, 1.0, config.divider)
            .color_raw::<UiBase>(config.composer_background)
            .color_raw::<UiHover>(config.row_hover)
            .with_transition::<UiHover>(16.0, 8.0)
            .with_interaction()
            .with_cursor_icon(CursorIcon::Pointer)
            .entity();
        let close = resources.interface.game_close;
        tree.in_parent(close, |tree| {
            tree.add_node()
                .absolute_fill(0.0)
                .with_text("put it away", 13.4)
                .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                .color_raw::<UiBase>(config.text_dim)
                .entity();
        });
    });
    tree.finish_subtree();

    for (root, place) in places {
        ink.build.commands.set(root, place);
    }
}

const BOARD_PLACES: usize = 8;

fn build_frames(ink: &mut Ink<'_, '_, '_, '_, '_, '_>, config: &Config) {
    for (index, program) in PROGRAMS.iter().enumerate() {
        let mut tree = UiTreeBuilder::new(ink.build.reborrow());
        let tree_root = tree.root_entity();
        let mut slot = FrameSlot {
            root: tree
                .add_node()
                .window(
                    Ab(vec2(120.0, 80.0)),
                    Ab(vec2(980.0, 620.0)),
                    Anchor::TopLeft,
                )
                .with_rect(8.0, 1.0, config.divider)
                .color_raw::<UiBase>(config.pane_background)
                .with_layer(UiLayer::FloatingPanels)
                .with_visible(false)
                .entity(),
            ..FrameSlot::default()
        };
        let root = slot.root;
        tree.in_parent(root, |tree| {
            slot.bar = tree
                .add_node()
                .window(
                    Ab(vec2(0.0, 0.0)),
                    Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, config.frame_bar_height)),
                    Anchor::TopLeft,
                )
                .with_rect(8.0, 0.0, config.transparent)
                .color_raw::<UiBase>(config.frame_bar)
                .with_effect(RectEffect::VerticalGradient {
                    secondary: config.pane_shade,
                })
                .entity();
            let bar = slot.bar;
            tree.in_parent(bar, |tree| {
                slot.title = tree
                    .add_node()
                    .window(
                        Ab(vec2(14.0, 0.0)),
                        Ab(vec2(360.0, config.frame_bar_height)),
                        Anchor::TopLeft,
                    )
                    .with_text(program.label, 14.0)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                    .color_raw::<UiBase>(config.text_dim)
                    .entity();
                slot.minimize =
                    build_frame_button(tree, config, -110.0, "\u{2212}", config.row_hover);
                slot.maximize =
                    build_frame_button(tree, config, -76.0, "\u{25a1}", config.row_hover);
                slot.close =
                    build_frame_button(tree, config, -38.0, "\u{2715}", config.close_hover);
            });

            slot.content = tree
                .add_node()
                .boundary(
                    Ab(vec2(0.0, config.frame_bar_height)),
                    Rl(vec2(100.0, 100.0)) + Ab(vec2(0.0, -config.frame_bar_height)),
                )
                .with_clip()
                .entity();

            slot.grip = tree
                .add_node()
                .window(
                    Rl(vec2(100.0, 100.0)) + Ab(vec2(-18.0, -18.0)),
                    Ab(vec2(18.0, 18.0)),
                    Anchor::TopLeft,
                )
                .with_rect(0.0, 0.0, config.transparent)
                .color_raw::<UiBase>(config.transparent)
                .color_raw::<UiHover>(config.row_hover)
                .with_interaction()
                .with_cursor_icon(CursorIcon::NwseResize)
                .entity();
        });
        tree.finish();

        ink.build.commands.set(
            slot.root,
            ProgramWindow {
                program: index,
                ..ProgramWindow::default()
            },
        );
        ink.build.commands.set(slot.root, slot);
        ink.gathered.frames.push((index, slot));
        ink.gathered.roots.push(tree_root);
        ink.build.commands.set(
            slot.minimize,
            FrameButton {
                program: index,
                kind: FrameButtonKind::Minimize,
            },
        );
        ink.build.commands.set(
            slot.maximize,
            FrameButton {
                program: index,
                kind: FrameButtonKind::Size,
            },
        );
        ink.build.commands.set(
            slot.close,
            FrameButton {
                program: index,
                kind: FrameButtonKind::Close,
            },
        );
    }
}

fn frame_content(frames: &[(usize, FrameSlot)], program: usize) -> Entity {
    frames
        .iter()
        .find(|(index, _)| *index == program)
        .map(|(_, slot)| slot.content)
        .unwrap_or_default()
}

fn build_frame_button(
    tree: &mut UiTreeBuilder,
    config: &Config,
    offset: f32,
    glyph: &str,
    hover: Vec4,
) -> Entity {
    let button = tree
        .add_node()
        .window(
            Rl(vec2(100.0, 0.0)) + Ab(vec2(offset, 5.0)),
            Ab(vec2(30.0, config.frame_bar_height - 10.0)),
            Anchor::TopLeft,
        )
        .with_rect(5.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.transparent)
        .color_raw::<UiHover>(hover)
        .color_raw::<UiPressed>(config.row_pressed)
        .with_transition::<UiHover>(20.0, 10.0)
        .with_interaction()
        .with_cursor_icon(CursorIcon::Pointer)
        .entity();
    tree.in_parent(button, |tree| {
        tree.add_node()
            .absolute_fill(0.0)
            .with_text(glyph, 13.4)
            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
            .color_raw::<UiBase>(config.text_dim)
            .color_raw::<UiHover>(config.text)
            .with_transition::<UiHover>(20.0, 10.0)
            .entity();
    });
    button
}

fn build_settings(resources: &mut AppResources, tree: &mut UiTreeBuilder, config: &Config) {
    resources.interface.settings_cog = tree
        .add_node()
        .window(
            Rl(vec2(100.0, 100.0)) + Ab(vec2(-56.0, -56.0)),
            Ab(vec2(38.0, 38.0)),
            Anchor::TopLeft,
        )
        .with_rect(19.0, 1.0, config.divider)
        .color_raw::<UiBase>(config.transparent)
        .color_raw::<UiHover>(config.row_hover)
        .with_transition::<UiHover>(16.0, 8.0)
        .with_interaction()
        .with_cursor_icon(CursorIcon::Pointer)
        .entity();
    let cog = resources.interface.settings_cog;
    tree.in_parent(cog, |tree| {
        tree.add_node()
            .absolute_fill(0.0)
            .with_icon(material::SETTINGS, 18.0)
            .color_raw::<UiBase>(config.text_faint)
            .entity();
    });

    let size = vec2(520.0, 372.0);
    let panel = tree
        .add_node()
        .window(
            Rl(vec2(100.0, 100.0)) + Ab(vec2(-size.x - 26.0, -size.y - 62.0)),
            Ab(size),
            Anchor::TopLeft,
        )
        .with_rect(9.0, 1.0, config.divider)
        .color_raw::<UiBase>(config.pane_background)
        .with_effect(RectEffect::VerticalGradient {
            secondary: config.pane_shade,
        })
        .with_visible(false)
        .entity();
    resources.interface.settings_panel = panel;

    tree.in_parent(panel, |tree| {
        tree.add_node()
            .window(Ab(vec2(22.0, 18.0)), Ab(vec2(400.0, 20.0)), Anchor::TopLeft)
            .with_text("SETTINGS", 12.2)
            .text_left()
            .color_raw::<UiBase>(config.text_faint)
            .entity();
        tree.add_node()
            .window(Ab(vec2(22.0, 42.0)), Ab(vec2(460.0, 20.0)), Anchor::TopLeft)
            .with_text("start autoplay session", 17.1)
            .text_left()
            .color_raw::<UiBase>(config.text)
            .entity();
        tree.add_node()
            .window(Ab(vec2(22.0, 64.0)), Ab(vec2(470.0, 18.0)), Anchor::TopLeft)
            .with_text("the week plays itself. pick where it stops.", 12.2)
            .text_left()
            .color_raw::<UiBase>(config.text_faint)
            .entity();
        tree.add_node()
            .window(
                Ab(vec2(22.0, 92.0)),
                Ab(vec2(size.x - 44.0, 1.0)),
                Anchor::TopLeft,
            )
            .with_rect(0.0, 0.0, config.transparent)
            .color_raw::<UiBase>(config.divider)
            .entity();

        for which in 0..crate::data::autoplay::OFFERED.len() {
            let row = tree
                .add_node()
                .window(
                    Ab(vec2(16.0, 104.0 + which as f32 * ENDING_ROW)),
                    Ab(vec2(size.x - 32.0, ENDING_ROW - 4.0)),
                    Anchor::TopLeft,
                )
                .with_rect(5.0, 0.0, config.transparent)
                .color_raw::<UiBase>(config.transparent)
                .color_raw::<UiHover>(config.row_hover)
                .with_transition::<UiHover>(16.0, 8.0)
                .with_interaction()
                .with_cursor_icon(CursorIcon::Pointer)
                .entity();
            let mut parts = EndingRow {
                which,
                root: row,
                ..Default::default()
            };
            tree.in_parent(row, |tree| {
                let ring = tree
                    .add_node()
                    .window(Ab(vec2(12.0, 13.0)), Ab(vec2(14.0, 14.0)), Anchor::TopLeft)
                    .with_rect(7.0, 1.0, config.text_faint)
                    .color_raw::<UiBase>(config.transparent)
                    .entity();
                parts.dot = tree.in_parent(ring, |tree| {
                    tree.add_node()
                        .window(Ab(vec2(3.0, 3.0)), Ab(vec2(8.0, 8.0)), Anchor::TopLeft)
                        .with_rect(4.0, 0.0, config.transparent)
                        .color_raw::<UiBase>(config.brand)
                        .with_visible(false)
                        .entity()
                });
                parts.label = tree
                    .add_node()
                    .window(Ab(vec2(38.0, 8.0)), Ab(vec2(420.0, 17.0)), Anchor::TopLeft)
                    .with_text("", 13.4)
                    .text_left()
                    .color_raw::<UiBase>(config.text)
                    .entity();
                parts.note = tree
                    .add_node()
                    .window(Ab(vec2(38.0, 25.0)), Ab(vec2(440.0, 15.0)), Anchor::TopLeft)
                    .with_text("", 11.2)
                    .text_left()
                    .with_text_overflow(TextOverflow::Ellipsis)
                    .color_raw::<UiBase>(config.text_faint)
                    .entity();
            });
            tree.context_reborrow().commands.set(row, parts);
        }

        resources.interface.settings_start = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 100.0)) + Ab(vec2(-186.0, -58.0)),
                Ab(vec2(164.0, 38.0)),
                Anchor::TopLeft,
            )
            .with_rect(6.0, 1.0, config.brand)
            .color_raw::<UiBase>(config.composer_background)
            .color_raw::<UiHover>(config.row_hover)
            .with_transition::<UiHover>(16.0, 8.0)
            .with_interaction()
            .with_cursor_icon(CursorIcon::Pointer)
            .entity();
        let start = resources.interface.settings_start;
        tree.in_parent(start, |tree| {
            tree.add_node()
                .absolute_fill(0.0)
                .with_text("START", 13.4)
                .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                .color_raw::<UiBase>(config.brand)
                .entity();
        });
    });
}

const ENDING_ROW: f32 = 46.0;

fn build_login(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let mut accounts: Vec<RowSlot> = Vec::new();
    let mut tree = UiTreeBuilder::new(ink.build.reborrow());
    ink.gathered.roots.push(tree.root_entity());
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.login_background)
        .with_effect(RectEffect::VerticalGradient {
            secondary: config.login_shade,
        })
        .with_layer(UiLayer::Popups)
        .entity();
    resources.interface.login = root;

    tree.in_parent(root, |tree| {
        build_settings(resources, tree, config);

        tree.add_node()
            .window(Ab(vec2(22.0, 16.0)), Ab(vec2(460.0, 16.0)), Anchor::TopLeft)
            .with_text("HOLLOWAY & SONS  Â·  WORKSTATION 14", 12.2)
            .text_left()
            .color_raw::<UiBase>(config.text_faint)
            .entity();
        tree.add_node()
            .window(Ab(vec2(22.0, 34.0)), Ab(vec2(460.0, 14.0)), Anchor::TopLeft)
            .with_text("this desk is reassigned automatically", 11.6)
            .text_left()
            .color_raw::<UiBase>(config.text_faint)
            .entity();

        let panel = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 50.0)) + Ab(vec2(-236.0, -216.0)),
                Ab(vec2(472.0, 432.0)),
                Anchor::TopLeft,
            )
            .with_rect(10.0, 1.0, config.divider)
            .color_raw::<UiBase>(config.pane_background)
            .with_effect(RectEffect::VerticalGradient {
                secondary: config.pane_shade,
            })
            .entity();

        tree.in_parent(panel, |tree| {
            tree.add_node()
                .window(Ab(vec2(24.0, 22.0)), Ab(vec2(424.0, 20.0)), Anchor::TopLeft)
                .with_text("choose an account", 16.5)
                .text_left()
                .color_raw::<UiBase>(config.text)
                .entity();

            let list = tree
                .add_node()
                .window(
                    Ab(vec2(14.0, 54.0)),
                    Ab(vec2(444.0, 248.0)),
                    Anchor::TopLeft,
                )
                .flow(FlowDirection::Vertical, 0.0, 4.0)
                .entity();
            tree.in_parent(list, |tree| {
                for _ in 0..ACCOUNTS.len() {
                    accounts.push(build_account_row(tree, config));
                }
            });

            tree.add_node()
                .window(
                    Ab(vec2(24.0, 314.0)),
                    Ab(vec2(424.0, 14.0)),
                    Anchor::TopLeft,
                )
                .with_text("PASSWORD", 11.0)
                .text_left()
                .color_raw::<UiBase>(config.text_faint)
                .entity();

            let field = tree
                .add_node()
                .window(
                    Ab(vec2(22.0, 332.0)),
                    Ab(vec2(330.0, 36.0)),
                    Anchor::TopLeft,
                )
                .with_rect(6.0, 1.0, config.brand_dim)
                .color_raw::<UiBase>(config.composer_background)
                .color_raw::<UiHover>(config.field_hover)
                .with_transition::<UiHover>(18.0, 9.0)
                .with_interaction()
                .with_cursor_icon(CursorIcon::Text)
                .entity();
            resources.interface.password_box = field;
            tree.in_parent(field, |tree| {
                resources.interface.password_field = tree
                    .add_node()
                    .window(Ab(vec2(14.0, 0.0)), Ab(vec2(302.0, 36.0)), Anchor::TopLeft)
                    .with_text("", 18.3)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                    .color_raw::<UiBase>(config.text)
                    .entity();
            });

            resources.interface.sign_in = tree
                .add_node()
                .window(
                    Ab(vec2(360.0, 332.0)),
                    Ab(vec2(90.0, 36.0)),
                    Anchor::TopLeft,
                )
                .with_rect(6.0, 1.0, config.brand_dim)
                .color_raw::<UiBase>(config.brand_dim)
                .color_raw::<UiHover>(config.icon_selected)
                .color_raw::<UiPressed>(config.row_pressed)
                .with_transition::<UiHover>(18.0, 9.0)
                .with_effect(RectEffect::VerticalGradient {
                    secondary: config.avatar_shade,
                })
                .with_interaction()
                .with_cursor_icon(CursorIcon::Pointer)
                .entity();
            let sign_in = resources.interface.sign_in;
            tree.in_parent(sign_in, |tree| {
                tree.add_node()
                    .absolute_fill(0.0)
                    .with_text("sign in", 13.4)
                    .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                    .color_raw::<UiBase>(config.text)
                    .entity();
            });

            resources.interface.refusal = tree
                .add_node()
                .window(
                    Ab(vec2(24.0, 376.0)),
                    Ab(vec2(424.0, 16.0)),
                    Anchor::TopLeft,
                )
                .with_text("", 12.8)
                .text_left()
                .color_raw::<UiBase>(config.accent_warn)
                .entity();

            tree.add_node()
                .window(
                    Ab(vec2(24.0, 398.0)),
                    Ab(vec2(424.0, 14.0)),
                    Anchor::TopLeft,
                )
                .with_text("type the password and press enter", 11.6)
                .text_left()
                .color_raw::<UiBase>(config.text_faint)
                .entity();
        });

        let note = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 50.0)) + Ab(vec2(248.0, -196.0)),
                Ab(vec2(168.0, 124.0)),
                Anchor::TopLeft,
            )
            .with_rect(1.0, 0.0, config.transparent)
            .color_raw::<UiBase>(config.sticky)
            .with_effect(RectEffect::VerticalGradient {
                secondary: config.sticky_shade,
            })
            .entity();
        resources.interface.sticky_note = tree.in_parent(note, |tree| {
            tree.add_node()
                .window(Ab(vec2(14.0, 16.0)), Ab(vec2(140.0, 96.0)), Anchor::TopLeft)
                .with_text("", 14.0)
                .with_text_wrap()
                .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
                .color_raw::<UiBase>(config.sticky_text)
                .entity()
        });

        resources.interface.quit_button = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 100.0)) + Ab(vec2(-60.0, -46.0)),
                Ab(vec2(120.0, 26.0)),
                Anchor::TopLeft,
            )
            .with_rect(5.0, 0.0, config.transparent)
            .color_raw::<UiBase>(config.transparent)
            .color_raw::<UiHover>(config.row_hover)
            .with_transition::<UiHover>(16.0, 8.0)
            .with_interaction()
            .with_cursor_icon(CursorIcon::Pointer)
            .entity();
        let quit_button = resources.interface.quit_button;
        tree.in_parent(quit_button, |tree| {
            tree.add_node()
                .absolute_fill(0.0)
                .with_text("shut down", 12.8)
                .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                .color_raw::<UiBase>(config.text_faint)
                .entity();
        });
    });
    tree.finish();

    let guest = crate::data::machine::index_of("guest");
    for (definition, slot) in accounts.into_iter().enumerate() {
        ink.build
            .commands
            .set(slot.root, AccountRow { definition, slot });
        ink.build
            .commands
            .set(slot.root, AccountOpened(definition == 0));
        if definition == guest {
            ink.build
                .commands
                .add_tag_type::<SelectedAccount>(slot.root);
        }
    }
}

fn build_account_row(tree: &mut UiTreeBuilder, config: &Config) -> RowSlot {
    let mut slot = RowSlot {
        root: tree
            .add_node()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 58.0)))
            .with_rect(7.0, 0.0, config.transparent)
            .color_raw::<UiBase>(config.transparent)
            .color_raw::<UiHover>(config.row_hover)
            .color_raw::<UiPressed>(config.row_pressed)
            .with_transition::<UiHover>(18.0, 9.0)
            .with_interaction()
            .with_cursor_icon(CursorIcon::Pointer)
            .entity(),
        ..RowSlot::default()
    };

    let root = slot.root;
    tree.in_parent(root, |tree| {
        slot.avatar = tree
            .add_node()
            .window(Ab(vec2(12.0, 9.0)), Ab(vec2(40.0, 40.0)), Anchor::TopLeft)
            .with_rect(20.0, 0.0, config.transparent)
            .color_raw::<UiBase>(config.brand_dim)
            .with_effect(RectEffect::VerticalGradient {
                secondary: config.avatar_shade,
            })
            .entity();
        let avatar = slot.avatar;
        slot.letter = tree.in_parent(avatar, |tree| {
            tree.add_node()
                .absolute_fill(0.0)
                .with_text("", 19.5)
                .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                .color_raw::<UiBase>(config.text)
                .entity()
        });
        slot.name = tree
            .add_node()
            .window(Ab(vec2(64.0, 12.0)), Ab(vec2(300.0, 18.0)), Anchor::TopLeft)
            .with_text("", 17.1)
            .text_left()
            .color_raw::<UiBase>(config.text)
            .entity();
        slot.status = tree
            .add_node()
            .window(Ab(vec2(64.0, 31.0)), Ab(vec2(300.0, 15.0)), Anchor::TopLeft)
            .with_text("", 12.8)
            .text_left()
            .color_raw::<UiBase>(config.text_faint)
            .entity();
        slot.badge = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-74.0, 20.0)),
                Ab(vec2(58.0, 18.0)),
                Anchor::TopLeft,
            )
            .with_rect(4.0, 0.0, config.transparent)
            .color_raw::<UiBase>(config.transparent)
            .entity();
        let badge = slot.badge;
        slot.badge_text = tree.in_parent(badge, |tree| {
            tree.add_node()
                .absolute_fill(0.0)
                .with_text("", 11.6)
                .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                .color_raw::<UiBase>(config.text_faint)
                .entity()
        });
    });
    slot
}

fn build_notices(resources: &mut AppResources, tree: &mut UiTreeBuilder, config: &Config) {
    let size = config.notices_size;
    let panel = tree
        .add_node()
        .window(
            Rl(vec2(100.0, 100.0))
                + Ab(vec2(-size.x - 12.0, -config.taskbar_height - size.y - 10.0)),
            Ab(size),
            Anchor::TopLeft,
        )
        .with_rect(9.0, 1.0, config.divider)
        .color_raw::<UiBase>(config.chrome)
        .with_effect(RectEffect::VerticalGradient {
            secondary: config.chrome_shade,
        })
        .with_layer(UiLayer::Popups)
        .with_visible(false)
        .entity();
    resources.interface.notices_panel = panel;

    tree.in_parent(panel, |tree| {
        tree.add_node()
            .window(Ab(vec2(18.0, 14.0)), Ab(vec2(200.0, 16.0)), Anchor::TopLeft)
            .with_text("NOTICES", 11.0)
            .text_left()
            .color_raw::<UiBase>(config.text_faint)
            .entity();
        resources.interface.notices_count = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-178.0, 14.0)),
                Ab(vec2(160.0, 16.0)),
                Anchor::TopLeft,
            )
            .with_text("", 11.0)
            .with_text_alignment(TextAlignment::Right, VerticalAlignment::Middle)
            .color_raw::<UiBase>(config.text_faint)
            .entity();
        tree.add_node()
            .window(
                Ab(vec2(14.0, 34.0)),
                Ab(vec2(size.x - 28.0, 1.0)),
                Anchor::TopLeft,
            )
            .with_rect(0.0, 0.0, config.transparent)
            .color_raw::<UiBase>(config.divider)
            .entity();

        for slot in 0..NOTICES_ROWS {
            let row = tree
                .add_node()
                .window(
                    Ab(vec2(12.0, NOTICES_TOP + slot as f32 * NOTICE_ROW)),
                    Ab(vec2(size.x - 24.0, NOTICE_ROW - 6.0)),
                    Anchor::TopLeft,
                )
                .with_rect(6.0, 0.0, config.transparent)
                .color_raw::<UiBase>(config.pane_background)
                .with_visible(false)
                .entity();
            let (stamp, who, body) = tree.in_parent(row, |tree| {
                let stamp = tree
                    .add_node()
                    .window(
                        Rl(vec2(100.0, 0.0)) + Ab(vec2(-150.0, 9.0)),
                        Ab(vec2(138.0, 14.0)),
                        Anchor::TopLeft,
                    )
                    .with_text("", 10.4)
                    .with_text_alignment(TextAlignment::Right, VerticalAlignment::Middle)
                    .color_raw::<UiBase>(config.text_faint)
                    .entity();
                let who = tree
                    .add_node()
                    .window(Ab(vec2(13.0, 9.0)), Ab(vec2(210.0, 15.0)), Anchor::TopLeft)
                    .with_text("", 12.2)
                    .text_left()
                    .with_text_overflow(TextOverflow::Ellipsis)
                    .color_raw::<UiBase>(config.brand)
                    .entity();
                let body = tree
                    .add_node()
                    .window(
                        Ab(vec2(13.0, 26.0)),
                        Ab(vec2(size.x - 50.0, 24.0)),
                        Anchor::TopLeft,
                    )
                    .with_text("", 11.6)
                    .with_text_wrap()
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
                    .color_raw::<UiBase>(config.text_dim)
                    .entity();
                (stamp, who, body)
            });
            tree.context_reborrow().commands.set(
                row,
                NoticeRow {
                    slot,
                    root: row,
                    stamp,
                    who,
                    body,
                },
            );
        }

        resources.interface.notices_more = tree
            .add_node()
            .window(
                Rl(vec2(0.0, 100.0)) + Ab(vec2(14.0, -26.0)),
                Ab(vec2(size.x - 28.0, 16.0)),
                Anchor::TopLeft,
            )
            .with_text("", 10.6)
            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
            .color_raw::<UiBase>(config.text_faint)
            .with_visible(false)
            .entity();
    });
}

const STORE_ROWS: usize = crate::data::store::MOST_LISTINGS;

pub const NOTICES_ROWS: usize = 7;
const NOTICES_TOP: f32 = 44.0;
const NOTICE_ROW: f32 = 58.0;

fn build_toasts(tree: &mut UiTreeBuilder, config: &Config) {
    for slot in 0..TOASTS {
        let bottom =
            -config.taskbar_height - 12.0 - (slot as f32 + 1.0) * (config.toast_size.y + 8.0);
        let root = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 100.0)) + Ab(vec2(-config.toast_size.x - 18.0, bottom)),
                Ab(config.toast_size),
                Anchor::TopLeft,
            )
            .with_rect(6.0, 1.0, config.brand)
            .color_raw::<UiBase>(config.pane_background)
            .color_raw::<UiHover>(config.row_hover)
            .with_transition::<UiHover>(18.0, 9.0)
            .with_effect(RectEffect::VerticalGradient {
                secondary: config.pane_shade,
            })
            .with_layer(UiLayer::Popups)
            .with_interaction()
            .with_cursor_icon(CursorIcon::Pointer)
            .with_visible(false)
            .entity();
        let (who, body) = tree.in_parent(root, |tree| {
            let who = tree
                .add_node()
                .window(Ab(vec2(14.0, 9.0)), Ab(vec2(288.0, 16.0)), Anchor::TopLeft)
                .with_text("", 12.8)
                .text_left()
                .color_raw::<UiBase>(config.brand)
                .entity();
            let body = tree
                .add_node()
                .window(Ab(vec2(14.0, 27.0)), Ab(vec2(288.0, 28.0)), Anchor::TopLeft)
                .with_text("", 12.2)
                .with_text_wrap()
                .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
                .color_raw::<UiBase>(config.text_dim)
                .entity();
            (who, body)
        });
        tree.context_reborrow().commands.set(
            root,
            ToastRow {
                slot,
                root,
                who,
                body,
            },
        );
    }
}

const TOASTS: usize = 3;

fn build_tonight(resources: &mut AppResources, tree: &mut UiTreeBuilder, config: &Config) {
    let note = tree
        .add_node()
        .window(
            Rl(vec2(100.0, 0.0)) + Ab(vec2(-292.0, 88.0)),
            Ab(vec2(268.0, 176.0)),
            Anchor::TopLeft,
        )
        .with_rect(1.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.sticky)
        .with_effect(RectEffect::VerticalGradient {
            secondary: config.sticky_shade,
        })
        .with_visible(false)
        .entity();
    resources.interface.tonight = note;

    tree.in_parent(note, |tree| {
        resources.interface.tonight_heading = tree
            .add_node()
            .window(Ab(vec2(18.0, 12.0)), Ab(vec2(232.0, 20.0)), Anchor::TopLeft)
            .with_text("", 15.9)
            .text_left()
            .color_raw::<UiBase>(config.sticky_author)
            .entity();
        tree.add_node()
            .window(Ab(vec2(18.0, 35.0)), Ab(vec2(232.0, 1.0)), Anchor::TopLeft)
            .with_rect(0.0, 0.0, config.transparent)
            .color_raw::<UiBase>(config.sticky_rule)
            .entity();

        for row in 0..crate::data::tonight::ROWS {
            let top =
                crate::data::tonight::FIRST_ROW + row as f32 * crate::data::tonight::ROW_PITCH;
            let line = tree
                .add_node()
                .window(Ab(vec2(18.0, top)), Ab(vec2(232.0, 29.0)), Anchor::TopLeft)
                .with_rect(0.0, 0.0, config.transparent)
                .color_raw::<UiBase>(config.transparent)
                .with_visible(false)
                .entity();
            let (tick, label) = tree.in_parent(line, |tree| {
                let square = tree
                    .add_node()
                    .window(Ab(vec2(0.0, 3.0)), Ab(vec2(16.0, 16.0)), Anchor::TopLeft)
                    .with_rect(2.0, 2.0, config.sticky_text)
                    .color_raw::<UiBase>(config.transparent)
                    .entity();
                let tick = tree.in_parent(square, |tree| {
                    tree.add_node()
                        .absolute_fill(0.0)
                        .with_text("", 15.0)
                        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                        .color_raw::<UiBase>(config.sticky_text)
                        .entity()
                });
                let label = tree
                    .add_node()
                    .window(Ab(vec2(26.0, 1.0)), Ab(vec2(206.0, 22.0)), Anchor::TopLeft)
                    .with_text("", 14.6)
                    .with_text_wrap()
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
                    .color_raw::<UiBase>(config.sticky_text)
                    .entity();
                tree.add_node()
                    .window(Ab(vec2(0.0, 27.0)), Ab(vec2(232.0, 1.0)), Anchor::TopLeft)
                    .with_rect(0.0, 0.0, config.transparent)
                    .color_raw::<UiBase>(config.sticky_rule)
                    .entity();
                (tick, label)
            });
            tree.context_reborrow().commands.set(
                line,
                ChecklistRow {
                    row,
                    root: line,
                    tick,
                    label,
                },
            );
        }

        resources.interface.tonight_done = tree
            .add_node()
            .window(Ab(vec2(18.0, 48.0)), Ab(vec2(232.0, 40.0)), Anchor::TopLeft)
            .with_text("", 14.6)
            .with_text_wrap()
            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
            .color_raw::<UiBase>(config.sticky_author)
            .with_visible(false)
            .entity();
    });
}

fn build_wallpaper(tree: &mut UiTreeBuilder, config: &Config) {
    for step in 1..8 {
        let down = step as f32 * 12.5;
        tree.add_node()
            .window(
                Rl(vec2(0.0, down)),
                Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 1.0)),
                Anchor::TopLeft,
            )
            .with_rect(0.0, 0.0, config.transparent)
            .color_raw::<UiBase>(config.wallpaper_grid)
            .entity();
    }
    for step in 1..10 {
        let across = step as f32 * 10.0;
        tree.add_node()
            .window(
                Rl(vec2(across, 0.0)),
                Rl(vec2(0.0, 100.0)) + Ab(vec2(1.0, 0.0)),
                Anchor::TopLeft,
            )
            .with_rect(0.0, 0.0, config.transparent)
            .color_raw::<UiBase>(config.wallpaper_grid)
            .entity();
    }

    tree.add_node()
        .window(Rl(vec2(0.0, 62.0)), Rl(vec2(100.0, 38.0)), Anchor::TopLeft)
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.wallpaper_horizon)
        .with_effect(RectEffect::VerticalGradient {
            secondary: config.desktop_background,
        })
        .entity();

    let mut halo = config.wallpaper_horizon;
    halo.w = 0.55;
    tree.add_node()
        .window(Rl(vec2(50.0, 46.0)), Ab(vec2(760.0, 760.0)), Anchor::Center)
        .with_rect(380.0, 0.0, config.transparent)
        .color_raw::<UiBase>(halo)
        .with_effect(RectEffect::RadialGlow {
            glow_color: halo,
            intensity: 1.0,
        })
        .entity();
    for ring in 0..9 {
        let size = 260.0 + ring as f32 * 150.0;
        let fade = 1.0 - ring as f32 / 9.0;
        let mut tint = config.wallpaper_ring;
        tint.w = 0.16 + 0.74 * fade * fade;
        tree.add_node()
            .window(Rl(vec2(50.0, 46.0)), Ab(vec2(size, size)), Anchor::Center)
            .with_rect(size * 0.5, 2.0, tint)
            .color_raw::<UiBase>(config.transparent)
            .entity();
    }

    for (across, down, run_x, run_y) in [
        (0.0f32, 0.0f32, 1.0f32, 1.0f32),
        (100.0, 0.0, -1.0, 1.0),
        (0.0, 100.0, 1.0, -1.0),
        (100.0, 100.0, -1.0, -1.0),
    ] {
        let corner = vec2(across, down);
        let reach = 46.0;
        let arm = vec2(run_x * reach, run_y * 2.0);
        let stem = vec2(run_x * 2.0, run_y * reach);
        tree.add_node()
            .window(
                Rl(corner) + Ab(vec2(run_x * 40.0, run_y * 40.0)),
                Ab(arm),
                Anchor::TopLeft,
            )
            .with_rect(0.0, 0.0, config.transparent)
            .color_raw::<UiBase>(config.wallpaper_grid)
            .entity();
        tree.add_node()
            .window(
                Rl(corner) + Ab(vec2(run_x * 40.0, run_y * 40.0)),
                Ab(stem),
                Anchor::TopLeft,
            )
            .with_rect(0.0, 0.0, config.transparent)
            .color_raw::<UiBase>(config.wallpaper_grid)
            .entity();
    }

    tree.add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.desktop_light)
        .with_effect(RectEffect::RadialGlow {
            glow_color: config.desktop_light,
            intensity: 0.9,
        })
        .entity();

    tree.add_node()
        .window(
            Rl(vec2(50.0, 46.0)) + Ab(vec2(5.0, 3.0)),
            Ab(vec2(900.0, 74.0)),
            Anchor::Center,
        )
        .with_text("HOLLOWAY & SONS", 58.0)
        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
        .color_raw::<UiBase>(config.wallpaper_ghost)
        .entity();
    tree.add_node()
        .window(Rl(vec2(50.0, 46.0)), Ab(vec2(900.0, 74.0)), Anchor::Center)
        .with_text("HOLLOWAY & SONS", 58.0)
        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
        .color_raw::<UiBase>(config.wallpaper_mark)
        .entity();
    tree.add_node()
        .window(
            Rl(vec2(50.0, 46.0)) + Ab(vec2(0.0, 40.0)),
            Ab(vec2(420.0, 1.0)),
            Anchor::Center,
        )
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.wallpaper_mark)
        .entity();
    tree.add_node()
        .window(
            Rl(vec2(50.0, 46.0)) + Ab(vec2(0.0, 58.0)),
            Ab(vec2(560.0, 18.0)),
            Anchor::Center,
        )
        .with_text("WORKSTATION  Â·  BUILD 3.07  Â·  DO NOT REMOVE", 14.0)
        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
        .color_raw::<UiBase>(config.wallpaper_mark)
        .entity();
}

fn build_desktop(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let mut buttons: Vec<(Entity, TaskbarButton)> = Vec::new();
    let mut icons: Vec<(Entity, usize)> = Vec::new();
    let mut tree = UiTreeBuilder::new(ink.build.reborrow());
    ink.gathered.roots.push(tree.root_entity());
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.desktop_background)
        .with_layer(UiLayer::Background)
        .with_visible(false)
        .entity();
    resources.interface.desktop = root;

    tree.in_parent(root, |tree| {
        build_wallpaper(tree, config);

        let plate = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-268.0, 22.0)),
                Ab(vec2(244.0, 52.0)),
                Anchor::TopLeft,
            )
            .with_rect(6.0, 1.0, config.divider)
            .color_raw::<UiBase>(config.pane_background)
            .with_effect(RectEffect::VerticalGradient {
                secondary: config.pane_shade,
            })
            .entity();
        tree.in_parent(plate, |tree| {
            tree.add_node()
                .window(Ab(vec2(14.0, 9.0)), Ab(vec2(216.0, 15.0)), Anchor::TopLeft)
                .with_text("HOLLOWAY & SONS", 12.2)
                .text_left()
                .color_raw::<UiBase>(config.text_dim)
                .entity();
            resources.interface.desktop_user = tree
                .add_node()
                .window(Ab(vec2(14.0, 27.0)), Ab(vec2(216.0, 16.0)), Anchor::TopLeft)
                .with_text("", 12.8)
                .text_left()
                .color_raw::<UiBase>(config.text_faint)
                .entity();
        });
        resources.interface.desktop_note = tree
            .add_node()
            .window(
                Rl(vec2(0.0, 100.0)) + Ab(vec2(22.0, -70.0)),
                Ab(vec2(560.0, 16.0)),
                Anchor::TopLeft,
            )
            .with_text("", 12.8)
            .text_left()
            .color_raw::<UiBase>(config.online)
            .entity();

        build_tonight(resources, tree, config);
        build_toasts(tree, config);
        build_notices(resources, tree, config);

        for (index, program) in PROGRAMS.iter().enumerate() {
            let at = vec2(
                18.0 + (index / ICON_COLUMN) as f32 * ICON_COLUMN_PITCH,
                22.0 + (index % ICON_COLUMN) as f32 * config.icon_pitch,
            );
            icons.push((
                build_icon(tree, config, at, program.glyph, program.label),
                index,
            ));
        }

        let taskbar = tree
            .add_node()
            .window(
                Rl(vec2(0.0, 100.0)) + Ab(vec2(0.0, -config.taskbar_height)),
                Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, config.taskbar_height)),
                Anchor::TopLeft,
            )
            .with_rect(0.0, 0.0, config.transparent)
            .color_raw::<UiBase>(config.taskbar)
            .with_effect(RectEffect::VerticalGradient {
                secondary: config.login_shade,
            })
            .entity();
        tree.in_parent(taskbar, |tree| {
            tree.add_node()
                .window(
                    Ab(vec2(0.0, 0.0)),
                    Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 1.0)),
                    Anchor::TopLeft,
                )
                .with_rect(0.0, 0.0, config.transparent)
                .color_raw::<UiBase>(config.divider)
                .entity();
            let start = tree
                .add_node()
                .window(
                    Ab(vec2(4.0, 4.0)),
                    Ab(vec2(
                        config.taskbar_height - 8.0,
                        config.taskbar_height - 8.0,
                    )),
                    Anchor::TopLeft,
                )
                .with_rect(6.0, 1.0, config.icon_edge)
                .color_raw::<UiBase>(config.brand_dim)
                .color_raw::<UiHover>(config.icon_selected)
                .with_transition::<UiHover>(18.0, 9.0)
                .with_effect(RectEffect::VerticalGradient {
                    secondary: config.avatar_shade,
                })
                .with_interaction()
                .with_cursor_icon(CursorIcon::Pointer)
                .entity();
            resources.interface.taskbar_start = start;
            tree.in_parent(start, |tree| {
                tree.add_node()
                    .absolute_fill(0.0)
                    .with_icon(material::GRID_ON, 19.0)
                    .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                    .color_raw::<UiBase>(config.text)
                    .entity();
            });
            tree.add_node()
                .window(
                    Ab(vec2(config.taskbar_height, 9.0)),
                    Ab(vec2(1.0, config.taskbar_height - 18.0)),
                    Anchor::TopLeft,
                )
                .with_rect(0.0, 0.0, config.transparent)
                .color_raw::<UiBase>(config.divider)
                .entity();
            let running = tree
                .add_node()
                .window(
                    Ab(vec2(config.taskbar_height + 8.0, 4.0)),
                    Rl(vec2(100.0, 0.0))
                        + Ab(vec2(
                            -config.taskbar_height - 356.0,
                            config.taskbar_height - 8.0,
                        )),
                    Anchor::TopLeft,
                )
                .flow(FlowDirection::Horizontal, 0.0, 4.0)
                .entity();
            tree.in_parent(running, |tree| {
                for (index, program) in PROGRAMS.iter().enumerate() {
                    let button = tree
                        .add_node()
                        .flow_child(Ab(vec2(
                            config.task_button_width,
                            config.taskbar_height - 8.0,
                        )))
                        .with_rect(5.0, 0.0, config.transparent)
                        .color_raw::<UiBase>(config.taskbar_idle)
                        .color_raw::<UiHover>(config.row_hover)
                        .with_transition::<UiHover>(16.0, 8.0)
                        .with_interaction()
                        .with_cursor_icon(CursorIcon::Pointer)
                        .with_visible(false)
                        .entity();
                    tree.in_parent(button, |tree| {
                        let lamp = tree
                            .add_node()
                            .window(Ab(vec2(0.0, 7.0)), Ab(vec2(3.0, 26.0)), Anchor::TopLeft)
                            .with_rect(2.0, 0.0, config.transparent)
                            .color_raw::<UiBase>(config.taskbar_active)
                            .with_visible(false)
                            .entity();
                        let mark = tree
                            .add_node()
                            .window(Ab(vec2(12.0, 4.0)), Ab(vec2(24.0, 32.0)), Anchor::TopLeft)
                            .with_icon(program.glyph, 16.0)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .color_raw::<UiBase>(config.brand)
                            .entity();
                        let label = tree
                            .add_node()
                            .window(
                                Ab(vec2(42.0, 4.0)),
                                Ab(vec2(config.task_button_width - 54.0, 32.0)),
                                Anchor::TopLeft,
                            )
                            .with_text(program.label, 14.6)
                            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                            .with_text_overflow(TextOverflow::Ellipsis)
                            .color_raw::<UiBase>(config.text_dim)
                            .entity();
                        let badge = tree
                            .add_node()
                            .window(
                                Rl(vec2(100.0, 0.0)) + Ab(vec2(-32.0, 10.0)),
                                Ab(vec2(22.0, 20.0)),
                                Anchor::TopLeft,
                            )
                            .with_rect(10.0, 0.0, config.transparent)
                            .color_raw::<UiBase>(config.badge)
                            .with_visible(false)
                            .entity();
                        let count = tree.in_parent(badge, |tree| {
                            tree.add_node()
                                .absolute_fill(0.0)
                                .with_text("", 11.0)
                                .with_text_alignment(
                                    TextAlignment::Center,
                                    VerticalAlignment::Middle,
                                )
                                .color_raw::<UiBase>(config.text)
                                .entity()
                        });
                        buttons.push((
                            button,
                            TaskbarButton {
                                program: index,
                                lamp,
                                mark,
                                label,
                                badge: LabeledSlot {
                                    root: badge,
                                    label: count,
                                },
                            },
                        ));
                    });
                }
            });

            let calendar = tree
                .add_node()
                .window(
                    Rl(vec2(100.0, 0.0)) + Ab(vec2(-198.0, 0.0)),
                    Ab(vec2(182.0, config.taskbar_height)),
                    Anchor::TopLeft,
                )
                .with_rect(4.0, 0.0, config.transparent)
                .color_raw::<UiBase>(config.transparent)
                .color_raw::<UiHover>(config.row_hover)
                .with_transition::<UiHover>(16.0, 8.0)
                .with_interaction()
                .with_cursor_icon(CursorIcon::Pointer)
                .entity();
            resources.interface.taskbar_calendar = calendar;
            tree.in_parent(calendar, |tree| {
                resources.interface.taskbar_clock = tree
                    .add_node()
                    .window(
                        Ab(vec2(0.0, 7.0)),
                        Rl(vec2(100.0, 0.0)) + Ab(vec2(-16.0, 17.0)),
                        Anchor::TopLeft,
                    )
                    .with_text("", 15.9)
                    .with_text_alignment(TextAlignment::Right, VerticalAlignment::Middle)
                    .color_raw::<UiBase>(config.text)
                    .entity();
                resources.interface.taskbar_date = tree
                    .add_node()
                    .window(
                        Ab(vec2(0.0, 25.0)),
                        Rl(vec2(100.0, 0.0)) + Ab(vec2(-16.0, 16.0)),
                        Anchor::TopLeft,
                    )
                    .with_text("", 13.4)
                    .with_text_alignment(TextAlignment::Right, VerticalAlignment::Middle)
                    .color_raw::<UiBase>(config.text_faint)
                    .entity();
            });
        });
    });
    tree.finish();

    for (button, parts) in buttons {
        ink.build.commands.set(button, parts);
    }
    for (icon, program) in icons {
        ink.build.commands.set(icon, ProgramIcon(program));
    }
}

fn build_sessions(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let content = frame_content(&ink.gathered.frames, program_index("sessions"));
    let mut tree = UiTreeBuilder::from_parent(ink.build.reborrow(), content);
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.pane_background)
        .with_effect(RectEffect::VerticalGradient {
            secondary: config.pane_shade,
        })
        .with_visible(false)
        .entity();
    resources.interface.sessions = root;

    tree.in_parent(root, |tree| {
        tree.add_node()
            .window(Ab(vec2(24.0, 18.0)), Ab(vec2(500.0, 20.0)), Anchor::TopLeft)
            .with_text("connection records", 17.1)
            .text_left()
            .color_raw::<UiBase>(config.text)
            .entity();
        tree.add_node()
            .window(Ab(vec2(24.0, 40.0)), Ab(vec2(560.0, 16.0)), Anchor::TopLeft)
            .with_text("kept by this machine, not by the server", 12.8)
            .text_left()
            .color_raw::<UiBase>(config.text_faint)
            .entity();
        let list = tree
            .add_node()
            .window(
                Ab(vec2(24.0, 78.0)),
                Ab(vec2(720.0, 400.0)),
                Anchor::TopLeft,
            )
            .flow(FlowDirection::Vertical, 0.0, 3.0)
            .entity();
        tree.in_parent(list, |tree| {
            for account in 0..ACCOUNTS.len() {
                let row = tree
                    .add_node()
                    .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 26.0)))
                    .with_text("", 14.0)
                    .text_left()
                    .color_raw::<UiBase>(config.text_dim)
                    .entity();
                tree.context_reborrow()
                    .commands
                    .set(row, SessionRow(account));
            }
        });
        resources.interface.sessions_total = tree
            .add_node()
            .window(
                Rl(vec2(0.0, 100.0)) + Ab(vec2(24.0, -64.0)),
                Ab(vec2(720.0, 18.0)),
                Anchor::TopLeft,
            )
            .with_text("", 13.4)
            .text_left()
            .color_raw::<UiBase>(config.text_faint)
            .entity();
    });
    tree.finish_subtree();
}

fn build_botterfly(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let content = frame_content(&ink.gathered.frames, program_index("botterfly"));
    let mut tree = UiTreeBuilder::from_parent(ink.build.reborrow(), content);
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.botterfly_background)
        .with_effect(RectEffect::VerticalGradient {
            secondary: config.botterfly_shade,
        })
        .with_visible(false)
        .entity();
    resources.interface.botterfly = root;

    tree.in_parent(root, |tree| {
        tree.add_node()
            .window(
                Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 40.0)),
                Ab(vec2(420.0, 34.0)),
                Anchor::TopCenter,
            )
            .with_text("Botterfly", 27.0)
            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
            .color_raw::<UiBase>(config.botterfly_text)
            .entity();
        tree.add_node()
            .window(
                Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 76.0)),
                Ab(vec2(520.0, 22.0)),
                Anchor::TopCenter,
            )
            .with_text("everybody ends up here eventually", 14.2)
            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
            .color_raw::<UiBase>(config.botterfly_faint)
            .entity();

        resources.interface.botterfly_count = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 128.0)),
                Ab(vec2(520.0, 52.0)),
                Anchor::TopCenter,
            )
            .with_text("8,814", 44.0)
            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
            .color_raw::<UiBase>(config.botterfly_accent)
            .entity();
        tree.add_node()
            .window(
                Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 182.0)),
                Ab(vec2(520.0, 20.0)),
                Anchor::TopCenter,
            )
            .with_text("colleagues are waiting for you", 13.4)
            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
            .color_raw::<UiBase>(config.botterfly_faint)
            .entity();

        resources.interface.botterfly_go = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 240.0)),
                Ab(vec2(320.0, 62.0)),
                Anchor::TopCenter,
            )
            .with_rect(31.0, 0.0, config.transparent)
            .color_raw::<UiBase>(config.botterfly_accent)
            .color_raw::<UiHover>(config.botterfly_accent_lit)
            .with_transition::<UiHover>(16.0, 8.0)
            .with_interaction()
            .with_cursor_icon(CursorIcon::Pointer)
            .entity();
        let go = resources.interface.botterfly_go;
        tree.in_parent(go, |tree| {
            tree.add_node()
                .absolute_fill(0.0)
                .with_text("JOIN THEM", 17.1)
                .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                .color_raw::<UiBase>(config.botterfly_background)
                .entity();
        });
        tree.add_node()
            .window(
                Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 316.0)),
                Ab(vec2(540.0, 18.0)),
                Anchor::TopCenter,
            )
            .with_text("uploading takes a moment and cannot be undone", 11.6)
            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
            .color_raw::<UiBase>(config.botterfly_faint)
            .entity();
    });
    tree.finish();
}

fn build_store(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let content = frame_content(&ink.gathered.frames, program_index("store"));
    let mut tree = UiTreeBuilder::from_parent(ink.build.reborrow(), content);
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.pane_background)
        .with_effect(RectEffect::VerticalGradient {
            secondary: config.pane_shade,
        })
        .with_visible(false)
        .entity();
    resources.interface.store = root;

    tree.in_parent(root, |tree| {
        tree.add_node()
            .window(Ab(vec2(24.0, 18.0)), Ab(vec2(560.0, 20.0)), Anchor::TopLeft)
            .with_text("software", 17.1)
            .text_left()
            .color_raw::<UiBase>(config.text)
            .entity();
        tree.add_node()
            .window(Ab(vec2(24.0, 40.0)), Ab(vec2(620.0, 16.0)), Anchor::TopLeft)
            .with_text(
                "what this network will give you. nothing here installs itself",
                12.8,
            )
            .text_left()
            .color_raw::<UiBase>(config.text_faint)
            .entity();
        resources.interface.store_field = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-286.0, 16.0)),
                Ab(vec2(262.0, 30.0)),
                Anchor::TopLeft,
            )
            .with_rect(15.0, 1.0, config.divider)
            .color_raw::<UiBase>(config.address_field)
            .color_raw::<UiHover>(config.field_hover)
            .with_transition::<UiHover>(16.0, 8.0)
            .with_interaction()
            .with_cursor_icon(CursorIcon::Text)
            .entity();
        let field = resources.interface.store_field;
        tree.in_parent(field, |tree| {
            resources.interface.store_caret = tree
                .add_node()
                .window(
                    Ab(vec2(14.0, 0.0)),
                    Rl(vec2(100.0, 0.0)) + Ab(vec2(-28.0, 30.0)),
                    Anchor::TopLeft,
                )
                .with_text("", 12.2)
                .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                .with_text_overflow(TextOverflow::Ellipsis)
                .color_raw::<UiBase>(config.text)
                .entity();
        });
        tree.add_node()
            .window(Ab(vec2(24.0, 66.0)), Ab(vec2(700.0, 1.0)), Anchor::TopLeft)
            .with_rect(0.0, 0.0, config.transparent)
            .color_raw::<UiBase>(config.divider)
            .entity();

        resources.interface.store_empty = tree
            .add_node()
            .window(Ab(vec2(24.0, 92.0)), Ab(vec2(700.0, 18.0)), Anchor::TopLeft)
            .with_text(
                "the list is empty. it only knows about what somebody has told you about",
                12.8,
            )
            .text_left()
            .color_raw::<UiBase>(config.text_faint)
            .with_visible(false)
            .entity();

        let holder = tree
            .add_node()
            .window(
                Ab(vec2(24.0, 84.0)),
                Rl(vec2(100.0, 100.0)) + Ab(vec2(-48.0, -108.0)),
                Anchor::TopLeft,
            )
            .entity();
        let wound = tree.in_parent(holder, |tree| tree.add_scroll_area_fill(0.0, 10.0));
        let list = scroll_content(tree, wound);
        tree.in_parent(list, |tree| {
            for listing in 0..STORE_ROWS {
                let card = tree
                    .add_node()
                    .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 86.0)))
                    .with_rect(6.0, 1.0, config.divider)
                    .color_raw::<UiBase>(config.composer_background)
                    .with_visible(false)
                    .entity();
                let mut row = StoreRow {
                    slot: listing,
                    listing,
                    root: card,
                    ..Default::default()
                };
                tree.in_parent(card, |tree| {
                    row.mark = tree
                        .add_node()
                        .window(Ab(vec2(14.0, 14.0)), Ab(vec2(44.0, 44.0)), Anchor::TopLeft)
                        .with_rect(6.0, 1.0, config.divider)
                        .color_raw::<UiBase>(config.pane_background)
                        .entity();
                    row.glyph = tree.in_parent(row.mark, |tree| {
                        tree.add_node()
                            .absolute_fill(0.0)
                            .with_icon(material::GRID_ON, 19.0)
                            .color_raw::<UiBase>(config.brand)
                            .entity()
                    });
                    row.title = tree
                        .add_node()
                        .window(Ab(vec2(70.0, 13.0)), Ab(vec2(400.0, 18.0)), Anchor::TopLeft)
                        .with_text("", 14.6)
                        .text_left()
                        .color_raw::<UiBase>(config.text)
                        .entity();
                    row.by = tree
                        .add_node()
                        .window(Ab(vec2(70.0, 32.0)), Ab(vec2(400.0, 14.0)), Anchor::TopLeft)
                        .with_text("", 11.2)
                        .text_left()
                        .color_raw::<UiBase>(config.text_faint)
                        .entity();
                    row.blurb = tree
                        .add_node()
                        .window(Ab(vec2(70.0, 49.0)), Ab(vec2(470.0, 16.0)), Anchor::TopLeft)
                        .with_text("", 12.4)
                        .text_left()
                        .with_text_overflow(TextOverflow::Ellipsis)
                        .color_raw::<UiBase>(config.text_dim)
                        .entity();
                    row.note = tree
                        .add_node()
                        .window(Ab(vec2(70.0, 66.0)), Ab(vec2(470.0, 14.0)), Anchor::TopLeft)
                        .with_text("", 10.6)
                        .text_left()
                        .with_text_overflow(TextOverflow::Ellipsis)
                        .color_raw::<UiBase>(config.text_faint)
                        .entity();
                    row.button = tree
                        .add_node()
                        .window(
                            Rl(vec2(100.0, 0.0)) + Ab(vec2(-134.0, 26.0)),
                            Ab(vec2(118.0, 32.0)),
                            Anchor::TopLeft,
                        )
                        .with_rect(5.0, 1.0, config.divider)
                        .color_raw::<UiBase>(config.pane_background)
                        .color_raw::<UiHover>(config.row_hover)
                        .with_transition::<UiHover>(16.0, 8.0)
                        .with_interaction()
                        .with_cursor_icon(CursorIcon::Pointer)
                        .entity();
                    tree.context_reborrow()
                        .commands
                        .set(row.button, StoreButton(listing));
                    tree.in_parent(row.button, |tree| {
                        row.button_label = tree
                            .add_node()
                            .absolute_fill(0.0)
                            .with_text("", 11.8)
                            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                            .color_raw::<UiBase>(config.brand)
                            .entity();
                    });
                });
                tree.context_reborrow().commands.set(card, row);
            }
        });
    });
    tree.finish();
}

fn build_cross(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let content = frame_content(&ink.gathered.frames, program_index("cross"));
    let mut tree = UiTreeBuilder::from_parent(ink.build.reborrow(), content);
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.pane_background)
        .with_effect(RectEffect::VerticalGradient {
            secondary: config.pane_shade,
        })
        .with_visible(false)
        .entity();
    resources.interface.cross = root;

    tree.in_parent(root, |tree| {
        tree.add_node()
            .window(Ab(vec2(24.0, 18.0)), Ab(vec2(560.0, 20.0)), Anchor::TopLeft)
            .with_text("cross-check", 17.1)
            .text_left()
            .color_raw::<UiBase>(config.text)
            .entity();
        tree.add_node()
            .window(Ab(vec2(24.0, 40.0)), Ab(vec2(620.0, 16.0)), Anchor::TopLeft)
            .with_text("what was said, against what this machine wrote down", 12.8)
            .text_left()
            .color_raw::<UiBase>(config.text_faint)
            .entity();

        let list = tree
            .add_node()
            .window(
                Ab(vec2(24.0, 76.0)),
                Ab(vec2(760.0, 430.0)),
                Anchor::TopLeft,
            )
            .flow(FlowDirection::Vertical, 0.0, 10.0)
            .entity();
        tree.in_parent(list, |tree| {
            for (pairing, _) in PAIRINGS.iter().enumerate() {
                let card = tree
                    .add_node()
                    .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 88.0)))
                    .with_rect(5.0, 1.0, config.divider)
                    .color_raw::<UiBase>(config.composer_background)
                    .with_visible(false)
                    .entity();
                let mut slot = PairingSlot {
                    root: card,
                    ..Default::default()
                };
                tree.in_parent(card, |tree| {
                    slot.source = tree
                        .add_node()
                        .window(Ab(vec2(14.0, 8.0)), Ab(vec2(430.0, 14.0)), Anchor::TopLeft)
                        .with_text("", 11.6)
                        .text_left()
                        .color_raw::<UiBase>(config.text_faint)
                        .entity();
                    slot.verdict = tree
                        .add_node()
                        .window(
                            Rl(vec2(100.0, 0.0)) + Ab(vec2(-116.0, 8.0)),
                            Ab(vec2(102.0, 14.0)),
                            Anchor::TopLeft,
                        )
                        .with_text("", 11.6)
                        .with_text_alignment(TextAlignment::Right, VerticalAlignment::Middle)
                        .color_raw::<UiBase>(config.text_faint)
                        .entity();
                    slot.claim = tree
                        .add_node()
                        .window(Ab(vec2(14.0, 24.0)), Ab(vec2(700.0, 16.0)), Anchor::TopLeft)
                        .with_text("", 14.0)
                        .text_left()
                        .color_raw::<UiBase>(config.text)
                        .entity();
                    slot.record = tree
                        .add_node()
                        .window(Ab(vec2(14.0, 43.0)), Ab(vec2(700.0, 16.0)), Anchor::TopLeft)
                        .with_text("", 14.0)
                        .text_left()
                        .color_raw::<UiBase>(config.text_dim)
                        .entity();
                    slot.reading = tree
                        .add_node()
                        .window(Ab(vec2(14.0, 64.0)), Ab(vec2(700.0, 16.0)), Anchor::TopLeft)
                        .with_text("", 12.2)
                        .text_left()
                        .color_raw::<UiBase>(config.text_faint)
                        .entity();
                });
                tree.context_reborrow()
                    .commands
                    .set(slot.root, PairingRow { pairing, slot });
            }
        });

        resources.interface.cross_summary = tree
            .add_node()
            .window(
                Rl(vec2(0.0, 100.0)) + Ab(vec2(24.0, -44.0)),
                Ab(vec2(760.0, 18.0)),
                Anchor::TopLeft,
            )
            .with_text("", 13.4)
            .text_left()
            .color_raw::<UiBase>(config.text_faint)
            .entity();
    });
    tree.finish_subtree();
}

fn build_files(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let content = frame_content(&ink.gathered.frames, program_index("files"));
    let mut tree = UiTreeBuilder::from_parent(ink.build.reborrow(), content);
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.pane_background)
        .with_effect(RectEffect::VerticalGradient {
            secondary: config.pane_shade,
        })
        .with_visible(false)
        .entity();
    resources.interface.files = root;

    tree.in_parent(root, |tree| {
        tree.add_node()
            .window(Ab(vec2(20.0, 16.0)), Ab(vec2(400.0, 20.0)), Anchor::TopLeft)
            .with_text("disk", 17.1)
            .text_left()
            .color_raw::<UiBase>(config.text)
            .entity();
        resources.interface.files_count = tree
            .add_node()
            .window(Ab(vec2(20.0, 38.0)), Ab(vec2(400.0, 16.0)), Anchor::TopLeft)
            .with_text("", 12.8)
            .text_left()
            .color_raw::<UiBase>(config.text_faint)
            .entity();

        let list = tree
            .add_node()
            .window(
                Ab(vec2(16.0, 70.0)),
                Rl(vec2(0.0, 100.0)) + Ab(vec2(228.0, -86.0)),
                Anchor::TopLeft,
            )
            .flow(FlowDirection::Vertical, 0.0, 2.0)
            .entity();
        tree.in_parent(list, |tree| {
            for index in 0..LEFTOVERS.len() {
                let row = tree
                    .add_node()
                    .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 38.0)))
                    .with_rect(4.0, 0.0, config.transparent)
                    .color_raw::<UiBase>(config.transparent)
                    .color_raw::<UiHover>(config.row_hover)
                    .color_raw::<UiPressed>(config.row_pressed)
                    .with_transition::<UiHover>(18.0, 9.0)
                    .with_interaction()
                    .with_cursor_icon(CursorIcon::Pointer)
                    .with_visible(false)
                    .entity();
                let mut slot = FileSlot {
                    root: row,
                    ..Default::default()
                };
                tree.in_parent(row, |tree| {
                    slot.name = tree
                        .add_node()
                        .window(Ab(vec2(10.0, 4.0)), Ab(vec2(200.0, 16.0)), Anchor::TopLeft)
                        .with_text("", 14.0)
                        .text_left()
                        .color_raw::<UiBase>(config.text_dim)
                        .entity();
                    slot.owner = tree
                        .add_node()
                        .window(Ab(vec2(10.0, 20.0)), Ab(vec2(200.0, 14.0)), Anchor::TopLeft)
                        .with_text("", 11.6)
                        .text_left()
                        .color_raw::<UiBase>(config.text_faint)
                        .entity();
                });
                tree.context_reborrow()
                    .commands
                    .set(row, FileRow { file: index, slot });
            }
        });

        let page = tree
            .add_node()
            .window(
                Ab(vec2(256.0, 70.0)),
                Rl(vec2(100.0, 100.0)) + Ab(vec2(-276.0, -86.0)),
                Anchor::TopLeft,
            )
            .with_rect(5.0, 1.0, config.divider)
            .color_raw::<UiBase>(config.composer_background)
            .entity();
        tree.in_parent(page, |tree| {
            resources.interface.file_name = tree
                .add_node()
                .window(Ab(vec2(18.0, 14.0)), Ab(vec2(400.0, 18.0)), Anchor::TopLeft)
                .with_text("", 15.2)
                .text_left()
                .color_raw::<UiBase>(config.text)
                .entity();
            resources.interface.file_stamp = tree
                .add_node()
                .window(
                    Rl(vec2(100.0, 0.0)) + Ab(vec2(-238.0, 15.0)),
                    Ab(vec2(220.0, 16.0)),
                    Anchor::TopLeft,
                )
                .with_text("", 11.6)
                .with_text_alignment(TextAlignment::Right, VerticalAlignment::Middle)
                .color_raw::<UiBase>(config.text_faint)
                .entity();
            resources.interface.file_body = tree
                .add_node()
                .window(
                    Ab(vec2(18.0, 42.0)),
                    Rl(vec2(100.0, 100.0)) + Ab(vec2(-36.0, -56.0)),
                    Anchor::TopLeft,
                )
                .with_text("", 14.0)
                .with_text_wrap()
                .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
                .color_raw::<UiBase>(config.text_dim)
                .entity();
        });
    });
    tree.finish_subtree();
}

const LINK_CAPACITY: usize = 6;

const ICON_COLUMN: usize = 8;
const ICON_COLUMN_PITCH: f32 = 100.0;

pub const NAV_LEFT: f32 = 82.0;
pub const NAV_GAP: f32 = 4.0;
pub const NAV_WIDEST: f32 = 176.0;

const BLOCK_CAPACITY: usize = 20;

fn build_chrome_button(
    tree: &mut UiTreeBuilder,
    config: &Config,
    at: Vec2,
    glyph: IconGlyph,
) -> Entity {
    let button = tree
        .add_node()
        .window(Ab(at), Ab(vec2(28.0, 28.0)), Anchor::TopLeft)
        .with_rect(14.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.transparent)
        .color_raw::<UiHover>(config.row_hover)
        .color_raw::<UiPressed>(config.row_pressed)
        .with_transition::<UiHover>(18.0, 9.0)
        .with_interaction()
        .with_cursor_icon(CursorIcon::Pointer)
        .entity();
    tree.in_parent(button, |tree| {
        tree.add_node()
            .absolute_fill(0.0)
            .with_icon(glyph, 16.0)
            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
            .color_raw::<UiBase>(config.text_dim)
            .color_raw::<UiHover>(config.text)
            .with_transition::<UiHover>(18.0, 9.0)
            .entity();
    });
    button
}

fn build_bookmark(
    tree: &mut UiTreeBuilder,
    config: &Config,
    width: f32,
    bookmark: usize,
    filed: bool,
) {
    let root = tree
        .add_node()
        .flow_child(Ab(vec2(width, 22.0)))
        .with_rect(4.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.transparent)
        .color_raw::<UiHover>(config.row_hover)
        .with_transition::<UiHover>(16.0, 8.0)
        .with_interaction()
        .with_cursor_icon(CursorIcon::Pointer)
        .with_visible(false)
        .entity();
    let label = tree.in_parent(root, |tree| {
        tree.add_node()
            .absolute_fill(0.0)
            .with_text("", 11.6)
            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
            .color_raw::<UiBase>(config.text_dim)
            .entity()
    });
    tree.context_reborrow().commands.set(
        root,
        BookmarkRow {
            bookmark,
            filed,
            slot: LabeledSlot { root, label },
        },
    );
}

fn build_browser(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let content = frame_content(&ink.gathered.frames, program_index("browser"));
    let mut tree = UiTreeBuilder::from_parent(ink.build.reborrow(), content);
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.chrome)
        .with_visible(false)
        .entity();
    resources.interface.browser = root;

    tree.in_parent(root, |tree| {
        let strip = tree
            .add_node()
            .window(
                Ab(vec2(0.0, 0.0)),
                Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 32.0)),
                Anchor::TopLeft,
            )
            .with_rect(0.0, 0.0, config.transparent)
            .color_raw::<UiBase>(config.chrome_shade)
            .entity();
        tree.in_parent(strip, |tree| {
            let tab = tree
                .add_node()
                .window(Ab(vec2(8.0, 5.0)), Ab(vec2(268.0, 27.0)), Anchor::TopLeft)
                .with_rect(7.0, 1.0, config.divider)
                .color_raw::<UiBase>(config.chrome)
                .with_effect(RectEffect::VerticalGradient {
                    secondary: config.chrome_shade,
                })
                .entity();
            tree.in_parent(tab, |tree| {
                resources.interface.browser_spinner = tree
                    .add_node()
                    .window(Ab(vec2(9.0, 0.0)), Ab(vec2(18.0, 27.0)), Anchor::TopLeft)
                    .with_text("", 12.0)
                    .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                    .color_raw::<UiBase>(config.brand)
                    .entity();
                resources.interface.browser_tab = tree
                    .add_node()
                    .window(Ab(vec2(30.0, 0.0)), Ab(vec2(226.0, 27.0)), Anchor::TopLeft)
                    .with_text("", 11.6)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                    .with_text_overflow(TextOverflow::Ellipsis)
                    .color_raw::<UiBase>(config.text_dim)
                    .entity();
            });
        });

        resources.interface.browser_back =
            build_chrome_button(tree, config, vec2(8.0, 38.0), material::ARROW_BACK);
        resources.interface.browser_forward =
            build_chrome_button(tree, config, vec2(42.0, 38.0), material::ARROW_FORWARD);
        resources.interface.browser_reload =
            build_chrome_button(tree, config, vec2(76.0, 38.0), material::REFRESH);
        resources.interface.browser_address = tree
            .add_node()
            .window(
                Ab(vec2(112.0, 38.0)),
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-124.0, 28.0)),
                Anchor::TopLeft,
            )
            .with_rect(14.0, 1.0, config.divider)
            .color_raw::<UiBase>(config.address_field)
            .with_interaction()
            .with_cursor_icon(CursorIcon::Text)
            .entity();
        let address = resources.interface.browser_address;
        tree.in_parent(address, |tree| {
            resources.interface.browser_caret = tree
                .add_node()
                .window(
                    Ab(vec2(14.0, 0.0)),
                    Rl(vec2(100.0, 0.0)) + Ab(vec2(-28.0, 28.0)),
                    Anchor::TopLeft,
                )
                .with_text("", 12.2)
                .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                .with_text_overflow(TextOverflow::Ellipsis)
                .color_raw::<UiBase>(config.text)
                .entity();
        });

        resources.interface.browser_progress = tree
            .add_node()
            .window(Ab(vec2(0.0, 70.0)), Ab(vec2(0.0, 2.0)), Anchor::TopLeft)
            .with_rect(0.0, 0.0, config.transparent)
            .color_raw::<UiBase>(config.progress)
            .with_visible(false)
            .entity();

        let bar = tree
            .add_node()
            .window(
                Ab(vec2(0.0, 72.0)),
                Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 30.0)),
                Anchor::TopLeft,
            )
            .with_rect(0.0, 0.0, config.transparent)
            .color_raw::<UiBase>(config.chrome_shade)
            .entity();
        tree.in_parent(bar, |tree| {
            let row = tree
                .add_node()
                .window(
                    Ab(vec2(8.0, 4.0)),
                    Rl(vec2(100.0, 0.0)) + Ab(vec2(-16.0, 22.0)),
                    Anchor::TopLeft,
                )
                .flow(FlowDirection::Horizontal, 0.0, 2.0)
                .entity();
            tree.in_parent(row, |tree| {
                for index in 0..BOOKMARKS.len() {
                    build_bookmark(tree, config, 108.0, index, false);
                }
                resources.interface.browser_folder = tree
                    .add_node()
                    .flow_child(Ab(vec2(112.0, 22.0)))
                    .with_rect(4.0, 0.0, config.transparent)
                    .color_raw::<UiBase>(config.transparent)
                    .color_raw::<UiHover>(config.row_hover)
                    .with_transition::<UiHover>(16.0, 8.0)
                    .with_interaction()
                    .with_cursor_icon(CursorIcon::Pointer)
                    .with_visible(false)
                    .entity();
                let folder = resources.interface.browser_folder;
                tree.in_parent(folder, |tree| {
                    tree.add_node()
                        .absolute_fill(0.0)
                        .with_text(FOLDER, 11.6)
                        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                        .color_raw::<UiBase>(config.text_dim)
                        .entity();
                });
            });
        });

        let body = tree
            .add_node()
            .boundary(Ab(vec2(0.0, 102.0)), Rl(vec2(100.0, 100.0)))
            .with_rect(0.0, 0.0, config.transparent)
            .color_raw::<UiBase>(config.page_body)
            .with_clip()
            .entity();
        resources.interface.browser_body = body;
        let page = tree.in_parent(body, |tree| {
            tree.add_node()
                .window(
                    Rl(vec2(50.0, 0.0)),
                    Rl(vec2(0.0, 100.0)) + Ab(vec2(config.page_width, 0.0)),
                    Anchor::TopCenter,
                )
                .with_rect(0.0, 1.0, config.page_edge)
                .color_raw::<UiBase>(config.page)
                .entity()
        });
        resources.interface.browser_page = page;
        tree.in_parent(page, |tree| {
            let band = tree
                .add_node()
                .window(
                    Ab(vec2(0.0, 0.0)),
                    Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 40.0)),
                    Anchor::TopLeft,
                )
                .with_rect(0.0, 0.0, config.transparent)
                .color_raw::<UiBase>(config.page_band)
                .entity();
            tree.in_parent(band, |tree| {
                tree.add_node()
                    .window(Ab(vec2(26.0, 16.0)), Ab(vec2(9.0, 9.0)), Anchor::TopLeft)
                    .with_rect(2.0, 0.0, config.transparent)
                    .color_raw::<UiBase>(config.page_mark)
                    .entity();
                resources.interface.browser_band = tree
                    .add_node()
                    .window(Ab(vec2(45.0, 0.0)), Ab(vec2(540.0, 40.0)), Anchor::TopLeft)
                    .with_text("", 14.2)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                    .with_text_overflow(TextOverflow::Ellipsis)
                    .color_raw::<UiBase>(config.page_text)
                    .entity();
                tree.add_node()
                    .window(
                        Rl(vec2(0.0, 100.0)) + Ab(vec2(0.0, -1.0)),
                        Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 1.0)),
                        Anchor::TopLeft,
                    )
                    .with_rect(0.0, 0.0, config.transparent)
                    .color_raw::<UiBase>(config.page_edge)
                    .entity();
            });

            let nav = tree
                .add_node()
                .window(
                    Ab(vec2(0.0, 40.0)),
                    Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 34.0)),
                    Anchor::TopLeft,
                )
                .with_rect(0.0, 0.0, config.transparent)
                .color_raw::<UiBase>(config.page_nav)
                .entity();
            tree.in_parent(nav, |tree| {
                tree.add_node()
                    .window(Ab(vec2(20.0, 0.0)), Ab(vec2(58.0, 34.0)), Anchor::TopLeft)
                    .with_text("GO TO", 10.4)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                    .color_raw::<UiBase>(config.page_faint)
                    .entity();
                let row = tree
                    .add_node()
                    .window(
                        Ab(vec2(NAV_LEFT, 5.0)),
                        Rl(vec2(100.0, 0.0)) + Ab(vec2(-NAV_LEFT - 18.0, 24.0)),
                        Anchor::TopLeft,
                    )
                    .flow(FlowDirection::Horizontal, 0.0, NAV_GAP)
                    .entity();
                tree.in_parent(row, |tree| {
                    for index in 0..LINK_CAPACITY {
                        let link = tree
                            .add_node()
                            .flow_child(Ab(vec2(132.0, 24.0)))
                            .with_rect(4.0, 1.0, config.page_edge)
                            .color_raw::<UiBase>(config.page_nav)
                            .color_raw::<UiHover>(config.page_band)
                            .with_transition::<UiHover>(16.0, 8.0)
                            .with_interaction()
                            .with_cursor_icon(CursorIcon::Pointer)
                            .with_visible(false)
                            .entity();
                        let label = tree.in_parent(link, |tree| {
                            tree.add_node()
                                .absolute_fill(0.0)
                                .with_text("", 11.8)
                                .with_text_alignment(
                                    TextAlignment::Center,
                                    VerticalAlignment::Middle,
                                )
                                .with_text_overflow(TextOverflow::Ellipsis)
                                .color_raw::<UiBase>(config.page_link)
                                .entity()
                        });
                        tree.context_reborrow().commands.set(
                            link,
                            LinkRow {
                                link: index,
                                slot: LabeledSlot { root: link, label },
                            },
                        );
                    }
                });
            });

            let blocks = tree
                .add_node()
                .window(
                    Ab(vec2(config.page_margin + 12.0, 92.0)),
                    Rl(vec2(100.0, 100.0)) + Ab(vec2(-2.0 * (config.page_margin + 12.0), -196.0)),
                    Anchor::TopLeft,
                )
                .flow(FlowDirection::Vertical, 0.0, 0.0)
                .entity();
            tree.in_parent(blocks, |tree| {
                for block in 0..BLOCK_CAPACITY {
                    let row = tree
                        .add_node()
                        .flow_child(Rl(vec2(100.0, 0.0)))
                        .flow(FlowDirection::Vertical, 0.0, 0.0)
                        .auto_size(AutoSizeMode::Height)
                        .with_rect(4.0, 0.0, config.transparent)
                        .color_raw::<UiBase>(config.transparent)
                        .color_raw::<UiHover>(config.transparent)
                        .with_transition::<UiHover>(16.0, 8.0)
                        .with_interaction()
                        .with_cursor_icon(CursorIcon::Pointer)
                        .with_visible(false)
                        .entity();
                    let (label, mark, right, rule, stripe, above, below) =
                        tree.in_parent(row, |tree| {
                            let mark = tree
                                .add_node()
                                .window(Ab(vec2(2.0, 11.0)), Ab(vec2(5.0, 5.0)), Anchor::TopLeft)
                                .with_rect(1.5, 0.0, config.transparent)
                                .color_raw::<UiBase>(config.page_mark)
                                .with_visible(false)
                                .entity();
                            let right = tree
                                .add_node()
                                .window(
                                    Rl(vec2(100.0, 0.0)) + Ab(vec2(-236.0, 6.0)),
                                    Ab(vec2(226.0, 18.0)),
                                    Anchor::TopLeft,
                                )
                                .with_text("", 11.6)
                                .with_text_alignment(
                                    TextAlignment::Right,
                                    VerticalAlignment::Middle,
                                )
                                .with_text_overflow(TextOverflow::Ellipsis)
                                .color_raw::<UiBase>(config.page_faint)
                                .with_visible(false)
                                .entity();
                            let above = tree
                                .add_node()
                                .flow_child(Rl(vec2(100.0, 0.0)))
                                .with_rect(0.0, 0.0, config.transparent)
                                .color_raw::<UiBase>(config.transparent)
                                .entity();
                            let stripe = tree
                                .add_node()
                                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 2.0)))
                                .with_rect(0.0, 0.0, config.transparent)
                                .color_raw::<UiBase>(config.page_mark)
                                .with_visible(false)
                                .entity();
                            let label = tree
                                .add_node()
                                .fill_width()
                                .auto_size(AutoSizeMode::Height)
                                .with_text("", 13.4)
                                .with_text_wrap()
                                .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
                                .color_raw::<UiBase>(config.page_text)
                                .entity();
                            let rule = tree
                                .add_node()
                                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 1.0)))
                                .with_rect(0.0, 0.0, config.transparent)
                                .color_raw::<UiBase>(config.page_rule)
                                .with_visible(false)
                                .entity();
                            let below = tree
                                .add_node()
                                .flow_child(Rl(vec2(100.0, 0.0)))
                                .with_rect(0.0, 0.0, config.transparent)
                                .color_raw::<UiBase>(config.transparent)
                                .entity();
                            (label, mark, right, rule, stripe, above, below)
                        });
                    tree.context_reborrow().commands.set(
                        row,
                        PageBlock {
                            block,
                            slot: LabeledSlot { root: row, label },
                            mark,
                            right,
                            rule,
                            stripe,
                            above,
                            below,
                        },
                    );
                }
            });

            resources.interface.browser_prompt = tree
                .add_node()
                .window(
                    Rl(vec2(0.0, 100.0)) + Ab(vec2(config.page_margin + 12.0, -122.0)),
                    Rl(vec2(100.0, 0.0)) + Ab(vec2(-2.0 * (config.page_margin + 12.0), 66.0)),
                    Anchor::TopLeft,
                )
                .with_rect(5.0, 1.0, config.page_faint)
                .color_raw::<UiBase>(config.transparent)
                .with_visible(false)
                .entity();
            let prompt = resources.interface.browser_prompt;
            tree.in_parent(prompt, |tree| {
                resources.interface.browser_prompt_label = tree
                    .add_node()
                    .window(Ab(vec2(12.0, 8.0)), Ab(vec2(300.0, 15.0)), Anchor::TopLeft)
                    .with_text("", 11.0)
                    .text_left()
                    .color_raw::<UiBase>(config.page_faint)
                    .entity();
                resources.interface.browser_prompt_field = tree
                    .add_node()
                    .window(
                        Ab(vec2(12.0, 26.0)),
                        Rl(vec2(100.0, 0.0)) + Ab(vec2(-108.0, 26.0)),
                        Anchor::TopLeft,
                    )
                    .with_rect(3.0, 1.0, config.page_faint)
                    .color_raw::<UiBase>(config.page)
                    .with_interaction()
                    .with_cursor_icon(CursorIcon::Text)
                    .entity();
                let field = resources.interface.browser_prompt_field;
                tree.in_parent(field, |tree| {
                    resources.interface.browser_prompt_caret = tree
                        .add_node()
                        .window(
                            Ab(vec2(9.0, 0.0)),
                            Rl(vec2(100.0, 0.0)) + Ab(vec2(-18.0, 26.0)),
                            Anchor::TopLeft,
                        )
                        .with_text("", 12.2)
                        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                        .color_raw::<UiBase>(config.page_text)
                        .entity();
                });
                resources.interface.browser_prompt_go = tree
                    .add_node()
                    .window(
                        Rl(vec2(100.0, 0.0)) + Ab(vec2(-86.0, 26.0)),
                        Ab(vec2(74.0, 26.0)),
                        Anchor::TopLeft,
                    )
                    .with_rect(3.0, 1.0, config.page_faint)
                    .color_raw::<UiBase>(config.page)
                    .color_raw::<UiHover>(config.card_shade)
                    .with_transition::<UiHover>(16.0, 8.0)
                    .with_interaction()
                    .with_cursor_icon(CursorIcon::Pointer)
                    .entity();
                let go = resources.interface.browser_prompt_go;
                tree.in_parent(go, |tree| {
                    tree.add_node()
                        .absolute_fill(0.0)
                        .with_text("submit", 11.6)
                        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                        .color_raw::<UiBase>(config.page_text)
                        .entity();
                });
            });

            resources.interface.browser_status = tree
                .add_node()
                .window(
                    Rl(vec2(0.0, 100.0)) + Ab(vec2(44.0, -30.0)),
                    Ab(vec2(560.0, 18.0)),
                    Anchor::TopLeft,
                )
                .with_text("", 10.5)
                .text_left()
                .color_raw::<UiBase>(config.page_faint)
                .entity();
        });

        resources.interface.browser_folder_panel = tree
            .add_node()
            .window(Ab(vec2(8.0, 100.0)), Ab(vec2(196.0, 8.0)), Anchor::TopLeft)
            .auto_size(AutoSizeMode::Height)
            .with_rect(5.0, 1.0, config.divider)
            .color_raw::<UiBase>(config.chrome)
            .with_visible(false)
            .entity();
        let panel = resources.interface.browser_folder_panel;
        tree.in_parent(panel, |tree| {
            let column = tree
                .add_node()
                .window(Ab(vec2(5.0, 5.0)), Ab(vec2(186.0, 8.0)), Anchor::TopLeft)
                .auto_size(AutoSizeMode::Height)
                .flow(FlowDirection::Vertical, 0.0, 2.0)
                .entity();
            tree.in_parent(column, |tree| {
                for index in 0..BOOKMARKS.len() {
                    build_bookmark(tree, config, 186.0, index, true);
                }
            });
        });
    });
    tree.finish_subtree();
}

fn build_terminated(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let mut tree = UiTreeBuilder::new(ink.build.reborrow());
    ink.gathered.roots.push(tree.root_entity());
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.terminal_black)
        .with_layer(UiLayer::Popups)
        .with_visible(false)
        .entity();
    resources.interface.terminated = root;
    tree.in_parent(root, |tree| {
        tree.add_node()
            .window(Rl(vec2(50.0, 50.0)), Ab(vec2(760.0, 26.0)), Anchor::Center)
            .with_text("[TERMINATED DUE TO RISK]", 19.5)
            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
            .color_raw::<UiBase>(config.terminal_red)
            .entity();
    });
    tree.finish_subtree();
}

fn build_office(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let content = frame_content(&ink.gathered.frames, program_index("office"));
    let mut tree = UiTreeBuilder::from_parent(ink.build.reborrow(), content);
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.pane_background)
        .with_effect(RectEffect::VerticalGradient {
            secondary: config.pane_shade,
        })
        .with_visible(false)
        .entity();
    resources.interface.office = root;

    tree.in_parent(root, |tree| {
        tree.add_node()
            .window(Ab(vec2(24.0, 16.0)), Ab(vec2(600.0, 20.0)), Anchor::TopLeft)
            .with_text(crate::data::office::TITLE, 15.9)
            .text_left()
            .color_raw::<UiBase>(config.brand)
            .entity();
        tree.add_node()
            .window(Ab(vec2(24.0, 37.0)), Ab(vec2(600.0, 16.0)), Anchor::TopLeft)
            .with_text(crate::data::office::BYLINE, 11.0)
            .text_left()
            .color_raw::<UiBase>(config.text_faint)
            .entity();

        resources.interface.office_name = tree
            .add_node()
            .window(Ab(vec2(24.0, 66.0)), Ab(vec2(600.0, 20.0)), Anchor::TopLeft)
            .with_text("", 14.6)
            .text_left()
            .color_raw::<UiBase>(config.text)
            .entity();
        resources.interface.office_body = tree
            .add_node()
            .window(
                Ab(vec2(24.0, 92.0)),
                Ab(vec2(430.0, 0.0)) + Rl(vec2(0.0, 100.0)) + Ab(vec2(0.0, -116.0)),
                Anchor::TopLeft,
            )
            .with_text("", 13.4)
            .with_text_wrap()
            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
            .color_raw::<UiBase>(config.text_dim)
            .entity();
        resources.interface.office_look = tree
            .add_node()
            .window(
                Ab(vec2(24.0, 92.0)),
                Ab(vec2(430.0, 0.0)) + Rl(vec2(0.0, 100.0)) + Ab(vec2(0.0, -116.0)),
                Anchor::TopLeft,
            )
            .with_text("", 13.4)
            .with_text_wrap()
            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
            .color_raw::<UiBase>(config.brand)
            .with_visible(false)
            .entity();

        let column = tree
            .add_node()
            .window(
                Ab(vec2(474.0, 66.0)),
                Rl(vec2(100.0, 100.0)) + Ab(vec2(-498.0, -90.0)),
                Anchor::TopLeft,
            )
            .flow(FlowDirection::Vertical, 0.0, 5.0)
            .entity();
        tree.in_parent(column, |tree| {
            for way in 0..OFFICE_WAYS {
                let row = tree
                    .add_node()
                    .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 30.0)))
                    .with_rect(5.0, 1.0, config.divider)
                    .color_raw::<UiBase>(config.composer_background)
                    .color_raw::<UiHover>(config.row_hover)
                    .color_raw::<UiPressed>(config.row_pressed)
                    .with_transition::<UiHover>(18.0, 9.0)
                    .with_interaction()
                    .with_cursor_icon(CursorIcon::Pointer)
                    .with_visible(false)
                    .entity();
                let label = tree.in_parent(row, |tree| {
                    tree.add_node()
                        .window(
                            Ab(vec2(12.0, 0.0)),
                            Rl(vec2(100.0, 0.0)) + Ab(vec2(-24.0, 30.0)),
                            Anchor::TopLeft,
                        )
                        .with_text("", 13.4)
                        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                        .color_raw::<UiBase>(config.text)
                        .entity()
                });
                tree.context_reborrow().commands.set(
                    row,
                    WayRow {
                        way,
                        slot: LabeledSlot { root: row, label },
                    },
                );
            }
            for thing in 0..OFFICE_THINGS {
                let row = tree
                    .add_node()
                    .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 28.0)))
                    .with_rect(5.0, 0.0, config.transparent)
                    .color_raw::<UiBase>(config.transparent)
                    .color_raw::<UiHover>(config.row_hover)
                    .color_raw::<UiPressed>(config.row_pressed)
                    .with_transition::<UiHover>(18.0, 9.0)
                    .with_interaction()
                    .with_cursor_icon(CursorIcon::Pointer)
                    .with_visible(false)
                    .entity();
                let label = tree.in_parent(row, |tree| {
                    tree.add_node()
                        .window(
                            Ab(vec2(12.0, 0.0)),
                            Rl(vec2(100.0, 0.0)) + Ab(vec2(-24.0, 28.0)),
                            Anchor::TopLeft,
                        )
                        .with_text("", 12.8)
                        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                        .color_raw::<UiBase>(config.text_dim)
                        .entity()
                });
                tree.context_reborrow().commands.set(
                    row,
                    ThingRow {
                        thing,
                        slot: LabeledSlot { root: row, label },
                    },
                );
            }
        });
    });
    tree.finish_subtree();
}

const OFFICE_WAYS: usize = crate::data::office::MOST_WAYS;
const OFFICE_THINGS: usize = crate::data::office::MOST_THINGS;

fn build_case(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let content = frame_content(&ink.gathered.frames, program_index("case"));
    let mut tree = UiTreeBuilder::from_parent(ink.build.reborrow(), content);
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.window_background)
        .with_visible(false)
        .entity();
    resources.interface.case = root;

    tree.in_parent(root, |tree| {
        tree.add_node()
            .window(Ab(vec2(22.0, 16.0)), Ab(vec2(760.0, 18.0)), Anchor::TopLeft)
            .with_text(crate::data::case::BANNER, 12.2)
            .text_left()
            .color_raw::<UiBase>(config.accent_warn)
            .entity();
        tree.add_node()
            .window(Ab(vec2(22.0, 36.0)), Ab(vec2(760.0, 18.0)), Anchor::TopLeft)
            .with_text(crate::data::case::NOTICE, 12.2)
            .text_left()
            .color_raw::<UiBase>(config.text_faint)
            .entity();

        let list = tree
            .add_node()
            .window(
                Ab(vec2(22.0, 68.0)),
                Ab(vec2(266.0, 0.0)) + Rl(vec2(0.0, 100.0)) + Ab(vec2(0.0, -92.0)),
                Anchor::TopLeft,
            )
            .flow(FlowDirection::Vertical, 0.0, 4.0)
            .entity();
        tree.in_parent(list, |tree| {
            for file in 0..crate::data::case::count() {
                let row = tree
                    .add_node()
                    .flow_child(Ab(vec2(266.0, 46.0)))
                    .with_rect(5.0, 1.0, config.divider)
                    .color_raw::<UiBase>(config.pane_background)
                    .color_raw::<UiHover>(config.row_hover)
                    .color_raw::<UiPressed>(config.row_pressed)
                    .with_transition::<UiHover>(18.0, 9.0)
                    .with_interaction()
                    .with_cursor_icon(CursorIcon::Pointer)
                    .entity();
                let label = tree.in_parent(row, |tree| {
                    let dossier = &crate::data::case::DOSSIERS[file];
                    tree.add_node()
                        .window(Ab(vec2(12.0, 6.0)), Ab(vec2(244.0, 17.0)), Anchor::TopLeft)
                        .with_text(dossier.handle, 14.0)
                        .text_left()
                        .color_raw::<UiBase>(if dossier.mine {
                            config.accent_warn
                        } else {
                            config.text
                        })
                        .entity();
                    tree.add_node()
                        .window(Ab(vec2(12.0, 24.0)), Ab(vec2(244.0, 15.0)), Anchor::TopLeft)
                        .with_text(dossier.summary, 11.6)
                        .text_left()
                        .color_raw::<UiBase>(config.text_faint)
                        .entity()
                });
                tree.context_reborrow().commands.set(
                    row,
                    CaseRow {
                        file,
                        slot: LabeledSlot { root: row, label },
                    },
                );
            }
        });

        resources.interface.case_title = tree
            .add_node()
            .window(
                Ab(vec2(306.0, 68.0)),
                Ab(vec2(600.0, 18.0)),
                Anchor::TopLeft,
            )
            .with_text("", 13.4)
            .text_left()
            .color_raw::<UiBase>(config.text_dim)
            .entity();
        resources.interface.case_body = tree
            .add_node()
            .window(
                Ab(vec2(306.0, 94.0)),
                Rl(vec2(100.0, 100.0)) + Ab(vec2(-328.0, -118.0)),
                Anchor::TopLeft,
            )
            .with_text("", 12.2)
            .with_text_wrap()
            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
            .color_raw::<UiBase>(config.text)
            .entity();
        tree.edit::<UiNodeContent>(resources.interface.case_body, |content| {
            if let UiNodeContent::Text { font_kind, .. } = content {
                *font_kind = nightshade_text::contract::data::text_data::FontKind::Mono;
            }
        });
    });
    tree.finish_subtree();
}

fn build_lights(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
    program: &str,
    goal: crate::data::lights::Goal,
) {
    let index = program_index(program);
    let content = frame_content(&ink.gathered.frames, index);
    let opening = match goal {
        crate::data::lights::Goal::Out => BOARDS[0],
        crate::data::lights::Goal::On => crate::data::lights::DARK,
    };
    let byline = match goal {
        crate::data::lights::Goal::Out => BYLINE,
        crate::data::lights::Goal::On => crate::data::lights::OTHER_BYLINE,
    };
    let mut tree = UiTreeBuilder::from_parent(ink.build.reborrow(), content);
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.pane_background)
        .with_effect(RectEffect::VerticalGradient {
            secondary: config.pane_shade,
        })
        .with_visible(false)
        .entity();
    match goal {
        crate::data::lights::Goal::Out => resources.interface.lights = root,
        crate::data::lights::Goal::On => resources.interface.lights_on = root,
    }

    tree.in_parent(root, |tree| {
        tree.add_node()
            .window(
                Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 26.0)),
                Ab(vec2(440.0, 22.0)),
                Anchor::TopCenter,
            )
            .with_text(crate::data::lights::instruction(goal), 17.1)
            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
            .color_raw::<UiBase>(config.text)
            .entity();
        tree.add_node()
            .window(
                Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 50.0)),
                Ab(vec2(480.0, 16.0)),
                Anchor::TopCenter,
            )
            .with_text(byline, 11.0)
            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
            .color_raw::<UiBase>(config.text_faint)
            .entity();

        let width = opening.len() as f32 * 66.0;
        let row = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 50.0)) + Ab(vec2(0.0, -12.0)),
                Ab(vec2(width, 62.0)),
                Anchor::Center,
            )
            .flow(FlowDirection::Horizontal, 0.0, 6.0)
            .entity();
        tree.in_parent(row, |tree| {
            for place in 0..opening.len() {
                let lamp = tree
                    .add_node()
                    .flow_child(Ab(vec2(60.0, 62.0)))
                    .with_rect(8.0, 1.0, config.divider)
                    .color_raw::<UiBase>(config.board_dark)
                    .color_raw::<UiHover>(config.row_hover)
                    .with_transition::<UiHover>(20.0, 10.0)
                    .with_interaction()
                    .with_cursor_icon(CursorIcon::Pointer)
                    .entity();
                let label = tree.in_parent(lamp, |tree| {
                    tree.add_node()
                        .absolute_fill(0.0)
                        .with_icon(material::LIGHTBULB, 24.0)
                        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                        .color_raw::<UiBase>(config.text_faint)
                        .entity()
                });
                tree.context_reborrow().commands.set(
                    lamp,
                    LampRow {
                        program: index,
                        place,
                        lit: opening.chars().nth(place) == Some('1'),
                        slot: LabeledSlot { root: lamp, label },
                    },
                );
            }
        });

        let says = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 50.0)) + Ab(vec2(0.0, 66.0)),
                Ab(vec2(560.0, 20.0)),
                Anchor::TopCenter,
            )
            .with_text("", 14.6)
            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
            .color_raw::<UiBase>(config.brand)
            .entity();

        if goal == crate::data::lights::Goal::On {
            resources.interface.lights_on_says = says;
            return;
        }
        resources.interface.lights_says = says;
        resources.interface.lights_moves = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 50.0)) + Ab(vec2(0.0, 42.0)),
                Ab(vec2(440.0, 18.0)),
                Anchor::TopCenter,
            )
            .with_text("", 13.4)
            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
            .color_raw::<UiBase>(config.text_dim)
            .entity();

        resources.interface.lights_again = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -74.0)),
                Ab(vec2(150.0, 32.0)),
                Anchor::TopCenter,
            )
            .with_rect(5.0, 1.0, config.divider)
            .color_raw::<UiBase>(config.composer_background)
            .color_raw::<UiHover>(config.row_hover)
            .color_raw::<UiPressed>(config.row_pressed)
            .with_transition::<UiHover>(18.0, 9.0)
            .with_interaction()
            .with_cursor_icon(CursorIcon::Pointer)
            .entity();
        let again = resources.interface.lights_again;
        tree.in_parent(again, |tree| {
            tree.add_node()
                .absolute_fill(0.0)
                .with_text("another board", 13.4)
                .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                .color_raw::<UiBase>(config.text_dim)
                .entity();
        });

        resources.interface.lights_count = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -32.0)),
                Ab(vec2(560.0, 18.0)),
                Anchor::TopCenter,
            )
            .with_text("", 11.6)
            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
            .color_raw::<UiBase>(config.text_faint)
            .entity();
    });
    tree.finish_subtree();
}

fn build_waiting(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let mut tree = UiTreeBuilder::new(ink.build.reborrow());
    ink.gathered.roots.push(tree.root_entity());
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.boot_background)
        .with_layer(UiLayer::Tooltips)
        .with_visible(false)
        .entity();
    resources.interface.waiting = root;

    tree.in_parent(root, |tree| {
        resources.interface.waiting_text = tree
            .add_node()
            .window(Rl(vec2(50.0, 50.0)), Ab(vec2(520.0, 24.0)), Anchor::Center)
            .with_text("", 14.0)
            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
            .color_raw::<UiBase>(config.text_faint)
            .entity();
    });
    tree.finish();
}

fn build_boot(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let mut tree = UiTreeBuilder::new(ink.build.reborrow());
    ink.gathered.roots.push(tree.root_entity());
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.boot_background)
        .with_layer(UiLayer::Tooltips)
        .with_visible(false)
        .entity();
    resources.interface.boot = root;

    tree.in_parent(root, |tree| {
        resources.interface.boot_text = tree
            .add_node()
            .window(
                Ab(vec2(52.0, 44.0)),
                Rl(vec2(100.0, 100.0)) + Ab(vec2(-104.0, -88.0)),
                Anchor::TopLeft,
            )
            .with_text("", 14.0)
            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
            .color_raw::<UiBase>(config.boot_text)
            .entity();
    });
    tree.finish();
}

fn build_over(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let mut tree = UiTreeBuilder::new(ink.build.reborrow());
    ink.gathered.roots.push(tree.root_entity());
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.login_background)
        .with_effect(RectEffect::VerticalGradient {
            secondary: config.login_shade,
        })
        .with_layer(UiLayer::Tooltips)
        .with_visible(false)
        .entity();
    resources.interface.over = root;

    tree.in_parent(root, |tree| {
        let panel = tree
            .add_node()
            .window(Rl(vec2(50.0, 50.0)), Ab(vec2(720.0, 560.0)), Anchor::Center)
            .with_rect(8.0, 1.0, config.divider)
            .color_raw::<UiBase>(config.pane_background)
            .with_effect(RectEffect::VerticalGradient {
                secondary: config.pane_shade,
            })
            .entity();
        tree.in_parent(panel, |tree| {
            resources.interface.over_heading = tree
                .add_node()
                .window(Ab(vec2(38.0, 32.0)), Ab(vec2(640.0, 24.0)), Anchor::TopLeft)
                .with_text("saturday", 19.5)
                .text_left()
                .color_raw::<UiBase>(config.brand)
                .entity();
            resources.interface.over_closing = tree
                .add_node()
                .window(
                    Ab(vec2(38.0, 62.0)),
                    Ab(vec2(640.0, 172.0)),
                    Anchor::TopLeft,
                )
                .with_text("", 12.6)
                .with_text_wrap()
                .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
                .color_raw::<UiBase>(config.text_dim)
                .entity();

            tree.add_node()
                .window(Ab(vec2(38.0, 244.0)), Ab(vec2(644.0, 1.0)), Anchor::TopLeft)
                .with_rect(0.0, 0.0, config.transparent)
                .color_raw::<UiBase>(config.divider)
                .entity();
            tree.add_node()
                .window(
                    Ab(vec2(38.0, 258.0)),
                    Ab(vec2(640.0, 18.0)),
                    Anchor::TopLeft,
                )
                .with_text("WHAT YOU WORKED OUT", 11.6)
                .text_left()
                .color_raw::<UiBase>(config.text_faint)
                .entity();

            let list = tree
                .add_node()
                .window(
                    Ab(vec2(38.0, 284.0)),
                    Ab(vec2(644.0, 180.0)),
                    Anchor::TopLeft,
                )
                .flow(FlowDirection::Vertical, 0.0, 2.0)
                .entity();
            tree.in_parent(list, |tree| {
                for lead in 0..LEADS.len() {
                    let row = tree
                        .add_node()
                        .flow_child(Ab(vec2(644.0, 18.0)))
                        .with_rect(0.0, 0.0, config.transparent)
                        .color_raw::<UiBase>(config.transparent)
                        .entity();
                    let label = tree.in_parent(row, |tree| {
                        tree.add_node()
                            .absolute_fill(0.0)
                            .with_text("", 12.4)
                            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                            .color_raw::<UiBase>(config.text_dim)
                            .entity()
                    });
                    tree.context_reborrow()
                        .commands
                        .set(row, TallyRow { lead, label });
                }
            });

            resources.interface.over_verdict = tree
                .add_node()
                .window(
                    Rl(vec2(0.0, 100.0)) + Ab(vec2(38.0, -96.0)),
                    Ab(vec2(644.0, 40.0)),
                    Anchor::TopLeft,
                )
                .with_text("", 14.6)
                .with_text_wrap()
                .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
                .color_raw::<UiBase>(config.text)
                .entity();
            resources.interface.over_session = tree
                .add_node()
                .window(
                    Rl(vec2(0.0, 100.0)) + Ab(vec2(38.0, -44.0)),
                    Ab(vec2(644.0, 18.0)),
                    Anchor::TopLeft,
                )
                .with_text("", 11.6)
                .text_left()
                .color_raw::<UiBase>(config.text_faint)
                .entity();
        });
    });
    tree.finish();
}

fn build_start_menu(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let root = resources.interface.desktop;
    let mut tree = UiTreeBuilder::from_parent(ink.build.reborrow(), root);
    let rows = PROGRAMS.len() as f32;
    let height = rows * 34.0 + 40.0;
    let panel = tree
        .add_node()
        .window(
            Rl(vec2(0.0, 100.0)) + Ab(vec2(6.0, -config.taskbar_height - height - 6.0)),
            Ab(vec2(232.0, height)),
            Anchor::TopLeft,
        )
        .with_rect(7.0, 1.0, config.divider)
        .color_raw::<UiBase>(config.chrome)
        .with_effect(RectEffect::VerticalGradient {
            secondary: config.chrome_shade,
        })
        .with_layer(UiLayer::Popups)
        .with_visible(false)
        .entity();
    resources.interface.start_menu = panel;

    tree.in_parent(panel, |tree| {
        tree.add_node()
            .window(Ab(vec2(14.0, 10.0)), Ab(vec2(200.0, 15.0)), Anchor::TopLeft)
            .with_text("HOLLOWAY & SONS", 11.0)
            .text_left()
            .color_raw::<UiBase>(config.text_faint)
            .entity();

        let list = tree
            .add_node()
            .window(
                Ab(vec2(6.0, 30.0)),
                Ab(vec2(220.0, rows * 34.0)),
                Anchor::TopLeft,
            )
            .flow(FlowDirection::Vertical, 0.0, 0.0)
            .entity();
        tree.in_parent(list, |tree| {
            for (index, program) in PROGRAMS.iter().enumerate() {
                let row = tree
                    .add_node()
                    .flow_child(Ab(vec2(220.0, 34.0)))
                    .with_rect(4.0, 0.0, config.transparent)
                    .color_raw::<UiBase>(config.transparent)
                    .color_raw::<UiHover>(config.row_hover)
                    .with_transition::<UiHover>(16.0, 8.0)
                    .with_interaction()
                    .with_cursor_icon(CursorIcon::Pointer)
                    .with_visible(false)
                    .entity();
                let label = tree.in_parent(row, |tree| {
                    tree.add_node()
                        .window(Ab(vec2(10.0, 2.0)), Ab(vec2(26.0, 30.0)), Anchor::TopLeft)
                        .with_icon(program.glyph, 15.0)
                        .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                        .color_raw::<UiBase>(config.brand)
                        .entity();
                    tree.add_node()
                        .window(Ab(vec2(42.0, 2.0)), Ab(vec2(168.0, 30.0)), Anchor::TopLeft)
                        .with_text(program.label, 13.4)
                        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                        .color_raw::<UiBase>(config.text_dim)
                        .entity()
                });
                tree.context_reborrow().commands.set(
                    row,
                    StartRow {
                        program: index,
                        slot: LabeledSlot { root: row, label },
                    },
                );
            }
        });

        resources.interface.start_divider = tree
            .add_node()
            .window(Ab(vec2(14.0, 0.0)), Ab(vec2(204.0, 1.0)), Anchor::TopLeft)
            .with_rect(0.0, 0.0, config.transparent)
            .color_raw::<UiBase>(config.divider)
            .entity();
        let (row, label) = build_menu_way(tree, config, material::TIMER, "sign off");
        resources.interface.start_signoff = row;
        resources.interface.start_signoff_label = label;
        resources.interface.start_signout =
            build_menu_way(tree, config, material::ARROW_BACK, "switch users").0;
    });
    tree.finish_subtree();
}

fn build_menu_way(
    tree: &mut UiTreeBuilder,
    config: &Config,
    glyph: IconGlyph,
    text: &'static str,
) -> (Entity, Entity) {
    let row = tree
        .add_node()
        .window(Ab(vec2(6.0, 0.0)), Ab(vec2(220.0, 30.0)), Anchor::TopLeft)
        .with_rect(4.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.transparent)
        .color_raw::<UiHover>(config.close_hover)
        .with_transition::<UiHover>(16.0, 8.0)
        .with_interaction()
        .with_cursor_icon(CursorIcon::Pointer)
        .entity();
    let label = tree.in_parent(row, |tree| {
        tree.add_node()
            .window(Ab(vec2(10.0, 0.0)), Ab(vec2(26.0, 30.0)), Anchor::TopLeft)
            .with_icon(glyph, 15.0)
            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
            .color_raw::<UiBase>(config.brand)
            .entity();
        tree.add_node()
            .window(Ab(vec2(42.0, 0.0)), Ab(vec2(168.0, 30.0)), Anchor::TopLeft)
            .with_text(text, 13.4)
            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
            .color_raw::<UiBase>(config.text_dim)
            .entity()
    });
    (row, label)
}

fn build_notes(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let content = frame_content(&ink.gathered.frames, program_index("notes"));
    let mut tree = UiTreeBuilder::from_parent(ink.build.reborrow(), content);
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.desktop_background)
        .with_effect(RectEffect::VerticalGradient {
            secondary: config.login_shade,
        })
        .with_visible(false)
        .entity();
    resources.interface.notes = root;

    tree.in_parent(root, |tree| {
        tree.add_node()
            .window(Ab(vec2(24.0, 18.0)), Ab(vec2(500.0, 20.0)), Anchor::TopLeft)
            .with_text("notes", 17.1)
            .text_left()
            .color_raw::<UiBase>(config.text)
            .entity();
        resources.interface.notes_count = tree
            .add_node()
            .window(Ab(vec2(24.0, 40.0)), Ab(vec2(560.0, 16.0)), Anchor::TopLeft)
            .with_text("", 12.8)
            .text_left()
            .color_raw::<UiBase>(config.text_faint)
            .entity();

        let wall = tree
            .add_node()
            .boundary(
                Ab(vec2(20.0, 74.0)),
                Rl(vec2(100.0, 100.0)) + Ab(vec2(-20.0, -20.0)),
            )
            .flow(FlowDirection::Horizontal, 0.0, 14.0)
            .flow_wrap()
            .entity();
        tree.in_parent(wall, |tree| {
            for note in 0..NOTES.len() {
                let card = tree
                    .add_node()
                    .flow_child(Ab(vec2(210.0, 150.0)))
                    .with_rect(2.0, 0.0, config.transparent)
                    .color_raw::<UiBase>(config.sticky)
                    .with_effect(RectEffect::VerticalGradient {
                        secondary: config.sticky_shade,
                    })
                    .with_visible(false)
                    .entity();
                let mut slot = (card, Entity::default(), Entity::default());
                tree.in_parent(card, |tree| {
                    slot.1 = tree
                        .add_node()
                        .window(Ab(vec2(14.0, 14.0)), Ab(vec2(182.0, 96.0)), Anchor::TopLeft)
                        .with_text("", 14.0)
                        .with_text_wrap()
                        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
                        .color_raw::<UiBase>(config.sticky_text)
                        .entity();
                    slot.2 = tree
                        .add_node()
                        .window(
                            Ab(vec2(14.0, 122.0)),
                            Ab(vec2(182.0, 14.0)),
                            Anchor::TopLeft,
                        )
                        .with_text("", 11.6)
                        .text_left()
                        .color_raw::<UiBase>(config.sticky_author)
                        .entity();
                });
                tree.context_reborrow().commands.set(
                    slot.0,
                    NoteCard {
                        note,
                        root: slot.0,
                        text: slot.1,
                        author: slot.2,
                    },
                );
            }
        });
    });
    tree.finish_subtree();
}

fn build_icon(
    tree: &mut UiTreeBuilder,
    config: &Config,
    at: Vec2,
    glyph: IconGlyph,
    label: &str,
) -> Entity {
    let root = tree
        .add_node()
        .window(Ab(at), Ab(vec2(84.0, 84.0)), Anchor::TopLeft)
        .with_rect(7.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.transparent)
        .color_raw::<UiHover>(config.icon_selected)
        .color_raw::<UiPressed>(config.row_pressed)
        .with_transition::<UiHover>(18.0, 9.0)
        .with_interaction()
        .with_cursor_icon(CursorIcon::Pointer)
        .entity();
    tree.in_parent(root, |tree| {
        let tile = tree
            .add_node()
            .window(Ab(vec2(22.0, 6.0)), Ab(vec2(40.0, 40.0)), Anchor::TopLeft)
            .with_rect(10.0, 1.0, config.icon_edge)
            .color_raw::<UiBase>(config.icon_tile)
            .with_effect(RectEffect::VerticalGradient {
                secondary: config.icon_tile_shade,
            })
            .entity();
        tree.in_parent(tile, |tree| {
            tree.add_node()
                .absolute_fill(0.0)
                .with_icon(glyph, 22.0)
                .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                .color_raw::<UiBase>(config.brand)
                .entity();
        });
        tree.add_node()
            .window(Ab(vec2(2.0, 54.0)), Ab(vec2(80.0, 26.0)), Anchor::TopLeft)
            .with_text(label, 12.8)
            .with_text_wrap()
            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Top)
            .color_raw::<UiBase>(config.text_dim)
            .entity();
    });
    root
}

fn build_board(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let content = frame_content(&ink.gathered.frames, program_index("board"));
    let mut tree = UiTreeBuilder::from_parent(ink.build.reborrow(), content);
    let root = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.board_background)
        .with_effect(RectEffect::VerticalGradient {
            secondary: config.sidebar_shade,
        })
        .with_visible(false)
        .entity();
    resources.interface.board = root;

    tree.in_parent(root, |tree| {
        tree.add_node()
            .window(Ab(vec2(24.0, 18.0)), Ab(vec2(400.0, 20.0)), Anchor::TopLeft)
            .with_text("what i have worked out", 17.1)
            .text_left()
            .color_raw::<UiBase>(config.text)
            .entity();
        resources.interface.board_progress = tree
            .add_node()
            .window(Ab(vec2(24.0, 40.0)), Ab(vec2(400.0, 16.0)), Anchor::TopLeft)
            .with_text("", 13.4)
            .text_left()
            .color_raw::<UiBase>(config.text_faint)
            .entity();
        let columns = tree
            .add_node()
            .boundary(
                Ab(vec2(20.0, 74.0)),
                Rl(vec2(100.0, 100.0)) + Ab(vec2(-20.0, -20.0)),
            )
            .flow(FlowDirection::Horizontal, 0.0, config.board_gutter)
            .entity();
        tree.in_parent(columns, |tree| {
            for (index, lead) in LEADS.iter().enumerate() {
                let column = tree
                    .add_node()
                    .flow_child(Ab(vec2(config.board_column_width, 0.0)))
                    .flow(FlowDirection::Vertical, 0.0, config.row_gutter)
                    .entity();
                tree.in_parent(column, |tree| {
                    let heading = tree
                        .add_node()
                        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 22.0)))
                        .with_text("", 14.6)
                        .text_left()
                        .color_raw::<UiBase>(config.text_dim)
                        .entity();
                    for note in 0..lead.notes.len() {
                        build_card(tree, config, lead_index(lead), note);
                    }
                    let footer = tree
                        .add_node()
                        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 18.0)))
                        .with_text("", 12.2)
                        .text_left()
                        .color_raw::<UiBase>(config.text_faint)
                        .entity();
                    tree.context_reborrow().commands.set(
                        column,
                        BoardColumn {
                            lead: index,
                            heading,
                            footer,
                        },
                    );
                });
            }
        });
    });
    tree.finish_subtree();
}

fn lead_index(lead: &crate::data::board::Lead) -> usize {
    LEADS
        .iter()
        .position(|entry| entry.id == lead.id)
        .unwrap_or_default()
}

fn build_card(tree: &mut UiTreeBuilder, config: &Config, lead: usize, note: usize) {
    let card = tree
        .add_node()
        .flow_child(Rl(vec2(100.0, 0.0)))
        .auto_size(AutoSizeMode::Height)
        .with_rect(3.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.card)
        .with_effect(RectEffect::VerticalGradient {
            secondary: config.card_shade,
        })
        .with_visible(false)
        .entity();
    let mut label = Entity::default();
    tree.in_parent(card, |tree| {
        label = tree
            .add_node()
            .fill_width()
            .auto_size(AutoSizeMode::Height)
            .auto_size_padding(vec2(9.0, 8.0))
            .with_text("", 13.4)
            .with_text_wrap()
            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Top)
            .color_raw::<UiBase>(config.card_text)
            .entity();
    });
    tree.context_reborrow()
        .commands
        .set(card, BoardCard { lead, note, label });
}

fn build_pointer(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    config: &Config,
) {
    let mut tree = UiTreeBuilder::new(ink.build.reborrow());
    ink.gathered.roots.push(tree.root_entity());
    let pointer = tree
        .add_node()
        .window(Ab(vec2(0.0, 0.0)), Ab(vec2(14.0, 20.0)), Anchor::TopLeft)
        .with_layer(UiLayer::Tooltips)
        .entity();
    resources.interface.pointer = pointer;
    tree.in_parent(pointer, |tree| {
        for row in 0..10 {
            let width = 2.0 + row as f32 * 1.1;
            tree.add_node()
                .window(
                    Ab(vec2(0.0, row as f32 * 1.6)),
                    Ab(vec2(width, 1.8)),
                    Anchor::TopLeft,
                )
                .with_rect(0.0, 0.0, config.transparent)
                .color_raw::<UiBase>(config.pointer)
                .entity();
        }
    });
    tree.finish();
}

fn build_marquee(interface: &mut Interface, config: &Config, tree: &mut UiTreeBuilder) {
    interface.marquee = tree
        .add_node()
        .window(
            Ab(vec2(0.0, 0.0)),
            Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, config.marquee_height)),
            Anchor::TopLeft,
        )
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.header_background)
        .with_effect(RectEffect::VerticalGradient {
            secondary: config.header_shade,
        })
        .entity();
    let band = interface.marquee;
    interface.marquee_text = tree.in_parent(band, |tree| {
        bottom_divider(tree, config);
        tree.add_node()
            .window(
                Ab(vec2(16.0, 0.0)),
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-32.0, config.marquee_height)),
                Anchor::TopLeft,
            )
            .with_text("", 12.2)
            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
            .color_raw::<UiBase>(config.text_dim)
            .entity()
    });
}

fn build_sidebar(
    interface: &mut Interface,
    script: &Script,
    config: &Config,
    tree: &mut UiTreeBuilder,
) {
    let (list_top, list_bottom) = sidebar_list_bounds(config);
    let inner = row_inner_width(config);

    let brand = tree
        .add_node()
        .window(
            Ab(vec2(0.0, 0.0)),
            Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, config.header_height)),
            Anchor::TopLeft,
        )
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.header_background)
        .with_effect(RectEffect::VerticalGradient {
            secondary: config.header_shade,
        })
        .entity();
    tree.in_parent(brand, |tree| {
        tree.add_node()
            .window(Ab(vec2(16.0, 9.0)), Ab(vec2(inner, 26.0)), Anchor::TopLeft)
            .with_text("iam", 22.0)
            .text_left()
            .color_raw::<UiBase>(config.brand)
            .entity();
        tree.add_node()
            .window(Ab(vec2(56.0, 18.0)), Ab(vec2(120.0, 14.0)), Anchor::TopLeft)
            .with_text("v0.9b", 12.2)
            .text_left()
            .color_raw::<UiBase>(config.text_faint)
            .entity();
        interface.night_label = tree
            .add_node()
            .window(Ab(vec2(16.0, 36.0)), Ab(vec2(inner, 14.0)), Anchor::TopLeft)
            .with_text("", 12.2)
            .text_left()
            .color_raw::<UiBase>(config.text_faint)
            .entity();
        bottom_divider(tree, config);
    });

    interface.offline_button = tree
        .add_node()
        .window(
            Ab(vec2(0.0, config.header_height)),
            Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, config.sidebar_toggle_height)),
            Anchor::TopLeft,
        )
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.toggle_base)
        .color_raw::<UiHover>(config.row_hover)
        .color_raw::<UiPressed>(config.row_pressed)
        .with_transition::<UiHover>(16.0, 8.0)
        .with_interaction()
        .with_cursor_icon(CursorIcon::Pointer)
        .entity();
    let offline_button = interface.offline_button;
    interface.offline_label = tree.in_parent(offline_button, |tree| {
        tree.add_node()
            .window(Ab(vec2(16.0, 8.0)), Ab(vec2(inner, 14.0)), Anchor::TopLeft)
            .with_text("+  SHOW OFFLINE", 12.2)
            .text_left()
            .color_raw::<UiBase>(config.text_faint)
            .entity()
    });

    let list_holder = tree
        .add_node()
        .boundary(
            Ab(vec2(0.0, list_top)),
            Rl(vec2(100.0, 100.0)) + Ab(vec2(0.0, -list_bottom)),
        )
        .entity();
    interface.sidebar_list =
        tree.in_parent(list_holder, |tree| tree.add_scroll_area_fill(8.0, 2.0));
    interface.sidebar_content = scroll_content(tree, interface.sidebar_list);

    let content = interface.sidebar_content;
    tree.in_parent(content, |tree| {
        interface.pending_heading = build_heading(tree, config);
        interface.pending_rows = (0..pending_capacity(script))
            .map(|_| build_row_slot(tree, config))
            .collect();
        interface.server_heading = build_heading(tree, config);
        interface.server_rows = (0..server_capacity(script))
            .map(|_| build_row_slot(tree, config))
            .collect();
        interface.channel_heading = build_heading(tree, config);
        interface.channel_rows = (0..channel_row_capacity(script))
            .map(|_| build_channel_row_slot(tree, config))
            .collect();
        interface.sidebar_groups = PRESENCE_GROUPS
            .iter()
            .map(|presence| SidebarGroup {
                presence: *presence,
                heading: build_heading(tree, config),
                rows: (0..group_capacity(script))
                    .map(|_| build_row_slot(tree, config))
                    .collect(),
            })
            .collect();
    });

    let footer_offset = config.sidebar_footer_height + config.sidebar_quit_height;
    let footer = tree
        .add_node()
        .window(
            Rl(vec2(0.0, 100.0)) + Ab(vec2(0.0, -footer_offset)),
            Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, config.sidebar_footer_height)),
            Anchor::TopLeft,
        )
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.header_background)
        .with_effect(RectEffect::VerticalGradient {
            secondary: config.header_shade,
        })
        .entity();
    tree.in_parent(footer, |tree| {
        top_divider(tree, config);
        interface.player_dot = tree
            .add_node()
            .window(Ab(vec2(16.0, 15.0)), Ab(vec2(8.0, 8.0)), Anchor::TopLeft)
            .with_rect(4.0, 0.0, config.transparent)
            .color_raw::<UiBase>(config.online)
            .entity();
        interface.player_handle = tree
            .add_node()
            .window(
                Ab(vec2(30.0, 10.0)),
                Ab(vec2(inner - 30.0, 17.0)),
                Anchor::TopLeft,
            )
            .with_text("", 15.9)
            .text_left()
            .color_raw::<UiBase>(config.text)
            .entity();
        interface.player_status = tree
            .add_node()
            .window(
                Ab(vec2(30.0, 28.0)),
                Ab(vec2(inner - 30.0, 14.0)),
                Anchor::TopLeft,
            )
            .with_text("", 12.2)
            .text_left()
            .color_raw::<UiBase>(config.text_faint)
            .entity();
    });
}

fn build_heading(tree: &mut UiTreeBuilder, config: &Config) -> Entity {
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, config.heading_height)))
        .with_text("", 11.6)
        .with_text_alignment(TextAlignment::Left, VerticalAlignment::Bottom)
        .color_raw::<UiBase>(config.text_faint)
        .with_visible(false)
        .entity()
}

fn build_row_slot(tree: &mut UiTreeBuilder, config: &Config) -> RowSlot {
    let avatar_size = config.avatar_size;
    let mut slot = RowSlot {
        root: tree
            .add_node()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, config.row_height)))
            .with_rect(6.0, 0.0, config.transparent)
            .color_raw::<UiBase>(config.row_base)
            .color_raw::<UiHover>(config.row_hover)
            .color_raw::<UiPressed>(config.row_pressed)
            .with_transition::<UiHover>(18.0, 9.0)
            .with_interaction()
            .with_cursor_icon(CursorIcon::Pointer)
            .with_visible(false)
            .entity(),
        ..RowSlot::default()
    };

    let root = slot.root;
    tree.in_parent(root, |tree| {
        slot.avatar = tree
            .add_node()
            .window(Ab(vec2(9.0, 9.0)), Ab(avatar_size), Anchor::TopLeft)
            .with_rect(avatar_size.x * 0.5, 0.0, config.transparent)
            .color_raw::<UiBase>(config.brand_dim)
            .with_effect(RectEffect::VerticalGradient {
                secondary: config.avatar_shade,
            })
            .with_clip()
            .entity();
        let avatar = slot.avatar;
        tree.in_parent(avatar, |tree| {
            slot.letter = tree
                .add_node()
                .absolute_fill(0.0)
                .with_text("", 18.3)
                .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                .color_raw::<UiBase>(config.text)
                .entity();
            let (head, body) = build_silhouette(tree, config);
            slot.silhouette_head = head;
            slot.silhouette_body = body;
        });
        slot.dot = tree
            .add_node()
            .window(
                Ab(vec2(9.0 + avatar_size.x - 11.0, 9.0 + avatar_size.y - 11.0)),
                Ab(vec2(12.0, 12.0)),
                Anchor::TopLeft,
            )
            .with_rect(6.0, 2.0, config.sidebar_background)
            .color_raw::<UiBase>(config.offline)
            .with_effect(RectEffect::RadialGlow {
                glow_color: config.online,
                intensity: config.dot_glow_strength,
            })
            .entity();
        slot.name = tree
            .add_node()
            .window(
                Ab(vec2(52.0, 10.0)),
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-86.0, 17.0)),
                Anchor::TopLeft,
            )
            .with_text("", 16.5)
            .text_left()
            .with_text_overflow(TextOverflow::Ellipsis)
            .color_raw::<UiBase>(config.text)
            .entity();
        slot.status = tree
            .add_node()
            .window(
                Ab(vec2(52.0, 28.0)),
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-86.0, 14.0)),
                Anchor::TopLeft,
            )
            .with_text("", 12.2)
            .text_left()
            .with_text_overflow(TextOverflow::Ellipsis)
            .color_raw::<UiBase>(config.text_faint)
            .entity();
        slot.badge = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-config.row_gutter - config.badge_min_width, 17.0)),
                Ab(vec2(config.badge_min_width, 18.0)),
                Anchor::TopLeft,
            )
            .with_rect(9.0, 0.0, config.transparent)
            .color_raw::<UiBase>(config.badge)
            .with_effect(RectEffect::RadialGlow {
                glow_color: config.badge,
                intensity: config.badge_glow_strength,
            })
            .with_visible(false)
            .entity();
        let badge = slot.badge;
        slot.badge_text = tree.in_parent(badge, |tree| {
            tree.add_node()
                .absolute_fill(0.0)
                .with_text("", 12.2)
                .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                .color_raw::<UiBase>(config.text)
                .entity()
        });
    });

    slot
}

fn build_silhouette(tree: &mut UiTreeBuilder, config: &Config) -> (Entity, Entity) {
    let size = config.avatar_size;
    let head_size = size.x * 0.34;
    let body_size = vec2(size.x * 0.56, size.y * 0.34);
    let head = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, size.y * 0.20)),
            Ab(vec2(head_size, head_size)),
            Anchor::TopCenter,
        )
        .with_rect(head_size * 0.5, 0.0, config.transparent)
        .color_raw::<UiBase>(config.silhouette_figure)
        .with_visible(false)
        .entity();
    let body = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -size.y * 0.16)),
            Ab(body_size),
            Anchor::BottomCenter,
        )
        .with_rect(body_size.x * 0.5, 0.0, config.transparent)
        .color_raw::<UiBase>(config.silhouette_figure)
        .with_visible(false)
        .entity();
    (head, body)
}

fn build_pane(
    interface: &mut Interface,
    script: &Script,
    config: &Config,
    tree: &mut UiTreeBuilder,
) -> Vec<(Entity, ChoiceSlot)> {
    let mut replies: Vec<(Entity, ChoiceSlot)> = Vec::new();
    let avatar_size = config.avatar_size;
    let header = tree
        .add_node()
        .window(
            Ab(vec2(0.0, 0.0)),
            Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, config.header_height)),
            Anchor::TopLeft,
        )
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.header_background)
        .with_effect(RectEffect::VerticalGradient {
            secondary: config.header_shade,
        })
        .entity();
    tree.in_parent(header, |tree| {
        interface.header_avatar = tree
            .add_node()
            .window(Ab(vec2(18.0, 12.0)), Ab(avatar_size), Anchor::TopLeft)
            .with_rect(avatar_size.x * 0.5, 0.0, config.transparent)
            .color_raw::<UiBase>(config.brand_dim)
            .with_clip()
            .entity();
        let avatar = interface.header_avatar;
        tree.in_parent(avatar, |tree| {
            interface.header_letter = tree
                .add_node()
                .absolute_fill(0.0)
                .with_text("", 18.3)
                .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                .color_raw::<UiBase>(config.text)
                .entity();
            let (head, body) = build_silhouette(tree, config);
            interface.header_silhouette_head = head;
            interface.header_silhouette_body = body;
        });
        interface.header_name = tree
            .add_node()
            .window(Ab(vec2(64.0, 11.0)), Ab(vec2(420.0, 19.0)), Anchor::TopLeft)
            .with_text("", 19.5)
            .text_left()
            .color_raw::<UiBase>(config.text)
            .entity();
        interface.header_status = tree
            .add_node()
            .window(Ab(vec2(64.0, 31.0)), Ab(vec2(620.0, 14.0)), Anchor::TopLeft)
            .with_text("", 13.4)
            .text_left()
            .with_text_overflow(TextOverflow::Ellipsis)
            .color_raw::<UiBase>(config.text_dim)
            .entity();
        bottom_divider(tree, config);
    });

    interface.log_holder = tree
        .add_node()
        .boundary(
            Ab(vec2(0.0, config.header_height)),
            Rl(vec2(100.0, 100.0)) + Ab(vec2(0.0, -config.header_height)),
        )
        .entity();
    let holder = interface.log_holder;
    interface.log_scroll = tree.in_parent(holder, |tree| {
        tree.add_scroll_area_fill(config.log_track_width, 8.0)
    });
    interface.log_content = scroll_content(tree, interface.log_scroll);

    interface.composer = tree
        .add_node()
        .window(
            Rl(vec2(0.0, 100.0)) + Ab(vec2(0.0, -config.choice_height)),
            Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, config.choice_height)),
            Anchor::TopLeft,
        )
        .flow(
            FlowDirection::Vertical,
            config.composer_padding,
            config.composer_spacing,
        )
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.composer_background)
        .entity();
    let composer = interface.composer;
    tree.in_parent(composer, |tree| {
        for slot in 0..choice_capacity(script) {
            let built = build_choice_slot(tree, config);
            replies.push((
                built.root,
                ChoiceSlot {
                    slot,
                    label: built.label,
                    choice: None,
                },
            ));
        }
        interface.placeholder_row = tree
            .add_node()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, config.choice_height)))
            .with_rect(8.0, 1.0, config.card_border)
            .color_raw::<UiBase>(config.transparent)
            .entity();
        let placeholder = interface.placeholder_row;
        interface.placeholder_label = tree.in_parent(placeholder, |tree| {
            tree.add_node()
                .window(
                    Ab(vec2(14.0, 0.0)),
                    Rl(vec2(100.0, 100.0)) + Ab(vec2(-28.0, 0.0)),
                    Anchor::TopLeft,
                )
                .with_text("", 14.6)
                .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                .with_text_overflow(TextOverflow::Ellipsis)
                .color_raw::<UiBase>(config.text_faint)
                .entity()
        });
    });

    interface.composer_divider = tree
        .add_node()
        .window(
            Rl(vec2(0.0, 100.0)) + Ab(vec2(0.0, -config.choice_height)),
            Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 1.0)),
            Anchor::TopLeft,
        )
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.divider)
        .entity();
    replies
}

fn build_channel_row_slot(tree: &mut UiTreeBuilder, config: &Config) -> RowSlot {
    let mut slot = RowSlot {
        root: tree
            .add_node()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, config.channel_row_height)))
            .with_rect(5.0, 0.0, config.transparent)
            .color_raw::<UiBase>(config.row_base)
            .color_raw::<UiHover>(config.row_hover)
            .color_raw::<UiPressed>(config.row_pressed)
            .with_transition::<UiHover>(18.0, 9.0)
            .with_interaction()
            .with_cursor_icon(CursorIcon::Pointer)
            .with_visible(false)
            .entity(),
        ..RowSlot::default()
    };

    let root = slot.root;
    let middle = (config.channel_row_height - 16.0) * 0.5;
    tree.in_parent(root, |tree| {
        slot.letter = tree
            .add_node()
            .window(
                Ab(vec2(14.0, middle)),
                Ab(vec2(14.0, 16.0)),
                Anchor::TopLeft,
            )
            .with_text("#", 17.1)
            .text_left()
            .color_raw::<UiBase>(config.text_faint)
            .entity();
        slot.name = tree
            .add_node()
            .window(
                Ab(vec2(30.0, middle)),
                Rl(vec2(100.0, 0.0)) + Ab(vec2(-66.0, 16.0)),
                Anchor::TopLeft,
            )
            .with_text("", 15.9)
            .text_left()
            .with_text_overflow(TextOverflow::Ellipsis)
            .color_raw::<UiBase>(config.text_dim)
            .entity();
        slot.badge = tree
            .add_node()
            .window(
                Rl(vec2(100.0, 0.0))
                    + Ab(vec2(
                        -config.row_gutter - config.badge_min_width,
                        (config.channel_row_height - 18.0) * 0.5,
                    )),
                Ab(vec2(config.badge_min_width, 18.0)),
                Anchor::TopLeft,
            )
            .with_rect(9.0, 0.0, config.transparent)
            .color_raw::<UiBase>(config.badge)
            .with_effect(RectEffect::RadialGlow {
                glow_color: config.badge,
                intensity: config.badge_glow_strength,
            })
            .with_visible(false)
            .entity();
        let badge = slot.badge;
        slot.badge_text = tree.in_parent(badge, |tree| {
            tree.add_node()
                .absolute_fill(0.0)
                .with_text("", 12.2)
                .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
                .color_raw::<UiBase>(config.text)
                .entity()
        });
    });

    slot
}

fn build_choice_slot(tree: &mut UiTreeBuilder, config: &Config) -> LabeledSlot {
    let root = tree
        .add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, config.choice_height)))
        .with_rect(8.0, 1.0, config.card_border)
        .color_raw::<UiBase>(config.choice_base)
        .color_raw::<UiHover>(config.choice_hover)
        .color_raw::<UiPressed>(config.choice_pressed)
        .with_transition::<UiHover>(18.0, 9.0)
        .with_interaction()
        .with_cursor_icon(CursorIcon::Pointer)
        .with_visible(false)
        .entity();
    let label = tree.in_parent(root, |tree| {
        tree.add_node()
            .window(
                Ab(vec2(14.0, 0.0)),
                Rl(vec2(100.0, 100.0)) + Ab(vec2(-28.0, 0.0)),
                Anchor::TopLeft,
            )
            .with_text("", 15.9)
            .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
            .with_text_overflow(TextOverflow::Ellipsis)
            .color_raw::<UiBase>(config.text)
            .entity()
    });
    LabeledSlot { root, label }
}

fn scroll_content(tree: &mut UiTreeBuilder, scroll: Entity) -> Entity {
    let content = widget::<UiScrollAreaData>(tree.context_reborrow().commands, scroll)
        .map(|data| data.content_entity);
    content.unwrap_or(scroll)
}

fn top_divider(tree: &mut UiTreeBuilder, config: &Config) {
    tree.add_node()
        .window(
            Ab(vec2(0.0, 0.0)),
            Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 1.0)),
            Anchor::TopLeft,
        )
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.divider)
        .entity();
}

fn bottom_divider(tree: &mut UiTreeBuilder, config: &Config) {
    tree.add_node()
        .window(
            Rl(vec2(0.0, 100.0)) + Ab(vec2(0.0, -1.0)),
            Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 1.0)),
            Anchor::TopLeft,
        )
        .with_rect(0.0, 0.0, config.transparent)
        .color_raw::<UiBase>(config.divider)
        .entity();
}
