pub use crate::contract::events::{CollisionEvent, CollisionEventKind, ContactForceEvent};
use rapier3d::prelude::*;
use std::sync::Mutex;

pub(crate) struct RawCollisionEvent {
    pub collider_a: ColliderHandle,
    pub collider_b: ColliderHandle,
    pub started: bool,
    pub is_sensor: bool,
}

pub(crate) struct RawContactForceEvent {
    pub collider_a: ColliderHandle,
    pub collider_b: ColliderHandle,
    pub total_force_magnitude: f32,
}

#[derive(Default)]
pub(crate) struct CollisionEventCollector {
    pub raw_collisions: Mutex<Vec<RawCollisionEvent>>,
    pub raw_contact_forces: Mutex<Vec<RawContactForceEvent>>,
}

impl CollisionEventCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn drain_raw_collisions(&self) -> Vec<RawCollisionEvent> {
        let mut lock = self.raw_collisions.lock().unwrap();
        std::mem::take(&mut *lock)
    }

    pub fn drain_raw_contact_forces(&self) -> Vec<RawContactForceEvent> {
        let mut lock = self.raw_contact_forces.lock().unwrap();
        std::mem::take(&mut *lock)
    }
}

impl EventHandler for CollisionEventCollector {
    fn handle_collision_event(
        &self,
        _bodies: &RigidBodySet,
        _colliders: &ColliderSet,
        event: rapier3d::geometry::CollisionEvent,
        _contact_pair: Option<&ContactPair>,
    ) {
        let (collider_a, collider_b, started, is_sensor) = match event {
            rapier3d::geometry::CollisionEvent::Started(a, b, flags) => {
                (a, b, true, flags.contains(CollisionEventFlags::SENSOR))
            }
            rapier3d::geometry::CollisionEvent::Stopped(a, b, flags) => {
                (a, b, false, flags.contains(CollisionEventFlags::SENSOR))
            }
        };
        let mut lock = self.raw_collisions.lock().unwrap();
        lock.push(RawCollisionEvent {
            collider_a,
            collider_b,
            started,
            is_sensor,
        });
    }

    fn handle_contact_force_event(
        &self,
        _dt: Real,
        _bodies: &RigidBodySet,
        _colliders: &ColliderSet,
        contact_pair: &ContactPair,
        total_force_magnitude: Real,
    ) {
        let mut lock = self.raw_contact_forces.lock().unwrap();
        lock.push(RawContactForceEvent {
            collider_a: contact_pair.collider1,
            collider_b: contact_pair.collider2,
            total_force_magnitude,
        });
    }
}
