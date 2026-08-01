use nightshade_camera::components::Camera;
use nightshade_render::components::light::Light;
use nightshade_render::components::mesh::RenderMesh;
use nightshade_render::data::bounding_volume::BoundingVolume;
use nightshade_render::data::material::Material;
use nightshade_scene::contract::primitives::{Name, Visibility};

use nightshade_scene::contract::LocalTransform;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prefab {
    pub name: String,
    pub root_nodes: Vec<PrefabNode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub material_variants: Vec<String>,
}

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
}

pub fn prefab_source_new(prefab_name: impl Into<String>) -> PrefabSource {
    PrefabSource {
        prefab_name: prefab_name.into(),
        source_path: None,
    }
}

pub fn prefab_source_with_path(
    prefab_name: impl Into<String>,
    source_path: impl Into<String>,
) -> PrefabSource {
    PrefabSource {
        prefab_name: prefab_name.into(),
        source_path: Some(source_path.into()),
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

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skin_index: Option<usize>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instances: Vec<nightshade_scene::contract::LocalTransform>,
}
