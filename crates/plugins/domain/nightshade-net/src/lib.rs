#[cfg(not(target_arch = "wasm32"))]
mod arena;
mod batch;
#[cfg(not(target_arch = "wasm32"))]
mod bridge;
#[cfg(not(target_arch = "wasm32"))]
mod broker;
#[cfg(not(target_arch = "wasm32"))]
mod bus;
#[cfg(not(target_arch = "wasm32"))]
mod client;
#[cfg(all(feature = "plugin", not(target_arch = "wasm32")))]
pub mod commands;
mod contract;
#[cfg(all(feature = "plugin", not(target_arch = "wasm32")))]
pub mod data;
#[cfg(not(target_arch = "wasm32"))]
mod lifecycle;
#[cfg(all(feature = "plugin", not(target_arch = "wasm32")))]
pub mod plugin;
#[cfg(not(target_arch = "wasm32"))]
mod reliability;
#[cfg(all(feature = "plugin", not(target_arch = "wasm32")))]
pub mod resources;
#[cfg(not(target_arch = "wasm32"))]
mod thread;
#[cfg(not(target_arch = "wasm32"))]
mod udp;
#[cfg(not(target_arch = "wasm32"))]
mod udp_client;
mod wire;

pub use bytes::Bytes;
pub use enum2contract;

pub use self::{
    contract::*,
    wire::{
        Message, decode_message, encode_hello, encode_publish_binary, encode_publish_text,
        encode_subscribe, encode_unsubscribe,
    },
};

#[cfg(not(target_arch = "wasm32"))]
pub use self::{
    batch::*, broker::*, bus::*, client::*, lifecycle::*, reliability::Delivery, thread::*,
    udp_client::*,
};

#[cfg(all(feature = "plugin", not(target_arch = "wasm32")))]
pub use self::{
    commands::{open_wire, publish, take_messages, take_notes},
    data::{Incoming, Note, Outgoing, Role, WireSettings},
    plugin::NetPlugin,
    resources::Wire,
};

#[derive(Debug)]
pub enum Error {
    NotConnected,
    MaxConnectionAttempts,
    AddressResolution,
    FrameTooLarge,
    TopicTooLong,
    Serialization(String),
    Transport(String),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotConnected => write!(formatter, "client is not connected"),
            Error::MaxConnectionAttempts => {
                write!(formatter, "maximum connection attempts reached")
            }
            Error::AddressResolution => write!(formatter, "could not resolve address"),
            Error::FrameTooLarge => write!(formatter, "frame length exceeds maximum"),
            Error::TopicTooLong => write!(formatter, "topic or identifier exceeds maximum length"),
            Error::Serialization(message) => write!(formatter, "serialization failed: {message}"),
            Error::Transport(message) => write!(formatter, "transport failed: {message}"),
            Error::Other(message) => write!(formatter, "{message}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<postcard::Error> for Error {
    fn from(error: postcard::Error) -> Self {
        Error::Serialization(error.to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Transport(error.to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::Serialization(error.to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<tokio::time::error::Elapsed> for Error {
    fn from(error: tokio::time::error::Elapsed) -> Self {
        Error::Transport(error.to_string())
    }
}

impl From<String> for Error {
    fn from(message: String) -> Self {
        Error::Other(message)
    }
}

impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Error::Other(message.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
