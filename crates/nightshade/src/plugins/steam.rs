//! Steamworks integration for achievements, stats, friends, and networking.
//!
//! Access Steam platform features through [`SteamResources`]:
//!
//! - Achievements: Unlock, clear, and query achievement status
//! - Stats: Track player statistics (int and float values)
//! - Friends: List friends and their online status
//! - Networking: Peer-to-peer messaging via Steam relay
//! - Rich Presence: Show custom status in friends list
//! - Overlays: Open Steam overlay dialogs
//!
//! Requires the `steam` feature and Steam client running.
//!
//! # Initialization
//!
//! Initialize Steam in your game's startup:
//!
//! ```ignore
//! fn initialize(&mut self, world: &mut World) {
//!     if let Err(error) = steam_initialize(&mut world.expect_resource_mut::<crate::plugins::steam::SteamResources>()) {
//!         eprintln!("Steam initialization failed: {error}");
//!         // Game can still run without Steam
//!     }
//!
//!     // Request stats/achievements from Steam servers
//!     steam_request_stats(&mut world.expect_resource_mut::<crate::plugins::steam::SteamResources>());
//! }
//! ```
//!
//! # Running Callbacks
//!
//! Call `steam_run_callbacks` each frame to process Steam events:
//!
//! ```ignore
//! fn run_systems(&mut self, world: &mut World) {
//!     steam_run_callbacks(&world.expect_resource_mut::<crate::plugins::steam::SteamResources>());
//! }
//! ```
//!
//! # Achievements
//!
//! Unlock and query achievements:
//!
//! ```ignore
//! // Unlock an achievement
//! steam_unlock_achievement(&world.expect_resource_mut::<crate::plugins::steam::SteamResources>(), "ACH_FIRST_KILL")?;
//! steam_store_stats(&world.expect_resource_mut::<crate::plugins::steam::SteamResources>())?;  // Sync to Steam
//!
//! // Check achievement status
//! steam_refresh_achievements(&mut world.expect_resource_mut::<crate::plugins::steam::SteamResources>());
//! for achievement in &world.expect_resource_mut::<crate::plugins::steam::SteamResources>().achievements {
//!     println!("{}: {}", achievement.api_name, achievement.achieved);
//! }
//!
//! // Clear achievement (for testing)
//! steam_clear_achievement(&world.expect_resource_mut::<crate::plugins::steam::SteamResources>(), "ACH_FIRST_KILL")?;
//! ```
//!
//! # Stats
//!
//! Track integer and float statistics:
//!
//! ```ignore
//! // Set stats
//! steam_set_stat_int(&world.expect_resource_mut::<crate::plugins::steam::SteamResources>(), "total_kills", 42)?;
//! steam_set_stat_float(&world.expect_resource_mut::<crate::plugins::steam::SteamResources>(), "play_time_hours", 10.5)?;
//! steam_store_stats(&world.expect_resource_mut::<crate::plugins::steam::SteamResources>())?;  // Sync to Steam
//!
//! // Read stats
//! steam_refresh_stats(&mut world.expect_resource_mut::<crate::plugins::steam::SteamResources>(), &["total_kills", "play_time_hours"]);
//! for stat in &world.expect_resource_mut::<crate::plugins::steam::SteamResources>().stats {
//!     match stat.value {
//!         StatValue::Int(value) => println!("{}: {}", stat.api_name, value),
//!         StatValue::Float(value) => println!("{}: {}", stat.api_name, value),
//!     }
//! }
//!
//! // Reset all stats (for testing)
//! steam_reset_all_stats(&world.expect_resource_mut::<crate::plugins::steam::SteamResources>(), true)?;  // true = also reset achievements
//! ```
//!
//! # Friends List
//!
//! Access the player's Steam friends:
//!
//! ```ignore
//! steam_refresh_friends(&mut world.expect_resource_mut::<crate::plugins::steam::SteamResources>());
//!
//! for friend in &world.expect_resource_mut::<crate::plugins::steam::SteamResources>().friends {
//!     let status = friend.state.as_str();  // "Online", "Away", etc.
//!     println!("{}: {}", friend.name, status);
//!
//!     if friend.state.is_online() {
//!         // Friend is available to play
//!     }
//! }
//! ```
//!
//! # P2P Networking
//!
//! Send messages to other Steam users via Steam's relay network:
//!
//! ```ignore
//! // Setup networking callbacks (once)
//! steam_setup_networking_callbacks(&mut world.expect_resource_mut::<crate::plugins::steam::SteamResources>());
//!
//! // Send message to a friend
//! let target_id = friend.steam_id;
//! steam_send_message(
//!     &mut world.expect_resource_mut::<crate::plugins::steam::SteamResources>(),
//!     target_id,
//!     b"Hello!",
//!     0,      // Channel
//!     true,   // Reliable
//! )?;
//!
//! // Receive messages each frame
//! steam_receive_messages(&mut world.expect_resource_mut::<crate::plugins::steam::SteamResources>(), 0, 100);  // Channel 0, max 100 messages
//!
//! for message in &world.expect_resource_mut::<crate::plugins::steam::SteamResources>().received_messages {
//!     if !message.is_outgoing {
//!         let text = String::from_utf8_lossy(&message.data);
//!         println!("From {}: {}", message.sender_name, text);
//!     }
//! }
//! steam_clear_messages(&mut world.expect_resource_mut::<crate::plugins::steam::SteamResources>());
//!
//! // Process incoming session requests
//! steam_process_pending_requests(&mut world.expect_resource_mut::<crate::plugins::steam::SteamResources>());
//!
//! // Check connection state
//! let state = steam_get_session_state(&world.expect_resource_mut::<crate::plugins::steam::SteamResources>(), target_id);
//! if state == SessionState::Connected {
//!     // Peer is connected
//! }
//!
//! // Close session
//! steam_close_session(&mut world.expect_resource_mut::<crate::plugins::steam::SteamResources>(), target_id);
//! ```
//!
//! # Rich Presence
//!
//! Show custom status in the Steam friends list:
//!
//! ```ignore
//! // Set status
//! steam_set_rich_presence(&world.expect_resource_mut::<crate::plugins::steam::SteamResources>(), "status", "In Battle - Wave 5");
//! steam_set_rich_presence(&world.expect_resource_mut::<crate::plugins::steam::SteamResources>(), "steam_display", "#StatusInGame");
//!
//! // Clear all rich presence
//! steam_clear_rich_presence(&world.expect_resource_mut::<crate::plugins::steam::SteamResources>());
//! ```
//!
//! # Steam Overlay
//!
//! Open Steam overlay dialogs:
//!
//! ```ignore
//! // Open invite dialog
//! steam_open_invite_dialog(&world.expect_resource_mut::<crate::plugins::steam::SteamResources>(), "+connect_lobby 12345");
//!
//! // Open chat with a user
//! steam_open_overlay_to_user(&world.expect_resource_mut::<crate::plugins::steam::SteamResources>(), "chat", friend.steam_id);
//! ```
//!
//! # User Information
//!
//! Access the current user's Steam info:
//!
//! ```ignore
//! let steam = &world.expect_resource_mut::<crate::plugins::steam::SteamResources>();
//! if steam_is_initialized(steam) {
//!     println!("User: {}", steam.user_name);
//!     println!("Steam ID: {:?}", steam.user_id);
//!     println!("App ID: {}", steam.app_id);
//! }
//! ```
//!
//! # Session States
//!
//! P2P connection states:
//!
//! | State | Description |
//! |-------|-------------|
//! | `None` | No session |
//! | `Connecting` | Connection in progress |
//! | `Connected` | Actively connected |
//! | `ClosedByPeer` | Remote closed the connection |
//! | `ProblemDetected` | Connection issue detected |
//! | `Failed` | Connection failed |
//!
//! # Error Handling
//!
//! Most of these functions return `Result<(), String>`. Always check for errors:
//!
//! ```ignore
//! if let Err(error) = steam_unlock_achievement(&world.expect_resource_mut::<crate::plugins::steam::SteamResources>(), "ACH_WIN") {
//!     eprintln!("Failed to unlock achievement: {error}");
//! }
//! ```
//!
//! # Testing Without Steam
//!
//! If Steam is not running, `steam_initialize` will fail gracefully. Check
//! `steam_is_initialized` before calling other functions to avoid errors.

