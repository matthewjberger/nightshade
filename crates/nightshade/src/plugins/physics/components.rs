//! Physics component definitions.

use nalgebra_glm::{Quat, Vec3};
use serde::{Deserialize, Serialize};

use super::types::{ColliderHandle, InteractionGroups, LockedAxes, RigidBodyHandle, RigidBodyType};

/// Rigid body component for physics simulation.
///
/// Defines a physics body that can be dynamic (affected by forces), kinematic
/// (animated but affecting others), or static (fixed in place). Attach alongside
/// a [`ColliderComponent`] to participate in collision detection and response.
#[derive(Debug, Clone, Serialize, Deserialize, enum2schema::Schema)]
pub struct RigidBodyComponent {
    /// Internal handle assigned by the physics engine (managed automatically).
    #[serde(skip)]
    #[schema(skip)]
    pub handle: Option<RigidBodyHandle>,
    /// Body type: Dynamic, KinematicPositionBased, KinematicVelocityBased, or Fixed.
    pub body_type: RigidBodyType,
    /// Initial position in world space.
    pub translation: [f32; 3],
    /// Initial orientation as quaternion [x, y, z, w].
    pub rotation: [f32; 4],
    /// Linear velocity in world units per second.
    pub linvel: [f32; 3],
    /// Angular velocity in radians per second.
    pub angvel: [f32; 3],
    /// Mass in kilograms (only affects dynamic bodies).
    pub mass: f32,
    /// Linear velocity decay rate. 0.0 leaves motion to drag-free space, higher
    /// values bleed off translation over time. Default: 0.0.
    #[serde(default)]
    pub linear_damping: f32,
    /// Angular velocity decay rate. 0.0 spins forever, higher values settle
    /// rotation over time. Default: 0.0.
    #[serde(default)]
    pub angular_damping: f32,
    /// Per-body multiplier on world gravity. 1.0 is normal, 0.0 floats, negative
    /// rises, 2.0 falls twice as hard. Default: 1.0.
    #[serde(default = "default_gravity_scale")]
    pub gravity_scale: f32,
    /// Constraints on which axes can translate or rotate.
    pub locked_axes: LockedAxes,
    /// Enable continuous collision detection. Costs CPU but prevents fast-moving bodies from
    /// tunneling through thin geometry. Default: false.
    #[serde(default)]
    pub ccd_enabled: bool,
}

fn default_gravity_scale() -> f32 {
    1.0
}

impl Default for RigidBodyComponent {
    fn default() -> Self {
        Self {
            handle: None,
            body_type: RigidBodyType::default(),
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            linvel: [0.0, 0.0, 0.0],
            angvel: [0.0, 0.0, 0.0],
            mass: 1.0,
            linear_damping: 0.0,
            angular_damping: 0.0,
            gravity_scale: 1.0,
            locked_axes: LockedAxes::default(),
            ccd_enabled: false,
        }
    }
}

impl RigidBodyComponent {
    /// Creates a dynamic rigid body affected by forces and gravity.
    pub fn new_dynamic() -> Self {
        Self {
            body_type: RigidBodyType::Dynamic,
            ..Default::default()
        }
    }

    /// Creates a kinematic rigid body driven by setting its translation each frame.
    pub fn new_kinematic() -> Self {
        Self {
            body_type: RigidBodyType::KinematicPositionBased,
            ..Default::default()
        }
    }

    /// Creates a kinematic rigid body driven by setting its linear velocity. Unlike
    /// the position-based kind, it responds to `set_linvel`, which is what scripts
    /// use to move it, and dynamic bodies cannot push it.
    pub fn new_kinematic_velocity() -> Self {
        Self {
            body_type: RigidBodyType::KinematicVelocityBased,
            ..Default::default()
        }
    }

    /// Creates a static (fixed) rigid body that never moves.
    pub fn new_static() -> Self {
        Self {
            body_type: RigidBodyType::Fixed,
            ..Default::default()
        }
    }

    /// Sets the initial translation.
    pub fn with_translation(mut self, x: f32, y: f32, z: f32) -> Self {
        self.translation = [x, y, z];
        self
    }

