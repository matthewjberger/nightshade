use crate::ecs::camera::components::Camera;
use crate::ecs::light::components::Light;
use crate::ecs::mesh::components::RenderMesh;
use crate::ecs::primitives::{Name, Visibility};
use crate::ecs::transform::components::LocalTransform;
use crate::render::bounding_volume::BoundingVolume;
use crate::render::material::Material;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrefabBodyType {
    Dynamic,
    Kinematic,
    Static,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefabRigidBody {
    pub body_type: PrefabBodyType,
    pub mass: f32,
}

impl Default for PrefabRigidBody {
    fn default() -> Self {
        Self {
            body_type: PrefabBodyType::Dynamic,
            mass: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefabCollider {
    pub shape: PrefabColliderShape,
    pub friction: f32,
    pub restitution: f32,
    pub density: f32,
    pub is_sensor: bool,
}

impl Default for PrefabCollider {
    fn default() -> Self {
        Self {
            shape: PrefabColliderShape::Cuboid {
                hx: 0.5,
                hy: 0.5,
                hz: 0.5,
            },
            friction: 0.5,
            restitution: 0.0,
            density: 1.0,
            is_sensor: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prefab {
    pub name: String,
    pub root_nodes: Vec<PrefabNode>,
    /// Names declared by `KHR_materials_variants` at the document root.
    /// Indices into this list match `MaterialVariants::mappings`'s
    /// `variant_indices`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub material_variants: Vec<String>,
}

/// A single variant override on a primitive node: when any of the listed
/// variant names is active, swap the entity's material to the supplied
/// [`Material`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefabMaterialVariant {
    pub variant_names: Vec<String>,
    pub material: Material,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefabNode {
    pub local_transform: LocalTransform,
    pub components: PrefabComponents,
    pub children: Vec<PrefabNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_index: Option<usize>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrefabSource {
    pub prefab_name: String,
    pub source_path: Option<String>,
    /// Stable identifier of the originating prefab file (the `prefab_id`
    /// in the `.nsprefab` SceneHeader). Persists the link from a
    /// spawned prefab instance back to its source so editor tooling
    /// (e.g. "refresh from source", "find all instances of prefab X")
    /// can reach across maps without relying on filesystem paths.
    #[cfg(feature = "scene_graph")]
    #[serde(default)]
    pub source_uuid: Option<crate::assets::asset::AssetUuid>,
}

pub fn prefab_source_new(prefab_name: impl Into<String>) -> PrefabSource {
    PrefabSource {
        prefab_name: prefab_name.into(),
        source_path: None,
        #[cfg(feature = "scene_graph")]
        source_uuid: None,
    }
}

pub fn prefab_source_with_path(
    prefab_name: impl Into<String>,
    source_path: impl Into<String>,
) -> PrefabSource {
    PrefabSource {
        prefab_name: prefab_name.into(),
        source_path: Some(source_path.into()),
        #[cfg(feature = "scene_graph")]
        source_uuid: None,
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrefabComponents {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Name>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub render_mesh: Option<RenderMesh>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub material: Option<Material>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub material_variants: Vec<PrefabMaterialVariant>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub camera: Option<Camera>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub light: Option<Light>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Visibility>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounding_volume: Option<BoundingVolume>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rigid_body: Option<PrefabRigidBody>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub collider: Option<PrefabCollider>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skin_index: Option<usize>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instances: Vec<crate::ecs::transform::components::LocalTransform>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldOverride {
    pub field_name: String,
    pub value: crate::assets::asset::TypedPayload,
}

/// The prefab's own copy of the collider shapes. The physics plugin owns the
/// live enum; this is what the format stores, and the variants match so the
/// serialized shape is unchanged.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrefabColliderShape {
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
