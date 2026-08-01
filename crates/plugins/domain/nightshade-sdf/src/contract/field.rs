use nalgebra_glm::{Mat4, Vec3, Vec4};

use crate::contract::render_data::SdfEditInstance;
use crate::contract::{SdfOperation, SdfShape};

pub const SDF_FAR: f32 = 1.0e6;

pub const SCALE_FLOOR: f32 = 1.0e-6;

const LACUNARITY: f32 = 2.0;

const LATTICE_RADIUS: f32 = 0.5;

const TURN_AXIS: Vec3 = Vec3::new(0.36, 0.48, 0.80);

const GOLDEN_ANGLE: f32 = 2.399_963;

const TOP_STEPS: u32 = 1024;

const TOP_CLOSE: f32 = 1.0e-3;

const TOP_CREEP: f32 = 0.01;

const TOP_THICK: f32 = 0.5;

const OCTAVE_SWAY: f32 = 0.25;

const BLEND_GUARD: f32 = 2.0;

const OCTAHEDRON_FACE: f32 = 0.577_350_26;

pub fn sdf_field(edits: &[SdfEditInstance], point: &Vec3, care: f32) -> f32 {
    let guarded = care + blend_guard(edits);
    let mut found = SDF_FAR;
    for edit in edits {
        found = edit_into(found, edit, point, guarded);
    }
    found
}

pub fn blend_guard(edits: &[SdfEditInstance]) -> f32 {
    edits
        .iter()
        .filter(|edit| edit.visible)
        .fold(0.0_f32, |widest, edit| widest.max(edit.blend))
        * BLEND_GUARD
}

fn edit_into(found: f32, edit: &SdfEditInstance, point: &Vec3, care: f32) -> f32 {
    if !edit.visible {
        return found;
    }
    let smoothing = edit.blend.max(0.0);
    let scale = uniform_scale(&edit.transform);
    if !matches!(edit.shape, SdfShape::Terrain { .. })
        && !(scale.is_finite() && scale > SCALE_FLOOR)
    {
        return found;
    }
    if edit.operation != SdfOperation::Intersect
        && let Some(reach) = edit.shape.reach()
    {
        let middle = edit.transform.column(3).xyz();
        let bound = reach * scale + smoothing;
        let gap = (point - middle).magnitude() - bound;
        if gap > found.abs() + smoothing {
            return found;
        }
    }
    let shape = shape_distance(edit, point, scale, care);
    match edit.operation {
        SdfOperation::Union => union(found, shape, smoothing),
        SdfOperation::Subtract => subtract(found, shape, smoothing),
        SdfOperation::Intersect => intersect(found, shape, smoothing),
    }
}

pub fn uniform_scale(transform: &Mat4) -> f32 {
    let along = [
        transform.column(0).xyz().magnitude(),
        transform.column(1).xyz().magnitude(),
        transform.column(2).xyz().magnitude(),
    ];
    along.iter().fold(0.0_f32, |most, one| most.max(*one))
}

pub fn field_doubt(edits: &[SdfEditInstance]) -> f32 {
    edits
        .iter()
        .filter(|edit| edit.visible)
        .fold(1.0_f32, |doubt, edit| match edit.shape {
            SdfShape::Terrain { octaves, gain, .. } => doubt.max(terrain_doubt(octaves, gain)),
            _ => doubt,
        })
}

fn terrain_doubt(octaves: u32, gain: f32) -> f32 {
    let steepest = (gain.abs() * LACUNARITY).powi(octaves.saturating_sub(1) as i32);
    steepest.max(1.0)
}

struct Terrain {
    base: f32,
    size: f32,
    octaves: u32,
    gain: f32,
    clip: f32,
    clip_smooth: f32,
    join_smooth: f32,
}

impl Terrain {
    fn from_shape(shape: &SdfShape) -> Option<Self> {
        let SdfShape::Terrain {
            base,
            size,
            octaves,
            gain,
            clip,
            clip_smooth,
            join_smooth,
            ..
        } = *shape
        else {
            return None;
        };
        Some(Self {
            base,
            size,
            octaves,
            gain,
            clip,
            clip_smooth,
            join_smooth,
        })
    }