    /// Sets the mass in kilograms.
    pub fn with_mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }

    /// Sets the linear damping (translation decay rate).
    pub fn with_linear_damping(mut self, damping: f32) -> Self {
        self.linear_damping = damping;
        self
    }

    /// Sets the angular damping (rotation decay rate).
    pub fn with_angular_damping(mut self, damping: f32) -> Self {
        self.angular_damping = damping;
        self
    }

    /// Sets the per-body gravity multiplier.
    pub fn with_gravity_scale(mut self, scale: f32) -> Self {
        self.gravity_scale = scale;
        self
    }

    /// Sets the initial rotation as a quaternion.
    pub fn with_rotation(mut self, x: f32, y: f32, z: f32, w: f32) -> Self {
        self.rotation = [x, y, z, w];
        self
    }

    /// Enables continuous collision detection for this body. Use for fast-moving bodies
    /// that would otherwise tunnel through thin geometry (bullets, fast props).
    pub fn with_ccd(mut self, enabled: bool) -> Self {
        self.ccd_enabled = enabled;
        self
    }

    /// Converts to a Rapier rigid body for simulation.
    #[cfg(feature = "physics")]
    pub(crate) fn to_rapier_rigid_body(&self) -> rapier3d::prelude::RigidBody {
        use rapier3d::prelude::*;

        let translation = vector![
            self.translation[0],
            self.translation[1],
            self.translation[2]
        ];
        let rotation =
            rapier3d::na::UnitQuaternion::from_quaternion(rapier3d::na::Quaternion::new(
                self.rotation[3],
                self.rotation[0],
                self.rotation[1],
                self.rotation[2],
            ));
        let position = rapier3d::na::Isometry3::from_parts(translation.into(), rotation);

        let rapier_body_type = super::types::rigid_body_type_to_rapier(self.body_type);
        let rapier_locked_axes = super::types::locked_axes_to_rapier(self.locked_axes);

        let mut rb = RigidBodyBuilder::new(rapier_body_type)
            .pose(position)
            .linvel(vector![self.linvel[0], self.linvel[1], self.linvel[2]])
            .angvel(vector![self.angvel[0], self.angvel[1], self.angvel[2]])
            .linear_damping(self.linear_damping)
            .angular_damping(self.angular_damping)
            .gravity_scale(self.gravity_scale)
            .locked_axes(rapier_locked_axes)
            .ccd_enabled(self.ccd_enabled)
            .build();

        if self.body_type == super::types::RigidBodyType::Dynamic {
            rb.set_additional_mass(self.mass, true);
        }

        rb
    }
}

use super::types::CharacterControllerConfig;

/// First-person character controller with kinematic movement.
///
/// Provides walking, jumping, crouching, and sprinting using Rapier's
/// kinematic character controller. Handles ground detection, slope limits,
/// step climbing, and collision response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, enum2schema::Schema)]
pub struct CharacterControllerComponent {
    /// Configuration for step height, slope limits, and autostep behavior.
    pub config: CharacterControllerConfig,
    /// Whether the character is currently touching the ground.
    pub grounded: bool,
    /// Current movement velocity.
    pub velocity: Vec3,
    /// Collision shape (typically a capsule).
    pub shape: ColliderShape,
    /// Maximum horizontal movement speed.
    pub max_speed: f32,
    /// Acceleration rate when moving.
    pub acceleration: f32,
    /// Upward velocity applied when jumping.
    pub jump_impulse: f32,
    /// Whether the character can currently jump (grounded and not recently jumped).
    pub can_jump: bool,
    /// Whether the character is currently crouching.
    pub is_crouching: bool,
    /// Whether the character is currently sprinting.
    pub is_sprinting: bool,
    /// Whether crouching is enabled.
    pub crouch_enabled: bool,
    /// Speed multiplier when crouching (typically < 1.0).
    pub crouch_speed_multiplier: f32,
    /// Speed multiplier when sprinting (typically > 1.0).
    pub sprint_speed_multiplier: f32,
    /// Capsule half-height when standing.
    pub standing_half_height: f32,
    /// Capsule half-height when crouching.
    pub crouching_half_height: f32,
    /// Internal state for crouch toggle detection.
    pub crouch_input_was_pressed: bool,
    /// Internal state for sprint toggle detection.
    pub sprint_input_was_pressed: bool,
    /// Internal state for jump detection.
    pub jump_input_was_pressed: bool,
    /// Overall character scale factor.
    pub scale: f32,
    /// Friction rate applied when grounded and at or below max_speed.
    /// Higher values = faster deceleration. Applied as `1.0 - (rate * dt)`.
    pub friction_rate: f32,
    /// Friction rate applied when grounded and above max_speed.
    /// Applied as exponential decay `exp(-rate * dt)`.
    pub above_max_friction_rate: f32,
    /// When false, the engine skips sprint/crouch/jump input for this entity.
    /// The game is responsible for setting is_crouching, is_sprinting, and velocity.y.
    /// The engine still handles WASD movement, acceleration, friction, gravity, and collision.
    pub engine_input_enabled: bool,
    /// When true, the engine skips gravity integration and the rapier
    /// character solver for this entity, letting game code teleport
    /// the player's transform without physics fighting it. Used by
    /// the noclip fly path.
    pub disable_gravity: bool,
    /// Camera-relative analog movement injected by game code, treated like a
    /// gamepad left stick: `x` steers right, `y` drives forward, magnitude
    /// clamped to one. Non-zero values override keyboard movement for the
    /// frame, which is how an on-screen joystick moves the player.
    pub external_movement: nalgebra_glm::Vec2,
    /// A one-shot jump request from game code, consumed by the input system on
    /// the next step. Lets an on-screen button jump without a Space key.
    pub external_jump: bool,
}

