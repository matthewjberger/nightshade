use crate::data::wire::{PeerId, Request, Update};
use crate::resources::{Incoming, Outgoing, Role, Wire};
use nightshade_net::{Broker, Client, ClientSettings, Message, Reception, Route, run_on_thread};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

const REQUEST_TOPIC: &str = "sandbox/request";
const STATE_TOPIC: &str = "sandbox/state";
const DEPARTED_TOPIC: &str = "net/peers/disconnected";
const CONNECT_RETRY_MILLISECONDS: u64 = 400;
const IDLE_POLL_MILLISECONDS: u64 = 2;

fn peer_topic(peer: PeerId) -> String {
    format!("sandbox/to/{peer}")
}

fn peer_of_identity(identity: &str) -> Option<PeerId> {
    let digits = identity.strip_prefix('p')?;
    let digits = digits.split('_').next()?;
    digits.parse().ok()
}

pub fn open_wire(role: Role, peer: PeerId, bind: String, connect: String) -> Wire {
    let (to_net, from_game) = unbounded_channel::<Outgoing>();
    let (to_game, from_net) = unbounded_channel::<Incoming>();

    let note = to_game.clone();
    let (stop, halt) = tokio::sync::oneshot::channel::<()>();
    let started = run_on_thread(run(role, peer, bind, connect, from_game, to_game, halt));
    let crew = match started {
        Ok(crew) => Some(crew),
        Err(error) => {
            note.send(Incoming::Note(format!("no network thread: {error}")))
                .ok();
            None
        }
    };

    Wire {
        role,
        peer,
        to_net,
        from_net,
        stop: Some(stop),
        crew,
    }
}

async fn run(
    role: Role,
    peer: PeerId,
    bind: String,
    connect: String,
    mut from_game: UnboundedReceiver<Outgoing>,
    to_game: UnboundedSender<Incoming>,
    mut halt: tokio::sync::oneshot::Receiver<()>,
) {
    let _broker = if role == Role::Host {
        match Broker::start(&bind).await {
            Ok(broker) => {
                let _ = to_game.send(Incoming::Note(format!("serving on {bind}")));
                Some(broker)
            }
            Err(error) => {
                let _ = to_game.send(Incoming::Note(format!(
                    "could not serve on {bind}: {error}"
                )));
                return;
            }
        }
    } else {
        None
    };

    let client = Client::new(&format!("p{peer}"), ClientSettings::default());
    loop {
        match client.connect(&connect).await {
            Ok(()) => break,
            Err(error) => {
                let _ = to_game.send(Incoming::Note(format!("joining {connect}: {error}")));
                tokio::select! {
                    _ = tokio::time::sleep(std::time::Duration::from_millis(
                        CONNECT_RETRY_MILLISECONDS,
                    )) => {}
                    _ = &mut halt => return,
                }
            }
        }
    }
    let _ = to_game.send(Incoming::Note(format!("connected to {connect}")));

    let subscribed = if role == Role::Host {
        client.subscribe(&[REQUEST_TOPIC, DEPARTED_TOPIC]).await
    } else {
        client
            .subscribe(&[STATE_TOPIC.to_string(), peer_topic(peer)])
            .await
    };
    if let Err(error) = subscribed {
        let _ = to_game.send(Incoming::Note(format!("could not subscribe: {error}")));
    }

    if role == Role::Guest {
        let _ = to_game.send(Incoming::Note("waiting for the host".to_string()));
    }

    loop {
        tokio::select! {
            _ = &mut halt => break,
            command = from_game.recv() => {
                let Some(command) = command else {
                    break;
                };
                send(&client, command).await;
            }
            reception = client.try_next_message() => {
                match reception {
                    Reception::Message(message) => receive(role, peer, &message, &to_game),
                    Reception::Empty => {
                        tokio::time::sleep(std::time::Duration::from_millis(
                            IDLE_POLL_MILLISECONDS,
                        ))
                        .await;
                    }
                    Reception::Disconnected => {
                        tokio::time::sleep(std::time::Duration::from_millis(
                            CONNECT_RETRY_MILLISECONDS,
                        ))
                        .await;
                    }
                }
            }
        }
    }
}

async fn send(client: &Client, command: Outgoing) {
    let _ = match command {
        Outgoing::Everyone(message) => client.publish(STATE_TOPIC, &message, Route::Global).await,
        Outgoing::One(target, message) => {
            client
                .publish(peer_topic(target), &message, Route::Global)
                .await
        }
        Outgoing::Host(message) => client.publish(REQUEST_TOPIC, &message, Route::Global).await,
    };
}

fn receive(role: Role, peer: PeerId, message: &Message, to_game: &UnboundedSender<Incoming>) {
    let topic = message.topic();
    if role == Role::Host {
        if topic == DEPARTED_TOPIC {
            if let Ok(report) = message.read_json::<nightshade_net::PeerDisconnectedPayload>()
                && let Some(gone) = peer_of_identity(&report.id)
                && gone != peer
            {
                let _ = to_game.send(Incoming::Departed(gone));
            }
            return;
        }
        let Some(sender) = peer_of_identity(message.sender()) else {
            return;
        };
        if let Ok(request) = message.read::<Request>() {
            let _ = to_game.send(Incoming::FromGuest(sender, request));
        }
        return;
    }
    if let Ok(update) = message.read::<Update>() {
        let _ = to_game.send(Incoming::FromHost(update));
    }
}
