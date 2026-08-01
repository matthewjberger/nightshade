use crate::data::{BODIES, BodyDefinition};
use nightshade::prelude::{Quat, Vec3, nalgebra_glm};

const VELOCITY_STEP: f32 = 0.05;
const MINIMUM_DISTANCE: f32 = 0.001;
const POLE_LIMIT: f32 = 0.985;

#[derive(Clone, Copy)]
pub struct Contact {
    pub body: usize,
    pub outward: Vec3,
    pub up: Vec3,
    pub height: f32,
    pub floor: f32,
    pub inside: bool,
    pub solid: bool,
}

impl Default for Contact {
    fn default() -> Self {
        Self {
            body: 0,
            outward: Vec3::y(),
            up: Vec3::y(),
            height: f32::MAX,
            floor: 0.0,
            inside: false,
            solid: false,
        }
    }
}

impl Contact {
    pub fn resting(&self, center: Vec3, clearance: f32) -> Vec3 {
        let distance = if self.inside {
            self.floor - clearance
        } else {
            self.floor + clearance
        };
        center + self.outward * distance
    }
}

pub fn body_position(index: usize, time: f32) -> Vec3 {
    let definition = &BODIES[index];
    let base = match definition.parent {
        Some(parent) => body_position(parent, time),
        None => Vec3::zeros(),
    };
    base + orbit_offset(definition, time)
}

pub fn body_velocity(index: usize, time: f32) -> Vec3 {
    (body_position(index, time + VELOCITY_STEP) - body_position(index, time - VELOCITY_STEP))
        / (2.0 * VELOCITY_STEP)
}

pub fn gravity_at(point: Vec3, time: f32) -> Vec3 {
    let mut total = Vec3::zeros();
    for (index, definition) in BODIES.iter().enumerate() {
        if definition.gravity <= 0.0 {
            continue;
        }
        let to_center = body_position(index, time) - point;
        let distance = to_center.magnitude();
        if distance < MINIMUM_DISTANCE {
            continue;
        }
        let radial = to_center / distance;
        let clamped = distance.max(definition.radius * 0.6);
        let pull = definition.gravity / (clamped * clamped);
        if definition.inner_radius > 0.0 && distance < definition.inner_radius {
            total -= radial * pull;
        } else {
            total += radial * pull;
        }
    }
    total
}

pub fn nearest_contact(point: Vec3, time: f32) -> Contact {
    let mut nearest = Contact::default();
    for (index, definition) in BODIES.iter().enumerate() {
        if definition.radius <= 0.0 {
            continue;
        }
        let candidate = contact_with(index, definition, point, time);
        if candidate.height < nearest.height {
            nearest = candidate;
        }
    }
    nearest
}

pub fn body_contact(index: usize, point: Vec3, time: f32) -> Contact {
    contact_with(index, &BODIES[index], point, time)
}

pub fn surface_position(body: usize, anchor: Vec3, height: f32, inside: bool, time: f32) -> Vec3 {
    let definition = &BODIES[body];
    let radial = normalized_or(anchor, Vec3::y());
    let distance = if inside {
        definition.inner_radius - height
    } else {
        definition.radius + height
    };
    body_position(body, time) + radial * distance
}

pub fn normalized_or(vector: Vec3, fallback: Vec3) -> Vec3 {
    let length = vector.magnitude();
    if length < MINIMUM_DISTANCE {
        fallback
    } else {
        vector / length
    }
}

pub fn tangent_frame(up: Vec3, yaw: f32) -> (Vec3, Vec3) {
    let mut reference = Vec3::y();
    if reference.dot(&up).abs() > POLE_LIMIT {
        reference = Vec3::x();
    }
    let east = normalized_or(reference.cross(&up), Vec3::x());
    let north = normalized_or(up.cross(&east), Vec3::z());
    let forward = normalized_or(north * yaw.cos() + east * yaw.sin(), north);
    let right = normalized_or(forward.cross(&up), east);
    (forward, right)
}

pub fn blend_rotation(current: Quat, goal: Quat, amount: f32) -> Quat {
    let aligned = if current.dot(&goal) < 0.0 {
        -goal
    } else {
        goal
    };
    (current * (1.0 - amount) + aligned * amount).normalize()
}

pub fn basis_rotation(forward: Vec3, up: Vec3) -> Quat {
    let back = normalized_or(-forward, Vec3::z());
    let mut right = up.cross(&back);
    if right.magnitude() < MINIMUM_DISTANCE {
        right = Vec3::x().cross(&back);
    }
    let right = normalized_or(right, Vec3::x());
    let corrected = back.cross(&right);
    nalgebra_glm::mat3_to_quat(&nalgebra_glm::Mat3::from_columns(&[right, corrected, back]))
        .normalize()
}

pub fn body_name(index: usize) -> &'static str {
    BODIES[index].name
}

pub fn body_radius(index: usize) -> f32 {
    BODIES[index].radius
}

pub fn drag_at(point: Vec3, time: f32) -> f32 {
    for (index, definition) in BODIES.iter().enumerate() {
        if definition.drag <= 0.0 {
            continue;
        }
        if (body_position(index, time) - point).magnitude() < definition.radius {
            return definition.drag;
        }
    }
    0.0
}

fn contact_with(index: usize, definition: &BodyDefinition, point: Vec3, time: f32) -> Contact {
    let offset = point - body_position(index, time);
    let distance = offset.magnitude();
    let radial = normalized_or(offset, Vec3::y());
    let solid = definition.drag <= 0.0 && !in_aperture(definition, radial);
    let inside = definition.inner_radius > 0.0 && distance < definition.inner_radius;
    let floor = if inside {
        definition.inner_radius
    } else {
        definition.radius
    };

    Contact {
        body: index,
        outward: radial,
        up: if inside { -radial } else { radial },
        height: if inside {
            floor - distance
        } else {
            distance - floor
        },
        floor,
        inside,
        solid,
    }
}

fn in_aperture(definition: &BodyDefinition, radial: Vec3) -> bool {
    definition.aperture < 1.0 && radial.y > definition.aperture
}

fn orbit_offset(definition: &BodyDefinition, time: f32) -> Vec3 {
    if definition.orbit_radius <= 0.0 {
        return Vec3::zeros();
    }
    let angle =
        definition.orbit_phase + time * std::f32::consts::TAU / definition.orbit_period.max(0.001);
    let stretch = definition.orbit_stretch;
    let flat_x =
        angle.cos() * definition.orbit_radius * stretch - definition.orbit_radius * (stretch - 1.0);
    let flat_z = angle.sin() * definition.orbit_radius;
    Vec3::new(
        flat_x,
        flat_z * definition.orbit_tilt.sin(),
        flat_z * definition.orbit_tilt.cos(),
    )
}
