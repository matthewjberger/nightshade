use crate::components::RowSlot;
use crate::data::SCRIPT;
use crate::data::machine::{Intro, Screen, Screensaver};
use crate::data::pipes::Segment;
use crate::data::script::{Fact, Presence, Script};
use nightshade::prelude::Entity;
use nightshade::prelude::Query;
use nightshade::prelude::{Vec2, Vec3, vec2, vec3};
use nightshade_ui::components::{UiLayoutNode, UiNodeContent};

pub struct Ink<'borrow, 'commands, 'nodes, 'contents, 'node, 'content> {
    pub build: nightshade_ui::commands::builder::UiBuildContext<'borrow, 'commands>,
    pub nodes: Query<'nodes, &'node mut UiLayoutNode>,
    pub contents: Query<'contents, &'content mut UiNodeContent>,
    pub gathered: &'borrow mut crate::systems::desk::Gathered,
}

pub const PLAY: usize = 1 << 20;

pub const SIGN_OFF: usize = usize::MAX;

pub fn choice_capacity(script: &Script) -> usize {
    let said = script
        .contacts
        .iter()
        .flat_map(|definition| {
            let goodnight = usize::from(definition.id == crate::data::script::HOST);
            definition
                .nodes
                .iter()
                .map(move |node| node.choices.len() + goodnight)
        })
        .max()
        .unwrap_or_default();
    let posted = script
        .servers
        .iter()
        .flat_map(|server| server.channels.iter())
        .map(|channel| {
            channel.questions.len()
                + crate::data::games::POSTED
                    .iter()
                    .filter(|posted| posted.room == channel.name)
                    .count()
        })
        .max()
        .unwrap_or_default();
    said.max(posted).max(1)
}

pub fn group_capacity(script: &Script) -> usize {
    script.contacts.len()
}

pub fn server_capacity(script: &Script) -> usize {
    script.servers.len()
}

pub fn pending_capacity(script: &Script) -> usize {
    script
        .contacts
        .iter()
        .filter(|definition| !definition.listed)
        .count()
}

pub fn channel_row_capacity(script: &Script) -> usize {
    script
        .servers
        .iter()
        .map(|server| server.channels.len())
        .max()
        .unwrap_or_default()
}

pub struct Desk {
    pub screen: Screen,
    pub store_at: Option<usize>,
    pub store_offered: Vec<usize>,
    pub store_typed: String,
    pub store_focused: bool,
    pub place: usize,
    pub looked_at: Option<usize>,
    pub case_file: usize,
    pub lamp_moves: usize,
    pub lamps_done: usize,
    pub menu_open: bool,
    pub playing: Option<usize>,
    pub board_won: bool,
}

impl Default for Desk {
    fn default() -> Self {
        Self {
            screen: Screen::Login,
            store_at: None,
            store_offered: Vec::new(),
            store_typed: String::new(),
            store_focused: false,
            place: 0,
            looked_at: None,
            case_file: 0,
            lamp_moves: 0,
            lamps_done: 0,
            menu_open: false,
            playing: None,
            board_won: false,
        }
    }
}

pub struct Boot {
    pub intro: Intro,
    pub intro_elapsed: f32,
    pub screensaver: Screensaver,
    pub screensaver_elapsed: f32,
    pub pipes: Vec<Segment>,
    pub pipes_rng: fastrand::Rng,
    pub pipes_heading: Vec3,
    pub pipes_enamel: u32,
    pub pipes_grown: f32,
    pub night_waiting: bool,
    pub took_the_screen: bool,
}

impl Default for Boot {
    fn default() -> Self {
        Self {
            intro: Intro::Waiting,
            intro_elapsed: 0.0,
            screensaver: Screensaver::Idle,
            screensaver_elapsed: 0.0,
            pipes: Vec::new(),
            pipes_rng: fastrand::Rng::with_seed(1),
            pipes_heading: vec3(1.0, 0.0, 0.0),
            pipes_enamel: 0,
            pipes_grown: 0.0,
            night_waiting: false,
            took_the_screen: false,
        }
    }
}

pub struct Login {
    pub settings_open: bool,
    pub typed: String,
    pub refusal: String,
    pub caret_phase: f32,
    pub caret_lit: bool,
}

impl Default for Login {
    fn default() -> Self {
        Self {
            settings_open: false,
            typed: String::new(),
            refusal: String::new(),
            caret_phase: 0.0,
            caret_lit: true,
        }
    }
}

pub struct Browser {
    pub browsing: Option<usize>,
    pub address: String,
    pub address_focused: bool,
    pub page_across: f32,
    pub loading: f32,
    pub load_wants: f32,
    pub history: Vec<usize>,
    pub history_at: usize,
    pub folder_open: bool,
    pub page_typed: String,
    pub page_answered: Option<usize>,
    pub page_refused: bool,
    pub not_found: bool,
}

impl Default for Browser {
    fn default() -> Self {
        Self {
            browsing: None,
            address: String::new(),
            address_focused: false,
            page_across: 0.0,
            loading: 0.0,
            load_wants: 0.0,
            history: Vec::new(),
            history_at: 0,
            folder_open: false,
            page_typed: String::new(),
            page_answered: None,
            page_refused: false,
            not_found: false,
        }
    }
}

