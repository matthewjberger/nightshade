use crate::contract::Route;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Role {
    #[default]
    Alone,
    Host,
    Guest,
}

pub struct WireSettings {
    pub role: Role,
    pub name: String,
    pub bind: String,
    pub connect: String,
    pub subscribe: Vec<String>,
}

impl Default for WireSettings {
    fn default() -> Self {
        Self {
            role: Role::Alone,
            name: "nightshade".to_string(),
            bind: "0.0.0.0:7777".to_string(),
            connect: "127.0.0.1:7777".to_string(),
            subscribe: Vec::new(),
        }
    }
}

pub struct Outgoing {
    pub topic: String,
    pub bytes: Vec<u8>,
    pub route: Route,
}

pub struct Incoming {
    pub topic: String,
    pub sender: String,
    pub bytes: bytes::Bytes,
}

pub enum Note {
    Serving(String),
    Connected(String),
    Waiting(String),
    Failed(String),
}

pub const CONNECT_RETRY_MILLISECONDS: u64 = 400;
