use steamworks::{
    Client, SteamId, networking_messages::NetworkingMessages, networking_types::NetworkingIdentity,
};

use crate::contract::*;
use crate::resources::SteamResources;

pub fn steam_is_initialized(steam: &SteamResources) -> bool {
    steam.initialized
}

pub fn steam_client(steam: &SteamResources) -> Option<&Client> {
    steam.client.as_ref()
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

pub fn steam_networking_messages(steam: &SteamResources) -> Option<NetworkingMessages> {
    steam.client.as_ref().map(|c| c.networking_messages())
}