pub struct Corner {
    pub revealed: Vec<Fact>,
    pub ticked: Vec<Fact>,
    pub notices: Vec<Notice>,
    pub notices_open: bool,
    pub notices_at: usize,
    pub notices_slide: f32,
    pub toasts: Vec<Toast>,
}

impl Default for Corner {
    fn default() -> Self {
        Self {
            revealed: Vec::new(),
            ticked: Vec::new(),
            notices: Vec::new(),
            notices_open: false,
            notices_at: 0,
            notices_slide: 0.0,
            toasts: Vec::new(),
        }
    }
}

pub struct Glass {
    pub alarm: f32,
    pub shutoff: f32,
    pub lens: f32,
    pub lens_at: Vec2,
}

impl Default for Glass {
    fn default() -> Self {
        Self {
            alarm: 0.0,
            shutoff: 0.0,
            lens: 0.0,
            lens_at: vec2(0.5, 0.5),
        }
    }
}
pub struct AppResources {
    pub script: &'static Script,
    pub desk: Desk,
    pub boot: Boot,
    pub login: Login,
    pub browser: Browser,
    pub corner: Corner,
    pub glass: Glass,
    pub session: Session,
    pub interface: Interface,
    pub redraw: Redraw,
    pub actions: Vec<Action>,
    pub autoplay: crate::data::autoplay::Autoplay,
}