    fn band(&self) -> f32 {
        let mut band = 0.0;
        let mut amplitude = self.size.abs();
        for _ in 0..self.octaves {
            band += amplitude
                * (self.clip.abs()
                    + LATTICE_RADIUS
                    + 0.25 * (self.clip_smooth.abs() + self.join_smooth.abs()));
            amplitude *= self.gain.abs();
        }
        band
    }
}

pub fn terrain_top(shape: &SdfShape, x: f32, z: f32) -> Option<f32> {
    let settings = Terrain::from_shape(shape)?;
    let band = settings.band();
    let floor = settings.base - band;
    let sample = |y: f32| terrain(&Vec3::new(x, y, z), &settings, f32::INFINITY);
    let mut y = settings.base + band;
    for _ in 0..TOP_STEPS {
        let value = sample(y);
        if value < TOP_CLOSE {
            if sample(y - TOP_THICK) < 0.0 {
                return Some(y);
            }
            y -= TOP_THICK;
            continue;
        }
        y -= value.max(TOP_CREEP);
        if y < floor {
            return Some(settings.base);
        }
    }
    Some(y)
}

fn turn(point: &Vec3, octave: u32) -> Vec3 {
    let angle = octave as f32 * GOLDEN_ANGLE;
    let (sine, cosine) = angle.sin_cos();
    point * cosine
        + TURN_AXIS.cross(point) * sine
        + TURN_AXIS * (TURN_AXIS.dot(point) * (1.0 - cosine))
}

fn shape_distance(edit: &SdfEditInstance, point: &Vec3, scale: f32, care: f32) -> f32 {
    if let Some(settings) = Terrain::from_shape(&edit.shape) {
        return terrain(point, &settings, care);
    }
    let inverse = nalgebra_glm::inverse(&edit.transform);
    let local = (inverse * Vec4::new(point.x, point.y, point.z, 1.0)).xyz();
    let reach = match edit.shape {
        SdfShape::Sphere { radius } => sphere(&local, radius),
        SdfShape::Cube {
            half_extents,
            rounding,
        } => cube(&local, &half_extents, rounding),
        SdfShape::Capsule {
            half_height,
            radius,
        } => capsule(&local, half_height, radius),
        SdfShape::Torus {
            major_radius,
            minor_radius,
        } => torus(&local, major_radius, minor_radius),
        SdfShape::Cylinder {
            half_height,
            radius,
        } => cylinder(&local, half_height, radius),
        SdfShape::Octahedron { radius, rounding } => octahedron(&local, radius) - rounding,
        SdfShape::Terrain { .. } => SDF_FAR,
    };
    reach * scale
}

fn sphere(point: &Vec3, radius: f32) -> f32 {
    point.magnitude() - radius
}

fn cube(point: &Vec3, half_extents: &Vec3, rounding: f32) -> f32 {
    let outside = Vec3::new(
        point.x.abs() - half_extents.x,
        point.y.abs() - half_extents.y,
        point.z.abs() - half_extents.z,
    );
    let corner = Vec3::new(outside.x.max(0.0), outside.y.max(0.0), outside.z.max(0.0)).magnitude();
    let inside = outside.x.max(outside.y).max(outside.z).min(0.0);
    corner + inside - rounding
}

fn capsule(point: &Vec3, half_height: f32, radius: f32) -> f32 {
    let along = Vec3::new(
        point.x,
        point.y - point.y.clamp(-half_height, half_height),
        point.z,
    );
    along.magnitude() - radius
}

fn torus(point: &Vec3, major_radius: f32, minor_radius: f32) -> f32 {
    let flat = (point.x * point.x + point.z * point.z).sqrt() - major_radius;
    (flat * flat + point.y * point.y).sqrt() - minor_radius
}

fn cylinder(point: &Vec3, half_height: f32, radius: f32) -> f32 {
    let across = (point.x * point.x + point.z * point.z).sqrt().abs() - radius;
    let up = point.y.abs() - half_height;
    let corner = (across.max(0.0) * across.max(0.0) + up.max(0.0) * up.max(0.0)).sqrt();
    across.max(up).min(0.0) + corner
}

