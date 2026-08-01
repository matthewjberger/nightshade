use crate::contract::*;
use std::sync::{Arc, Mutex};
use steamworks::{Client, SteamId};

pub struct SteamResources {
    pub(crate) client: Option<Client>,
    pub initialized: bool,
    pub user_name: String,
    pub user_id: Option<SteamId>,
    pub app_id: u32,
    pub stats_received: bool,
    pub achievements: Vec<AchievementInfo>,
    pub stats: Vec<StatInfo>,
    pub friends: Vec<FriendInfo>,
    pub initialization_error: Option<String>,
    pub pending_session_requests: Arc<Mutex<Vec<PendingSessionRequest>>>,
    pub received_messages: Vec<NetworkMessage>,
    pub active_sessions: Vec<ActiveSession>,
    pub(crate) networking_callbacks_registered: bool,
}

impl Default for SteamResources {
    fn default() -> Self {
        Self {
            client: None,
            initialized: false,
            user_name: String::new(),
            user_id: None,
            app_id: 0,
            stats_received: false,
            achievements: Vec::new(),
            stats: Vec::new(),
            friends: Vec::new(),
            initialization_error: None,
            pending_session_requests: Arc::new(Mutex::new(Vec::new())),
            received_messages: Vec::new(),
            active_sessions: Vec::new(),
            networking_callbacks_registered: false,
        }
    }
}
