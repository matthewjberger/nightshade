use steamworks::{FriendState, SteamId};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PersonaState {
    Offline,
    Online,
    Busy,
    Away,
    Snooze,
    LookingToTrade,
    LookingToPlay,
}

impl From<FriendState> for PersonaState {
    fn from(state: FriendState) -> Self {
        match state {
            FriendState::Offline => PersonaState::Offline,
            FriendState::Online => PersonaState::Online,
            FriendState::Busy => PersonaState::Busy,
            FriendState::Away => PersonaState::Away,
            FriendState::Snooze => PersonaState::Snooze,
            FriendState::LookingToTrade => PersonaState::LookingToTrade,
            FriendState::LookingToPlay => PersonaState::LookingToPlay,
            FriendState::Invisible => PersonaState::Offline,
        }
    }
}

impl PersonaState {
    pub fn as_str(&self) -> &'static str {
        match self {
            PersonaState::Offline => "Offline",
            PersonaState::Online => "Online",
            PersonaState::Busy => "Busy",
            PersonaState::Away => "Away",
            PersonaState::Snooze => "Snooze",
            PersonaState::LookingToTrade => "Looking to Trade",
            PersonaState::LookingToPlay => "Looking to Play",
        }
    }

    pub fn is_online(&self) -> bool {
        !matches!(self, PersonaState::Offline)
    }
}

#[derive(Clone)]
pub struct AchievementInfo {
    pub api_name: String,
    pub achieved: bool,
}

#[derive(Clone)]
pub struct StatInfo {
    pub api_name: String,
    pub value: StatValue,
}

#[derive(Clone, Copy)]
pub enum StatValue {
    Int(i32),
    Float(f32),
}

#[derive(Clone)]
pub struct FriendInfo {
    pub name: String,
    pub steam_id: steamworks::SteamId,
    pub state: PersonaState,
}

#[derive(Clone)]
pub struct NetworkMessage {
    pub sender_name: String,
    pub sender_id: SteamId,
    pub data: Vec<u8>,
    pub channel: u32,
    pub is_outgoing: bool,
}

#[derive(Clone)]
pub struct PendingSessionRequest {
    pub remote_id: SteamId,
    pub remote_name: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SessionState {
    #[default]
    None,
    Connecting,
    Connected,
    ClosedByPeer,
    ProblemDetected,
    Failed,
}

#[derive(Clone)]
pub struct ActiveSession {
    pub steam_id: SteamId,
    pub name: String,
    pub state: SessionState,
}