fn octahedron(point: &Vec3, radius: f32) -> f32 {
    let folded = Vec3::new(point.x.abs(), point.y.abs(), point.z.abs());
    let plane = folded.x + folded.y + folded.z - radius;
    let turned = if 3.0 * folded.x < plane {
        folded
    } else if 3.0 * folded.y < plane {
        Vec3::new(folded.y, folded.z, folded.x)
    } else if 3.0 * folded.z < plane {
        Vec3::new(folded.z, folded.x, folded.y)
    } else {
        return plane * OCTAHEDRON_FACE;
    };
    let along = (0.5 * (turned.z - turned.y + radius)).clamp(0.0, radius);
    Vec3::new(turned.x, turned.y - radius + along, turned.z - along).magnitude()
}

fn hash(cell: &Vec3) -> f32 {
    let scattered = Vec3::new(
        part(cell.x * 0.1031),
        part(cell.y * 0.1030),
        part(cell.z * 0.0973),
    );
    let turned = Vec3::new(scattered.y, scattered.z, scattered.x) + Vec3::repeat(33.33);
    let shared = scattered.dot(&turned);
    let mixed = scattered + Vec3::repeat(shared);
    part((mixed.x + mixed.y) * mixed.z)
}

fn part(value: f32) -> f32 {
    value - value.floor()
}

fn lattice(point: &Vec3) -> f32 {
    let cell = Vec3::new(point.x.floor(), point.y.floor(), point.z.floor());
    let offset = point - cell;
    let mut nearest = SDF_FAR;
    for corner in 0..8_u32 {
        let at = Vec3::new(
            (corner & 1) as f32,
            ((corner >> 1) & 1) as f32,
            (corner >> 2) as f32,
        );
        let radius = LATTICE_RADIUS * hash(&(cell + at));
        nearest = nearest.min((offset - at).magnitude() - radius);
    }
    nearest
}

fn terrain(point: &Vec3, settings: &Terrain, care: f32) -> f32 {
    let mut found = point.y - settings.base;
    let mut amplitude = settings.size;
    let mut frequency = 1.0 / settings.size;
    let budget = if settings.gain.abs() < 1.0 {
        (settings.clip.abs()
            + OCTAVE_SWAY * (settings.clip_smooth.abs() + settings.join_smooth.abs()))
            / (1.0 - settings.gain.abs())
    } else {
        f32::INFINITY
    };
    for octave in 0..settings.octaves {
        let spheres = lattice(&(turn(point, octave) * frequency)) * amplitude;
        let clipped = intersect(
            spheres,
            found - settings.clip * amplitude,
            settings.clip_smooth * amplitude,
        );
        found = union(found, clipped, settings.join_smooth * amplitude);
        amplitude *= settings.gain;
        frequency *= LACUNARITY;
        let sway = amplitude.abs() * budget;
        if found > sway + care {
            return found - sway;
        }
        if found < -(sway + care) {
            return found;
        }
    }
    found
}

fn mix(low: f32, high: f32, share: f32) -> f32 {
    low + (high - low) * share
}

fn union(world: f32, shape: f32, smoothing: f32) -> f32 {
    if smoothing <= 0.0 {
        return world.min(shape);
    }
    let share = (0.5 + 0.5 * (shape - world) / smoothing).clamp(0.0, 1.0);
    mix(shape, world, share) - smoothing * share * (1.0 - share)
}

fn subtract(world: f32, shape: f32, smoothing: f32) -> f32 {
    if smoothing <= 0.0 {
        return world.max(-shape);
    }
    let share = (0.5 - 0.5 * (world + shape) / smoothing).clamp(0.0, 1.0);
    mix(world, -shape, share) + smoothing * share * (1.0 - share)
}

fn intersect(world: f32, shape: f32, smoothing: f32) -> f32 {
    if smoothing <= 0.0 {
        return world.max(shape);
    }
    let share = (0.5 - 0.5 * (shape - world) / smoothing).clamp(0.0, 1.0);
    mix(shape, world, share) + smoothing * share * (1.0 - share)
}
