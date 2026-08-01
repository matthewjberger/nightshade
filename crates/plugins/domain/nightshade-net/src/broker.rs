use crate::{
    BrokerContract, Result,
    arena::{Arena, PeerId},
    bridge::{Bridge, BridgeCommand, BridgeRegistry, connect_bridge, spawn_bridge},
    wire::{
        DELIVER_PAYLOAD_OFFSET, FLAG_BINARY, FLAG_LOCAL_ONLY, Frame, FrameReader, FrameView,
        MAX_WRITE_BATCH, encode_deliver_header, encode_deliver_text, encode_forward_header,
        parse_frame, write_frames,
    },
};
use bytes::{Bytes, BytesMut};
use smallvec::SmallVec;
use socket2::{Domain, Socket, TcpKeepalive, Type};
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
    net::{TcpListener, TcpStream, tcp::OwnedWriteHalf},
    sync::{
        mpsc::{self, Receiver, Sender, UnboundedSender, error::TrySendError},
        oneshot, watch,
    },
};

const BROKER_ID: &str = "broker";
const CONTROL_TOPIC_PREFIX: &str = "net/";
const BROKER_EVENT_CAPACITY: usize = 1024;
const CONTROL_QUEUE_CAPACITY: usize = 256;
const DATA_QUEUE_CAPACITY: usize = 1024;
const REORDER_WINDOW: u64 = 1024;
const REORDER_WORDS: usize = (REORDER_WINDOW / 64) as usize;
const MAX_TRACKED_ORIGINS: usize = 1024;

pub struct Broker {
    pub(crate) sender: Sender<BrokerEvent>,
    pub(crate) shutdown_sender: watch::Sender<bool>,
}

impl Broker {
    pub async fn start(address: &str) -> Result<Broker> {
        let listener = create_listener(resolve_address(address).await?)?;
        let (event_sender, event_receiver) = mpsc::channel(BROKER_EVENT_CAPACITY);
        let (shutdown_sender, shutdown_receiver) = watch::channel(false);
        let instance_id = uuid::Uuid::new_v4().as_u128();
        tokio::spawn(broker_loop(
            event_receiver,
            shutdown_receiver.clone(),
            instance_id,
        ));
        tokio::spawn(accept_loop(
            listener,
            event_sender.clone(),
            shutdown_receiver,
        ));
        Ok(Broker {
            sender: event_sender,
            shutdown_sender,
        })
    }

    pub fn is_running(&self) -> bool {
        !self.sender.is_closed()
    }

    pub fn stop(&self) {
        let _ = self.shutdown_sender.send(true);
    }
}

pub(crate) enum PeerWriter {
    Tcp(OwnedWriteHalf),
    Udp {
        outbound: mpsc::Sender<(SocketAddr, Bytes)>,
        address: SocketAddr,
    },
}

impl PeerWriter {
    async fn send_frames(&mut self, frames: &[Frame]) -> Result<()> {
        match self {
            PeerWriter::Tcp(writer) => write_frames(writer, frames).await,
            PeerWriter::Udp { outbound, address } => {
                for frame in frames {
                    if outbound
                        .send((*address, frame.to_contiguous()))
                        .await
                        .is_err()
                    {
                        return Err(crate::Error::Transport(
                            "the datagram listener stopped".to_string(),
                        ));
                    }
                }
                Ok(())
            }
        }
    }
}

pub(crate) enum BrokerEvent {
    Hello {
        name: Arc<str>,
        writer: PeerWriter,
        shutdown: oneshot::Receiver<()>,
        registered: oneshot::Sender<PeerId>,
    },
    Frame {
        peer: PeerId,
        name: Arc<str>,
        frame: BytesMut,
        view: FrameView,
    },
}

struct OriginProgress {
    watermark: u64,
    window: [u64; REORDER_WORDS],
    last_seen: u64,
}

impl OriginProgress {
    fn new(last_seen: u64) -> Self {
        Self {
            watermark: 0,
            window: [0; REORDER_WORDS],
            last_seen,
        }
    }

