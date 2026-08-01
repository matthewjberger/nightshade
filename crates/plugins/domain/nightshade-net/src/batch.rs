#[cfg(not(target_arch = "wasm32"))]
use crate::{Client, Result, Route};
#[cfg(not(target_arch = "wasm32"))]
use serde::Serialize;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};

#[cfg(not(target_arch = "wasm32"))]
pub struct Batch<T> {
    pub topic: String,
    pub route: Route,
    pub items: Vec<T>,
    pub max_items: usize,
    pub flush_interval: Duration,
    pub last_flush: Instant,
}

#[cfg(not(target_arch = "wasm32"))]
impl<T> Batch<T> {
    pub fn new(topic: &str, route: Route, max_items: usize, flush_interval: Duration) -> Batch<T> {
        Batch {
            topic: topic.to_string(),
            route,
            items: Vec::new(),
            max_items,
            flush_interval,
            last_flush: Instant::now(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T: Serialize> Batch<T> {
    pub async fn push(&mut self, client: &Client, item: T) -> Result<()> {
        self.items.push(item);
        if self.items.len() >= self.max_items || self.last_flush.elapsed() >= self.flush_interval {
            self.flush(client).await?;
        }
        Ok(())
    }

    pub async fn flush(&mut self, client: &Client) -> Result<()> {
        if self.items.is_empty() {
            return Ok(());
        }
        let payload = postcard::to_allocvec(&self.items)?;
        client
            .publish_bytes(&self.topic, &payload, self.route)
            .await?;
        self.items.clear();
        self.last_flush = Instant::now();
        Ok(())
    }
}
