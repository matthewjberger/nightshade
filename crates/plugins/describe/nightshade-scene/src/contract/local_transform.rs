use nalgebra_glm::{Mat4, Quat, Vec3, quat_to_mat4};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, enum2schema::Schema)]
#[serde(default)]
pub struct LocalTransform {
    #[schema(type = "array", items = "number", len = 3)]
    pub translation: Vec3,
    #[schema(type = "array", items = "number", len = 4)]
    pub rotation: Quat,
    #[schema(type = "array", items = "number", len = 3)]
    pub scale: Vec3,
}

impl Default for LocalTransform {
    fn default() -> Self {
        Self {
            translation: Vec3::zeros(),
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }
}

impl LocalTransform {
    pub fn new(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            ..Default::default()
        }
    }

    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            rotation,
            ..Default::default()
        }
    }

    pub fn from_scale(scale: Vec3) -> Self {
        Self {
            scale,
            ..Default::default()
        }
    }

    pub fn from_matrix(matrix: &Mat4) -> Self {
        let axis_x = Vec3::new(matrix[(0, 0)], matrix[(1, 0)], matrix[(2, 0)]);
        let axis_y = Vec3::new(matrix[(0, 1)], matrix[(1, 1)], matrix[(2, 1)]);
        let axis_z = Vec3::new(matrix[(0, 2)], matrix[(1, 2)], matrix[(2, 2)]);
        let scale = Vec3::new(axis_x.norm(), axis_y.norm(), axis_z.norm());

        let normalized = |axis: Vec3, length: f32| {
            if length.abs() < f32::EPSILON {
                axis
            } else {
                axis / length
            }
        };
        let rotation = nalgebra_glm::mat3_to_quat(&nalgebra_glm::Mat3::from_columns(&[
            normalized(axis_x, scale.x),
            normalized(axis_y, scale.y),
            normalized(axis_z, scale.z),
        ]))
        .normalize();

        Self {
            translation: Vec3::new(matrix[(0, 3)], matrix[(1, 3)], matrix[(2, 3)]),
            rotation,
            scale,
        }
    }

    pub fn euler_degrees(&self) -> Vec3 {
        let quaternion = &self.rotation;

        let sin_roll = 2.0 * (quaternion.w * quaternion.i + quaternion.j * quaternion.k);
        let cos_roll = 1.0 - 2.0 * (quaternion.i * quaternion.i + quaternion.j * quaternion.j);
        let roll = sin_roll.atan2(cos_roll);

        let sin_pitch = 2.0 * (quaternion.w * quaternion.j - quaternion.k * quaternion.i);
        let pitch = if sin_pitch.abs() >= 1.0 {
            std::f32::consts::FRAC_PI_2.copysign(sin_pitch)
        } else {
            sin_pitch.asin()
        };

        let sin_yaw = 2.0 * (quaternion.w * quaternion.k + quaternion.i * quaternion.j);
        let cos_yaw = 1.0 - 2.0 * (quaternion.j * quaternion.j + quaternion.k * quaternion.k);
        let yaw = sin_yaw.atan2(cos_yaw);

        Vec3::new(roll.to_degrees(), pitch.to_degrees(), yaw.to_degrees())
    }

    pub fn rotation_from_euler_degrees(angles: Vec3) -> Quat {
        nalgebra_glm::quat_angle_axis(angles.z.to_radians(), &Vec3::z())
            * nalgebra_glm::quat_angle_axis(angles.y.to_radians(), &Vec3::y())
            * nalgebra_glm::quat_angle_axis(angles.x.to_radians(), &Vec3::x())
    }

    pub fn as_matrix(&self) -> Mat4 {
        let mut matrix = quat_to_mat4(&self.rotation.normalize());
        matrix[(0, 0)] *= self.scale.x;
        matrix[(1, 0)] *= self.scale.x;
        matrix[(2, 0)] *= self.scale.x;
        matrix[(0, 1)] *= self.scale.y;
        matrix[(1, 1)] *= self.scale.y;
        matrix[(2, 1)] *= self.scale.y;
        matrix[(0, 2)] *= self.scale.z;
        matrix[(1, 2)] *= self.scale.z;
        matrix[(2, 2)] *= self.scale.z;
        matrix[(0, 3)] = self.translation.x;
        matrix[(1, 3)] = self.translation.y;
        matrix[(2, 3)] = self.translation.z;
        matrix
    }

    pub fn right_vector(&self) -> Vec3 {
        super::extract_right_vector(&self.as_matrix())
    }

    pub fn up_vector(&self) -> Vec3 {
        super::extract_up_vector(&self.as_matrix())
    }

    pub fn forward_vector(&self) -> Vec3 {
        super::extract_forward_vector(&self.as_matrix())
    }

    pub fn rotate(&mut self, angle: f32, axis: &Vec3) {
        self.rotation = nalgebra_glm::quat_angle_axis(angle, axis) * self.rotation;
    }
}
