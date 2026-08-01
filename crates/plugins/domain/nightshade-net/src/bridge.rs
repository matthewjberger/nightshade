use crate::{Client, ClientSettings, wire::Frame};
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender, channel};

const BRIDGE_QUEUE_CAPACITY: usize = 1024;
const BRIDGE_RECONNECT_INTERVAL: Duration = Duration::from_secs(2);

pub(crate) enum BridgeCommand {
    Forward(Frame),
    CloseAndNotify,
    CloseLocal,
}

pub(crate) struct Bridge {
    pub(crate) id: Box<str>,
    pub(crate) target_address: Box<str>,
    pub(crate) commands: Sender<BridgeCommand>,
}

#[derive(Default)]
pub(crate) struct BridgeRegistry {
    bridges: Vec<Bridge>,
}

impl BridgeRegistry {
    pub(crate) fn insert(&mut self, bridge: Bridge) {
        self.bridges.push(bridge);
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.bridges.is_empty()
    }

    pub(crate) fn ids(&self) -> Vec<String> {
        self.bridges
            .iter()
            .map(|bridge| bridge.id.to_string())
            .collect()
    }

    pub(crate) fn iter(&self) -> std::slice::Iter<'_, Bridge> {
        self.bridges.iter()
    }

    pub(crate) fn close_by_address(&mut self, target_address: &str, notify_remote: bool) {
        self.bridges.retain(|bridge| {
            if &*bridge.target_address == target_address {
                let command = if notify_remote {
                    BridgeCommand::CloseAndNotify
                } else {
                    BridgeCommand::CloseLocal
                };
                let _ = bridge.commands.try_send(command);
                false
            } else {
                true
            }
        });
    }

    pub(crate) fn close_by_id(&mut self, id: &str) {
        self.bridges.retain(|bridge| {
            if &*bridge.id == id {
                let _ = bridge.commands.try_send(BridgeCommand::CloseLocal);
                false
            } else {
                true
            }
        });
    }
}

pub(crate) async fn connect_bridge(
    override_id: Option<String>,
    target_address: &str,
) -> Option<(Client, String)> {
    let settings = ClientSettings {
        autoreconnect: false,
        max_connection_attempts: Some(0),
        ..Default::default()
    };
    let client = match override_id {
        Some(id) => Client::with_id(id, settings),
        None => Client::new("bridge", settings),
    };
    let id = client.id().to_string();
    if client.connect(target_address).await.is_err() {
        return None;
    }
    Some((client, id))
}

pub(crate) fn spawn_bridge(client: Client, target_address: String) -> Sender<BridgeCommand> {
    let (sender, receiver) = channel(BRIDGE_QUEUE_CAPACITY);
    tokio::spawn(bridge_task(client, receiver, target_address));
    sender
}

async fn bridge_task(
    client: Client,
    mut commands: Receiver<BridgeCommand>,
    target_address: String,
) {
    let mut reconnect = tokio::time::interval(BRIDGE_RECONNECT_INTERVAL);
    loop {
        tokio::select! {
            command = commands.recv() => match command {
                Some(BridgeCommand::Forward(frame)) => {
                    let _ = client.send_bridge_frame(frame).await;
                }
                Some(BridgeCommand::CloseAndNotify) => {
                    let _ = client.notify_close_bridge().await;
                    break;
                }
                Some(BridgeCommand::CloseLocal) | None => break,
            },
            _ = reconnect.tick() => {
                if !client.is_connected().await {
                    let _ = client.connect(&target_address).await;
                }
            }
        }
    }
}
