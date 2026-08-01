use crate::{Client, Message, Reception, Result, Route};
use serde::Serialize;
use std::future::Future;

pub trait Publisher: Sync {
    fn publish_encoded(
        &self,
        topic: &str,
        payload: &[u8],
        binary: bool,
        route: Route,
    ) -> impl Future<Output = Result<()>> + Send;

    fn publish<T: Serialize + Sync>(
        &self,
        topic: &str,
        value: &T,
        route: Route,
    ) -> impl Future<Output = Result<()>> + Send {
        async move {
            let payload = postcard::to_allocvec(value)?;
            self.publish_encoded(topic, &payload, true, route).await
        }
    }

    fn publish_json<T: Serialize + Sync>(
        &self,
        topic: &str,
        value: &T,
        route: Route,
    ) -> impl Future<Output = Result<()>> + Send {
        async move {
            let payload = serde_json::to_vec(value)?;
            self.publish_encoded(topic, &payload, false, route).await
        }
    }

    fn publish_bytes(
        &self,
        topic: &str,
        payload: &[u8],
        route: Route,
    ) -> impl Future<Output = Result<()>> + Send {
        self.publish_encoded(topic, payload, true, route)
    }

    fn publish_text(
        &self,
        topic: &str,
        payload: &str,
        route: Route,
    ) -> impl Future<Output = Result<()>> + Send {
        self.publish_encoded(topic, payload.as_bytes(), false, route)
    }
}

pub trait Subscriber: Sync {
    fn subscribe(&self, topics: &[String]) -> impl Future<Output = Result<()>> + Send;

    fn unsubscribe(&self, topics: &[String]) -> impl Future<Output = Result<()>> + Send;

    fn next_message(&self) -> impl Future<Output = Option<Message>> + Send;

    fn try_next_message(&self) -> impl Future<Output = Reception> + Send;
}

pub trait MessageBus: Publisher + Subscriber {}

impl<T: Publisher + Subscriber> MessageBus for T {}

impl Publisher for Client {
    fn publish_encoded(
        &self,
        topic: &str,
        payload: &[u8],
        binary: bool,
        route: Route,
    ) -> impl Future<Output = Result<()>> + Send {
        Client::publish_encoded(self, topic, payload, binary, route)
    }
}

impl Subscriber for Client {
    fn subscribe(&self, topics: &[String]) -> impl Future<Output = Result<()>> + Send {
        Client::subscribe(self, topics)
    }

    fn unsubscribe(&self, topics: &[String]) -> impl Future<Output = Result<()>> + Send {
        Client::unsubscribe(self, topics)
    }

    fn next_message(&self) -> impl Future<Output = Option<Message>> + Send {
        Client::next_message(self)
    }

    fn try_next_message(&self) -> impl Future<Output = Reception> + Send {
        Client::try_next_message(self)
    }
}
