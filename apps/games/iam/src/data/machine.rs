use crate::data::script::{Fact, Script};

pub fn is_guest(signed_in: Option<usize>, screen: Screen) -> bool {
    screen == Screen::Desktop && signed_in == Some(index_of("guest"))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Login,
    Desktop,
    Over,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Intro {
    #[default]
    Waiting,
    Booting,
    Started,
    Playing,
    ToBlack,
    FromBlack,
    Done,
}

pub const TO_BLACK_SECONDS: f32 = 0.55;
pub const FROM_BLACK_SECONDS: f32 = 0.85;

pub const HOLD_BOOTING: f32 = 2.6;
pub const HOLD_LOGO: f32 = 2.0;

pub const HOLD_STARTED: f32 = 1.9;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Screensaver {
    #[default]
    Idle,
    Rising,
    Running,
    Falling,
}

pub const PIPES_RISE_SECONDS: f32 = 1.1;
pub const PIPES_RUN_SECONDS: f32 = 6.0;
pub const PIPES_HOLD_SECONDS: f32 = 1.2;
pub const PIPES_FALL_SECONDS: f32 = 0.9;

pub struct Account {
    pub id: &'static str,
    pub display_name: &'static str,
    pub handle: &'static str,
    pub note: &'static str,
    pub password: &'static str,
    pub understands: &'static [Fact],
    pub locked_note: &'static str,
    pub archive: Option<&'static Script>,
}

pub const STICKY_NOTE: &str = "guest / guest\nleave it, they reuse the desk\n- r";

pub const ACCOUNTS: &[Account] = &[
    Account {
        id: "guest",
        display_name: "Guest",
        handle: "newuser_41",
        note: "provisioned automatically",
        password: "guest",
        understands: &[],
        locked_note: "",
        archive: None,
    },
    Account {
        id: "r_mears",
        display_name: "r.mears",
        handle: "newuser_40",
        note: "signed in 3:07 AM",
        password: "8815",
        understands: &["knows_the_count_was_wrong"],
        locked_note: "password hint: how many of us are there",
        archive: None,
    },
    Account {
        id: "d_okonkwo",
        display_name: "d.okonkwo",
        handle: "newuser_39",
        note: "signed in 3:07 AM",
        password: "zbguen",
        understands: &["knows_the_name_is_rotated"],
        locked_note: "password hint: the toolbox will read it for you",
        archive: None,
    },
    Account {
        id: "j_alderete",
        display_name: "j.alderete",
        handle: "newuser_38",
        note: "signed in 3:07 AM",
        password: "eleven",
        understands: &["knows_how_many_lines"],
        locked_note: "password hint: sundial posted, and four were kept",
        archive: None,
    },
];

pub fn index_of(id: &str) -> usize {
    ACCOUNTS
        .iter()
        .position(|account| account.id == id)
        .unwrap_or_default()
}

pub fn opens(account: &Account, typed: &str) -> bool {
    typed.trim().eq_ignore_ascii_case(account.password)
}
