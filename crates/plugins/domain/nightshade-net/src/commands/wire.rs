use crate::data::{CONNECT_RETRY_MILLISECONDS, Incoming, Note, Outgoing, Role, WireSettings};
use crate::resources::Wire;
use crate::thread::run_on_thread;
use crate::{Broker, Client, ClientSettings, Route};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

pub fn open_wire(settings: WireSettings) -> Wire {
    let (to_net, from_game) = unbounded_channel::<Outgoing>();
    let (to_game, from_net) = unbounded_channel::<Incoming>();
    let (told, notes) = unbounded_channel::<Note>();
    let (stop, halt) = tokio::sync::oneshot::channel::<()>();
    let role = settings.role;
    let spoken = told.clone();
    let started = run_on_thread(run(settings, from_game, to_game, told, halt));
    let crew = match started {
        Ok(crew) => Some(crew),
        Err(error) => {
            let _ = spoken.send(Note::Failed(format!("no network thread: {error}")));
            None
        }
    };
    Wire {
        role,
        to_net,
        from_net,
        notes,
        stop: Some(stop),
        crew,
    }
}

pub fn publish(wire: &Wire, topic: impl Into<String>, bytes: Vec<u8>, route: Route) {
    let _ = wire.to_net.send(Outgoing {
        topic: topic.into(),
        bytes,
        route,
    });
}

pub fn take_messages(wire: &mut Wire) -> Vec<Incoming> {
    let mut arrived = Vec::new();
    while let Ok(message) = wire.from_net.try_recv() {
        arrived.push(message);
    }
    arrived
}

pub fn take_notes(wire: &mut Wire) -> Vec<Note> {
    let mut told = Vec::new();
    while let Ok(note) = wire.notes.try_recv() {
        told.push(note);
    }
    told
}

async fn run(
    settings: WireSettings,
    mut from_game: UnboundedReceiver<Outgoing>,
    to_game: UnboundedSender<Incoming>,
    told: UnboundedSender<Note>,
    mut halt: tokio::sync::oneshot::Receiver<()>,
) {
    let _broker = if settings.role == Role::Host {
        match Broker::start(&settings.bind).await {
            Ok(broker) => {
                let _ = told.send(Note::Serving(settings.bind.clone()));
                Some(broker)
            }
            Err(error) => {
                let _ = told.send(Note::Failed(format!(
                    "could not serve on {}: {error}",
                    settings.bind
                )));
                return;
            }
        }
    } else {
        None
    };

    let client = Client::new(&settings.name, ClientSettings::default());
    loop {
        match client.connect(&settings.connect).await {
            Ok(()) => break,
            Err(error) => {
                let _ = told.send(Note::Waiting(format!(
                    "joining {}: {error}",
                    settings.connect
                )));
                tokio::select! {
                    _ = tokio::time::sleep(std::time::Duration::from_millis(
                        CONNECT_RETRY_MILLISECONDS,
                    )) => {}
                    _ = &mut halt => return,
                }
            }
        }
    }
    let _ = told.send(Note::Connected(settings.connect.clone()));

    if !settings.subscribe.is_empty()
        && let Err(error) = client.subscribe(&settings.subscribe).await
    {
        let _ = told.send(Note::Failed(format!("could not subscribe: {error}")));
    }

    loop {
        tokio::select! {
            _ = &mut halt => break,
            outgoing = from_game.recv() => {
                let Some(outgoing) = outgoing else {
                    break;
                };
                let _ = client
                    .publish_bytes(&outgoing.topic, &outgoing.bytes, outgoing.route)
                    .await;
            }
            reception = client.next_message() => {
                let Some(message) = reception else {
                    continue;
                };
                let payload = message
                    .bytes()
                    .map(bytes::Bytes::copy_from_slice)
                    .or_else(|| message.text().map(|text| bytes::Bytes::copy_from_slice(text.as_bytes())))
                    .unwrap_or_default();
                let _ = to_game.send(Incoming {
                    topic: message.topic().to_string(),
                    sender: message.sender().to_string(),
                    bytes: payload,
                });
            }
        }
    }
}
