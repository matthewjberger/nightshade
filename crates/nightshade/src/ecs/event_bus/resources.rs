use std::any::Any;
use std::collections::VecDeque;

pub use crate::platform::input::events::{InputEvent, KeyState};

pub enum Message {
    Input {
        event: InputEvent,
    },
    App {
        type_name: &'static str,
        payload: Box<dyn Any + Send + Sync>,
    },
}

pub struct EventBus {
    pub messages: VecDeque<Message>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