impl Default for AppResources {
    fn default() -> Self {
        Self {
            script: &SCRIPT,
            desk: Desk::default(),
            boot: Boot::default(),
            login: Login::default(),
            browser: Browser::default(),
            corner: Corner::default(),
            glass: Glass::default(),
            session: Session::default(),
            interface: Interface::default(),
            redraw: redraw_everything(),
            actions: Vec::new(),
            autoplay: crate::data::autoplay::Autoplay::default(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Cue {
    Message,
    Invite,
    Boot,
    Refused,
}

#[derive(Clone, Copy, Debug)]
pub enum Action {
    ToggleProgram(usize),
    MinimizeProgram(usize),
    CloseProgram(usize),
    SignOut,
    SignOff,
    SignIn,
    SelectAccount(usize),
    SizeWindow(usize, crate::components::WindowMode),
    PressBoard(usize),
    PressLamp(usize, usize),
    OpenCase(usize),
    ToggleSettings,
    WantEnding(usize),
    StartAutoplay,
    Install(usize),
    Upload,
    OpenStore(&'static str),
    GoPlace(usize),
    LookAt(usize),
    OpenFile(usize),
    NextLamps,
    CloseGame,
    OpenThread(Entity),
    OpenServer(Entity),
    OpenChannel(Entity),
    Reply(Entity),
    AcceptInvite(Entity),
    Ask(Entity),
    ToggleOffline,
    Quit,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    #[default]
    Nothing,
    Thread(Entity),
    Server(Entity),
}

#[derive(Clone, Copy)]
pub enum ToastTarget {
    Thread(Entity),
    Room { server: Entity, channel: Entity },
    Store(&'static str),
    Nothing,
}

pub struct Notice {
    pub who: String,
    pub text: String,
    pub stamp: String,
}

pub struct Toast {
    pub who: String,
    pub text: String,
    pub opens: ToastTarget,
    pub age: f32,
}

#[derive(Default)]
pub struct Session {
    pub view: View,
    pub facts: Vec<Fact>,
    pub remarked: Vec<&'static str>,
    pub handle: &'static str,
    pub night: usize,
    pub show_offline: bool,
    pub terminated: bool,
    pub ending: crate::data::endings::Ending,
    pub marquee_phase: f32,
    pub finished: bool,
    pub held_open: bool,
    pub ambience_started: bool,
    pub room_sound: Option<Entity>,
    pub room_settled: bool,
    pub hum_sound: Option<Entity>,
    pub hum_retiring: Option<Entity>,
    pub hum_seconds: f32,
    pub hum_elapsed: f32,
    pub hum_due: f32,
    pub cues: Vec<Cue>,
    pub typing_shown: bool,
    pub dots_phase: f32,
    pub clock_seconds: f32,
    pub last_bottom: f32,
    pub pin_log: bool,
}

pub fn knows(session: &Session, fact: Fact) -> bool {
    session.facts.contains(&fact)
}

pub fn establish(session: &mut Session, fact: Fact) -> bool {
    if knows(session, fact) {
        return false;
    }
    session.facts.push(fact);
    true
}

pub fn learn(resources: &mut AppResources, facts: &[Fact]) {
    for fact in facts {
        if establish(&mut resources.session, fact) {
            resources.redraw.sidebar = true;
        }
    }
}

pub fn open_thread(session: &Session) -> Option<Entity> {
    match session.view {
        View::Thread(entity) => Some(entity),
        _ => None,
    }
}

pub fn open_server(session: &Session) -> Option<Entity> {
    match session.view {
        View::Server(entity) => Some(entity),
        _ => None,
    }
}

#[derive(Default, Clone)]
pub struct SidebarGroup {
    pub presence: Presence,
    pub heading: Entity,
    pub rows: Vec<RowSlot>,
}

#[derive(Default)]
pub struct Interface {
    pub shell: Entity,
    pub login: Entity,
    pub password_box: Entity,
    pub password_field: Entity,
    pub sign_in: Entity,
    pub settings_cog: Entity,
    pub settings_panel: Entity,
    pub settings_start: Entity,

    pub refusal: Entity,
    pub sticky_note: Entity,
    pub desktop: Entity,
    pub desktop_user: Entity,
    pub desktop_note: Entity,
    pub tonight: Entity,
    pub tonight_heading: Entity,
    pub tonight_done: Entity,
    pub taskbar_calendar: Entity,
    pub taskbar_clock: Entity,
    pub taskbar_date: Entity,
    pub taskbar_start: Entity,
    pub start_menu: Entity,
    pub start_signout: Entity,
    pub start_signoff: Entity,
    pub start_signoff_label: Entity,
    pub start_divider: Entity,
    pub sessions: Entity,
    pub sessions_total: Entity,

    pub waiting: Entity,
    pub waiting_text: Entity,

    pub boot: Entity,
    pub boot_text: Entity,

    pub over: Entity,
    pub over_heading: Entity,
    pub over_closing: Entity,
    pub over_verdict: Entity,
    pub over_session: Entity,

    pub lights: Entity,
    pub lights_on: Entity,
    pub lights_on_says: Entity,
    pub case: Entity,
    pub case_body: Entity,
    pub case_title: Entity,
    pub office: Entity,
    pub office_name: Entity,
    pub office_body: Entity,
    pub office_look: Entity,
    pub notices_panel: Entity,
    pub notices_count: Entity,
    pub notices_more: Entity,
    pub player_dot: Entity,
    pub terminated: Entity,
    pub lights_moves: Entity,
    pub lights_says: Entity,
    pub lights_count: Entity,
    pub lights_again: Entity,

    pub occupied: Entity,
    pub occupied_handle: Entity,
    pub occupied_body: Entity,

    pub browser: Entity,
    pub browser_back: Entity,
    pub browser_forward: Entity,
    pub browser_reload: Entity,
    pub browser_address: Entity,
    pub browser_caret: Entity,
    pub browser_spinner: Entity,
    pub browser_progress: Entity,
    pub browser_tab: Entity,
    pub botterfly: Entity,
    pub botterfly_count: Entity,
    pub botterfly_go: Entity,
    pub store: Entity,
    pub store_empty: Entity,
    pub store_field: Entity,
    pub store_caret: Entity,
    pub browser_body: Entity,
    pub browser_page: Entity,
    pub browser_band: Entity,
    pub browser_status: Entity,
    pub browser_folder: Entity,
    pub browser_folder_panel: Entity,
    pub browser_prompt: Entity,
    pub browser_prompt_label: Entity,
    pub browser_prompt_field: Entity,
    pub browser_prompt_caret: Entity,
    pub browser_prompt_go: Entity,

    pub files: Entity,
    pub file_name: Entity,
    pub file_stamp: Entity,
    pub file_body: Entity,
    pub files_count: Entity,

    pub cross: Entity,
    pub cross_summary: Entity,
    pub notes: Entity,
    pub notes_count: Entity,
    pub board: Entity,
    pub board_progress: Entity,
    pub pointer: Entity,
    pub sidebar_list: Entity,
    pub sidebar_content: Entity,
    pub pending_heading: Entity,
    pub pending_rows: Vec<RowSlot>,
    pub server_heading: Entity,
    pub server_rows: Vec<RowSlot>,
    pub sidebar_groups: Vec<SidebarGroup>,
    pub offline_button: Entity,
    pub offline_label: Entity,
    pub night_label: Entity,
    pub player_handle: Entity,
    pub player_status: Entity,
    pub marquee: Entity,
    pub marquee_text: Entity,
    pub quit_button: Entity,
    pub header_avatar: Entity,
    pub header_letter: Entity,
    pub header_silhouette_head: Entity,
    pub header_silhouette_body: Entity,
    pub header_name: Entity,
    pub header_status: Entity,
    pub channel_heading: Entity,
    pub channel_rows: Vec<RowSlot>,
    pub log_holder: Entity,
    pub log_scroll: Entity,
    pub log_content: Entity,
    pub typing_bubble: Entity,
    pub typing_dots: Entity,
    pub game_panel: Entity,
    pub game_title: Entity,
    pub game_blurb: Entity,
    pub game_prize: Entity,
    pub game_close: Entity,
    pub composer: Entity,
    pub composer_divider: Entity,
    pub placeholder_row: Entity,
    pub placeholder_label: Entity,
}

#[derive(Default)]
pub struct Redraw {
    pub machine: bool,
    pub sidebar: bool,
    pub header: bool,
    pub composer: bool,
}

pub fn redraw_everything() -> Redraw {
    Redraw {
        machine: true,
        sidebar: true,
        header: true,
        composer: true,
    }
}