    fn test(&self, sequence: u64) -> bool {
        let bit = sequence % REORDER_WINDOW;
        self.window[(bit / 64) as usize] >> (bit % 64) & 1 != 0
    }

    fn set(&mut self, sequence: u64) {
        let bit = sequence % REORDER_WINDOW;
        self.window[(bit / 64) as usize] |= 1 << (bit % 64);
    }

    fn clear(&mut self, sequence: u64) {
        let bit = sequence % REORDER_WINDOW;
        self.window[(bit / 64) as usize] &= !(1 << (bit % 64));
    }

    fn advance(&mut self) {
        while self.test(self.watermark + 1) {
            self.clear(self.watermark + 1);
            self.watermark += 1;
        }
    }

    fn record(&mut self, sequence: u64) -> bool {
        if sequence <= self.watermark {
            return false;
        }
        if sequence > self.watermark + REORDER_WINDOW {
            let new_watermark = sequence - REORDER_WINDOW;
            if new_watermark - self.watermark >= REORDER_WINDOW {
                self.window = [0; REORDER_WORDS];
            } else {
                let mut cursor = self.watermark + 1;
                while cursor <= new_watermark {
                    self.clear(cursor);
                    cursor += 1;
                }
            }
            self.watermark = new_watermark;
            self.advance();
        }
        if sequence <= self.watermark || self.test(sequence) {
            return false;
        }
        self.set(sequence);
        self.advance();
        true
    }
}

struct PeerSlot {
    name: Arc<str>,
    control: Sender<Frame>,
    data: Sender<Frame>,
}

struct TopicState {
    name: Box<str>,
    subscribers: SmallVec<[PeerId; 4]>,
}

struct WildcardSubscription {
    pattern: Box<str>,
    peer: PeerId,
}

struct BrokerState {
    instance_id: u128,
    publish_counter: u64,
    origin_progress: HashMap<u128, OriginProgress>,
    origin_tick: u64,
    peers: Arena<PeerSlot>,
    peer_names: HashMap<Arc<str>, PeerId>,
    topic_index: HashMap<Box<str>, u32>,
    topics: Vec<Option<TopicState>>,
    free_topics: Vec<u32>,
    wildcards: Vec<WildcardSubscription>,
    bridges: BridgeRegistry,
    disconnect_sender: UnboundedSender<PeerId>,
}

pub(crate) async fn resolve_address(address: &str) -> Result<SocketAddr> {
    Ok(crate::client::resolve_addresses(address).await?[0])
}

fn create_listener(socket_address: SocketAddr) -> Result<TcpListener> {
    let socket = Socket::new(Domain::for_address(socket_address), Type::STREAM, None)?;
    socket.set_reuse_address(true)?;
    socket.bind(&socket_address.into())?;
    socket.set_keepalive(true)?;
    let keepalive = TcpKeepalive::new()
        .with_time(Duration::from_secs(30))
        .with_interval(Duration::from_secs(1));
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    let keepalive = keepalive.with_retries(3);
    socket.set_tcp_keepalive(&keepalive)?;
    socket.listen(1024)?;
    let std_listener: std::net::TcpListener = socket.into();
    std_listener.set_nonblocking(true)?;
    Ok(TcpListener::from_std(std_listener)?)
}

async fn accept_loop(
    listener: TcpListener,
    event_sender: Sender<BrokerEvent>,
    mut shutdown: watch::Receiver<bool>,
) {
    loop {
        tokio::select! {
            accepted = listener.accept() => match accepted {
                Ok((stream, address)) => {
                    if !address.ip().is_loopback() {
                        let _ = stream.set_nodelay(true);
                    }
                    tokio::spawn(connection_task(event_sender.clone(), stream));
                }
                Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
            },
            _ = shutdown.changed() => break,
        }
    }
}

async fn connection_task(event_sender: Sender<BrokerEvent>, stream: TcpStream) {
    let (read_half, write_half) = stream.into_split();
    let mut reader = FrameReader::new(read_half);
    let mut writer: Option<PeerWriter> = Some(PeerWriter::Tcp(write_half));
    let mut peer: Option<(PeerId, Arc<str>)> = None;
    let mut shutdown_signal = None;
    loop {
        let Ok(frame) = reader.next_frame().await else {
            break;
        };
        if !process_incoming_frame(
            frame,
            &event_sender,
            &mut writer,
            &mut peer,
            &mut shutdown_signal,
        )
        .await
        {
            break;
        }
    }
    drop(shutdown_signal);
}

