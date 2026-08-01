use std::sync::{Arc, Mutex};

pub fn report(notice: &Arc<Mutex<Option<String>>>, message: impl Into<String>) {
    if let Ok(mut guard) = notice.lock() {
        *guard = Some(message.into());
    }
}
