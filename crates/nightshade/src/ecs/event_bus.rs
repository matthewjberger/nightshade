//! Event bus for cross-system communication.
//!
//! A simple message queue for decoupling systems and passing events:
//!
//! - [`EventBus`]: Message queue resource
//! - [`Message`]: Event wrapper (input or custom app events)
//! - [`publish_event`]: Send a message to the bus
//! - [`publish_app_event`]: Send a typed custom event
//!
//! # Publishing Events
//!
//! Send custom events from anywhere:
//!
//! ```ignore
//! use nightshade::ecs::event_bus::publish_app_event;
//!
//! struct EnemyDied { entity: Entity, score: i32 }
//!
//! // When an enemy is defeated
//! publish_app_event(world, EnemyDied { entity, score: 100 });
//! ```
//!
//! # Handling Events
//!
//! Process events in your game loop:
//!
//! ```ignore
//! fn run_systems(&mut self, world: &mut World) {
//!     while let Some(message) = world.expect_resource_mut::<crate::ecs::event_bus::resources::EventBus>().messages.pop_front() {
//!         match message {
//!             Message::Input { event } => {
//!                 // Handle input events
//!             }
//!             Message::App { type_name, payload } => {
//!                 // Handle custom events
//!                 if type_name == std::any::type_name::<EnemyDied>() {
//!                     if let Some(event) = payload.downcast_ref::<EnemyDied>() {
//!                         self.score += event.score;
//!                     }
//!                 }
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! # Message Types
//!
//! | Variant | Description |
//! |---------|-------------|
//! | `Message::Input` | Input events (keyboard, mouse, gamepad) |
//! | `Message::App` | Custom application events with type-erased payload |
//!
//! # Pattern: Event-Driven Systems
//!
//! ```ignore
//! struct PlayerDamaged { amount: i32 }
//! struct PlayerHealed { amount: i32 }
//! struct GameOver;
//!
//! // In combat system
//! if hit_detected {
//!     player_health -= damage;
//!     publish_app_event(world, PlayerDamaged { amount: damage });
//!     if player_health <= 0 {
//!         publish_app_event(world, GameOver);
//!     }
//! }
//!
//! // In UI system
//! for message in world.expect_resource_mut::<crate::ecs::event_bus::resources::EventBus>().messages.iter() {
//!     if let Message::App { type_name, payload } = message {
//!         if type_name == std::any::type_name::<PlayerDamaged>() {
//!             // Flash health bar red
//!         }
//!     }
//! }
//! ```
//!
//! [`EventBus`]: resources::EventBus
//! [`Message`]: resources::Message
//! [`publish_event`]: commands::publish_event
//! [`publish_app_event`]: commands::publish_app_event

pub mod commands;
pub mod resources;

pub use commands::*;
pub use resources::*;

use winit::event::WindowEvent;

pub enum EngineEvent<'a> {
    Window(&'a WindowEvent),
}
