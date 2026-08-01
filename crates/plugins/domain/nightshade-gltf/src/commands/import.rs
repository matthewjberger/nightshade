use gltf::{Document, Gltf, Node, Primitive, buffer, image, import_buffers};
use nalgebra_glm::{Mat4, Quat, Vec2, Vec3, vec2, vec3};
use nightshade_animation::contract::{
    AnimationChannel, AnimationClip, AnimationInterpolation, AnimationProperty, AnimationSampler,
    AnimationSamplerOutput,
};
use nightshade_camera::components::{Camera, OrthographicCamera, PerspectiveCamera, Projection};
use nightshade_graphics::resources::loading::{
    SourceImageId, TextureProductionPlan, TextureRecipe,
};
use nightshade_render::components::light::{Light, LightType};
use nightshade_render::components::mesh::RenderMesh;
use nightshade_render::components::mesh::{
    Mesh, MorphTarget, MorphTargetData, SkinData, SkinnedVertex, Vertex,
};
use nightshade_render::data::material::Material;
use nightshade_render::data::material::{AlphaMode, TextureTransform};
use nightshade_render::data::texture_data::{
    SamplerFilter, SamplerSettings, SamplerWrap, TextureUsage,
};
use nightshade_scene::contract::primitives::{Name, Visibility};

use nightshade_scene::contract::LocalTransform;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

use nightshade_prefab::contract::{Prefab, PrefabComponents, PrefabNode};

type EncodedImages = Vec<Option<Vec<u8>>>;
type GltfImport = (Document, Vec<buffer::Data>, EncodedImages);

struct TextureSpec {
    recipe: TextureRecipe,
    usage: TextureUsage,
    sampler: SamplerSettings,
}

type TextureSpecMap = HashMap<String, TextureSpec>;

mod accessors;
mod animation;
mod materials;
mod nodes;
mod primitives;
mod read;
mod scene;
mod skin;
mod tangents;
mod textures;
mod uri;

use accessors::*;
use animation::*;
use materials::*;
use nodes::*;
use primitives::*;
use read::*;
use scene::*;
use skin::*;
use tangents::*;
use textures::*;
use uri::*;

pub use nightshade_prefab::data::{ModelImport, ModelSkin};

pub fn import_gltf_from_path(
    path: &std::path::Path,
) -> Result<ModelImport, Box<dyn std::error::Error>> {
    let (doc, buffers, encoded_images) = import_lenient_from_path(path)?;

    let canonical_path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let namespace = canonical_path
        .to_str()
        .unwrap_or("unnamed")
        .replace('\\', "/")
        .replace('/', "::");

    process_gltf_data(doc, &buffers, encoded_images, &namespace)
}

pub fn import_gltf_from_bytes(bytes: &[u8]) -> Result<ModelImport, Box<dyn std::error::Error>> {
    let (doc, buffers, encoded_images) = import_lenient_from_slice(bytes)?;
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    let namespace = format!("bytes_{:016x}", hasher.finish());
    process_gltf_data(doc, &buffers, encoded_images, &namespace)
}

pub fn import_gltf_with_resources(
    gltf_bytes: &[u8],
    resources: &HashMap<String, Vec<u8>>,
) -> Result<ModelImport, Box<dyn std::error::Error>> {
    let Gltf { document, blob } = Gltf::from_slice_without_validation(gltf_bytes)?;

    let mut buffers: Vec<buffer::Data> = Vec::with_capacity(document.buffers().count());
    for buffer in document.buffers() {
        match buffer.source() {
            buffer::Source::Bin => {
                let bin = blob
                    .as_ref()
                    .ok_or("glTF references binary chunk but none was provided")?;
                buffers.push(buffer::Data(bin.clone()));
            }
            buffer::Source::Uri(uri) => {
                let bytes = resolve_uri_bytes(uri, resources)?;
                buffers.push(buffer::Data(bytes));
            }
        }
    }

    let mut encoded_images: EncodedImages = Vec::with_capacity(document.images().count());
    for img in document.images() {
        let encoded: Vec<u8> = match img.source() {
            image::Source::Uri { uri, .. } => resolve_uri_bytes(uri, resources)?,
            image::Source::View { view, .. } => {
                let buffer = &buffers[view.buffer().index()];
                let start = view.offset();
                let end = start + view.length();
                buffer.0[start..end].to_vec()
            }
        };
        encoded_images.push(Some(encoded));
    }

    let mut hasher = DefaultHasher::new();
    gltf_bytes.hash(&mut hasher);
    let namespace = format!("bytes_{:016x}", hasher.finish());
    process_gltf_data(document, &buffers, encoded_images, &namespace)
}
