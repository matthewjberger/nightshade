use crate::{Client, ClientSettings, Message, Result};
use std::{
    future::Future,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interrupt {
    UpdateImmediately,
    Stop,
}

pub trait Lifecycle: Send {
    fn initialize(&mut self, _client: &Client) -> impl Future<Output = ()> + Send {
        std::future::ready(())
    }

    fn update(&mut self, _client: &Client) -> impl Future<Output = ()> + Send {
        std::future::ready(())
    }

    fn receive_message(
        &mut self,
        _message: &Message,
        _client: &Client,
    ) -> impl Future<Output = Option<Interrupt>> + Send {
        std::future::ready(None)
    }
}

#[derive(Debug, Clone)]
pub struct LifecycleSettings {
    pub name: String,
    pub broker_address: String,
    pub update_interval: Duration,
}

pub async fn run_lifecycle(
    mut lifecycle: impl Lifecycle,
    settings: LifecycleSettings,
) -> Result<()> {
    let client = Client::new(&settings.name, ClientSettings::default());
    client.connect(&settings.broker_address).await?;
    lifecycle.initialize(&client).await;
    loop {
        let deadline = Instant::now() + settings.update_interval;
        let interrupt = receive_messages_until_deadline(&mut lifecycle, &client, deadline).await;
        if interrupt == Some(Interrupt::Stop) {
            return Ok(());
        }
        lifecycle.update(&client).await;
    }
}

async fn receive_messages_until_deadline(
    lifecycle: &mut impl Lifecycle,
    client: &Client,
    deadline: Instant,
) -> Option<Interrupt> {
    loop {
        let now = Instant::now();
        if now >= deadline {
            return None;
        }
        let remaining = deadline - now;
        match tokio::time::timeout(remaining, client.next_message()).await {
            Ok(Some(message)) => match lifecycle.receive_message(&message, client).await {
                Some(Interrupt::UpdateImmediately) => {
                    return Some(Interrupt::UpdateImmediately);
                }
                Some(Interrupt::Stop) => return Some(Interrupt::Stop),
                None => {}
            },
            _ => return None,
        }
    }
}
