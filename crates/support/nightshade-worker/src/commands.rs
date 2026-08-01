use crate::data::Job;
#[cfg(not(target_arch = "wasm32"))]
use crate::resources::Bench;
use crate::resources::{Entry, Held, Queue, Workers};
use std::collections::{BinaryHeap, HashSet};
use std::sync::{Arc, Condvar, Mutex};

#[cfg(not(target_arch = "wasm32"))]
pub fn workers_hire<J: Job>(cap: usize) -> Workers<J> {
    let queue = Arc::new(Queue {
        waiting: Mutex::new(Held {
            jobs: BinaryHeap::new(),
            stopping: false,
        }),
        ready: Condvar::new(),
    });
    let finished = Arc::new(Mutex::new(Vec::with_capacity(cap)));
    let hands = crate::resources::hire_hands(cap);
    let mut crew = Vec::with_capacity(hands);
    for _ in 0..hands {
        let queue = Arc::clone(&queue);
        let finished = Arc::clone(&finished);
        crew.push(std::thread::spawn(move || labor(queue, finished)));
    }
    Workers {
        queue,
        finished,
        flight: HashSet::new(),
        order: 0,
        era: 0,
        crew,
    }
}

#[cfg(target_arch = "wasm32")]
pub fn workers_hire<J: Job>(cap: usize) -> Workers<J> {
    Workers {
        queue: Arc::new(Queue {
            waiting: Mutex::new(Held {
                jobs: BinaryHeap::new(),
                stopping: false,
            }),
            ready: Condvar::new(),
        }),
        finished: Arc::new(Mutex::new(Vec::with_capacity(cap))),
        flight: HashSet::new(),
        order: 0,
        era: 0,
        tools: <J::Tools as Default>::default(),
    }
}

pub fn workers_submit<J: Job>(workers: &mut Workers<J>, job: J, rank: i32) -> bool {
    let key = job.key();
    if !workers.flight.insert(key) {
        return false;
    }
    workers.order += 1;
    let entry = Entry {
        rank,
        order: workers.order,
        era: workers.era,
        key,
        job,
    };
    workers
        .queue
        .waiting
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .jobs
        .push(entry);
    workers.queue.ready.notify_one();
    true
}

pub fn workers_cancel<J: Job>(workers: &mut Workers<J>, key: J::Key) -> bool {
    if !workers.flight.contains(&key) {
        return false;
    }
    let mut held = workers
        .queue
        .waiting
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let before = held.jobs.len();
    let kept: BinaryHeap<Entry<J>> = std::mem::take(&mut held.jobs)
        .into_iter()
        .filter(|entry| entry.key != key)
        .collect();
    let dropped = kept.len() < before;
    held.jobs = kept;
    drop(held);
    if dropped {
        workers.flight.remove(&key);
    }
    dropped
}

pub fn workers_clear<J: Job>(workers: &mut Workers<J>) {
    workers
        .queue
        .waiting
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .jobs
        .clear();
    workers
        .finished
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clear();
    workers.flight.clear();
    workers.era += 1;
}

pub fn workers_take<J: Job>(workers: &mut Workers<J>, limit: usize) -> Vec<J::Done> {
    #[cfg(target_arch = "wasm32")]
    labor_now(workers, limit);
    let taken: Vec<(u64, J::Key, J::Done)> = {
        let mut finished = workers
            .finished
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let count = limit.min(finished.len());
        finished.drain(..count).collect()
    };
    let mut done = Vec::with_capacity(taken.len());
    for (era, key, result) in taken {
        if era != workers.era {
            continue;
        }
        workers.flight.remove(&key);
        done.push(result);
    }
    done
}

#[cfg(not(target_arch = "wasm32"))]
fn labor<J: Job>(queue: Arc<Queue<J>>, finished: Bench<J>) {
    let mut tools = <J::Tools as Default>::default();
    loop {
        let entry = {
            let mut held = queue
                .waiting
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            loop {
                if held.stopping {
                    return;
                }
                if let Some(entry) = held.jobs.pop() {
                    break entry;
                }
                held = queue
                    .ready
                    .wait(held)
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
            }
        };
        let key = entry.key;
        let era = entry.era;
        let done = entry.job.run(&mut tools);
        finished
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push((era, key, done));
    }
}

#[cfg(target_arch = "wasm32")]
fn labor_now<J: Job>(workers: &mut Workers<J>, limit: usize) {
    for _ in 0..limit {
        let taken = workers
            .queue
            .waiting
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .jobs
            .pop();
        let Some(entry) = taken else {
            return;
        };
        let key = entry.key;
        let era = entry.era;
        let done = entry.job.run(&mut workers.tools);
        workers
            .finished
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push((era, key, done));
    }
}
