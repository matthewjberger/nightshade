use crate::{
    Result,
    arena::PeerId,
    broker::{Broker, BrokerEvent, PeerWriter, process_incoming_frame, resolve_address},
    reliability::{Delivery, Endpoint},
};
use bytes::{Bytes, BytesMut};
use std::{
    collections::{HashMap, hash_map::Entry},
    net::SocketAddr,
    sync::Arc,
    time::Duration,
};
use tokio::{
    net::UdpSocket,
    sync::{
        mpsc::{self, Sender},
        oneshot, watch,
    },
    time::Instant,
};

const RECEIVE_BUFFER: usize = 65_536;
const OUTBOUND_CAPACITY: usize = 1024;
const RETRANSMIT_INTERVAL: Duration = Duration::from_millis(20);
const PEER_TIMEOUT: Duration = Duration::from_secs(15);
const UNREGISTERED_TIMEOUT: Duration = Duration::from_secs(3);
const CONTROL_TOPIC_PREFIX: &[u8] = b"net/";

const KIND_OFFSET: usize = 4;
const KIND_DELIVER: u8 = 5;
const HEADER_LENGTH: usize = 8;

impl Broker {
    pub async fn start_udp_listener(&self, address: &str) -> Result<()> {
        let socket = Arc::new(UdpSocket::bind(resolve_address(address).await?).await?);
        let (outbound_sender, outbound_receiver) = mpsc::channel(OUTBOUND_CAPACITY);
        tokio::spawn(udp_listen_loop(
            socket,
            self.sender.clone(),
            outbound_sender,
            outbound_receiver,
            self.shutdown_sender.subscribe(),
        ));
        Ok(())
    }
}

struct UdpPeer {
    endpoint: Endpoint,
    writer: Option<PeerWriter>,
    peer: Option<(PeerId, Arc<str>)>,
    shutdown_signal: Option<oneshot::Sender<()>>,
    registered: bool,
    last_seen: Instant,
}

impl UdpPeer {
    fn new(outbound: Sender<(SocketAddr, Bytes)>, address: SocketAddr) -> Self {
        Self {
            endpoint: Endpoint::new(),
            writer: Some(PeerWriter::Udp { outbound, address }),
            peer: None,
            shutdown_signal: None,
            registered: false,
            last_seen: Instant::now(),
        }
    }
}

async fn udp_listen_loop(
    socket: Arc<UdpSocket>,
    event_sender: Sender<BrokerEvent>,
    outbound_sender: Sender<(SocketAddr, Bytes)>,
    mut outbound_receiver: mpsc::Receiver<(SocketAddr, Bytes)>,
    mut shutdown: watch::Receiver<bool>,
) {
    let mut peers: HashMap<SocketAddr, UdpPeer> = HashMap::new();
    let mut retransmit = tokio::time::interval(RETRANSMIT_INTERVAL);
    let started = Instant::now();
    let mut scratch = vec![0_u8; RECEIVE_BUFFER];
    loop {
        let now_millis = started.elapsed().as_millis() as u64;
        tokio::select! {
            received = socket.recv_from(&mut scratch) => {
                let Ok((length, address)) = received else {
                    continue;
                };
                let datagram = Bytes::copy_from_slice(&scratch[..length]);
                receive_datagram(
                    &socket,
                    &event_sender,
                    &outbound_sender,
                    &mut peers,
                    address,
                    datagram,
                    now_millis,
                )
                .await;
            }
            outbound = outbound_receiver.recv() => {
                let Some((address, frame)) = outbound else {
                    break;
                };
                if let Some(peer) = peers.get_mut(&address) {
                    let delivery = if frame_is_control(&frame) {
                        Delivery::ReliableOrdered
                    } else {
                        Delivery::Unreliable
                    };
                    peer.endpoint.send(delivery, &frame);
                    flush_peer(&socket, address, peer, now_millis).await;
                }
            }
            _ = retransmit.tick() => {
                let mut addresses: Vec<SocketAddr> = Vec::new();
                for (address, peer) in peers.iter_mut() {
                    if peer.endpoint.has_outgoing() {
                        addresses.push(*address);
                    }
                }
                for address in addresses {
                    if let Some(peer) = peers.get_mut(&address) {
                        flush_peer(&socket, address, peer, now_millis).await;
                    }
                }
                expire_idle_peers(&mut peers);
            }
            _ = shutdown.changed() => break,
        }
    }
}

async fn receive_datagram(
    socket: &Arc<UdpSocket>,
    event_sender: &Sender<BrokerEvent>,
    outbound_sender: &Sender<(SocketAddr, Bytes)>,
    peers: &mut HashMap<SocketAddr, UdpPeer>,
    address: SocketAddr,
    datagram: Bytes,
    now_millis: u64,
) {
    let peer = match peers.entry(address) {
        Entry::Occupied(occupied) => occupied.into_mut(),
        Entry::Vacant(vacant) => vacant.insert(UdpPeer::new(outbound_sender.clone(), address)),
    };
    peer.last_seen = Instant::now();

    let mut delivered: Vec<Bytes> = Vec::new();
    peer.endpoint.receive(now_millis, &datagram, &mut delivered);
    for frame in delivered {
        let mut frame_buffer = BytesMut::with_capacity(frame.len());
        frame_buffer.extend_from_slice(&frame);
        let alive = process_incoming_frame(
            frame_buffer,
            event_sender,
            &mut peer.writer,
            &mut peer.peer,
            &mut peer.shutdown_signal,
        )
        .await;
        if peer.peer.is_some() {
            peer.registered = true;
        }
        if !alive {
            break;
        }
    }
    flush_peer(socket, address, peer, now_millis).await;
}

async fn flush_peer(socket: &UdpSocket, address: SocketAddr, peer: &mut UdpPeer, now_millis: u64) {
    let mut datagrams: Vec<Bytes> = Vec::new();
    peer.endpoint.drain_outgoing(now_millis, &mut datagrams);
    for datagram in datagrams {
        let _ = socket.send_to(&datagram, address).await;
    }
}

fn expire_idle_peers(peers: &mut HashMap<SocketAddr, UdpPeer>) {
    let now = Instant::now();
    peers.retain(|_address, peer| {
        let timeout = if peer.registered {
            PEER_TIMEOUT
        } else {
            UNREGISTERED_TIMEOUT
        };
        now.duration_since(peer.last_seen) < timeout
    });
}

fn frame_is_control(frame: &[u8]) -> bool {
    if frame.len() < HEADER_LENGTH || frame[KIND_OFFSET] != KIND_DELIVER {
        return false;
    }
    let topic_length = u16::from_be_bytes([frame[6], frame[7]]) as usize;
    let topic_end = HEADER_LENGTH + topic_length;
    if frame.len() < topic_end {
        return false;
    }
    frame[HEADER_LENGTH..topic_end].starts_with(CONTROL_TOPIC_PREFIX)
}