pub(crate) async fn process_incoming_frame(
    frame: BytesMut,
    event_sender: &Sender<BrokerEvent>,
    writer: &mut Option<PeerWriter>,
    peer: &mut Option<(PeerId, Arc<str>)>,
    shutdown_signal: &mut Option<oneshot::Sender<()>>,
) -> bool {
    let Some(view) = parse_frame(&frame) else {
        return false;
    };
    match view {
        FrameView::Hello { id } => {
            let Some(peer_writer) = writer.take() else {
                return true;
            };
            let Ok(name) = std::str::from_utf8(&frame[id]) else {
                return false;
            };
            let name: Arc<str> = Arc::from(name);
            let (shutdown_sender, shutdown_receiver) = oneshot::channel();
            let (registered_sender, registered_receiver) = oneshot::channel();
            let forwarded = event_sender
                .send(BrokerEvent::Hello {
                    name: name.clone(),
                    writer: peer_writer,
                    shutdown: shutdown_receiver,
                    registered: registered_sender,
                })
                .await;
            if forwarded.is_err() {
                return false;
            }
            *shutdown_signal = Some(shutdown_sender);
            let Ok(id) = registered_receiver.await else {
                return false;
            };
            *peer = Some((id, name));
            true
        }
        view => {
            let Some((peer, name)) = peer.as_ref() else {
                return true;
            };
            event_sender
                .send(BrokerEvent::Frame {
                    peer: *peer,
                    name: name.clone(),
                    frame,
                    view,
                })
                .await
                .is_ok()
        }
    }
}

async fn connection_writer_task(
    mut control: Receiver<Frame>,
    mut data: Receiver<Frame>,
    mut writer: PeerWriter,
    mut shutdown: oneshot::Receiver<()>,
    disconnect_sender: UnboundedSender<PeerId>,
    peer_id: PeerId,
) {
    let mut frames: Vec<Frame> = Vec::with_capacity(MAX_WRITE_BATCH);
    loop {
        frames.clear();
        tokio::select! {
            biased;
            _ = &mut shutdown => break,
            frame = control.recv() => match frame {
                Some(frame) => frames.push(frame),
                None => break,
            },
            frame = data.recv() => match frame {
                Some(frame) => frames.push(frame),
                None => break,
            },
        }
        while frames.len() < MAX_WRITE_BATCH
            && let Ok(frame) = control.try_recv()
        {
            frames.push(frame);
        }
        while frames.len() < MAX_WRITE_BATCH
            && let Ok(frame) = data.try_recv()
        {
            frames.push(frame);
        }
        if writer.send_frames(&frames).await.is_err() {
            break;
        }
    }
    let _ = disconnect_sender.send(peer_id);
}

async fn broker_loop(
    mut events: Receiver<BrokerEvent>,
    mut shutdown: watch::Receiver<bool>,
    instance_id: u128,
) {
    let (disconnect_sender, mut disconnect_receiver) = mpsc::unbounded_channel();
    let mut state = BrokerState {
        instance_id,
        publish_counter: 0,
        origin_progress: HashMap::new(),
        origin_tick: 0,
        peers: Arena::default(),
        peer_names: HashMap::new(),
        topic_index: HashMap::new(),
        topics: Vec::new(),
        free_topics: Vec::new(),
        wildcards: Vec::new(),
        bridges: BridgeRegistry::default(),
        disconnect_sender,
    };
    loop {
        tokio::select! {
            event = events.recv() => match event {
                Some(event) => {
                    let _ = handle_broker_event(&mut state, event).await;
                }
                None => break,
            },
            disconnect = disconnect_receiver.recv() => {
                if let Some(peer_id) = disconnect {
                    remove_disconnected_peer(&mut state, peer_id);
                }
            },
            _ = shutdown.changed() => break,
        }
    }
}