impl Default for CharacterControllerComponent {
    fn default() -> Self {
        Self {
            config: CharacterControllerConfig::default(),
            grounded: false,
            velocity: Vec3::zeros(),
            shape: ColliderShape::Capsule {
                half_height: 0.9,
                radius: 0.3,
            },
            max_speed: 5.0,
            acceleration: 50.0,
            jump_impulse: 8.0,
            can_jump: false,
            is_crouching: false,
            is_sprinting: false,
            crouch_enabled: true,
            crouch_speed_multiplier: 0.5,
            sprint_speed_multiplier: 1.6,
            standing_half_height: 0.9,
            crouching_half_height: 0.45,
            crouch_input_was_pressed: false,
            sprint_input_was_pressed: false,
            jump_input_was_pressed: false,
            scale: 1.0,
            friction_rate: 8.0,
            above_max_friction_rate: 1.5,
            engine_input_enabled: true,
            disable_gravity: false,
            external_movement: nalgebra_glm::Vec2::zeros(),
            external_jump: false,
        }
    }
}

impl CharacterControllerComponent {
    /// Creates a character controller with a capsule collision shape.
    pub fn new_capsule(half_height: f32, radius: f32) -> Self {
        let character_height = 2.0 * half_height + 2.0 * radius;
        let scale = character_height / 2.4;

        Self {
            config: CharacterControllerConfig::default(),
            grounded: false,
            velocity: Vec3::zeros(),
            shape: ColliderShape::Capsule {
                half_height,
                radius,
            },
            max_speed: 5.0,
            acceleration: 50.0,
            jump_impulse: 8.0,
            can_jump: false,
            is_crouching: false,
            is_sprinting: false,
            crouch_enabled: true,
            crouch_speed_multiplier: 0.5,
            sprint_speed_multiplier: 1.6,
            standing_half_height: half_height,
            crouching_half_height: half_height * 0.5,
            crouch_input_was_pressed: false,
            sprint_input_was_pressed: false,
            jump_input_was_pressed: false,
            scale,
            friction_rate: 8.0,
            above_max_friction_rate: 1.5,
            engine_input_enabled: true,
            disable_gravity: false,
            external_movement: nalgebra_glm::Vec2::zeros(),
            external_jump: false,
        }
    }
}

