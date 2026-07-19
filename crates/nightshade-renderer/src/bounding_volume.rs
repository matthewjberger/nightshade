//! Bounding volumes for culling and picking: an oriented box and a combined
//! box-plus-sphere volume, with transform and ray-intersection math.

use nalgebra_glm::Mat3;
use nalgebra_glm::{Mat4, Quat, Vec3, Vec4};

/// Box defined by a center, half-extents, and an orientation.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, enum2schema::Schema)]
#[serde(default)]
pub struct OrientedBoundingBox {
    /// Center in the current space.
    #[schema(type = "array", items = "number", len = 3)]
    pub center: Vec3,
    /// Half-sizes along the box's local axes.
    #[schema(type = "array", items = "number", len = 3)]
    pub half_extents: Vec3,
    /// Rotation from local axes to the current space.
    #[schema(type = "array", items = "number", len = 4)]
    pub orientation: Quat,
}

impl Default for OrientedBoundingBox {
    fn default() -> Self {
        Self {
            center: Vec3::zeros(),
            half_extents: Vec3::new(0.5, 0.5, 0.5),
            orientation: Quat::identity(),
        }
    }
}

impl OrientedBoundingBox {
    /// Builds a box from center, half-extents, and orientation.
    pub fn new(center: Vec3, half_extents: Vec3, orientation: Quat) -> Self {
        Self {
            center,
            half_extents,
            orientation,
        }
    }

    /// Builds an axis-aligned box spanning `min` to `max`.
    pub fn from_aabb(min: Vec3, max: Vec3) -> Self {
        let center = (min + max) * 0.5;
        let half_extents = (max - min) * 0.5;
        Self {
            center,
            half_extents,
            orientation: Quat::identity(),
        }
    }

    /// Returns the box transformed by `matrix`, absorbing scale into the half-extents.
    pub fn transform(&self, matrix: &Mat4) -> Self {
        let transformed_center =
            (matrix * Vec4::new(self.center.x, self.center.y, self.center.z, 1.0)).xyz();

        let right = Vec3::new(matrix[(0, 0)], matrix[(1, 0)], matrix[(2, 0)]);
        let up = Vec3::new(matrix[(0, 1)], matrix[(1, 1)], matrix[(2, 1)]);
        let forward = Vec3::new(matrix[(0, 2)], matrix[(1, 2)], matrix[(2, 2)]);

        let scale_x = nalgebra_glm::length(&right);
        let scale_y = nalgebra_glm::length(&up);
        let scale_z = nalgebra_glm::length(&forward);

        let rotation_matrix = Mat3::new(
            right.x / scale_x,
            up.x / scale_y,
            forward.x / scale_z,
            right.y / scale_x,
            up.y / scale_y,
            forward.y / scale_z,
            right.z / scale_x,
            up.z / scale_y,
            forward.z / scale_z,
        );

        let orientation = nalgebra_glm::mat3_to_quat(&rotation_matrix) * self.orientation;

        let scaled_half_extents = Vec3::new(
            self.half_extents.x * scale_x,
            self.half_extents.y * scale_y,
            self.half_extents.z * scale_z,
        );

        Self {
            center: transformed_center,
            half_extents: scaled_half_extents,
            orientation,
        }
    }

    /// The eight world-space corners of the box.
    pub fn get_corners(&self) -> [Vec3; 8] {
        let rotation_matrix = nalgebra_glm::quat_to_mat3(&self.orientation);

        let x_axis = rotation_matrix.column(0).into_owned() * self.half_extents.x;
        let y_axis = rotation_matrix.column(1).into_owned() * self.half_extents.y;
        let z_axis = rotation_matrix.column(2).into_owned() * self.half_extents.z;

        [
            self.center - x_axis - y_axis - z_axis,
            self.center + x_axis - y_axis - z_axis,
            self.center - x_axis + y_axis - z_axis,
            self.center + x_axis + y_axis - z_axis,
            self.center - x_axis - y_axis + z_axis,
            self.center + x_axis - y_axis + z_axis,
            self.center - x_axis + y_axis + z_axis,
            self.center + x_axis + y_axis + z_axis,
        ]
    }