async fn handle_broker_event(state: &mut BrokerState, event: BrokerEvent) -> Result<()> {
    match event {
        BrokerEvent::Hello {
            name,
            writer,
            shutdown,
            registered,
        } => {
            let (topic, mut payload) = BrokerContract::peer_connected();
            payload.id = name.to_string();
            let peer_id = establish_peer(state, name, writer, shutdown);
            let _ = registered.send(peer_id);
            publish_from_broker(state, BROKER_ID, &topic, &payload.to_json()?)?;
        }
        BrokerEvent::Frame {
            peer,
            name,
            frame,
            view,
        } => {
            handle_peer_frame(state, peer, &name, frame, view).await?;
        }
    }
    Ok(())
}

async fn handle_peer_frame(
    state: &mut BrokerState,
    peer: PeerId,
    peer_name: &str,
    frame: BytesMut,
    view: FrameView,
) -> Result<()> {
    match view {
        FrameView::Hello { .. } => {}
        FrameView::Subscribe { topic } => {
            let Ok(topic_name) = std::str::from_utf8(&frame[topic]) else {
                return Ok(());
            };
            subscribe_peer(state, peer, topic_name);
        }
        FrameView::Unsubscribe { topic } => {
            let Ok(topic_name) = std::str::from_utf8(&frame[topic]) else {
                return Ok(());
            };
            unsubscribe_peer(state, peer, topic_name);
        }
        FrameView::Publish {
            flags,
            topic,
            payload,
        } => {
            let frozen = frame.freeze();
            let Ok(topic_name) = std::str::from_utf8(&frozen[topic.clone()]) else {
                return Ok(());
            };
            if topic_name.starts_with(CONTROL_TOPIC_PREFIX) {
                answer_introspection_requests(state, topic_name)?;
            }
            if flags & FLAG_LOCAL_ONLY == 0 {
                let sequence = next_sequence(state);
                let visited = [state.instance_id];
                forward_to_bridges(
                    state,
                    peer_name,
                    &frozen[topic.clone()],
                    flags & FLAG_BINARY,
                    sequence,
                    &visited,
                    frozen.slice(payload.clone()),
                )?;
            }
            let header = encode_deliver_header(
                &frozen[topic],
                peer_name.as_bytes(),
                flags & FLAG_BINARY,
                payload.len(),
            )?;
            deliver(
                state,
                topic_name,
                &Frame {
                    header,
                    payload: frozen.slice(payload),
                },
            );
        }
        FrameView::Forward {
            flags,
            sequence,
            visited,
            topic,
            payload,
        } => {
            let frozen = frame.freeze();
            let visited_bytes = &frozen[visited];
            let (hops, _) = visited_bytes.as_chunks::<16>();
            if hops
                .iter()
                .any(|hop| u128::from_be_bytes(*hop) == state.instance_id)
            {
                return Ok(());
            }
            let Some(origin_chunk) = hops.first() else {
                return Ok(());
            };
            let origin = u128::from_be_bytes(*origin_chunk);
            if !record_seen_message(state, origin, sequence) {
                return Ok(());
            }
            let Ok(topic_name) = std::str::from_utf8(&frozen[topic.clone()]) else {
                return Ok(());
            };
            answer_introspection_requests(state, topic_name)?;
            let mut extended: SmallVec<[u128; 8]> = visited_bytes
                .as_chunks::<16>()
                .0
                .iter()
                .map(|hop| u128::from_be_bytes(*hop))
                .collect();
            extended.push(state.instance_id);
            forward_to_bridges(
                state,
                peer_name,
                &frozen[topic.clone()],
                flags & FLAG_BINARY,
                sequence,
                &extended,
                frozen.slice(payload.clone()),
            )?;
            let header = encode_deliver_header(
                &frozen[topic],
                peer_name.as_bytes(),
                flags & FLAG_BINARY,
                payload.len(),
            )?;
            deliver(
                state,
                topic_name,
                &Frame {
                    header,
                    payload: frozen.slice(payload),
                },
            );
        }
        FrameView::OpenBridge {
            ack,
            source_address,
            target_address,
        } => {
            let Ok(source) = std::str::from_utf8(&frame[source_address]) else {
                return Ok(());
            };
            let Ok(target) = std::str::from_utf8(&frame[target_address]) else {
                return Ok(());
            };
            let source = source.to_string();
            let target = target.to_string();
            open_broker_bridge(state, peer_name.to_string(), target, source, ack).await?;
        }
        FrameView::CloseBridge {
            ack,
            target_address,
        } => {
            if ack {
                state.bridges.close_by_id(peer_name);
            } else {
                let Ok(target) = std::str::from_utf8(&frame[target_address]) else {
                    return Ok(());
                };
                state.bridges.close_by_address(target, true);
            }
        }
    }
    Ok(())
}

