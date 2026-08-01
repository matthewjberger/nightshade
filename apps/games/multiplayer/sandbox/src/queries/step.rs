use crate::data::wire::{WANT_CROUCH, WANT_JUMP};
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade_physics::prelude::{
    PhysicsWorld, physics_world_capsule_contacts, physics_world_cast_ray,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct Motion {
    pub position: Vec3,
    pub velocity: Vec3,
    pub facing: f32,
    pub crouch: f32,
    pub grounded: bool,
    pub jump_hold: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Wish {
    pub walk: Vec3,
    pub wants: u8,
}

impl Wish {
    pub fn jump(&self) -> bool {
        self.wants & WANT_JUMP != 0
    }

    pub fn crouch(&self) -> bool {
        self.wants & WANT_CROUCH != 0
    }
}

pub fn step_character(motion: Motion, wish: Wish, delta: f32, physics: &PhysicsWorld) -> Motion {
    let delta = delta.min(theme::MAX_STEP_SECONDS);
    let mut position = motion.position;
    let mut velocity = motion.velocity;
    let mut grounded = motion.grounded;
    let mut crouch = motion.crouch;
    let mut facing = motion.facing;
    let mut jump_hold = (motion.jump_hold - delta).max(0.0);
    let was_grounded = grounded;

    let crouch_target = if wish.crouch() && grounded { 1.0 } else { 0.0 };
    crouch += (crouch_target - crouch) * (1.0 - (-theme::CROUCH_SHARPNESS * delta).exp());

    let mut walk = wish.walk;
    walk.y = 0.0;
    if walk.norm() > 1.0 {
        walk = walk.normalize();
    }
    let top_speed =
        theme::MAX_GROUND_SPEED * (1.0 - theme::CROUCH_SPEED_PENALTY * crouch.clamp(0.0, 1.0));
    let target = walk * top_speed;
    let acceleration = if grounded {
        theme::GROUND_ACCELERATION
    } else {
        theme::AIR_ACCELERATION
    };
    let blend = (acceleration * delta).min(1.0);
    velocity.x += (target.x - velocity.x) * blend;
    velocity.z += (target.z - velocity.z) * blend;
    if grounded && walk.norm() < 0.01 {
        let friction = 1.0 - (theme::GROUND_FRICTION * delta).min(1.0);
        velocity.x *= friction;
        velocity.z *= friction;
    }

    velocity.y += physics.gravity.y * delta;

    if wish.jump() && grounded {
        velocity.y = theme::JUMP_IMPULSE;
        jump_hold = theme::JUMP_SUPPRESS_SECONDS;
    }

    let probe_origin = position + Vec3::new(0.0, theme::RIDE_PROBE_HEIGHT, 0.0);
    let ground_distance = ground_below(physics, probe_origin);
    if let Some(distance) = ground_distance
        && jump_hold <= 0.0
    {
        let displacement = theme::RIDE_PROBE_HEIGHT - distance;
        let spring = displacement * theme::LIFTER_SPRING_STIFFNESS
            - velocity.y * theme::LIFTER_SPRING_DAMPING;
        velocity.y += spring * delta;
        grounded = distance < theme::RIDE_PROBE_HEIGHT + theme::GROUNDED_MARGIN;
    } else {
        grounded = false;
    }
    if !was_grounded && grounded {
        velocity.y = velocity.y.max(0.0);
    }

    position += velocity * delta;

    for _ in 0..theme::BUMPER_RESOLVE_ITERATIONS {
        let contacts = physics_world_capsule_contacts(
            physics,
            position + Vec3::new(0.0, theme::BUMPER_CENTER_HEIGHT, 0.0),
            theme::BUMPER_HALF_HEIGHT,
            theme::BUMPER_RADIUS,
            None,
        );
        if contacts.is_empty() {
            break;
        }
        for contact in contacts {
            position += contact.normal * contact.depth;
            let into_surface = velocity.dot(&contact.normal);
            if into_surface < 0.0 {
                velocity -= contact.normal * into_surface;
            }
        }
    }

    let planar = Vec3::new(velocity.x, 0.0, velocity.z);
    if planar.norm() > theme::FACING_SPEED_THRESHOLD {
        let wanted = planar.x.atan2(planar.z);
        let mut difference = wanted - facing;
        while difference > std::f32::consts::PI {
            difference -= std::f32::consts::TAU;
        }
        while difference < -std::f32::consts::PI {
            difference += std::f32::consts::TAU;
        }
        facing += difference * (1.0 - (-theme::FACING_TURN_SHARPNESS * delta).exp());
    }

    if position.y < theme::FALL_FLOOR {
        position = Vec3::new(0.0, 2.0, 0.0);
        velocity = Vec3::zeros();
    }

    Motion {
        position,
        velocity,
        facing,
        crouch,
        grounded,
        jump_hold,
    }
}

fn ground_below(physics: &PhysicsWorld, probe_origin: Vec3) -> Option<f32> {
    let radius = theme::LIFTER_RADIUS * 0.6;
    let offsets = [
        Vec3::zeros(),
        Vec3::new(radius, 0.0, 0.0),
        Vec3::new(-radius, 0.0, 0.0),
        Vec3::new(0.0, 0.0, radius),
        Vec3::new(0.0, 0.0, -radius),
    ];
    let reach = theme::RIDE_PROBE_HEIGHT + theme::LIFTER_PROBE_DISTANCE;
    let mut nearest: Option<f32> = None;
    for offset in offsets {
        if let Some(hit) = physics_world_cast_ray(
            physics,
            probe_origin + offset,
            Vec3::new(0.0, -1.0, 0.0),
            reach,
            None,
        ) {
            nearest = Some(nearest.map_or(hit.distance, |best: f32| best.min(hit.distance)));
        }
    }
    nearest
}
