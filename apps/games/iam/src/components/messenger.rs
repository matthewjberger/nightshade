use crate::data::script::{Effect, Post, Presence};
use nightshade::prelude::Entity;

#[derive(Default, Clone, Debug)]
pub struct Contact {
    pub definition: usize,
}

#[derive(Default, Clone, Debug)]
pub struct ContactPresence(pub Presence);

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Listing {
    #[default]
    Hidden,
    Pending,
    Listed,
}

#[derive(Default, Clone, Debug)]
pub struct ContactListing(pub Listing);

#[derive(Default, Clone, Debug)]
pub struct ContactDeactivated(pub bool);

#[derive(Default, Clone, Debug)]
pub struct ContactAutomated(pub bool);

#[derive(Default, Clone, Copy)]
pub struct ContactGave(pub bool);

#[derive(Default, Clone, Copy)]
pub struct ContactSpent(pub bool);

#[derive(Default, Clone, Copy)]
pub struct ContactPersuaded(pub bool);

#[derive(Default, Clone, Copy)]
pub struct ContactBurned(pub bool);

#[derive(Default, Clone, Debug)]
pub struct ContactUnread(pub u32);

#[derive(Default, Clone, Debug)]
pub struct ContactInvite(pub &'static str);

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Speaker {
    #[default]
    Contact,
    Player,
    Notice,
}

#[derive(Default, Clone, Debug)]
pub struct TranscriptEntry {
    pub speaker: Speaker,
    pub stamp: String,
    pub text: String,
}

#[derive(Default, Clone, Debug)]
pub struct Transcript {
    pub entries: Vec<TranscriptEntry>,
}

#[derive(Default, Clone, Debug)]
pub struct Conversation {
    pub node: Option<usize>,
    pub delivered: usize,
    pub timer: f32,
    pub segment: usize,
    pub opened: bool,
    pub awaiting_reply: bool,
}

#[derive(Default, Clone, Debug)]
pub struct ThreadView {
    pub column: Entity,
    pub drawn: usize,
    pub last_speaker: Option<Speaker>,
}

#[derive(Default, Clone, Debug)]
pub struct Server {
    pub definition: usize,
}

#[derive(Default, Clone, Debug)]
pub struct ServerListed(pub bool);

#[derive(Default, Clone, Debug)]
pub struct ServerOpenChannel(pub Option<Entity>);

#[derive(Default, Clone, Debug)]
pub struct Channel {
    pub server: Entity,
    pub server_definition: usize,
    pub definition: usize,
}

#[derive(Clone, Debug)]
pub struct FeedEntry {
    pub author: &'static str,
    pub stamp: String,
    pub text: &'static str,
    pub mine: bool,
}

#[derive(Default, Clone, Debug)]
pub struct ChannelFeed {
    pub column: Entity,
    pub log: Vec<FeedEntry>,
    pub drawn: usize,
    pub delivered: usize,
    pub held: Vec<usize>,
    pub queue: Vec<&'static Post>,
    pub queued_effects: &'static [Effect],
    pub timer: f32,
    pub last_author: Option<&'static str>,
}

#[derive(Default, Clone, Debug)]
pub struct ChannelAsked(pub Vec<usize>);

#[derive(Default, Clone, Debug)]
pub struct ChannelUnread(pub u32);

#[derive(Default, Clone, Copy)]
pub struct RowContact(pub Option<Entity>);

#[derive(Default, Clone, Copy)]
pub struct RowServer(pub Option<Entity>);

#[derive(Default, Clone, Copy)]
pub struct RowChannel(pub Option<Entity>);

#[derive(Default, Clone, Copy)]
pub struct ChoiceSlot {
    pub slot: usize,
    pub label: Entity,
    pub choice: Option<usize>,
}
