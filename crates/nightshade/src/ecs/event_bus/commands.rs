use crate::ecs::world::{World, events::Message};
use std::any::Any;

pub fn publish_event(world: &mut World, message: Message) {
    world
        .res_mut::<crate::ecs::event_bus::resources::EventBus>()
        .messages
        .push_back(message);
}

pub fn publish_app_event<T: Any + Send + Sync>(world: &mut World, event: T) {
    world
        .res_mut::<crate::ecs::event_bus::resources::EventBus>()
        .messages
        .push_back(Message::App {
            type_name: std::any::type_name::<T>(),
            payload: Box::new(event),
        });
}