fn next_sequence(state: &mut BrokerState) -> u64 {
    state.publish_counter += 1;
    state.publish_counter
}

fn record_seen_message(state: &mut BrokerState, origin: u128, sequence: u64) -> bool {
    state.origin_tick += 1;
    let tick = state.origin_tick;
    if !state.origin_progress.contains_key(&origin) {
        if state.origin_progress.len() >= MAX_TRACKED_ORIGINS
            && let Some(stalest) = state
                .origin_progress
                .iter()
                .min_by_key(|(_, progress)| progress.last_seen)
                .map(|(tracked, _)| *tracked)
        {
            state.origin_progress.remove(&stalest);
        }
        state
            .origin_progress
            .insert(origin, OriginProgress::new(tick));
    }
    let Some(progress) = state.origin_progress.get_mut(&origin) else {
        return true;
    };
    progress.last_seen = tick;
    progress.record(sequence)
}

fn establish_peer(
    state: &mut BrokerState,
    name: Arc<str>,
    writer: PeerWriter,
    shutdown: oneshot::Receiver<()>,
) -> PeerId {
    if let Some(previous) = state.peer_names.remove(&name)
        && state.peers.remove(previous).is_some()
    {
        purge_subscriptions(state, previous);
    }
    let (control_sender, control_receiver) = mpsc::channel(CONTROL_QUEUE_CAPACITY);
    let (data_sender, data_receiver) = mpsc::channel(DATA_QUEUE_CAPACITY);
    let peer_id = state.peers.insert(PeerSlot {
        name: name.clone(),
        control: control_sender,
        data: data_sender,
    });
    state.peer_names.insert(name, peer_id);
    tokio::spawn(connection_writer_task(
        control_receiver,
        data_receiver,
        writer,
        shutdown,
        state.disconnect_sender.clone(),
        peer_id,
    ));
    peer_id
}

fn remove_disconnected_peer(state: &mut BrokerState, peer_id: PeerId) {
    let Some(slot) = state.peers.remove(peer_id) else {
        return;
    };
    if state.peer_names.get(&slot.name) == Some(&peer_id) {
        state.peer_names.remove(&slot.name);
    }
    purge_subscriptions(state, peer_id);
    let (topic, mut payload) = BrokerContract::peer_disconnected();
    payload.id = slot.name.to_string();
    if let Ok(text) = payload.to_json() {
        let _ = publish_from_broker(state, BROKER_ID, &topic, &text);
    }
}

fn purge_subscriptions(state: &mut BrokerState, peer_id: PeerId) {
    state.wildcards.retain(|wildcard| wildcard.peer != peer_id);
    for index in 0..state.topics.len() {
        let is_empty = {
            let Some(topic) = state.topics[index].as_mut() else {
                continue;
            };
            topic
                .subscribers
                .retain(|subscriber| *subscriber != peer_id);
            topic.subscribers.is_empty()
        };
        if is_empty {
            release_topic(state, index as u32);
        }
    }
}

fn release_topic(state: &mut BrokerState, index: u32) {
    if let Some(topic) = state.topics[index as usize].take() {
        state.topic_index.remove(&topic.name);
        state.free_topics.push(index);
    }
}

fn is_wildcard(topic: &str) -> bool {
    topic
        .split('/')
        .any(|segment| segment == "+" || segment == "#")
}

