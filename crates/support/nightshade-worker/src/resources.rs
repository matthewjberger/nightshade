use crate::data::Job;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use std::sync::{Arc, Condvar, Mutex};

pub(crate) type Bench<J> = Arc<Mutex<Vec<(u64, <J as Job>::Key, <J as Job>::Done)>>>;

pub struct Workers<J: Job> {
    pub(crate) queue: Arc<Queue<J>>,
    pub(crate) finished: Bench<J>,
    pub(crate) flight: HashSet<J::Key>,
    pub(crate) order: u64,
    pub(crate) era: u64,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) crew: Vec<std::thread::JoinHandle<()>>,
    #[cfg(target_arch = "wasm32")]
    pub(crate) tools: J::Tools,
}

pub(crate) struct Queue<J: Job> {
    pub(crate) waiting: Mutex<Held<J>>,
    pub(crate) ready: Condvar,
}

pub(crate) struct Held<J: Job> {
    pub(crate) jobs: BinaryHeap<Entry<J>>,
    pub(crate) stopping: bool,
}

pub(crate) struct Entry<J: Job> {
    pub(crate) rank: i32,
    pub(crate) order: u64,
    pub(crate) era: u64,
    pub(crate) key: J::Key,
    pub(crate) job: J,
}

impl<J: Job> Ord for Entry<J> {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .rank
            .cmp(&self.rank)
            .then_with(|| other.order.cmp(&self.order))
    }
}

impl<J: Job> PartialOrd for Entry<J> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<J: Job> PartialEq for Entry<J> {
    fn eq(&self, other: &Self) -> bool {
        self.rank == other.rank && self.order == other.order
    }
}

impl<J: Job> Eq for Entry<J> {}

impl<J: Job> Drop for Workers<J> {
    fn drop(&mut self) {
        let mut held = self
            .queue
            .waiting
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        held.stopping = true;
        held.jobs.clear();
        drop(held);
        self.queue.ready.notify_all();
        #[cfg(not(target_arch = "wasm32"))]
        {
            let hands = self.crew.len();
            for hand in self.crew.drain(..) {
                hand.join().ok();
            }
            release_hands(hands);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn budget() -> usize {
    let cores = std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1);
    (cores / crate::data::CREW_DIVISOR).max(1)
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn free_hands() -> usize {
    let hired = HIRED
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    budget().saturating_sub(*hired)
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn hire_hands(cap: usize) -> usize {
    let mut hired = HIRED
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let free = budget().saturating_sub(*hired).max(1);
    let hands = cap.max(1).min(free);
    *hired += hands;
    hands
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn release_hands(hands: usize) {
    let mut hired = HIRED
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *hired = hired.saturating_sub(hands);
}

#[cfg(not(target_arch = "wasm32"))]
static HIRED: Mutex<usize> = Mutex::new(0);
