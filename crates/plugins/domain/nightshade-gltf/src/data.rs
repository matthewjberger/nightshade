use std::collections::HashMap;

use nalgebra_glm::{Mat4, Quat, Vec3};
use nightshade_animation::contract::AnimationClip;
use nightshade_render::data::geometry::Mesh;
use nightshade_render::data::material::Material;

pub const LOAD_CREW: usize = 2;
pub const LOAD_LIMIT: usize = 1;

pub struct Gltf;

#[derive(Debug, Default, Clone)]
pub struct SceneExport {
    pub name: String,
    pub nodes: Vec<ExportNode>,
    pub roots: Vec<usize>,
    pub meshes: Vec<ExportMesh>,
    pub materials: Vec<Material>,
    pub images: Vec<ExportImage>,
    pub skins: Vec<ExportSkin>,
    pub animations: Vec<AnimationClip>,
}

#[derive(Debug, Clone)]
pub struct ExportNode {
    pub name: String,
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub children: Vec<usize>,
    pub mesh: Option<usize>,
    pub skin: Option<usize>,
    pub camera: Option<ExportCamera>,
    pub light: Option<ExportLight>,
    pub morph_weights: Vec<f32>,
}

impl Default for ExportNode {
    fn default() -> Self {
        Self {
            name: String::new(),
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
            children: Vec::new(),
            mesh: None,
            skin: None,
            camera: None,
            light: None,
            morph_weights: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExportMesh {
    pub name: String,
    pub mesh: Mesh,
    pub material: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ExportImage {
    pub name: String,
    pub png: Vec<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct ExportSkin {
    pub name: Option<String>,
    pub joints: Vec<usize>,
    pub inverse_bind_matrices: Vec<Mat4>,
    pub skeleton: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
pub enum ExportCamera {
    Perspective {
        field_of_view: f32,
        aspect_ratio: Option<f32>,
        near: f32,
        far: Option<f32>,
    },
    Orthographic {
        horizontal_magnification: f32,
        vertical_magnification: f32,
        near: f32,
        far: f32,
    },
}

#[derive(Debug, Clone)]
pub struct ExportLight {
    pub name: Option<String>,
    pub kind: ExportLightKind,
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: Option<f32>,
}

#[derive(Debug, Clone, Copy)]
pub enum ExportLightKind {
    Directional,
    Point,
    Spot { inner_cone: f32, outer_cone: f32 },
}

impl SceneExport {
    pub fn image_index(&self) -> HashMap<String, usize> {
        self.images
            .iter()
            .enumerate()
            .map(|(index, image)| (image.name.clone(), index))
            .collect()
    }
}