fn topic_matches(pattern: &str, topic: &str) -> bool {
    let mut pattern_segments = pattern.split('/');
    let mut topic_segments = topic.split('/');
    loop {
        match (pattern_segments.next(), topic_segments.next()) {
            (Some("#"), _) => return true,
            (Some("+"), Some(_)) => continue,
            (Some(pattern_segment), Some(topic_segment)) if pattern_segment == topic_segment => {
                continue;
            }
            (None, None) => return true,
            _ => return false,
        }
    }
}

fn subscribe_peer(state: &mut BrokerState, peer_id: PeerId, topic_name: &str) {
    if state.peers.get(peer_id).is_none() {
        return;
    }
    if is_wildcard(topic_name) {
        let already = state
            .wildcards
            .iter()
            .any(|wildcard| wildcard.peer == peer_id && &*wildcard.pattern == topic_name);
        if !already {
            state.wildcards.push(WildcardSubscription {
                pattern: Box::from(topic_name),
                peer: peer_id,
            });
        }
        return;
    }
    let index = match state.topic_index.get(topic_name) {
        Some(&index) => index,
        None => {
            let index = match state.free_topics.pop() {
                Some(index) => index,
                None => {
                    state.topics.push(None);
                    (state.topics.len() - 1) as u32
                }
            };
            state.topics[index as usize] = Some(TopicState {
                name: Box::from(topic_name),
                subscribers: SmallVec::new(),
            });
            state.topic_index.insert(Box::from(topic_name), index);
            index
        }
    };
    let Some(topic) = state.topics[index as usize].as_mut() else {
        return;
    };
    if !topic.subscribers.contains(&peer_id) {
        topic.subscribers.push(peer_id);
    }
}

fn unsubscribe_peer(state: &mut BrokerState, peer_id: PeerId, topic_name: &str) {
    if is_wildcard(topic_name) {
        state
            .wildcards
            .retain(|wildcard| !(wildcard.peer == peer_id && &*wildcard.pattern == topic_name));
        return;
    }
    let Some(&index) = state.topic_index.get(topic_name) else {
        return;
    };
    let Some(topic) = state.topics[index as usize].as_mut() else {
        return;
    };
    topic
        .subscribers
        .retain(|subscriber| *subscriber != peer_id);
    if topic.subscribers.is_empty() {
        release_topic(state, index);
    }
}

fn answer_introspection_requests(state: &mut BrokerState, topic_name: &str) -> Result<()> {
    if topic_name == BrokerContract::request_subscriptions_topic() {
        let (report_topic, mut payload) = BrokerContract::report_subscriptions();
        let mut subscriptions: Vec<(String, Vec<String>)> = state
            .topics
            .iter()
            .flatten()
            .map(|topic| {
                (
                    topic.name.to_string(),
                    topic
                        .subscribers
                        .iter()
                        .filter_map(|&subscriber| {
                            state
                                .peers
                                .get(subscriber)
                                .map(|slot| slot.name.to_string())
                        })
                        .collect(),
                )
            })
            .collect();
        let mut wildcard_groups: HashMap<&str, Vec<String>> = HashMap::new();
        for wildcard in &state.wildcards {
            if let Some(slot) = state.peers.get(wildcard.peer) {
                wildcard_groups
                    .entry(&wildcard.pattern)
                    .or_default()
                    .push(slot.name.to_string());
            }
        }
        subscriptions.extend(
            wildcard_groups
                .into_iter()
                .map(|(pattern, subscribers)| (pattern.to_string(), subscribers)),
        );
        payload.subscriptions = subscriptions;
        publish_from_broker(state, BROKER_ID, &report_topic, &payload.to_json()?)?;
    } else if topic_name == BrokerContract::request_peers_topic() {
        let (report_topic, mut payload) = BrokerContract::report_peers();
        payload.peers = state
            .peers
            .iter()
            .map(|(_, slot)| slot.name.to_string())
            .collect();
        publish_from_broker(state, BROKER_ID, &report_topic, &payload.to_json()?)?;
    } else if topic_name == BrokerContract::request_bridges_topic() {
        let (report_topic, mut payload) = BrokerContract::report_bridges();
        payload.bridges = state.bridges.ids();
        publish_from_broker(state, BROKER_ID, &report_topic, &payload.to_json()?)?;
    }
    Ok(())
}