    /// Returns the nearest positive ray hit distance, or `None` when the ray misses.
    pub fn intersect_ray(&self, ray_origin: Vec3, ray_direction: Vec3) -> Option<f32> {
        let inv_orientation = self.orientation.conjugate();
        let local_ray_origin =
            nalgebra_glm::quat_rotate_vec3(&inv_orientation, &(ray_origin - self.center));
        let local_ray_direction = nalgebra_glm::quat_rotate_vec3(&inv_orientation, &ray_direction);

        let inv_dir = Vec3::new(
            if local_ray_direction.x.abs() > 1e-10 {
                1.0 / local_ray_direction.x
            } else {
                f32::INFINITY
            },
            if local_ray_direction.y.abs() > 1e-10 {
                1.0 / local_ray_direction.y
            } else {
                f32::INFINITY
            },
            if local_ray_direction.z.abs() > 1e-10 {
                1.0 / local_ray_direction.z
            } else {
                f32::INFINITY
            },
        );

        let t_min = (-self.half_extents - local_ray_origin).component_mul(&inv_dir);
        let t_max = (self.half_extents - local_ray_origin).component_mul(&inv_dir);

        let t1 = Vec3::new(
            t_min.x.min(t_max.x),
            t_min.y.min(t_max.y),
            t_min.z.min(t_max.z),
        );

        let t2 = Vec3::new(
            t_min.x.max(t_max.x),
            t_min.y.max(t_max.y),
            t_min.z.max(t_max.z),
        );

        let t_near = t1.x.max(t1.y).max(t1.z);
        let t_far = t2.x.min(t2.y).min(t2.z);

        if t_near <= t_far && t_far >= 0.0 {
            if t_near >= 0.0 {
                Some(t_near)
            } else {
                Some(t_far)
            }
        } else {
            None
        }
    }
}

/// An oriented box paired with a bounding-sphere radius.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, enum2schema::Schema)]
#[serde(default)]
pub struct BoundingVolume {
    /// Oriented box.
    pub obb: OrientedBoundingBox,
    /// Radius of the enclosing sphere about the box center.
    pub sphere_radius: f32,
}

impl Default for BoundingVolume {
    fn default() -> Self {
        Self {
            obb: OrientedBoundingBox::default(),
            sphere_radius: 1.0,
        }
    }
}

impl BoundingVolume {
    /// Builds a volume from an oriented box and sphere radius.
    pub fn new(obb: OrientedBoundingBox, sphere_radius: f32) -> Self {
        Self { obb, sphere_radius }
    }

    /// Bounding volume for a named primitive mesh type.
    pub fn from_mesh_type(mesh_type: &str) -> Self {
        let (half_extents, sphere_radius) = match mesh_type {
            "Cube" => {
                let half = 0.5;
                (Vec3::new(half, half, half), half * 1.732_050_8)
            }
            "Sphere" => {
                let radius = 1.0;
                (Vec3::new(radius, radius, radius), radius)
            }
            "Cylinder" => {
                let radius = 0.5f32;
                let half_height = 0.5f32;
                (
                    Vec3::new(radius, half_height, radius),
                    (radius * radius + half_height * half_height).sqrt(),
                )
            }
            "Torus" => {
                let major = 1.0;
                let minor = 0.3;
                let outer = major + minor;
                (Vec3::new(outer, minor, outer), outer)
            }
            "Cone" => {
                let radius = 0.5f32;
                let half_height = 0.5f32;
                (
                    Vec3::new(radius, half_height, radius),
                    (radius * radius + half_height * half_height).sqrt(),
                )
            }
            "Plane" | "SubdividedPlane" => (Vec3::new(1.0, 1.0, 1.0), 1.732),
            _ => (Vec3::new(0.5, 0.5, 0.5), 0.866),
        };

        Self {
            obb: OrientedBoundingBox::new(Vec3::zeros(), half_extents, Quat::identity()),
            sphere_radius,
        }
    }

    /// Returns the volume transformed by `matrix`, scaling the sphere by the largest axis scale.
    pub fn transform(&self, matrix: &Mat4) -> Self {
        let transformed_obb = self.obb.transform(matrix);

        let scale_x =
            nalgebra_glm::length(&Vec3::new(matrix[(0, 0)], matrix[(1, 0)], matrix[(2, 0)]));
        let scale_y =
            nalgebra_glm::length(&Vec3::new(matrix[(0, 1)], matrix[(1, 1)], matrix[(2, 1)]));
        let scale_z =
            nalgebra_glm::length(&Vec3::new(matrix[(0, 2)], matrix[(1, 2)], matrix[(2, 2)]));
        let max_scale = scale_x.max(scale_y).max(scale_z);

        Self {
            obb: transformed_obb,
            sphere_radius: self.sphere_radius * max_scale,
        }
    }
}
