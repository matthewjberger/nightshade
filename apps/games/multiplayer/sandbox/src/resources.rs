use crate::data::assets::MODEL_COUNT;
use crate::data::wire::{Body, PeerId, Request, Seat, Thing, Update};
use nalgebra_glm::Vec3;
use nightshade::prelude::Entity;
use nightshade_animation::contract::AnimationClip;
use nightshade_cli::clap;
use nightshade_load::LoadHandle;
use nightshade_prefab::ModelSkin;
use nightshade_prefab::Prefab;
use std::collections::{HashMap, VecDeque};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

#[derive(clap::Args, Clone, Debug, Default)]
pub struct Arguments {
    #[arg(long, help = "Serve on this port and play as the host")]
    pub host: Option<u16>,
    #[arg(long, help = "Join the host at this address, as host:port or host")]
    pub join: Option<String>,
    #[arg(long, help = "The name the other players see")]
    pub name: Option<String>,
    #[arg(long, help = "Open in the world instead of the title screen")]
    pub play: bool,
    #[arg(
        long,
        help = "Serve the world with no window and no one at the keyboard"
    )]
    pub serve: bool,
    #[arg(
        long,
        help = "Walk the character in a circle with no one at the keyboard"
    )]
    pub wander: bool,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    #[default]
    Title,
    Playing,
    Paused,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Role {
    Host,
    Guest,
}

pub enum Outgoing {
    Everyone(Update),
    One(PeerId, Update),
    Host(Request),
}

pub enum Incoming {
    FromGuest(PeerId, Request),
    FromHost(Update),
    Departed(PeerId),
    Note(String),
}

pub struct Wire {
    pub role: Role,
    pub peer: PeerId,
    pub to_net: UnboundedSender<Outgoing>,
    pub from_net: UnboundedReceiver<Incoming>,
    pub stop: Option<tokio::sync::oneshot::Sender<()>>,
    pub crew: Option<std::thread::JoinHandle<()>>,
}

impl Drop for Wire {
    fn drop(&mut self) {
        drop(self.stop.take());
        if let Some(crew) = self.crew.take() {
            crew.join().ok();
        }
    }
}

#[derive(Clone)]
pub struct ModelKit {
    pub prefab: Prefab,
    pub clips: Vec<AnimationClip>,
    pub skins: Vec<ModelSkin>,
}

pub struct Models {
    pub handles: Vec<Option<LoadHandle>>,
    pub kits: Vec<Option<ModelKit>>,
    pub failed: Vec<bool>,
    pub requested: Vec<bool>,
}

impl Default for Models {
    fn default() -> Self {
        Self {
            handles: vec![None; MODEL_COUNT],
            kits: vec![None; MODEL_COUNT],
            failed: vec![false; MODEL_COUNT],
            requested: vec![false; MODEL_COUNT],
        }
    }
}

#[derive(Clone, Copy)]
pub struct Sent {
    pub sequence: u32,
    pub walk: Vec3,
    pub wants: u8,
}

#[derive(Default, Clone)]
pub struct Mailbox {
    pub bodies: HashMap<PeerId, Body>,
    pub things: HashMap<u32, Thing>,
}

#[derive(Clone, Copy)]
pub struct Shot {
    pub peer: PeerId,
    pub tick: u32,
    pub origin: Vec3,
    pub direction: Vec3,
}

pub struct Session {
    pub screen: Screen,
    pub local: Option<Entity>,
    pub camera: Option<Entity>,
    pub bodies: HashMap<PeerId, Entity>,
    pub seats: Vec<Seat>,
    pub name: String,
    pub tick: u32,
    pub accumulator: f32,
    pub stepping: bool,
    pub sequence: u32,
    pub sent: Vec<Sent>,
    pub latest: HashMap<PeerId, (u32, Vec3, u8)>,
    pub predict: bool,
    pub interpolate: bool,
    pub clock: f32,
    pub chat: Vec<String>,
    pub typing: bool,
    pub note: String,
    pub spawned: Vec<Body>,
    pub history: VecDeque<(u32, Vec<Body>)>,
    pub shots: Vec<Shot>,
    pub things: HashMap<u32, Entity>,
    pub replicas: Vec<Thing>,
    pub next_thing: u32,
    pub mailed: HashMap<PeerId, Mailbox>,
    pub aim_origin: Vec3,
    pub aim_direction: Vec3,
    pub fired: bool,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            screen: Screen::Title,
            local: None,
            camera: None,
            bodies: HashMap::new(),
            seats: Vec::new(),
            name: String::new(),
            tick: 0,
            accumulator: 0.0,
            stepping: false,
            sequence: 0,
            sent: Vec::new(),
            latest: HashMap::new(),
            predict: true,
            interpolate: true,
            clock: 0.0,
            chat: Vec::new(),
            typing: false,
            note: String::new(),
            spawned: Vec::new(),
            history: VecDeque::new(),
            shots: Vec::new(),
            things: HashMap::new(),
            replicas: Vec::new(),
            next_thing: 1,
            mailed: HashMap::new(),
            aim_origin: Vec3::zeros(),
            aim_direction: -Vec3::z(),
            fired: false,
        }
    }
}

#[derive(Default, Clone)]
pub struct UiHandles {
    pub title_root: Option<Entity>,
    pub title_play: Option<Entity>,
    pub title_quit: Option<Entity>,
    pub hud_root: Option<Entity>,
    pub hud_roster: Vec<Entity>,
    pub hud_chat: Vec<Entity>,
    pub hud_status: Option<Entity>,
    pub hud_note: Option<Entity>,
    pub hud_input: Option<Entity>,
    pub pause_root: Option<Entity>,
    pub pause_resume: Option<Entity>,
    pub pause_predict: Option<Entity>,
    pub pause_interpolate: Option<Entity>,
    pub pause_quit: Option<Entity>,
}
