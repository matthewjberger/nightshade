use std::sync::Arc;

use steamworks::{
    Client, FriendFlags, SteamId,
    networking_types::{NetworkingIdentity, SendFlags},
};

use crate::contract::*;
use crate::resources::SteamResources;

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