/// Collision shape component with physical material properties.
///
/// Defines the collision geometry and surface properties for an entity.
/// Can be attached to a rigid body for physics response, or used standalone
/// for trigger volumes (sensors).
#[derive(Debug, Clone, Serialize, Deserialize, enum2schema::Schema)]
pub struct ColliderComponent {
    /// Internal handle assigned by the physics engine (managed automatically).
    #[serde(skip)]
    #[schema(skip)]
    pub handle: Option<ColliderHandle>,
    /// Collision geometry shape.
    pub shape: ColliderShape,
    /// Friction coefficient (0.0 = frictionless, 1.0 = high friction).
    pub friction: f32,
    /// Restitution (bounciness) coefficient (0.0 = no bounce, 1.0 = perfectly elastic).
    pub restitution: f32,
    /// Density used to compute mass from volume (for dynamic bodies).
    pub density: f32,
    /// When true, detects overlaps but doesn't generate collision response.
    pub is_sensor: bool,
    /// Bitmask for filtering which colliders can collide.
    pub collision_groups: InteractionGroups,
    /// Bitmask for filtering which colliders participate in constraint solving.
    pub solver_groups: InteractionGroups,
}

impl Default for ColliderComponent {
    fn default() -> Self {
        Self {
            handle: None,
            shape: ColliderShape::Cuboid {
                hx: 0.5,
                hy: 0.5,
                hz: 0.5,
            },
            friction: 0.5,
            restitution: 0.0,
            density: 1.0,
            is_sensor: false,
            collision_groups: InteractionGroups::all(),
            solver_groups: InteractionGroups::all(),
        }
    }
}

/// Collision shape geometry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, enum2schema::Schema)]
pub enum ColliderShape {
    /// Sphere defined by radius.
    Ball { radius: f32 },
    /// Box defined by half-extents along each axis.
    Cuboid { hx: f32, hy: f32, hz: f32 },
    /// Capsule (cylinder with hemispherical caps) aligned to Y axis.
    Capsule { half_height: f32, radius: f32 },
    /// Cylinder aligned to Y axis.
    Cylinder { half_height: f32, radius: f32 },
    /// Cone pointing up along Y axis.
    Cone { half_height: f32, radius: f32 },
    /// Convex hull computed from a set of vertices.
    ConvexMesh { vertices: Vec<[f32; 3]> },
    /// Triangle mesh for static geometry.
    TriMesh {
        vertices: Vec<[f32; 3]>,
        indices: Vec<[u32; 3]>,
    },
    /// Height field terrain.
    HeightField {
        nrows: usize,
        ncols: usize,
        heights: Vec<f32>,
        scale: [f32; 3],
    },
}

impl ColliderComponent {
    /// Creates a sphere collider.
    pub fn new_ball(radius: f32) -> Self {
        Self {
            shape: ColliderShape::Ball { radius },
            ..Default::default()
        }
    }

    /// Creates a box collider with the given half-extents.
    pub fn new_cuboid(hx: f32, hy: f32, hz: f32) -> Self {
        Self {
            shape: ColliderShape::Cuboid { hx, hy, hz },
            ..Default::default()
        }
    }

    /// Creates a capsule collider aligned to the Y axis.
    pub fn new_capsule(half_height: f32, radius: f32) -> Self {
        Self {
            shape: ColliderShape::Capsule {
                half_height,
                radius,
            },
            ..Default::default()
        }
    }

    /// Creates a cylinder collider aligned to the Y axis.
    pub fn new_cylinder(half_height: f32, radius: f32) -> Self {
        Self {
            shape: ColliderShape::Cylinder {
                half_height,
                radius,
            },
            ..Default::default()
        }
    }

    /// Creates a cone collider pointing up along the Y axis.
    pub fn new_cone(half_height: f32, radius: f32) -> Self {
        Self {
            shape: ColliderShape::Cone {
                half_height,
                radius,
            },
            ..Default::default()
        }
    }