fn publish_from_broker(
    state: &mut BrokerState,
    skip_bridge: &str,
    topic: &str,
    payload: &str,
) -> Result<()> {
    let sequence = next_sequence(state);
    let frame = encode_deliver_text(topic, BROKER_ID, payload)?;
    let topic_range = DELIVER_PAYLOAD_OFFSET..DELIVER_PAYLOAD_OFFSET + topic.len();
    let payload_start = topic_range.end + 2 + BROKER_ID.len();
    let visited = [state.instance_id];
    forward_to_bridges(
        state,
        skip_bridge,
        &frame[topic_range.clone()],
        0,
        sequence,
        &visited,
        frame.slice(payload_start..),
    )?;
    deliver(state, topic, &Frame::single(frame));
    Ok(())
}

fn forward_to_bridges(
    state: &BrokerState,
    skip_name: &str,
    topic_bytes: &[u8],
    flags: u8,
    sequence: u64,
    visited: &[u128],
    payload: Bytes,
) -> Result<()> {
    if state.bridges.is_empty() {
        return Ok(());
    }
    let header = encode_forward_header(topic_bytes, flags, sequence, visited, payload.len())?;
    let frame = Frame { header, payload };
    for bridge in state.bridges.iter() {
        if &*bridge.id != skip_name {
            let _ = bridge
                .commands
                .try_send(BridgeCommand::Forward(frame.clone()));
        }
    }
    Ok(())
}

fn send_to_peer(state: &BrokerState, peer_id: PeerId, control: bool, frame: &Frame) {
    let Some(peer) = state.peers.get(peer_id) else {
        return;
    };
    let queue = if control { &peer.control } else { &peer.data };
    if let Err(TrySendError::Full(_)) = queue.try_send(frame.clone())
        && control
    {
        let _ = state.disconnect_sender.send(peer_id);
    }
}

fn deliver(state: &BrokerState, topic_name: &str, frame: &Frame) {
    let control = topic_name.starts_with(CONTROL_TOPIC_PREFIX);
    let exact = state
        .topic_index
        .get(topic_name)
        .and_then(|&index| state.topics[index as usize].as_ref());

    if state.wildcards.is_empty() {
        if let Some(topic) = exact {
            for &peer_id in &topic.subscribers {
                send_to_peer(state, peer_id, control, frame);
            }
        }
        return;
    }

    let mut delivered: SmallVec<[PeerId; 8]> = SmallVec::new();
    if let Some(topic) = exact {
        for &peer_id in &topic.subscribers {
            delivered.push(peer_id);
            send_to_peer(state, peer_id, control, frame);
        }
    }
    for wildcard in &state.wildcards {
        if !delivered.contains(&wildcard.peer) && topic_matches(&wildcard.pattern, topic_name) {
            delivered.push(wildcard.peer);
            send_to_peer(state, wildcard.peer, control, frame);
        }
    }
}

async fn open_broker_bridge(
    state: &mut BrokerState,
    requester: String,
    target_address: String,
    source_address: String,
    ack: bool,
) -> Result<()> {
    state.bridges.close_by_address(&target_address, !ack);
    let override_id = if ack { Some(requester.clone()) } else { None };
    let Some((client, bridge_id)) = connect_bridge(override_id, &target_address).await else {
        return Ok(());
    };
    if ack {
        let (topic, mut payload) = BrokerContract::bridge_created();
        payload.id.clone_from(&bridge_id);
        payload.source_address = source_address;
        payload.target_address = target_address.clone();
        publish_from_broker(state, &requester, &topic, &payload.to_json()?)?;
    } else {
        client
            .open_bridge_acked(&target_address, &source_address, true)
            .await?;
    }
    let commands = spawn_bridge(client, target_address.clone());
    state.bridges.insert(Bridge {
        id: bridge_id.into(),
        target_address: target_address.into(),
        commands,
    });
    Ok(())
}
