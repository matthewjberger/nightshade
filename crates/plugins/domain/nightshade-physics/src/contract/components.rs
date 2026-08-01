use nalgebra_glm;
use nalgebra_glm::{Quat, Vec3};
use serde::{Deserialize, Serialize};

use super::types::{ColliderHandle, InteractionGroups, LockedAxes, RigidBodyHandle, RigidBodyType};

#[derive(Debug, Clone, Serialize, Deserialize, enum2schema::Schema)]
pub struct RigidBodyComponent {
    #[serde(skip)]
    #[schema(skip)]
    pub handle: Option<RigidBodyHandle>,
    pub body_type: RigidBodyType,
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub linvel: [f32; 3],
    pub angvel: [f32; 3],
    pub mass: f32,
    #[serde(default)]
    pub linear_damping: f32,
    #[serde(default)]
    pub angular_damping: f32,
    #[serde(default = "default_gravity_scale")]
    pub gravity_scale: f32,
    pub locked_axes: LockedAxes,
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
    pub fn new_dynamic() -> Self {
        Self {
            body_type: RigidBodyType::Dynamic,
            ..Default::default()
        }
    }

    pub fn new_kinematic() -> Self {
        Self {
            body_type: RigidBodyType::KinematicPositionBased,
            ..Default::default()
        }
    }

    pub fn new_kinematic_velocity() -> Self {
        Self {
            body_type: RigidBodyType::KinematicVelocityBased,
            ..Default::default()
        }
    }

    pub fn new_static() -> Self {
        Self {
            body_type: RigidBodyType::Fixed,
            ..Default::default()
        }
    }

    pub fn with_translation(mut self, x: f32, y: f32, z: f32) -> Self {
        self.translation = [x, y, z];
        self
    }

    pub fn with_mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }

    pub fn with_linear_damping(mut self, damping: f32) -> Self {
        self.linear_damping = damping;
        self
    }

    pub fn with_angular_damping(mut self, damping: f32) -> Self {
        self.angular_damping = damping;
        self
    }

    pub fn with_gravity_scale(mut self, scale: f32) -> Self {
        self.gravity_scale = scale;
        self
    }

    pub fn with_rotation(mut self, x: f32, y: f32, z: f32, w: f32) -> Self {
        self.rotation = [x, y, z, w];
        self
    }

    pub fn with_ccd(mut self, enabled: bool) -> Self {
        self.ccd_enabled = enabled;
        self
    }
}

use super::types::CharacterControllerConfig;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, enum2schema::Schema)]
pub struct CharacterControllerComponent {
    pub config: CharacterControllerConfig,
    pub grounded: bool,
    pub velocity: Vec3,
    pub shape: ColliderShape,
    pub max_speed: f32,
    pub acceleration: f32,
    pub jump_impulse: f32,
    pub can_jump: bool,
    pub is_crouching: bool,
    pub is_sprinting: bool,
    pub crouch_enabled: bool,
    pub crouch_speed_multiplier: f32,
    pub sprint_speed_multiplier: f32,
    pub standing_half_height: f32,
    pub crouching_half_height: f32,
    pub scale: f32,
    pub friction_rate: f32,
    pub above_max_friction_rate: f32,
    pub engine_input_enabled: bool,
    pub disable_gravity: bool,
    pub external_movement: nalgebra_glm::Vec2,
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

#[derive(Debug, Clone, Serialize, Deserialize, enum2schema::Schema)]
pub struct ColliderComponent {
    #[serde(skip)]
    #[schema(skip)]
    pub handle: Option<ColliderHandle>,
    pub shape: ColliderShape,
    pub friction: f32,
    pub restitution: f32,
    pub density: f32,
    pub is_sensor: bool,
    pub collision_groups: InteractionGroups,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, enum2schema::Schema)]
pub enum ColliderShape {
    Ball {
        radius: f32,
    },
    Cuboid {
        hx: f32,
        hy: f32,
        hz: f32,
    },
    Capsule {
        half_height: f32,
        radius: f32,
    },
    Cylinder {
        half_height: f32,
        radius: f32,
    },
    Cone {
        half_height: f32,
        radius: f32,
    },
    ConvexMesh {
        vertices: Vec<[f32; 3]>,
    },
    TriMesh {
        vertices: Vec<[f32; 3]>,
        indices: Vec<[u32; 3]>,
    },
    HeightField {
        nrows: usize,
        ncols: usize,
        heights: Vec<f32>,
        scale: [f32; 3],
    },
}

impl ColliderComponent {
    pub fn new_ball(radius: f32) -> Self {
        Self {
            shape: ColliderShape::Ball { radius },
            ..Default::default()
        }
    }

    pub fn new_cuboid(hx: f32, hy: f32, hz: f32) -> Self {
        Self {
            shape: ColliderShape::Cuboid { hx, hy, hz },
            ..Default::default()
        }
    }

    pub fn new_capsule(half_height: f32, radius: f32) -> Self {
        Self {
            shape: ColliderShape::Capsule {
                half_height,
                radius,
            },
            ..Default::default()
        }
    }

    pub fn new_cylinder(half_height: f32, radius: f32) -> Self {
        Self {
            shape: ColliderShape::Cylinder {
                half_height,
                radius,
            },
            ..Default::default()
        }
    }

    pub fn new_cone(half_height: f32, radius: f32) -> Self {
        Self {
            shape: ColliderShape::Cone {
                half_height,
                radius,
            },
            ..Default::default()
        }
    }

    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction;
        self
    }

    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution;
        self
    }

    pub fn with_density(mut self, density: f32) -> Self {
        self.density = density;
        self
    }

    pub fn as_sensor(mut self) -> Self {
        self.is_sensor = true;
        self
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct CollisionListener;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct PhysicsInterpolation {
    pub previous_translation: Vec3,
    pub previous_rotation: Quat,
    pub current_translation: Vec3,
    pub current_rotation: Quat,
    pub enabled: bool,
}

impl PhysicsInterpolation {
    pub fn new() -> Self {
        Self {
            previous_translation: Vec3::zeros(),
            previous_rotation: Quat::identity(),
            current_translation: Vec3::zeros(),
            current_rotation: Quat::identity(),
            enabled: true,
        }
    }

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

    pub fn update_previous(&mut self) {
        self.previous_translation = self.current_translation;
        self.previous_rotation = self.current_rotation;
    }

    pub fn update_current(&mut self, translation: Vec3, rotation: Quat) {
        self.current_translation = translation;
        self.current_rotation = rotation;
    }
}