    /// Sets the friction coefficient.
    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction;
        self
    }

    /// Sets the restitution (bounciness) coefficient.
    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution;
        self
    }

    /// Sets the density for mass computation.
    pub fn with_density(mut self, density: f32) -> Self {
        self.density = density;
        self
    }

    /// Marks this collider as a sensor (trigger volume).
    pub fn as_sensor(mut self) -> Self {
        self.is_sensor = true;
        self
    }

    /// Converts to a Rapier collider for simulation.
    #[cfg(feature = "physics")]
    pub(crate) fn to_rapier_collider(&self) -> rapier3d::prelude::Collider {
        use rapier3d::prelude::{ColliderBuilder, DMatrix, Point, Real, SharedShape, vector};

        let shape: SharedShape = match &self.shape {
            ColliderShape::Ball { radius } => SharedShape::ball(*radius),
            ColliderShape::Cuboid { hx, hy, hz } => SharedShape::cuboid(*hx, *hy, *hz),
            ColliderShape::Capsule {
                half_height,
                radius,
            } => SharedShape::capsule_y(*half_height, *radius),
            ColliderShape::Cylinder {
                half_height,
                radius,
            } => SharedShape::cylinder(*half_height, *radius),
            ColliderShape::Cone {
                half_height,
                radius,
            } => SharedShape::cone(*half_height, *radius),
            ColliderShape::ConvexMesh { vertices } => {
                let points: Vec<Point<Real>> = vertices
                    .iter()
                    .map(|v| Point::new(v[0], v[1], v[2]))
                    .collect();
                SharedShape::convex_hull(&points).unwrap_or_else(|| SharedShape::ball(0.1))
            }
            ColliderShape::TriMesh { vertices, indices } => {
                let points: Vec<Point<Real>> = vertices
                    .iter()
                    .map(|v| Point::new(v[0], v[1], v[2]))
                    .collect();
                SharedShape::trimesh(points, indices.clone())
                    .unwrap_or_else(|_| SharedShape::ball(0.1))
            }
            ColliderShape::HeightField {
                nrows,
                ncols,
                heights,
                scale,
            } => SharedShape::heightfield(
                DMatrix::from_vec(*nrows, *ncols, heights.clone()),
                vector![scale[0], scale[1], scale[2]],
            ),
        };

        let rapier_collision_groups =
            super::types::interaction_groups_to_rapier(self.collision_groups);
        let rapier_solver_groups = super::types::interaction_groups_to_rapier(self.solver_groups);

        ColliderBuilder::new(shape)
            .friction(self.friction)
            .restitution(self.restitution)
            .density(self.density)
            .sensor(self.is_sensor)
            .collision_groups(rapier_collision_groups)
            .solver_groups(rapier_solver_groups)
            .build()
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct CollisionListener;

/// Interpolation state for smooth rendering between physics updates.
///
/// Physics runs at a fixed timestep, but rendering may occur at different rates.
/// This component stores the previous and current physics state to allow smooth
/// interpolation based on the accumulated time since the last physics step.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct PhysicsInterpolation {
    /// Position at the previous physics step.
    pub previous_translation: Vec3,
    /// Rotation at the previous physics step.
    pub previous_rotation: Quat,
    /// Position at the current physics step.
    pub current_translation: Vec3,
    /// Rotation at the current physics step.
    pub current_rotation: Quat,
    /// Whether interpolation is active.
    pub enabled: bool,
}

impl PhysicsInterpolation {
    /// Creates a new interpolation state with interpolation enabled.
    pub fn new() -> Self {
        Self {
            previous_translation: Vec3::zeros(),
            previous_rotation: Quat::identity(),
            current_translation: Vec3::zeros(),
            current_rotation: Quat::identity(),
            enabled: true,
        }
    }

    /// Computes the interpolated transform at the given alpha (0.0 = previous, 1.0 = current).
    pub fn interpolate(&self, alpha: f32) -> (Vec3, Quat) {
        if !self.enabled {
            return (self.current_translation, self.current_rotation);
        }

        let translation =
            nalgebra_glm::lerp(&self.previous_translation, &self.current_translation, alpha);
        let rotation =
            nalgebra_glm::quat_slerp(&self.previous_rotation, &self.current_rotation, alpha);
        (translation, rotation)
    }

    /// Copies current state to previous (called before each physics step).
    pub fn update_previous(&mut self) {
        self.previous_translation = self.current_translation;
        self.previous_rotation = self.current_rotation;
    }

    /// Updates the current physics state (called after each physics step).
    pub fn update_current(&mut self, translation: Vec3, rotation: Quat) {
        self.current_translation = translation;
        self.current_rotation = rotation;
    }
}

nightshade_ecs::impl_component!(
    RigidBodyComponent,
    ColliderComponent,
    CharacterControllerComponent,
    CollisionListener,
    PhysicsInterpolation,
);
