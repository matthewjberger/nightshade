use crate::data::{Incoming, Note, Outgoing, Role};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot;

pub struct Wire {
    pub role: Role,
    pub to_net: UnboundedSender<Outgoing>,
    pub from_net: UnboundedReceiver<Incoming>,
    pub notes: UnboundedReceiver<Note>,
    pub stop: Option<oneshot::Sender<()>>,
    pub crew: Option<std::thread::JoinHandle<()>>,
}

impl Drop for Wire {
    fn drop(&mut self) {
        drop(self.stop.take());
        if let Some(crew) = self.crew.take() {
            let _ = crew.join();
        }
    }
}
