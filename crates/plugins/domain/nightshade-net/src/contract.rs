use enum2contract::EnumContract;
use serde::{Deserialize, Serialize};

#[derive(Debug, EnumContract, Serialize, Deserialize)]
pub enum BrokerContract {
    #[topic("net/peers/connected")]
    PeerConnected { id: String },

    #[topic("net/peers/disconnected")]
    PeerDisconnected { id: String },

    #[topic("net/peers/request")]
    RequestPeers,

    #[topic("net/peers/report")]
    ReportPeers { peers: Vec<String> },

    #[topic("net/subscriptions/request")]
    RequestSubscriptions,

    #[topic("net/subscriptions/report")]
    ReportSubscriptions {
        subscriptions: Vec<(String, Vec<String>)>,
    },

    #[topic("net/bridges/created")]
    BridgeCreated {
        id: String,
        source_address: String,
        target_address: String,
    },

    #[topic("net/bridges/request")]
    RequestBridges,

    #[topic("net/bridges/report")]
    ReportBridges { bridges: Vec<String> },
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Route {
    #[default]
    Global,

    Local,
}