use std::sync::{Arc, Mutex};
use steamworks::{
    Client, FriendFlags, FriendState, SteamId,
    networking_messages::NetworkingMessages,
    networking_types::{NetworkingIdentity, SendFlags},
};

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

pub struct SteamResources {
    client: Option<Client>,
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
    networking_callbacks_registered: bool,
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

pub fn steam_initialize(steam: &mut SteamResources) -> Result<(), String> {
    match Client::init() {
        Ok(client) => {
            let user = client.user();
            let utils = client.utils();
            let friends_interface = client.friends();

            steam.user_id = Some(user.steam_id());
            steam.user_name = friends_interface.name();
            steam.app_id = utils.app_id().0;

            steam.client = Some(client);
            steam.initialized = true;

            tracing::info!(
                "Steam initialized successfully for user: {}",
                steam.user_name
            );
            tracing::info!("App ID: {}", steam.app_id);

            Ok(())
        }
        Err(error) => {
            let error_msg = format!("Failed to initialize Steam: {error:?}");
            tracing::error!("{}", error_msg);
            steam.initialization_error = Some(error_msg.clone());
            Err(error_msg)
        }
    }
}

pub fn steam_run_callbacks(steam: &SteamResources) {
    if let Some(client) = &steam.client {
        client.run_callbacks();
    }
}

pub fn steam_request_stats(steam: &mut SteamResources) {
    steam_refresh_achievements(steam);
}

pub fn steam_refresh_achievements(steam: &mut SteamResources) {
    let Some(client) = &steam.client else {
        return;
    };

    let user_stats = client.user_stats();

    let num_achievements = user_stats.get_num_achievements().unwrap_or(0);
    if num_achievements == 0 {
        return;
    }

    let Some(achievement_names) = user_stats.get_achievement_names() else {
        return;
    };

    steam.achievements.clear();
    for name in achievement_names {
        let achieved = user_stats.achievement(&name).get().unwrap_or(false);

        steam.achievements.push(AchievementInfo {
            api_name: name,
            achieved,
        });
    }

    steam.stats_received = true;
}

pub fn steam_refresh_stats(steam: &mut SteamResources, stat_names: &[&str]) {
    let Some(client) = &steam.client else {
        return;
    };

    let user_stats = client.user_stats();

    steam.stats.clear();
    for &name in stat_names {
        if let Ok(value) = user_stats.get_stat_i32(name) {
            steam.stats.push(StatInfo {
                api_name: name.to_string(),
                value: StatValue::Int(value),
            });
        } else if let Ok(value) = user_stats.get_stat_f32(name) {
            steam.stats.push(StatInfo {
                api_name: name.to_string(),
                value: StatValue::Float(value),
            });
        }
    }
}

pub fn steam_unlock_achievement(steam: &SteamResources, name: &str) -> Result<(), String> {
    let Some(client) = &steam.client else {
        return Err("Steam not initialized".to_string());
    };

    let user_stats = client.user_stats();
    user_stats
        .achievement(name)
        .set()
        .map_err(|_| format!("Failed to unlock achievement: {name}"))?;

    Ok(())
}

pub fn steam_clear_achievement(steam: &SteamResources, name: &str) -> Result<(), String> {
    let Some(client) = &steam.client else {
        return Err("Steam not initialized".to_string());
    };

    let user_stats = client.user_stats();
    user_stats
        .achievement(name)
        .clear()
        .map_err(|_| format!("Failed to clear achievement: {name}"))?;

    Ok(())
}

pub fn steam_set_stat_int(steam: &SteamResources, name: &str, value: i32) -> Result<(), String> {
    let Some(client) = &steam.client else {
        return Err("Steam not initialized".to_string());
    };

    let user_stats = client.user_stats();
    user_stats
        .set_stat_i32(name, value)
        .map_err(|_| format!("Failed to set stat: {name}"))?;

    Ok(())
}

pub fn steam_set_stat_float(steam: &SteamResources, name: &str, value: f32) -> Result<(), String> {
    let Some(client) = &steam.client else {
        return Err("Steam not initialized".to_string());
    };

    let user_stats = client.user_stats();
    user_stats
        .set_stat_f32(name, value)
        .map_err(|_| format!("Failed to set stat: {name}"))?;

    Ok(())
}

pub fn steam_store_stats(steam: &SteamResources) -> Result<(), String> {
    let Some(client) = &steam.client else {
        return Err("Steam not initialized".to_string());
    };

    let user_stats = client.user_stats();
    user_stats
        .store_stats()
        .map_err(|_| "Failed to store stats".to_string())?;

    Ok(())
}

pub fn steam_reset_all_stats(steam: &SteamResources, achievements_too: bool) -> Result<(), String> {
    let Some(client) = &steam.client else {
        return Err("Steam not initialized".to_string());
    };

    let user_stats = client.user_stats();
    user_stats
        .reset_all_stats(achievements_too)
        .map_err(|_| "Failed to reset stats".to_string())?;

    Ok(())
}

pub fn steam_refresh_friends(steam: &mut SteamResources) {
    let Some(client) = &steam.client else {
        return;
    };

    let friends_interface = client.friends();
    let friend_list = friends_interface.get_friends(FriendFlags::IMMEDIATE);

    steam.friends.clear();
    for friend in friend_list {
        let name = friend.name();
        let state = friend.state();

        steam.friends.push(FriendInfo {
            name,
            steam_id: friend.id(),
            state: state.into(),
        });
    }

    steam.friends.sort_by(|a, b| {
        let a_online = a.state.is_online();
        let b_online = b.state.is_online();
        match (a_online, b_online) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });
}

pub fn steam_is_initialized(steam: &SteamResources) -> bool {
    steam.initialized
}

pub fn steam_client(steam: &SteamResources) -> Option<&Client> {
    steam.client.as_ref()
}

pub fn steam_setup_networking_callbacks(steam: &mut SteamResources) {
    if steam.networking_callbacks_registered {
        return;
    }

    let Some(client) = &steam.client else {
        return;
    };

    let networking = client.networking_messages();
    let pending_requests = Arc::clone(&steam.pending_session_requests);

    networking.session_request_callback(move |request| {
        let remote_identity = request.remote();
        if let Some(steam_id) = remote_identity.steam_id() {
            if let Ok(mut requests) = pending_requests.lock()
                && !requests.iter().any(|r| r.remote_id == steam_id)
            {
                requests.push(PendingSessionRequest {
                    remote_id: steam_id,
                    remote_name: format!("{:?}", steam_id),
                });
            }
            request.accept();
        }
    });

    networking.session_failed_callback(|info| {
        if let Some(identity) = info.identity_remote()
            && let Some(steam_id) = identity.steam_id()
        {
            tracing::warn!("Session failed with peer: {:?}", steam_id);
        }
    });

    steam.networking_callbacks_registered = true;
    tracing::info!("Steam networking callbacks registered");
}

pub fn steam_send_message(
    steam: &mut SteamResources,
    target: SteamId,
    data: &[u8],
    channel: u32,
    reliable: bool,
) -> Result<(), String> {
    let Some(client) = &steam.client else {
        return Err("Steam not initialized".to_string());
    };

    let networking = client.networking_messages();
    let identity = NetworkingIdentity::new_steam_id(target);

    let flags = if reliable {
        SendFlags::RELIABLE_NO_NAGLE | SendFlags::AUTO_RESTART_BROKEN_SESSION
    } else {
        SendFlags::UNRELIABLE_NO_NAGLE
    };

    networking
        .send_message_to_user(identity, flags, data, channel)
        .map_err(|error| format!("Failed to send message: {error:?}"))?;

    let target_name = steam
        .friends
        .iter()
        .find(|f| f.steam_id == target)
        .map(|f| f.name.clone())
        .unwrap_or_else(|| format!("{:?}", target));

    steam.received_messages.push(NetworkMessage {
        sender_name: steam.user_name.clone(),
        sender_id: steam.user_id.unwrap_or(target),
        data: data.to_vec(),
        channel,
        is_outgoing: true,
    });

    if !steam.active_sessions.iter().any(|s| s.steam_id == target) {
        steam.active_sessions.push(ActiveSession {
            steam_id: target,
            name: target_name,
            state: SessionState::Connecting,
        });
    }

    Ok(())
}

pub fn steam_receive_messages(steam: &mut SteamResources, channel: u32, max_messages: usize) {
    let Some(client) = &steam.client else {
        return;
    };

    let networking = client.networking_messages();
    let messages = networking.receive_messages_on_channel(channel, max_messages);

    for message in messages {
        let identity = message.identity_peer();
        if let Some(steam_id) = identity.steam_id() {
            let sender_name = steam
                .friends
                .iter()
                .find(|f| f.steam_id == steam_id)
                .map(|f| f.name.clone())
                .unwrap_or_else(|| format!("{:?}", steam_id));

            steam.received_messages.push(NetworkMessage {
                sender_name,
                sender_id: steam_id,
                data: message.data().to_vec(),
                channel: message.channel() as u32,
                is_outgoing: false,
            });

            if let Some(session) = steam
                .active_sessions
                .iter_mut()
                .find(|s| s.steam_id == steam_id)
            {
                session.state = SessionState::Connected;
            } else {
                let name = steam
                    .friends
                    .iter()
                    .find(|f| f.steam_id == steam_id)
                    .map(|f| f.name.clone())
                    .unwrap_or_else(|| format!("{:?}", steam_id));

                steam.active_sessions.push(ActiveSession {
                    steam_id,
                    name,
                    state: SessionState::Connected,
                });
            }
        }
    }
}

pub fn steam_process_pending_requests(steam: &mut SteamResources) {
    if let Ok(mut requests) = steam.pending_session_requests.lock() {
        for request in requests.drain(..) {
            if !steam
                .active_sessions
                .iter()
                .any(|s| s.steam_id == request.remote_id)
            {
                steam.active_sessions.push(ActiveSession {
                    steam_id: request.remote_id,
                    name: request.remote_name,
                    state: SessionState::Connected,
                });
            }
        }
    }
}

pub fn steam_close_session(steam: &mut SteamResources, steam_id: SteamId) {
    if steam.client.is_some() {
        unsafe {
            let net = steamworks_sys::SteamAPI_SteamNetworkingMessages_SteamAPI_v002();
            if !net.is_null() {
                let mut identity: steamworks_sys::SteamNetworkingIdentity = std::mem::zeroed();
                steamworks_sys::SteamAPI_SteamNetworkingIdentity_SetSteamID64(
                    &mut identity,
                    steam_id.raw(),
                );
                steamworks_sys::SteamAPI_ISteamNetworkingMessages_CloseSessionWithUser(
                    net, &identity,
                );
            }
        }
    }
    steam.active_sessions.retain(|s| s.steam_id != steam_id);
}

pub fn steam_close_all_sessions(steam: &mut SteamResources) {
    if steam.client.is_some() {
        unsafe {
            let net = steamworks_sys::SteamAPI_SteamNetworkingMessages_SteamAPI_v002();
            if !net.is_null() {
                for session in &steam.active_sessions {
                    let mut identity: steamworks_sys::SteamNetworkingIdentity = std::mem::zeroed();
                    steamworks_sys::SteamAPI_SteamNetworkingIdentity_SetSteamID64(
                        &mut identity,
                        session.steam_id.raw(),
                    );
                    steamworks_sys::SteamAPI_ISteamNetworkingMessages_CloseSessionWithUser(
                        net, &identity,
                    );
                }
            }
        }
    }
    steam.active_sessions.clear();
}

pub fn steam_clear_messages(steam: &mut SteamResources) {
    steam.received_messages.clear();
}

pub fn steam_get_session_state(steam: &SteamResources, steam_id: SteamId) -> SessionState {
    let Some(client) = &steam.client else {
        return SessionState::None;
    };

    let networking = client.networking_messages();
    let identity = NetworkingIdentity::new_steam_id(steam_id);

    let (state, _, _) = networking.get_session_connection_info(&identity);

    match state {
        steamworks::networking_types::NetworkingConnectionState::None => SessionState::None,
        steamworks::networking_types::NetworkingConnectionState::Connecting => {
            SessionState::Connecting
        }
        steamworks::networking_types::NetworkingConnectionState::FindingRoute => {
            SessionState::Connecting
        }
        steamworks::networking_types::NetworkingConnectionState::Connected => {
            SessionState::Connected
        }
        steamworks::networking_types::NetworkingConnectionState::ClosedByPeer => {
            SessionState::ClosedByPeer
        }
        steamworks::networking_types::NetworkingConnectionState::ProblemDetectedLocally => {
            SessionState::ProblemDetected
        }
    }
}

pub fn steam_refresh_session_states(steam: &mut SteamResources) {
    let Some(client) = &steam.client else {
        return;
    };

    let networking = client.networking_messages();

    for session in &mut steam.active_sessions {
        let identity = NetworkingIdentity::new_steam_id(session.steam_id);
        let (state, _, _) = networking.get_session_connection_info(&identity);

        session.state = match state {
            steamworks::networking_types::NetworkingConnectionState::None => SessionState::None,
            steamworks::networking_types::NetworkingConnectionState::Connecting => {
                SessionState::Connecting
            }
            steamworks::networking_types::NetworkingConnectionState::FindingRoute => {
                SessionState::Connecting
            }
            steamworks::networking_types::NetworkingConnectionState::Connected => {
                SessionState::Connected
            }
            steamworks::networking_types::NetworkingConnectionState::ClosedByPeer => {
                SessionState::ClosedByPeer
            }
            steamworks::networking_types::NetworkingConnectionState::ProblemDetectedLocally => {
                SessionState::ProblemDetected
            }
        };
    }

    steam
        .active_sessions
        .retain(|s| !matches!(s.state, SessionState::None | SessionState::ClosedByPeer));
}

pub fn steam_networking_messages(steam: &SteamResources) -> Option<NetworkingMessages> {
    steam.client.as_ref().map(|c| c.networking_messages())
}

pub fn steam_open_invite_dialog(steam: &SteamResources, connect_string: &str) {
    let Some(client) = &steam.client else {
        return;
    };
    client
        .friends()
        .activate_invite_dialog_connect_string(connect_string);
}

pub fn steam_open_overlay_to_user(steam: &SteamResources, dialog: &str, user: SteamId) {
    let Some(client) = &steam.client else {
        return;
    };
    client.friends().activate_game_overlay_to_user(dialog, user);
}

pub fn steam_set_rich_presence(steam: &SteamResources, key: &str, value: &str) {
    let Some(client) = &steam.client else {
        return;
    };
    client.friends().set_rich_presence(key, Some(value));
}

pub fn steam_clear_rich_presence(steam: &SteamResources) {
    let Some(client) = &steam.client else {
        return;
    };
    client.friends().clear_rich_presence();
}

/// Installs Steam: initializes the client at startup (failing gracefully
/// when Steam is not running) and pumps its callbacks at the top of every
/// frame. Without this plugin the engine never touches Steamworks.
pub struct SteamPlugin;

impl crate::app::Plugin for SteamPlugin {
    fn build(&self, app: &mut crate::app::App) {
        app.add_system(
            crate::app::Stage::Startup,
            |world: &mut crate::ecs::World| {
                if let Err(error) = steam_initialize(
                    &mut world.expect_resource_mut::<crate::plugins::steam::SteamResources>(),
                ) {
                    tracing::warn!("Steam unavailable: {error}");
                }
            },
        );
        app.add_system(crate::app::Stage::First, |world: &mut crate::ecs::World| {
            steam_run_callbacks(
                &world.expect_resource_mut::<crate::plugins::steam::SteamResources>(),
            );
        });
    }
}
