use crate::data::Job;
use crate::resources::Workers;

pub fn workers_in_flight<J: Job>(workers: &Workers<J>) -> usize {
    workers.flight.len()
}

pub fn workers_holds<J: Job>(workers: &Workers<J>, key: J::Key) -> bool {
    workers.flight.contains(&key)
}

pub fn workers_queued<J: Job>(workers: &Workers<J>) -> usize {
    workers
        .queue
        .waiting
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .jobs
        .len()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn workers_free() -> usize {
    crate::resources::free_hands()
}

#[cfg(target_arch = "wasm32")]
pub fn workers_free() -> usize {
    0
}

pub fn workers_waiting<J: Job>(workers: &Workers<J>) -> Vec<J::Key> {
    workers
        .queue
        .waiting
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .jobs
        .iter()
        .map(|entry| entry.key)
        .collect()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn workers_hands<J: Job>(workers: &Workers<J>) -> usize {
    workers.crew.len()
}

#[cfg(target_arch = "wasm32")]
pub fn workers_hands<J: Job>(_workers: &Workers<J>) -> usize {
    0
}
