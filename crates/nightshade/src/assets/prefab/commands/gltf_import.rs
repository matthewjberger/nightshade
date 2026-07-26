use crate::assets::loading::{SourceImageId, TextureProductionPlan, TextureRecipe};
use crate::ecs::animation::components::{
    AnimationChannel, AnimationClip, AnimationInterpolation, AnimationProperty, AnimationSampler,
    AnimationSamplerOutput,
};
use crate::ecs::camera::components::{Camera, OrthographicCamera, PerspectiveCamera, Projection};
use crate::ecs::light::components::{Light, LightType};
use crate::ecs::mesh::components::RenderMesh;
use crate::ecs::mesh::components::{
    Mesh, MorphTarget, MorphTargetData, SkinData, SkinnedVertex, Vertex,
};
use crate::ecs::primitives::{Name, Visibility};
use crate::ecs::transform::components::LocalTransform;
use crate::render::material::Material;
use crate::render::material::{AlphaMode, TextureTransform};
use crate::render::texture_data::{SamplerFilter, SamplerSettings, SamplerWrap, TextureUsage};
use gltf::{Document, Gltf, Node, Primitive, buffer, image, import_buffers};
use nalgebra_glm::{Mat4, Quat, Vec2, Vec3, vec2, vec3};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

use super::super::components::{Prefab, PrefabComponents, PrefabNode};

type EncodedImages = Vec<Option<Vec<u8>>>;
type GltfImport = (Document, Vec<buffer::Data>, EncodedImages);

struct TextureSpec {
    recipe: TextureRecipe,
    usage: TextureUsage,
    sampler: SamplerSettings,
}

type TextureSpecMap = HashMap<String, TextureSpec>;

fn import_lenient_from_path(path: &Path) -> Result<GltfImport, Box<dyn std::error::Error>> {
    let base = path.parent();
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let Gltf { document, blob } = Gltf::from_reader_without_validation(reader)?;
    let buffers = import_buffers(&document, base, blob)?;
    let encoded_images = extract_encoded_images_from_path(&document, base, &buffers)?;
    Ok((document, buffers, encoded_images))
}

fn import_lenient_from_slice(bytes: &[u8]) -> Result<GltfImport, Box<dyn std::error::Error>> {
    let Gltf { document, blob } = Gltf::from_slice_without_validation(bytes)?;
    let buffers = import_buffers(&document, None, blob)?;
    let encoded_images = extract_encoded_images_from_path(&document, None, &buffers)?;
    Ok((document, buffers, encoded_images))
}

fn extract_encoded_images_from_path(
    doc: &Document,
    base: Option<&Path>,
    buffers: &[buffer::Data],
) -> Result<EncodedImages, Box<dyn std::error::Error>> {
    let mut out = Vec::with_capacity(doc.images().count());
    for img in doc.images() {
        let bytes = match img.source() {
            image::Source::Uri { uri, .. } => {
                if let Some(rest) = uri.strip_prefix("data:") {
                    decode_data_uri_payload(rest)?
                } else if let Some(base) = base {
                    let decoded = percent_decode(uri);
                    let path = base.join(decoded);
                    std::fs::read(path)?
                } else {
                    return Err(format!("no base path for image URI '{}'", uri).into());
                }
            }
            image::Source::View { view, .. } => {
                let buffer = &buffers[view.buffer().index()];
                let start = view.offset();
                let end = start + view.length();
                buffer.0[start..end].to_vec()
            }
        };
        out.push(Some(bytes));
    }
    Ok(out)
}

fn decode_data_uri_payload(rest: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let comma = rest
        .find(',')
        .ok_or("malformed data URI: missing comma separator")?;
    let header = &rest[..comma];
    let payload = &rest[comma + 1..];
    if header.split(';').any(|piece| piece == "base64") {
        use base64::Engine;
        Ok(base64::engine::general_purpose::STANDARD.decode(payload)?)
    } else {
        Ok(percent_decode(payload).into_bytes())
    }
}

fn sampler_settings_from_gltf(sampler: &gltf::texture::Sampler) -> SamplerSettings {
    let wrap = |mode: gltf::texture::WrappingMode| match mode {
        gltf::texture::WrappingMode::ClampToEdge => SamplerWrap::ClampToEdge,
        gltf::texture::WrappingMode::MirroredRepeat => SamplerWrap::MirroredRepeat,
        gltf::texture::WrappingMode::Repeat => SamplerWrap::Repeat,
    };
    let mag = match sampler.mag_filter() {
        Some(gltf::texture::MagFilter::Nearest) => SamplerFilter::Nearest,
        _ => SamplerFilter::Linear,
    };
    let (min, mipmap) = match sampler.min_filter() {
        Some(gltf::texture::MinFilter::Nearest) => (SamplerFilter::Nearest, SamplerFilter::Nearest),
        Some(gltf::texture::MinFilter::Linear) => (SamplerFilter::Linear, SamplerFilter::Linear),
        Some(gltf::texture::MinFilter::NearestMipmapNearest) => {
            (SamplerFilter::Nearest, SamplerFilter::Nearest)
        }
        Some(gltf::texture::MinFilter::LinearMipmapNearest) => {
            (SamplerFilter::Linear, SamplerFilter::Nearest)
        }
        Some(gltf::texture::MinFilter::NearestMipmapLinear) => {
            (SamplerFilter::Nearest, SamplerFilter::Linear)
        }
        Some(gltf::texture::MinFilter::LinearMipmapLinear) | None => {
            (SamplerFilter::Linear, SamplerFilter::Linear)
        }
    };
    SamplerSettings {
        wrap_u: wrap(sampler.wrap_s()),
        wrap_v: wrap(sampler.wrap_t()),
        mag_filter: mag,
        min_filter: min,
        mipmap_filter: mipmap,
    }
}

fn texture_transform_from_info(info: &gltf::texture::Info) -> TextureTransform {
    let base_uv = info.tex_coord();
    match info.texture_transform() {
        Some(t) => TextureTransform {
            offset: t.offset(),
            rotation: t.rotation(),
            scale: t.scale(),
            uv_set: t.tex_coord().unwrap_or(base_uv),
        },
        None => TextureTransform {
            offset: [0.0, 0.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
            uv_set: base_uv,
        },
    }
}

fn texture_transform_from_extension_value(
    extension: Option<&serde_json::Value>,
    base_uv: u32,
) -> TextureTransform {
    let Some(transform) = extension else {
        return TextureTransform {
            offset: [0.0, 0.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
            uv_set: base_uv,
        };
    };

    let offset = transform
        .get("offset")
        .and_then(|v| v.as_array())
        .filter(|array| array.len() >= 2)
        .map(|array| {
            [
                array[0].as_f64().unwrap_or(0.0) as f32,
                array[1].as_f64().unwrap_or(0.0) as f32,
            ]
        })
        .unwrap_or([0.0, 0.0]);

    let rotation = transform
        .get("rotation")
        .and_then(|v| v.as_f64())
        .map(|value| value as f32)
        .unwrap_or(0.0);

    let scale = transform
        .get("scale")
        .and_then(|v| v.as_array())
        .filter(|array| array.len() >= 2)
        .map(|array| {
            [
                array[0].as_f64().unwrap_or(1.0) as f32,
                array[1].as_f64().unwrap_or(1.0) as f32,
            ]
        })
        .unwrap_or([1.0, 1.0]);

    let uv_set = transform
        .get("texCoord")
        .and_then(|v| v.as_u64())
        .map(|value| value as u32)
        .unwrap_or(base_uv);

    TextureTransform {
        offset,
        rotation,
        scale,
        uv_set,
    }
}

fn texture_transform_from_normal(info: &gltf::material::NormalTexture) -> TextureTransform {
    texture_transform_from_extension_value(
        info.extension_value("KHR_texture_transform"),
        info.tex_coord(),
    )
}

fn texture_transform_from_occlusion(info: &gltf::material::OcclusionTexture) -> TextureTransform {
    texture_transform_from_extension_value(
        info.extension_value("KHR_texture_transform"),
        info.tex_coord(),
    )
}

fn pack_textures_rg(
    source_a_name: Option<&str>,
    source_b_name: Option<&str>,
    namespace: &str,
    output_name_prefix: &str,
    output_usage: TextureUsage,
    texture_specs: &mut TextureSpecMap,
) -> Option<String> {
    let a_image = source_a_name.and_then(|name| direct_image_source(name, texture_specs));
    let b_image = source_b_name.and_then(|name| direct_image_source(name, texture_specs));
    if a_image.is_none() && b_image.is_none() {
        return None;
    }
    let sampler = source_a_name
        .or(source_b_name)
        .and_then(|name| texture_specs.get(name).map(|spec| spec.sampler))
        .unwrap_or_default();
    let mut hasher = DefaultHasher::new();
    source_a_name.hash(&mut hasher);
    source_b_name.hash(&mut hasher);
    output_name_prefix.hash(&mut hasher);
    let hash = hasher.finish();
    let name = format!(
        "{}::{}_{:x}_{}",
        namespace,
        output_name_prefix,
        hash,
        sampler.signature()
    );
    texture_specs.entry(name.clone()).or_insert(TextureSpec {
        recipe: TextureRecipe::PackRg {
            a: a_image,
            b: b_image,
        },
        usage: output_usage,
        sampler,
    });
    Some(name)
}

fn pack_textures_rgb_a(
    source_rgb_name: Option<&str>,
    source_a_name: Option<&str>,
    namespace: &str,
    output_name_prefix: &str,
    output_usage: TextureUsage,
    texture_specs: &mut TextureSpecMap,
) -> Option<String> {
    let rgb_image = source_rgb_name.and_then(|name| direct_image_source(name, texture_specs));
    let alpha_image = source_a_name.and_then(|name| direct_image_source(name, texture_specs));
    if rgb_image.is_none() && alpha_image.is_none() {
        return None;
    }
    let sampler = source_rgb_name
        .or(source_a_name)
        .and_then(|name| texture_specs.get(name).map(|spec| spec.sampler))
        .unwrap_or_default();
    let mut hasher = DefaultHasher::new();
    source_rgb_name.hash(&mut hasher);
    source_a_name.hash(&mut hasher);
    output_name_prefix.hash(&mut hasher);
    let hash = hasher.finish();
    let name = format!(
        "{}::{}_{:x}_{}",
        namespace,
        output_name_prefix,
        hash,
        sampler.signature()
    );
    texture_specs.entry(name.clone()).or_insert(TextureSpec {
        recipe: TextureRecipe::PackRgbA {
            rgb: rgb_image,
            alpha: alpha_image,
        },
        usage: output_usage,
        sampler,
    });
    Some(name)
}

fn direct_image_source(name: &str, texture_specs: &TextureSpecMap) -> Option<SourceImageId> {
    match texture_specs.get(name)?.recipe {
        TextureRecipe::Direct { image } => Some(image),
        _ => None,
    }
}

fn register_texture(
    texture: gltf::Texture<'_>,
    usage: TextureUsage,
    namespace: &str,
    image_index_to_id: &HashMap<usize, SourceImageId>,
    texture_specs: &mut TextureSpecMap,
) -> Option<String> {
    let image_index = texture.source()?.index();
    let image = *image_index_to_id.get(&image_index)?;
    let usage_suffix = match usage {
        TextureUsage::Color => "_srgb",
        TextureUsage::Linear => "_linear",
    };
    let sampler_settings = sampler_settings_from_gltf(&texture.sampler());
    let name = format!(
        "{}::img{}{}_{}",
        namespace,
        image_index,
        usage_suffix,
        sampler_settings.signature()
    );
    texture_specs.entry(name.clone()).or_insert(TextureSpec {
        recipe: TextureRecipe::Direct { image },
        usage,
        sampler: sampler_settings,
    });
    Some(name)
}

fn texture_transform_from_json(value: &serde_json::Value, base_uv: u32) -> TextureTransform {
    let ext = value
        .get("extensions")
        .and_then(|extensions| extensions.get("KHR_texture_transform"));
    let Some(ext) = ext else {
        return TextureTransform {
            offset: [0.0, 0.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
            uv_set: base_uv,
        };
    };
    let offset = ext
        .get("offset")
        .and_then(|o| o.as_array())
        .map(|arr| {
            [
                arr.first().and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                arr.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
            ]
        })
        .unwrap_or([0.0, 0.0]);
    let rotation = ext.get("rotation").and_then(|r| r.as_f64()).unwrap_or(0.0) as f32;
    let scale = ext
        .get("scale")
        .and_then(|s| s.as_array())
        .map(|arr| {
            [
                arr.first().and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
                arr.get(1).and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
            ]
        })
        .unwrap_or([1.0, 1.0]);
    let uv_set = ext
        .get("texCoord")
        .and_then(|t| t.as_u64())
        .map(|v| v as u32)
        .unwrap_or(base_uv);
    TextureTransform {
        offset,
        rotation,
        scale,
        uv_set,
    }
}

fn resolve_extension_texture(
    info: &serde_json::Value,
    doc: &Document,
    usage: TextureUsage,
    namespace: &str,
    image_index_to_id: &HashMap<usize, SourceImageId>,
    texture_specs: &mut TextureSpecMap,
) -> Option<(String, TextureTransform)> {
    let texture_index = info.get("index").and_then(|v| v.as_u64())? as usize;
    let base_uv = info
        .get("texCoord")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .unwrap_or(0);
    let gltf_texture = doc.textures().nth(texture_index)?;
    let name = register_texture(
        gltf_texture,
        usage,
        namespace,
        image_index_to_id,
        texture_specs,
    )?;
    let transform = texture_transform_from_json(info, base_uv);
    Some((name, transform))
}

#[derive(Debug, Clone)]
pub struct GltfSkin {
    pub name: Option<String>,
    pub joints: Vec<usize>,
    pub inverse_bind_matrices: Vec<Mat4>,
}

pub struct GltfLoadResult {
    pub prefabs: Vec<Prefab>,
    pub meshes: HashMap<String, Mesh>,
    pub materials: Vec<Material>,
    pub encoded_images: Vec<Vec<u8>>,
    pub texture_plan: Vec<TextureProductionPlan>,
    pub animations: Vec<AnimationClip>,
    pub skins: Vec<GltfSkin>,
    pub node_to_skin: HashMap<usize, usize>,
    pub node_to_morph_target_count: HashMap<usize, usize>,
    pub node_count: usize,
    /// Heuristic exposure value derived from the brightest punctual light
    /// intensity or emissive strength in the asset. Around 1.0 for assets
    /// using normalized lighting; small (e.g. 0.001) for assets using
    /// physical glTF units (lux/candela).
    pub suggested_exposure: f32,
}

pub fn import_gltf_from_path(
    path: &std::path::Path,
) -> Result<GltfLoadResult, Box<dyn std::error::Error>> {
    let (doc, buffers, encoded_images) = import_lenient_from_path(path)?;

    let canonical_path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let namespace = canonical_path
        .to_str()
        .unwrap_or("unnamed")
        .replace('\\', "/")
        .replace('/', "::");

    process_gltf_data(doc, &buffers, encoded_images, &namespace)
}

pub fn import_gltf_from_bytes(bytes: &[u8]) -> Result<GltfLoadResult, Box<dyn std::error::Error>> {
    let (doc, buffers, encoded_images) = import_lenient_from_slice(bytes)?;
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    let namespace = format!("bytes_{:016x}", hasher.finish());
    process_gltf_data(doc, &buffers, encoded_images, &namespace)
}

/// Import a `.gltf` file plus its external resources from in-memory bytes.
///
/// `resources` maps URI strings (as they appear in the glTF JSON, percent-decoded)
/// to their bytes. Use this when the engine has no filesystem access (e.g. WASM)
/// and the user has supplied the `.gltf` together with its `.bin` and texture files.
pub fn import_gltf_with_resources(
    gltf_bytes: &[u8],
    resources: &HashMap<String, Vec<u8>>,
) -> Result<GltfLoadResult, Box<dyn std::error::Error>> {
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

fn resolve_uri_bytes(
    uri: &str,
    resources: &HashMap<String, Vec<u8>>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if let Some(rest) = uri.strip_prefix("data:") {
        let comma = rest
            .find(',')
            .ok_or("malformed data URI: missing comma separator")?;
        let header = &rest[..comma];
        let payload = &rest[comma + 1..];
        if header.split(';').any(|piece| piece == "base64") {
            use base64::Engine;
            return Ok(base64::engine::general_purpose::STANDARD.decode(payload)?);
        }
        return Ok(percent_decode(payload).into_bytes());
    }

    let decoded = normalize_relative_path(&percent_decode(uri));
    if let Some(bytes) = resources.get(&decoded) {
        return Ok(bytes.clone());
    }
    Err(format!("missing external resource for URI '{}'", uri).into())
}

fn normalize_relative_path(input: &str) -> String {
    let normalized = input.replace('\\', "/");
    let mut parts: Vec<&str> = Vec::new();
    for piece in normalized.split('/') {
        match piece {
            "" | "." => continue,
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    parts.join("/")
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%'
            && index + 2 < bytes.len()
            && let (Some(high), Some(low)) =
                (hex_value(bytes[index + 1]), hex_value(bytes[index + 2]))
        {
            out.push((high << 4) | low);
            index += 3;
            continue;
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| input.to_string())
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn process_gltf_data(
    doc: Document,
    buffers: &[buffer::Data],
    encoded_images: EncodedImages,
    namespace: &str,
) -> Result<GltfLoadResult, Box<dyn std::error::Error>> {
    let mut meshes = HashMap::new();
    let mut materials = Vec::new();
    let mut texture_specs: TextureSpecMap = HashMap::new();
    let mut mesh_index_to_primitives: HashMap<usize, Vec<(String, usize)>> = HashMap::new();
    let mut gltf_material_index_to_deduplicated_index: HashMap<usize, usize> = HashMap::new();

    let mut mesh_to_skin: HashMap<usize, usize> = HashMap::new();
    let mut node_to_morph_target_count: HashMap<usize, usize> = HashMap::new();
    for node in doc.nodes() {
        if let (Some(mesh_index), Some(skin_index)) = (
            node.mesh().map(|m| m.index()),
            node.skin().map(|s| s.index()),
        ) {
            mesh_to_skin.insert(mesh_index, skin_index);
        }
        if let Some(mesh) = node.mesh() {
            let morph_count = mesh
                .primitives()
                .next()
                .map(|p| p.morph_targets().count())
                .unwrap_or(0);
            if morph_count > 0 {
                node_to_morph_target_count.insert(node.index(), morph_count);
            }
        }
    }

    let image_count = encoded_images.len();
    let mut image_index_to_id: HashMap<usize, SourceImageId> = HashMap::new();
    let mut encoded_image_arena: Vec<Vec<u8>> = Vec::with_capacity(image_count);
    for (index, opt_bytes) in encoded_images.into_iter().enumerate() {
        if let Some(bytes) = opt_bytes {
            let id = SourceImageId(encoded_image_arena.len() as u32);
            encoded_image_arena.push(bytes);
            image_index_to_id.insert(index, id);
        }
    }

    for material in doc.materials() {
        let pbr = material.pbr_metallic_roughness();
        let base_color = pbr.base_color_factor();
        let emissive = material.emissive_factor();
        let mut base_color_array = [base_color[0], base_color[1], base_color[2], base_color[3]];
        let mut roughness_value = pbr.roughness_factor();
        let mut metallic_value = pbr.metallic_factor();

        let (mut base_texture, mut base_texture_transform) = pbr
            .base_color_texture()
            .map(|tex| {
                let transform = texture_transform_from_info(&tex);
                (
                    register_texture(
                        tex.texture(),
                        TextureUsage::Color,
                        namespace,
                        &image_index_to_id,
                        &mut texture_specs,
                    ),
                    transform,
                )
            })
            .unwrap_or((None, TextureTransform::IDENTITY));

        let (emissive_texture, emissive_texture_transform) = material
            .emissive_texture()
            .map(|tex| {
                let transform = texture_transform_from_info(&tex);
                (
                    register_texture(
                        tex.texture(),
                        TextureUsage::Color,
                        namespace,
                        &image_index_to_id,
                        &mut texture_specs,
                    ),
                    transform,
                )
            })
            .unwrap_or((None, TextureTransform::IDENTITY));

        let (normal_texture, normal_texture_transform) = material
            .normal_texture()
            .map(|tex| {
                let transform = texture_transform_from_normal(&tex);
                (
                    register_texture(
                        tex.texture(),
                        TextureUsage::Linear,
                        namespace,
                        &image_index_to_id,
                        &mut texture_specs,
                    ),
                    transform,
                )
            })
            .unwrap_or((None, TextureTransform::IDENTITY));

        let normal_scale = material
            .normal_texture()
            .map(|tex| tex.scale())
            .unwrap_or(1.0);

        let (mut metallic_roughness_texture, mut metallic_roughness_texture_transform) = pbr
            .metallic_roughness_texture()
            .map(|tex| {
                let transform = texture_transform_from_info(&tex);
                (
                    register_texture(
                        tex.texture(),
                        TextureUsage::Linear,
                        namespace,
                        &image_index_to_id,
                        &mut texture_specs,
                    ),
                    transform,
                )
            })
            .unwrap_or((None, TextureTransform::IDENTITY));

        let (occlusion_texture, occlusion_texture_transform) = material
            .occlusion_texture()
            .map(|tex| {
                let transform = texture_transform_from_occlusion(&tex);
                (
                    register_texture(
                        tex.texture(),
                        TextureUsage::Linear,
                        namespace,
                        &image_index_to_id,
                        &mut texture_specs,
                    ),
                    transform,
                )
            })
            .unwrap_or((None, TextureTransform::IDENTITY));

        let occlusion_strength = material
            .occlusion_texture()
            .map(|tex| tex.strength())
            .unwrap_or(1.0);

        let transmission_factor = material
            .transmission()
            .map(|t| t.transmission_factor())
            .unwrap_or(0.0);

        let (transmission_texture, transmission_texture_transform) = material
            .transmission()
            .and_then(|t| {
                t.transmission_texture().map(|tex| {
                    let transform = texture_transform_from_info(&tex);
                    (
                        register_texture(
                            tex.texture(),
                            TextureUsage::Linear,
                            namespace,
                            &image_index_to_id,
                            &mut texture_specs,
                        ),
                        transform,
                    )
                })
            })
            .unwrap_or((None, TextureTransform::IDENTITY));

        let thickness = material
            .volume()
            .map(|v| v.thickness_factor())
            .unwrap_or(0.0);

        let (thickness_texture, thickness_texture_transform) = material
            .volume()
            .and_then(|v| {
                v.thickness_texture().map(|tex| {
                    let transform = texture_transform_from_info(&tex);
                    (
                        register_texture(
                            tex.texture(),
                            TextureUsage::Linear,
                            namespace,
                            &image_index_to_id,
                            &mut texture_specs,
                        ),
                        transform,
                    )
                })
            })
            .unwrap_or((None, TextureTransform::IDENTITY));

        let attenuation_color = material
            .volume()
            .map(|v| v.attenuation_color())
            .unwrap_or([1.0, 1.0, 1.0]);

        let attenuation_distance = material
            .volume()
            .map(|v| v.attenuation_distance())
            .unwrap_or(f32::INFINITY);

        let ior = material.ior().unwrap_or(1.5);

        let specular_factor = material
            .specular()
            .map(|s| s.specular_factor())
            .unwrap_or(1.0);

        let specular_color_factor = material
            .specular()
            .map(|s| s.specular_color_factor())
            .unwrap_or([1.0, 1.0, 1.0]);

        let (specular_texture, specular_texture_transform) = material
            .specular()
            .and_then(|s| {
                s.specular_texture().map(|tex| {
                    let transform = texture_transform_from_info(&tex);
                    (
                        register_texture(
                            tex.texture(),
                            TextureUsage::Linear,
                            namespace,
                            &image_index_to_id,
                            &mut texture_specs,
                        ),
                        transform,
                    )
                })
            })
            .unwrap_or((None, TextureTransform::IDENTITY));

        let (specular_color_texture, specular_color_texture_transform) = material
            .specular()
            .and_then(|s| {
                s.specular_color_texture().map(|tex| {
                    let transform = texture_transform_from_info(&tex);
                    (
                        register_texture(
                            tex.texture(),
                            TextureUsage::Color,
                            namespace,
                            &image_index_to_id,
                            &mut texture_specs,
                        ),
                        transform,
                    )
                })
            })
            .unwrap_or((None, TextureTransform::IDENTITY));

        let emissive_strength = material.emissive_strength().unwrap_or(1.0);

        if let Some(spec_gloss) = material.pbr_specular_glossiness() {
            let diffuse_factor = spec_gloss.diffuse_factor();
            let specular_factor_sg = spec_gloss.specular_factor();
            let glossiness_factor = spec_gloss.glossiness_factor();

            let (converted_base_rgb, converted_metallic) =
                crate::ecs::material::spec_gloss::spec_gloss_to_metallic_roughness(
                    diffuse_factor,
                    specular_factor_sg,
                );

            base_color_array = [
                converted_base_rgb[0],
                converted_base_rgb[1],
                converted_base_rgb[2],
                diffuse_factor[3],
            ];
            metallic_value = converted_metallic;
            roughness_value = (1.0 - glossiness_factor).clamp(0.0, 1.0);

            let diffuse_texture_info = spec_gloss.diffuse_texture();
            let spec_gloss_texture_info = spec_gloss.specular_glossiness_texture();
            let diffuse_image = diffuse_texture_info.as_ref().and_then(|tex| {
                image_index_to_id
                    .get(&tex.texture().source()?.index())
                    .copied()
            });
            let spec_gloss_image = spec_gloss_texture_info.as_ref().and_then(|tex| {
                image_index_to_id
                    .get(&tex.texture().source()?.index())
                    .copied()
            });

            if diffuse_image.is_some() || spec_gloss_image.is_some() {
                let mat_index = material.index().unwrap_or(usize::MAX);
                let base_name = format!("{}::sg_converted_base_{}", namespace, mat_index);
                let mr_name = format!("{}::sg_converted_mr_{}", namespace, mat_index);

                let diffuse_sampler = diffuse_texture_info
                    .as_ref()
                    .map(|tex| sampler_settings_from_gltf(&tex.texture().sampler()))
                    .unwrap_or(SamplerSettings::DEFAULT);
                let spec_gloss_sampler = spec_gloss_texture_info
                    .as_ref()
                    .map(|tex| sampler_settings_from_gltf(&tex.texture().sampler()))
                    .unwrap_or(SamplerSettings::DEFAULT);

                texture_specs.insert(
                    base_name.clone(),
                    TextureSpec {
                        recipe: TextureRecipe::SpecGlossBase {
                            diffuse: diffuse_image,
                            diffuse_factor,
                            spec_gloss: spec_gloss_image,
                            specular_factor: specular_factor_sg,
                            glossiness_factor,
                        },
                        usage: TextureUsage::Color,
                        sampler: diffuse_sampler,
                    },
                );
                texture_specs.insert(
                    mr_name.clone(),
                    TextureSpec {
                        recipe: TextureRecipe::SpecGlossMr {
                            diffuse: diffuse_image,
                            diffuse_factor,
                            spec_gloss: spec_gloss_image,
                            specular_factor: specular_factor_sg,
                            glossiness_factor,
                        },
                        usage: TextureUsage::Linear,
                        sampler: spec_gloss_sampler,
                    },
                );

                base_texture = Some(base_name);
                base_texture_transform = diffuse_texture_info
                    .as_ref()
                    .map(texture_transform_from_info)
                    .unwrap_or(TextureTransform::IDENTITY);
                metallic_roughness_texture = Some(mr_name);
                metallic_roughness_texture_transform = spec_gloss_texture_info
                    .as_ref()
                    .map(texture_transform_from_info)
                    .unwrap_or(TextureTransform::IDENTITY);
            }
        }

        let diffuse_trans_ext = material.extension_value("KHR_materials_diffuse_transmission");
        let diffuse_transmission_factor = diffuse_trans_ext
            .and_then(|v| v.get("diffuseTransmissionFactor"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        let diffuse_transmission_color_factor = diffuse_trans_ext
            .and_then(|v| v.get("diffuseTransmissionColorFactor"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                [
                    arr.first().and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
                    arr.get(1).and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
                    arr.get(2).and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
                ]
            })
            .unwrap_or([1.0, 1.0, 1.0]);
        let (diffuse_transmission_texture, diffuse_transmission_texture_transform) =
            diffuse_trans_ext
                .and_then(|v| v.get("diffuseTransmissionTexture"))
                .and_then(|info| {
                    resolve_extension_texture(
                        info,
                        &doc,
                        TextureUsage::Linear,
                        namespace,
                        &image_index_to_id,
                        &mut texture_specs,
                    )
                })
                .map(|(name, transform)| (Some(name), transform))
                .unwrap_or((None, TextureTransform::IDENTITY));
        let (diffuse_transmission_color_texture_pre, diffuse_transmission_color_texture_transform) =
            diffuse_trans_ext
                .and_then(|v| v.get("diffuseTransmissionColorTexture"))
                .and_then(|info| {
                    resolve_extension_texture(
                        info,
                        &doc,
                        TextureUsage::Color,
                        namespace,
                        &image_index_to_id,
                        &mut texture_specs,
                    )
                })
                .map(|(name, transform)| (Some(name), transform))
                .unwrap_or((None, TextureTransform::IDENTITY));
        let diffuse_transmission_color_texture = pack_textures_rgb_a(
            diffuse_transmission_color_texture_pre.as_deref(),
            diffuse_transmission_texture.as_deref(),
            namespace,
            "diffuse_transmission_packed",
            TextureUsage::Color,
            &mut texture_specs,
        );
        let diffuse_transmission_texture: Option<String> = None;

        let dispersion = material
            .extension_value("KHR_materials_dispersion")
            .and_then(|v| v.get("dispersion"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;

        let anisotropy_ext = material.extension_value("KHR_materials_anisotropy");
        let anisotropy_strength = anisotropy_ext
            .and_then(|v| v.get("anisotropyStrength"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        let anisotropy_rotation = anisotropy_ext
            .and_then(|v| v.get("anisotropyRotation"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        let (anisotropy_texture, anisotropy_texture_transform) = anisotropy_ext
            .and_then(|v| v.get("anisotropyTexture"))
            .and_then(|info| {
                resolve_extension_texture(
                    info,
                    &doc,
                    TextureUsage::Linear,
                    namespace,
                    &image_index_to_id,
                    &mut texture_specs,
                )
            })
            .map(|(name, transform)| (Some(name), transform))
            .unwrap_or((None, TextureTransform::IDENTITY));

        let iridescence_ext = material.extension_value("KHR_materials_iridescence");
        let iridescence_factor = iridescence_ext
            .and_then(|v| v.get("iridescenceFactor"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        let iridescence_ior = iridescence_ext
            .and_then(|v| v.get("iridescenceIor"))
            .and_then(|v| v.as_f64())
            .unwrap_or(1.3) as f32;
        let iridescence_thickness_minimum = iridescence_ext
            .and_then(|v| v.get("iridescenceThicknessMinimum"))
            .and_then(|v| v.as_f64())
            .unwrap_or(100.0) as f32;
        let iridescence_thickness_maximum = iridescence_ext
            .and_then(|v| v.get("iridescenceThicknessMaximum"))
            .and_then(|v| v.as_f64())
            .unwrap_or(400.0) as f32;
        let (iridescence_texture_pre, iridescence_texture_transform) = iridescence_ext
            .and_then(|v| v.get("iridescenceTexture"))
            .and_then(|info| {
                resolve_extension_texture(
                    info,
                    &doc,
                    TextureUsage::Linear,
                    namespace,
                    &image_index_to_id,
                    &mut texture_specs,
                )
            })
            .map(|(name, transform)| (Some(name), transform))
            .unwrap_or((None, TextureTransform::IDENTITY));
        let (iridescence_thickness_texture_pre, iridescence_thickness_texture_transform) =
            iridescence_ext
                .and_then(|v| v.get("iridescenceThicknessTexture"))
                .and_then(|info| {
                    resolve_extension_texture(
                        info,
                        &doc,
                        TextureUsage::Linear,
                        namespace,
                        &image_index_to_id,
                        &mut texture_specs,
                    )
                })
                .map(|(name, transform)| (Some(name), transform))
                .unwrap_or((None, TextureTransform::IDENTITY));
        let iridescence_texture = pack_textures_rg(
            iridescence_texture_pre.as_deref(),
            iridescence_thickness_texture_pre.as_deref(),
            namespace,
            "iridescence_packed",
            TextureUsage::Linear,
            &mut texture_specs,
        );
        let iridescence_thickness_texture: Option<String> = None;

        let sheen_ext = material.extension_value("KHR_materials_sheen");
        let sheen_color_factor = sheen_ext
            .and_then(|v| v.get("sheenColorFactor"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                [
                    arr.first().and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                    arr.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                    arr.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                ]
            })
            .unwrap_or([0.0, 0.0, 0.0]);
        let sheen_roughness_factor = sheen_ext
            .and_then(|v| v.get("sheenRoughnessFactor"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        let (sheen_color_texture_pre, sheen_color_texture_transform) = sheen_ext
            .and_then(|v| v.get("sheenColorTexture"))
            .and_then(|info| {
                resolve_extension_texture(
                    info,
                    &doc,
                    TextureUsage::Color,
                    namespace,
                    &image_index_to_id,
                    &mut texture_specs,
                )
            })
            .map(|(name, transform)| (Some(name), transform))
            .unwrap_or((None, TextureTransform::IDENTITY));
        let (sheen_roughness_texture_pre, sheen_roughness_texture_transform) = sheen_ext
            .and_then(|v| v.get("sheenRoughnessTexture"))
            .and_then(|info| {
                resolve_extension_texture(
                    info,
                    &doc,
                    TextureUsage::Linear,
                    namespace,
                    &image_index_to_id,
                    &mut texture_specs,
                )
            })
            .map(|(name, transform)| (Some(name), transform))
            .unwrap_or((None, TextureTransform::IDENTITY));
        let sheen_color_texture = pack_textures_rgb_a(
            sheen_color_texture_pre.as_deref(),
            sheen_roughness_texture_pre.as_deref(),
            namespace,
            "sheen_packed",
            TextureUsage::Color,
            &mut texture_specs,
        );
        let sheen_roughness_texture: Option<String> = None;

        let clearcoat_ext = material.extension_value("KHR_materials_clearcoat");
        let clearcoat_factor = clearcoat_ext
            .and_then(|v| v.get("clearcoatFactor"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        let clearcoat_roughness_factor = clearcoat_ext
            .and_then(|v| v.get("clearcoatRoughnessFactor"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        let (clearcoat_texture_pre, clearcoat_texture_transform) = clearcoat_ext
            .and_then(|v| v.get("clearcoatTexture"))
            .and_then(|info| {
                resolve_extension_texture(
                    info,
                    &doc,
                    TextureUsage::Linear,
                    namespace,
                    &image_index_to_id,
                    &mut texture_specs,
                )
            })
            .map(|(name, transform)| (Some(name), transform))
            .unwrap_or((None, TextureTransform::IDENTITY));
        let (clearcoat_roughness_texture_pre, clearcoat_roughness_texture_transform) =
            clearcoat_ext
                .and_then(|v| v.get("clearcoatRoughnessTexture"))
                .and_then(|info| {
                    resolve_extension_texture(
                        info,
                        &doc,
                        TextureUsage::Linear,
                        namespace,
                        &image_index_to_id,
                        &mut texture_specs,
                    )
                })
                .map(|(name, transform)| (Some(name), transform))
                .unwrap_or((None, TextureTransform::IDENTITY));
        let (clearcoat_normal_texture, clearcoat_normal_texture_transform) = clearcoat_ext
            .and_then(|v| v.get("clearcoatNormalTexture"))
            .and_then(|info| {
                resolve_extension_texture(
                    info,
                    &doc,
                    TextureUsage::Linear,
                    namespace,
                    &image_index_to_id,
                    &mut texture_specs,
                )
            })
            .map(|(name, transform)| (Some(name), transform))
            .unwrap_or((None, TextureTransform::IDENTITY));
        let clearcoat_normal_scale = clearcoat_ext
            .and_then(|v| v.get("clearcoatNormalTexture"))
            .and_then(|info| info.get("scale"))
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0) as f32;
        let clearcoat_texture = pack_textures_rg(
            clearcoat_texture_pre.as_deref(),
            clearcoat_roughness_texture_pre.as_deref(),
            namespace,
            "clearcoat_packed",
            TextureUsage::Linear,
            &mut texture_specs,
        );
        let clearcoat_roughness_texture: Option<String> = clearcoat_texture.clone();

        let world_material = Material {
            base_color: base_color_array,
            emissive_factor: [emissive[0], emissive[1], emissive[2]],
            alpha_mode: match material.alpha_mode() {
                gltf::material::AlphaMode::Opaque => AlphaMode::Opaque,
                gltf::material::AlphaMode::Mask => AlphaMode::Mask,
                gltf::material::AlphaMode::Blend => AlphaMode::Blend,
            },
            alpha_cutoff: material.alpha_cutoff().unwrap_or(0.5),
            blend_opaque_alpha_threshold: 0.99,
            base_texture,
            base_texture_transform,
            emissive_texture,
            emissive_texture_transform,
            normal_texture,
            normal_texture_transform,
            normal_scale,
            normal_map_flip_y: false,
            normal_map_two_component: false,
            detail_base_texture: None,
            detail_normal_texture: None,
            detail_uv_scale: 4.0,
            height_texture: None,
            parallax_scale: 0.0,
            reflection_box_min: [0.0, 0.0, 0.0],
            reflection_box_max: [0.0, 0.0, 0.0],
            metallic_roughness_texture,
            metallic_roughness_texture_transform,
            occlusion_texture,
            occlusion_texture_transform,
            occlusion_strength,
            roughness: roughness_value,
            metallic: metallic_value,
            unlit: material.unlit(),
            double_sided: material.double_sided(),
            transmission_factor,
            transmission_texture,
            transmission_texture_transform,
            thickness,
            thickness_texture,
            thickness_texture_transform,
            attenuation_color,
            attenuation_distance,
            ior,
            specular_factor,
            specular_color_factor,
            specular_texture,
            specular_texture_transform,
            specular_color_texture,
            specular_color_texture_transform,
            emissive_strength,
            diffuse_transmission_factor,
            diffuse_transmission_texture,
            diffuse_transmission_texture_transform,
            diffuse_transmission_color_factor,
            diffuse_transmission_color_texture,
            diffuse_transmission_color_texture_transform,
            dispersion,
            anisotropy_strength,
            anisotropy_rotation,
            anisotropy_texture,
            anisotropy_texture_transform,
            iridescence_factor,
            iridescence_texture,
            iridescence_texture_transform,
            iridescence_ior,
            iridescence_thickness_minimum,
            iridescence_thickness_maximum,
            iridescence_thickness_texture,
            iridescence_thickness_texture_transform,
            sheen_color_factor,
            sheen_color_texture,
            sheen_color_texture_transform,
            sheen_roughness_factor,
            sheen_roughness_texture,
            sheen_roughness_texture_transform,
            clearcoat_factor,
            clearcoat_texture,
            clearcoat_texture_transform,
            clearcoat_roughness_factor,
            clearcoat_roughness_texture,
            clearcoat_roughness_texture_transform,
            clearcoat_normal_texture,
            clearcoat_normal_texture_transform,
            clearcoat_normal_scale,
        };

        let deduplicated_index =
            if let Some(existing_index) = materials.iter().position(|m| m == &world_material) {
                if let Some(mat_index) = material.index() {
                    tracing::debug!(
                        "Material {} is identical to material {}, reusing",
                        mat_index,
                        existing_index
                    );
                }
                existing_index
            } else {
                let new_index = materials.len();
                materials.push(world_material);
                new_index
            };

        if let Some(mat_index) = material.index() {
            gltf_material_index_to_deduplicated_index.insert(mat_index, deduplicated_index);
        }
    }

    let document_variant_names: Vec<String> = doc
        .as_json()
        .extensions
        .as_ref()
        .and_then(|exts| exts.others.get("KHR_materials_variants"))
        .and_then(|v| v.get("variants"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|entry| entry.get("name").and_then(|n| n.as_str()))
                .map(|name| name.to_string())
                .collect()
        })
        .unwrap_or_default();

    let mut mesh_index_to_primitive_variants: MeshVariantMap = HashMap::new();

    for mesh in doc.meshes() {
        let base_mesh_name = mesh
            .name()
            .unwrap_or(&format!("Mesh_{}", mesh.index()))
            .to_string();

        let mut primitive_names = Vec::new();
        let mut primitive_variants: Vec<Vec<(Vec<u32>, u32)>> = Vec::new();
        let default_morph_weights: Option<Vec<f32>> = mesh.weights().map(|w| w.to_vec());

        for (primitive_index, primitive) in mesh.primitives().enumerate() {
            if let Some(mut mesh_primitive) =
                load_gltf_primitive(&primitive, buffers, default_morph_weights.as_deref())
            {
                if let Some(&skin_index) = mesh_to_skin.get(&mesh.index())
                    && let Some(skin_data) = &mut mesh_primitive.skin_data
                {
                    skin_data.skin_index = Some(skin_index);
                }

                let base_primitive_name = if mesh.primitives().len() > 1 {
                    format!("{}_{}", base_mesh_name, primitive_index)
                } else {
                    base_mesh_name.clone()
                };
                let primitive_name = format!("{}::{}", namespace, base_primitive_name);

                let gltf_material_index = primitive.material().index().unwrap_or(0);
                let material_index = gltf_material_index_to_deduplicated_index
                    .get(&gltf_material_index)
                    .copied()
                    .unwrap_or(0);
                primitive_names.push((primitive_name.clone(), material_index));

                let mut mappings: Vec<(Vec<u32>, u32)> = Vec::new();
                if let Some(mappings_value) = primitive
                    .extension_value("KHR_materials_variants")
                    .and_then(|v| v.get("mappings"))
                    .and_then(|v| v.as_array())
                {
                    for entry in mappings_value {
                        let Some(material_idx) = entry.get("material").and_then(|v| v.as_u64())
                        else {
                            continue;
                        };
                        let Some(variant_arr) = entry.get("variants").and_then(|v| v.as_array())
                        else {
                            continue;
                        };
                        let variant_indices: Vec<u32> = variant_arr
                            .iter()
                            .filter_map(|i| i.as_u64().map(|n| n as u32))
                            .collect();
                        if variant_indices.is_empty() {
                            continue;
                        }
                        let dedup = gltf_material_index_to_deduplicated_index
                            .get(&(material_idx as usize))
                            .copied()
                            .unwrap_or(material_idx as usize);
                        mappings.push((variant_indices, dedup as u32));
                    }
                }
                primitive_variants.push(mappings);

                meshes.insert(primitive_name, mesh_primitive);
            }
        }

        if !primitive_names.is_empty() {
            mesh_index_to_primitives.insert(mesh.index(), primitive_names);
            mesh_index_to_primitive_variants.insert(mesh.index(), primitive_variants);
        }
    }

    let mut prefabs = Vec::new();

    let point_intensity_scale =
        point_light_intensity_scale_for_generator(doc.as_json().asset.generator.as_deref());

    let conversion_context = NodeConversionContext {
        doc: &doc,
        buffers,
        mesh_index_to_primitives: &mesh_index_to_primitives,
        mesh_index_to_primitive_variants: &mesh_index_to_primitive_variants,
        materials: &materials,
        document_variant_names: &document_variant_names,
        point_intensity_scale,
    };

    for scene in doc.scenes() {
        let scene_name = scene
            .name()
            .unwrap_or(&format!("Scene_{}", scene.index()))
            .to_string();

        let mut root_nodes = Vec::new();

        for node in scene.nodes() {
            convert_node_to_prefab_nodes(&conversion_context, &node, &mut root_nodes);
        }

        prefabs.push(Prefab {
            name: scene_name,
            root_nodes,
            material_variants: document_variant_names.clone(),
        });
    }

    let animations: Vec<AnimationClip> = doc
        .animations()
        .map(|animation| load_gltf_animation(&animation, buffers, &node_to_morph_target_count))
        .collect();

    let skins: Vec<GltfSkin> = doc
        .skins()
        .map(|skin| load_gltf_skin(&skin, buffers))
        .collect();

    let mut node_to_skin: HashMap<usize, usize> = HashMap::new();
    for node in doc.nodes() {
        if let Some(skin) = node.skin() {
            node_to_skin.insert(node.index(), skin.index());
        }
    }

    let node_count = doc.nodes().count();

    let mut max_intensity: f32 = 1.0;
    if let Some(lights) = doc.lights() {
        for light in lights {
            max_intensity = max_intensity.max(light.intensity());
        }
    }
    for material in materials.iter() {
        max_intensity = max_intensity.max(material.emissive_strength);
    }
    let suggested_exposure = if max_intensity > 10.0 {
        (1.0 / max_intensity).clamp(1e-4, 1.0)
    } else {
        1.0
    };

    let texture_plan: Vec<TextureProductionPlan> = texture_specs
        .into_iter()
        .map(|(name, spec)| TextureProductionPlan {
            name,
            recipe: spec.recipe,
            usage: spec.usage,
            sampler: spec.sampler,
        })
        .collect();

    Ok(GltfLoadResult {
        prefabs,
        meshes,
        materials,
        encoded_images: encoded_image_arena,
        texture_plan,
        animations,
        skins,
        node_to_skin,
        node_to_morph_target_count,
        node_count,
        suggested_exposure,
    })
}

type VariantMappings = Vec<(Vec<u32>, u32)>;
type MeshVariantMap = HashMap<usize, Vec<VariantMappings>>;

fn build_prefab_material_variants(
    mappings: &[(Vec<u32>, u32)],
    materials: &[Material],
    variant_names: &[String],
) -> Vec<crate::assets::prefab::components::PrefabMaterialVariant> {
    let mut out = Vec::with_capacity(mappings.len());
    for (variant_indices, material_index) in mappings {
        if let Some(material) = materials.get(*material_index as usize) {
            let names: Vec<String> = variant_indices
                .iter()
                .filter_map(|index| variant_names.get(*index as usize).cloned())
                .collect();
            if names.is_empty() {
                continue;
            }
            out.push(crate::assets::prefab::components::PrefabMaterialVariant {
                variant_names: names,
                material: material.clone(),
            });
        }
    }
    out
}

fn read_accessor_floats(
    doc: &Document,
    buffers: &[buffer::Data],
    accessor_index: usize,
    components_per_element: usize,
) -> Option<Vec<f32>> {
    let accessor = doc.accessors().nth(accessor_index)?;
    if accessor.data_type() != gltf::accessor::DataType::F32 {
        tracing::warn!(
            "EXT_mesh_gpu_instancing: quantized component types are not supported (accessor {})",
            accessor_index
        );
        return None;
    }
    let view = accessor.view()?;
    let buffer_data = buffers.get(view.buffer().index())?;
    let buffer_bytes = &buffer_data.0;
    let element_bytes = components_per_element * 4;
    let stride = view.stride().unwrap_or(element_bytes);
    let start = view.offset() + accessor.offset();
    let mut out = Vec::with_capacity(accessor.count() * components_per_element);
    for element_index in 0..accessor.count() {
        let element_offset = start + element_index * stride;
        if element_offset + element_bytes > buffer_bytes.len() {
            return None;
        }
        for component_index in 0..components_per_element {
            let byte_offset = element_offset + component_index * 4;
            let bytes = &buffer_bytes[byte_offset..byte_offset + 4];
            out.push(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]));
        }
    }
    Some(out)
}

fn read_gpu_instancing_transforms(
    doc: &Document,
    buffers: &[buffer::Data],
    node: &Node,
) -> Vec<LocalTransform> {
    let extension = match node.extension_value("EXT_mesh_gpu_instancing") {
        Some(value) => value,
        None => return Vec::new(),
    };
    let attributes = match extension.get("attributes").and_then(|v| v.as_object()) {
        Some(map) => map,
        None => return Vec::new(),
    };

    let translation_accessor = attributes
        .get("TRANSLATION")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize);
    let rotation_accessor = attributes
        .get("ROTATION")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize);
    let scale_accessor = attributes
        .get("SCALE")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize);

    let translations =
        translation_accessor.and_then(|index| read_accessor_floats(doc, buffers, index, 3));
    let rotations =
        rotation_accessor.and_then(|index| read_accessor_floats(doc, buffers, index, 4));
    let scales = scale_accessor.and_then(|index| read_accessor_floats(doc, buffers, index, 3));

    let instance_count = translations
        .as_ref()
        .map(|values| values.len() / 3)
        .or_else(|| rotations.as_ref().map(|values| values.len() / 4))
        .or_else(|| scales.as_ref().map(|values| values.len() / 3))
        .unwrap_or(0);

    if instance_count == 0 {
        return Vec::new();
    }

    let mut out = Vec::with_capacity(instance_count);
    for instance_index in 0..instance_count {
        let translation = translations
            .as_ref()
            .map(|values| {
                Vec3::new(
                    values[instance_index * 3],
                    values[instance_index * 3 + 1],
                    values[instance_index * 3 + 2],
                )
            })
            .unwrap_or_else(Vec3::zeros);
        let rotation = rotations
            .as_ref()
            .map(|values| {
                Quat::new(
                    values[instance_index * 4 + 3],
                    values[instance_index * 4],
                    values[instance_index * 4 + 1],
                    values[instance_index * 4 + 2],
                )
            })
            .unwrap_or_else(Quat::identity);
        let scale = scales
            .as_ref()
            .map(|values| {
                Vec3::new(
                    values[instance_index * 3],
                    values[instance_index * 3 + 1],
                    values[instance_index * 3 + 2],
                )
            })
            .unwrap_or_else(|| Vec3::new(1.0, 1.0, 1.0));
        out.push(LocalTransform {
            translation,
            rotation,
            scale,
        });
    }
    out
}

struct NodeConversionContext<'a> {
    doc: &'a Document,
    buffers: &'a [buffer::Data],
    mesh_index_to_primitives: &'a HashMap<usize, Vec<(String, usize)>>,
    mesh_index_to_primitive_variants: &'a MeshVariantMap,
    materials: &'a [Material],
    document_variant_names: &'a [String],
    point_intensity_scale: f32,
}

fn convert_node_to_prefab_nodes(
    context: &NodeConversionContext<'_>,
    node: &Node,
    output_nodes: &mut Vec<PrefabNode>,
) {
    let (translation, rotation, scale) = node.transform().decomposed();

    let quat = Quat::new(rotation[3], rotation[0], rotation[1], rotation[2]);

    let local_transform = LocalTransform {
        translation: Vec3::new(translation[0], translation[1], translation[2]),
        rotation: quat,
        scale: Vec3::new(scale[0], scale[1], scale[2]),
    };

    let node_name = node.name().map(|s| s.to_string());

    let node_skin_index = node.skin().map(|s| s.index());

    let instance_transforms = read_gpu_instancing_transforms(context.doc, context.buffers, node);

    if let Some(mesh) = node.mesh() {
        let mesh_variants = context.mesh_index_to_primitive_variants.get(&mesh.index());
        if let Some(primitives) = context.mesh_index_to_primitives.get(&mesh.index()) {
            if primitives.len() == 1 {
                let (mesh_name, material_index) = &primitives[0];
                let mut components = PrefabComponents::default();

                if let Some(name_str) = &node_name {
                    components.name = Some(Name(name_str.clone()));
                }

                components.render_mesh = Some(RenderMesh::new(mesh_name.clone()));

                components.material = Some(
                    context
                        .materials
                        .get(*material_index)
                        .cloned()
                        .unwrap_or_else(Material::default),
                );

                if let Some(mappings) = mesh_variants.and_then(|v| v.first()) {
                    components.material_variants = build_prefab_material_variants(
                        mappings,
                        context.materials,
                        context.document_variant_names,
                    );
                }

                if let Some(camera) = node.camera() {
                    components.camera = Some(load_gltf_camera(&camera));
                }

                if let Some(light) = node.light() {
                    components.light = Some(load_gltf_light(&light, context.point_intensity_scale));
                }

                components.visibility = Some(Visibility { visible: true });
                components.skin_index = node_skin_index;
                components.instances = instance_transforms.clone();

                let mut children = Vec::new();
                for child in node.children() {
                    convert_node_to_prefab_nodes(context, &child, &mut children);
                }

                output_nodes.push(PrefabNode {
                    local_transform,
                    components,
                    children,
                    node_index: Some(node.index()),
                });
            } else {
                let mut parent_components = PrefabComponents::default();
                if let Some(name_str) = &node_name {
                    parent_components.name = Some(Name(name_str.clone()));
                }

                if let Some(camera) = node.camera() {
                    parent_components.camera = Some(load_gltf_camera(&camera));
                }

                if let Some(light) = node.light() {
                    parent_components.light =
                        Some(load_gltf_light(&light, context.point_intensity_scale));
                }

                parent_components.skin_index = node_skin_index;

                let mut children = Vec::new();

                for (index, (mesh_name, material_index)) in primitives.iter().enumerate() {
                    let mut primitive_components = PrefabComponents {
                        name: Some(Name(format!("Primitive_{}", index))),
                        render_mesh: Some(RenderMesh::new(mesh_name.clone())),
                        visibility: Some(Visibility { visible: true }),
                        skin_index: node_skin_index,
                        instances: instance_transforms.clone(),
                        ..Default::default()
                    };

                    primitive_components.material = Some(
                        context
                            .materials
                            .get(*material_index)
                            .cloned()
                            .unwrap_or_else(Material::default),
                    );

                    if let Some(mappings) = mesh_variants.and_then(|v| v.get(index)) {
                        primitive_components.material_variants = build_prefab_material_variants(
                            mappings,
                            context.materials,
                            context.document_variant_names,
                        );
                    }

                    children.push(PrefabNode {
                        local_transform: LocalTransform::default(),
                        components: primitive_components,
                        children: Vec::new(),
                        node_index: None,
                    });
                }

                for child in node.children() {
                    convert_node_to_prefab_nodes(context, &child, &mut children);
                }

                output_nodes.push(PrefabNode {
                    local_transform,
                    components: parent_components,
                    children,
                    node_index: Some(node.index()),
                });
            }
        } else {
            let mut components = PrefabComponents::default();
            if let Some(name_str) = &node_name {
                components.name = Some(Name(name_str.clone()));
            }

            if let Some(camera) = node.camera() {
                components.camera = Some(load_gltf_camera(&camera));
            }

            if let Some(light) = node.light() {
                components.light = Some(load_gltf_light(&light, context.point_intensity_scale));
            }

            components.skin_index = node_skin_index;

            let mut children = Vec::new();
            for child in node.children() {
                convert_node_to_prefab_nodes(context, &child, &mut children);
            }

            output_nodes.push(PrefabNode {
                local_transform,
                components,
                children,
                node_index: Some(node.index()),
            });
        }
    } else {
        let mut components = PrefabComponents::default();
        if let Some(name_str) = &node_name {
            components.name = Some(Name(name_str.clone()));
        }

        if let Some(camera) = node.camera() {
            components.camera = Some(load_gltf_camera(&camera));
        }

        if let Some(light) = node.light() {
            components.light = Some(load_gltf_light(&light, context.point_intensity_scale));
        }

        components.skin_index = node_skin_index;

        let mut children = Vec::new();
        for child in node.children() {
            convert_node_to_prefab_nodes(context, &child, &mut children);
        }

        output_nodes.push(PrefabNode {
            local_transform,
            components,
            children,
            node_index: Some(node.index()),
        });
    }
}

fn load_gltf_light(
    light: &gltf::khr_lights_punctual::Light<'_>,
    point_intensity_scale: f32,
) -> Light {
    let kind = light.kind();
    let (light_type, inner_cone_angle, outer_cone_angle) = match kind {
        gltf::khr_lights_punctual::Kind::Directional => (LightType::Directional, 0.0, 0.0),
        gltf::khr_lights_punctual::Kind::Point => (LightType::Point, 0.0, 0.0),
        gltf::khr_lights_punctual::Kind::Spot {
            inner_cone_angle,
            outer_cone_angle,
        } => (LightType::Spot, inner_cone_angle, outer_cone_angle),
    };

    let color = light.color();
    let mut intensity = light.intensity();

    if !matches!(light_type, LightType::Directional) {
        intensity *= point_intensity_scale;
    }

    let range = light.range().unwrap_or(0.0);

    Light {
        light_type,
        color: Vec3::new(color[0], color[1], color[2]),
        intensity,
        range,
        inner_cone_angle,
        outer_cone_angle,
        cast_shadows: matches!(light_type, LightType::Directional),
        shadow_bias: 0.0005,
        shadow_resolution: 0,
        shadow_distance: 0.0,
        cookie_texture: None,
        ..Default::default()
    }
}

/// Older Blender glTF exporters (< v3.0) write point/spot light intensity
/// in lumens; the glTF spec calls for candela. Modern Blender (3.0+) and
/// Khronos sample exporters write spec-compliant candela. We sniff the
/// `asset.generator` string and apply the lumens â†’ candela conversion
/// (divide by 4Ï€) only when the file came from old Blender.
fn point_light_intensity_scale_for_generator(generator: Option<&str>) -> f32 {
    let Some(generator) = generator else {
        return 1.0;
    };
    if !generator.contains("Khronos glTF Blender I/O") {
        return 1.0;
    }
    let Some(version_part) = generator.rsplit('v').next() else {
        return 1.0;
    };
    let Some(major_str) = version_part.split('.').next() else {
        return 1.0;
    };
    let Ok(major) = major_str.parse::<u32>() else {
        return 1.0;
    };
    if major < 3 {
        1.0 / (4.0 * std::f32::consts::PI)
    } else {
        1.0
    }
}

fn load_gltf_camera(camera: &gltf::Camera<'_>) -> Camera {
    let projection = match camera.projection() {
        gltf::camera::Projection::Perspective(persp) => {
            Projection::Perspective(PerspectiveCamera {
                aspect_ratio: persp.aspect_ratio(),
                y_fov_rad: persp.yfov(),
                z_far: persp.zfar(),
                z_near: persp.znear(),
            })
        }
        gltf::camera::Projection::Orthographic(ortho) => {
            Projection::Orthographic(OrthographicCamera {
                x_mag: ortho.xmag(),
                y_mag: ortho.ymag(),
                z_far: ortho.zfar(),
                z_near: ortho.znear(),
            })
        }
    };

    Camera {
        projection,
        smoothing: None,
    }
}

fn load_gltf_primitive(
    primitive: &Primitive,
    buffers: &[buffer::Data],
    default_morph_weights: Option<&[f32]>,
) -> Option<Mesh> {
    let reader = primitive.reader(|buffer| buffers.get(buffer.index()).map(|b| &b.0[..]));

    let positions: Vec<Vec3> = reader
        .read_positions()?
        .map(|p| vec3(p[0], p[1], p[2]))
        .collect();

    if positions.is_empty() {
        tracing::warn!("glTF primitive has no positions, skipping");
        return None;
    }

    let normals: Vec<Vec3> = if let Some(normals_iter) = reader.read_normals() {
        let normals: Vec<_> = normals_iter.map(|n| vec3(n[0], n[1], n[2])).collect();

        if normals.len() != positions.len() {
            tracing::warn!("glTF primitive has mismatched normal count, generating normals");
            generate_normals(&positions, &reader.read_indices())
        } else {
            normals
        }
    } else {
        generate_normals(&positions, &reader.read_indices())
    };

    let mut texcoords: Vec<Vec2> = vec![vec2(0.0, 0.0); positions.len()];

    if let Some(tex_iter) = reader.read_tex_coords(0) {
        let coords: Vec<_> = tex_iter.into_f32().map(|t| vec2(t[0], t[1])).collect();

        if coords.len() == positions.len() {
            texcoords = coords;
        } else {
            tracing::warn!("glTF primitive has mismatched texcoord count, using defaults");
        }
    }

    let mut texcoords_1: Vec<Vec2> = vec![vec2(0.0, 0.0); positions.len()];

    if let Some(tex_iter) = reader.read_tex_coords(1) {
        let coords: Vec<_> = tex_iter.into_f32().map(|t| vec2(t[0], t[1])).collect();

        if coords.len() == positions.len() {
            texcoords_1 = coords;
        }
    }

    let colors: Vec<[f32; 4]> = if let Some(color_iter) = reader.read_colors(0) {
        let color_data: Vec<[f32; 4]> = color_iter.into_rgba_f32().collect();
        if color_data.len() == positions.len() {
            color_data
        } else {
            vec![[1.0, 1.0, 1.0, 1.0]; positions.len()]
        }
    } else {
        vec![[1.0, 1.0, 1.0, 1.0]; positions.len()]
    };

    let raw_indices: Vec<u32> = if let Some(indices_iter) = reader.read_indices() {
        indices_iter.into_u32().collect()
    } else {
        (0..positions.len() as u32).collect()
    };

    if raw_indices.is_empty() {
        tracing::warn!("glTF primitive has no indices, skipping");
        return None;
    }

    let has_normal_texture = primitive.material().normal_texture().is_some();

    let raw_tangents: Option<Vec<[f32; 4]>> =
        reader.read_tangents().map(|iter| iter.collect::<Vec<_>>());
    let tangents_were_provided = raw_tangents
        .as_ref()
        .is_some_and(|tangents| tangents.len() == positions.len());
    let tangents: Vec<[f32; 4]> = match raw_tangents {
        Some(tangent_data) if tangent_data.len() == positions.len() => tangent_data,
        _ => vec![[1.0, 0.0, 0.0, 1.0]; positions.len()],
    };

    let has_joints = reader.read_joints(0).is_some();
    let joint_indices_opt: Option<Vec<[u32; 4]>> = if has_joints {
        let mut joint_indices: Vec<[u32; 4]> = vec![[0, 0, 0, 0]; positions.len()];
        if let Some(joints_iter) = reader.read_joints(0) {
            let joints: Vec<[u16; 4]> = joints_iter.into_u16().collect();
            if joints.len() == positions.len() {
                for (vertex_index, joint) in joints.iter().enumerate() {
                    joint_indices[vertex_index] = [
                        joint[0] as u32,
                        joint[1] as u32,
                        joint[2] as u32,
                        joint[3] as u32,
                    ];
                }
            }
        }
        Some(joint_indices)
    } else {
        None
    };

    let joint_weights_opt: Option<Vec<[f32; 4]>> = if has_joints {
        let joint_weights: Vec<[f32; 4]> = if let Some(weights_iter) = reader.read_weights(0) {
            let weights: Vec<[f32; 4]> = weights_iter.into_f32().collect();
            if weights.len() == positions.len() {
                weights
                    .into_iter()
                    .map(|w| {
                        let sum = w[0] + w[1] + w[2] + w[3];
                        if sum > 0.0 {
                            [w[0] / sum, w[1] / sum, w[2] / sum, w[3] / sum]
                        } else {
                            [1.0, 0.0, 0.0, 0.0]
                        }
                    })
                    .collect()
            } else {
                vec![[1.0, 0.0, 0.0, 0.0]; positions.len()]
            }
        } else {
            vec![[1.0, 0.0, 0.0, 0.0]; positions.len()]
        };
        Some(joint_weights)
    } else {
        None
    };

    let mode = primitive.mode();
    let bbox_extent = bounding_box_extent(&positions);
    let mut attributes = PrimitiveAttributes {
        positions,
        normals,
        texcoords,
        texcoords_1,
        colors,
        tangents,
        joint_indices: joint_indices_opt,
        joint_weights: joint_weights_opt,
    };
    let indices = convert_primitive_to_triangles(mode, &raw_indices, &mut attributes, bbox_extent);

    if indices.is_empty() {
        tracing::warn!("glTF primitive has no triangles after mode conversion, skipping");
        return None;
    }

    if !tangents_were_provided
        && has_normal_texture
        && matches!(
            mode,
            gltf::mesh::Mode::Triangles
                | gltf::mesh::Mode::TriangleStrip
                | gltf::mesh::Mode::TriangleFan
        )
    {
        attributes.tangents = generate_mikktspace_tangents(
            &attributes.positions,
            &attributes.normals,
            &attributes.texcoords,
            &indices,
        );
    }

    let PrimitiveAttributes {
        positions,
        normals,
        texcoords,
        texcoords_1,
        colors,
        tangents,
        joint_indices: joint_indices_opt,
        joint_weights: joint_weights_opt,
    } = attributes;

    let skin_data = if let (Some(joint_indices), Some(joint_weights)) =
        (joint_indices_opt, joint_weights_opt)
    {
        let skinned_vertices: Vec<SkinnedVertex> = positions
            .iter()
            .zip(normals.iter())
            .zip(texcoords.iter())
            .zip(texcoords_1.iter())
            .zip(tangents.iter())
            .zip(colors.iter())
            .zip(joint_indices.iter())
            .zip(joint_weights.iter())
            .map(
                |(
                    (
                        (((((position, normal), tex_coords), tex_coords_1), tangent), color),
                        joint_indices,
                    ),
                    joint_weights,
                )| {
                    SkinnedVertex {
                        position: [position.x, position.y, position.z],
                        normal: [normal.x, normal.y, normal.z],
                        tex_coords: [tex_coords.x, tex_coords.y],
                        tex_coords_1: [tex_coords_1.x, tex_coords_1.y],
                        tangent: *tangent,
                        color: *color,
                        joint_indices: *joint_indices,
                        joint_weights: *joint_weights,
                    }
                },
            )
            .collect();
        Some(SkinData::new(skinned_vertices))
    } else {
        None
    };

    if positions.len() != normals.len() || positions.len() != texcoords.len() {
        tracing::error!("glTF primitive has inconsistent vertex attribute lengths");
        return None;
    }

    let vertices: Vec<Vertex> = positions
        .iter()
        .zip(&normals)
        .zip(&texcoords)
        .zip(&texcoords_1)
        .zip(&tangents)
        .zip(&colors)
        .map(
            |(((((position, normal), tex_coords), tex_coords_1), tangent), color)| Vertex {
                position: [position.x, position.y, position.z],
                normal: [normal.x, normal.y, normal.z],
                tex_coords: [tex_coords.x, tex_coords.y],
                tex_coords_1: [tex_coords_1.x, tex_coords_1.y],
                tangent: *tangent,
                color: *color,
            },
        )
        .collect();

    let mut min = nalgebra_glm::vec3(f32::MAX, f32::MAX, f32::MAX);
    let mut max = nalgebra_glm::vec3(f32::MIN, f32::MIN, f32::MIN);

    for position in &positions {
        min.x = min.x.min(position.x);
        min.y = min.y.min(position.y);
        min.z = min.z.min(position.z);

        max.x = max.x.max(position.x);
        max.y = max.y.max(position.y);
        max.z = max.z.max(position.z);
    }

    let (center, half_extents, sphere_radius) = if positions.is_empty() {
        (
            nalgebra_glm::Vec3::zeros(),
            nalgebra_glm::Vec3::zeros(),
            0.0,
        )
    } else {
        let center = (min + max) * 0.5;
        let half_extents = (max - min) * 0.5;
        let sphere_radius = nalgebra_glm::length(&half_extents);
        (center, half_extents, sphere_radius)
    };

    let bounding_volume = crate::render::bounding_volume::BoundingVolume {
        obb: crate::render::bounding_volume::OrientedBoundingBox {
            center,
            half_extents,
            orientation: nalgebra_glm::quat_identity(),
        },
        sphere_radius,
    };

    let morph_targets = load_morph_targets(
        primitive,
        buffers,
        &positions,
        &normals,
        default_morph_weights,
    );

    let mut mesh = Mesh::with_bounding_volume(vertices, indices, bounding_volume);
    mesh.skin_data = skin_data;
    mesh.morph_targets = morph_targets;

    Some(mesh)
}

struct PrimitiveAttributes {
    positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    texcoords: Vec<Vec2>,
    texcoords_1: Vec<Vec2>,
    colors: Vec<[f32; 4]>,
    tangents: Vec<[f32; 4]>,
    joint_indices: Option<Vec<[u32; 4]>>,
    joint_weights: Option<Vec<[f32; 4]>>,
}

fn bounding_box_extent(positions: &[Vec3]) -> f32 {
    if positions.is_empty() {
        return 1.0;
    }
    let mut min = vec3(f32::MAX, f32::MAX, f32::MAX);
    let mut max = vec3(f32::MIN, f32::MIN, f32::MIN);
    for position in positions {
        min.x = min.x.min(position.x);
        min.y = min.y.min(position.y);
        min.z = min.z.min(position.z);
        max.x = max.x.max(position.x);
        max.y = max.y.max(position.y);
        max.z = max.z.max(position.z);
    }
    let extent = max - min;
    extent.x.max(extent.y).max(extent.z).max(1.0e-6)
}

fn convert_primitive_to_triangles(
    mode: gltf::mesh::Mode,
    raw_indices: &[u32],
    attributes: &mut PrimitiveAttributes,
    bbox_extent: f32,
) -> Vec<u32> {
    use gltf::mesh::Mode;
    match mode {
        Mode::Triangles => {
            let triangle_count = raw_indices.len() / 3;
            raw_indices[..triangle_count * 3].to_vec()
        }
        Mode::TriangleStrip => triangle_strip_to_list(raw_indices),
        Mode::TriangleFan => triangle_fan_to_list(raw_indices),
        Mode::Points => points_to_triangles(raw_indices, attributes, bbox_extent * 0.025),
        Mode::Lines => {
            let segments: Vec<(u32, u32)> = raw_indices
                .chunks_exact(2)
                .map(|chunk| (chunk[0], chunk[1]))
                .collect();
            line_segments_to_triangles(&segments, attributes, bbox_extent * 0.005)
        }
        Mode::LineStrip => {
            if raw_indices.len() < 2 {
                return Vec::new();
            }
            let segments: Vec<(u32, u32)> = (0..raw_indices.len() - 1)
                .map(|segment_index| (raw_indices[segment_index], raw_indices[segment_index + 1]))
                .collect();
            line_segments_to_triangles(&segments, attributes, bbox_extent * 0.005)
        }
        Mode::LineLoop => {
            if raw_indices.len() < 2 {
                return Vec::new();
            }
            let mut segments: Vec<(u32, u32)> = (0..raw_indices.len() - 1)
                .map(|segment_index| (raw_indices[segment_index], raw_indices[segment_index + 1]))
                .collect();
            segments.push((raw_indices[raw_indices.len() - 1], raw_indices[0]));
            line_segments_to_triangles(&segments, attributes, bbox_extent * 0.005)
        }
    }
}

fn triangle_strip_to_list(strip_indices: &[u32]) -> Vec<u32> {
    if strip_indices.len() < 3 {
        return Vec::new();
    }
    let mut output = Vec::with_capacity((strip_indices.len() - 2) * 3);
    for triangle_offset in 0..strip_indices.len() - 2 {
        let (a, b, c) = if triangle_offset.is_multiple_of(2) {
            (
                strip_indices[triangle_offset],
                strip_indices[triangle_offset + 1],
                strip_indices[triangle_offset + 2],
            )
        } else {
            (
                strip_indices[triangle_offset + 1],
                strip_indices[triangle_offset],
                strip_indices[triangle_offset + 2],
            )
        };
        if a == b || b == c || a == c {
            continue;
        }
        output.push(a);
        output.push(b);
        output.push(c);
    }
    output
}

fn triangle_fan_to_list(fan_indices: &[u32]) -> Vec<u32> {
    if fan_indices.len() < 3 {
        return Vec::new();
    }
    let mut output = Vec::with_capacity((fan_indices.len() - 2) * 3);
    let center = fan_indices[0];
    for triangle_offset in 1..fan_indices.len() - 1 {
        let edge_start = fan_indices[triangle_offset];
        let edge_end = fan_indices[triangle_offset + 1];
        if center == edge_start || edge_start == edge_end || center == edge_end {
            continue;
        }
        output.push(center);
        output.push(edge_start);
        output.push(edge_end);
    }
    output
}

fn points_to_triangles(
    point_indices: &[u32],
    attributes: &mut PrimitiveAttributes,
    radius: f32,
) -> Vec<u32> {
    let octahedron_offsets = [
        vec3(radius, 0.0, 0.0),
        vec3(-radius, 0.0, 0.0),
        vec3(0.0, radius, 0.0),
        vec3(0.0, -radius, 0.0),
        vec3(0.0, 0.0, radius),
        vec3(0.0, 0.0, -radius),
    ];
    let octahedron_triangles: [[u32; 3]; 8] = [
        [0, 2, 4],
        [2, 1, 4],
        [1, 3, 4],
        [3, 0, 4],
        [2, 0, 5],
        [1, 2, 5],
        [3, 1, 5],
        [0, 3, 5],
    ];

    let mut new_attributes = PrimitiveAttributes {
        positions: Vec::new(),
        normals: Vec::new(),
        texcoords: Vec::new(),
        texcoords_1: Vec::new(),
        colors: Vec::new(),
        tangents: Vec::new(),
        joint_indices: attributes.joint_indices.as_ref().map(|_| Vec::new()),
        joint_weights: attributes.joint_weights.as_ref().map(|_| Vec::new()),
    };
    let mut output_indices = Vec::new();

    for &point_index in point_indices {
        let source = point_index as usize;
        if source >= attributes.positions.len() {
            continue;
        }
        let center = attributes.positions[source];
        let texcoord = attributes.texcoords[source];
        let texcoord_1 = attributes.texcoords_1[source];
        let color = attributes.colors[source];
        let tangent = attributes.tangents[source];
        let source_joint_indices = attributes
            .joint_indices
            .as_ref()
            .map(|joints| joints[source]);
        let source_joint_weights = attributes
            .joint_weights
            .as_ref()
            .map(|weights| weights[source]);

        let base = new_attributes.positions.len() as u32;
        for offset in &octahedron_offsets {
            new_attributes.positions.push(center + offset);
            new_attributes.normals.push(nalgebra_glm::normalize(offset));
            new_attributes.texcoords.push(texcoord);
            new_attributes.texcoords_1.push(texcoord_1);
            new_attributes.colors.push(color);
            new_attributes.tangents.push(tangent);
            if let (Some(target), Some(value)) =
                (new_attributes.joint_indices.as_mut(), source_joint_indices)
            {
                target.push(value);
            }
            if let (Some(target), Some(value)) =
                (new_attributes.joint_weights.as_mut(), source_joint_weights)
            {
                target.push(value);
            }
        }
        for triangle in &octahedron_triangles {
            output_indices.push(base + triangle[0]);
            output_indices.push(base + triangle[1]);
            output_indices.push(base + triangle[2]);
        }
    }

    *attributes = new_attributes;
    output_indices
}

fn line_segments_to_triangles(
    segments: &[(u32, u32)],
    attributes: &mut PrimitiveAttributes,
    radius: f32,
) -> Vec<u32> {
    let prism_triangles: [[u32; 3]; 12] = [
        [0, 3, 2],
        [0, 2, 1],
        [4, 5, 6],
        [4, 6, 7],
        [0, 4, 7],
        [0, 7, 3],
        [1, 2, 6],
        [1, 6, 5],
        [0, 1, 5],
        [0, 5, 4],
        [3, 7, 6],
        [3, 6, 2],
    ];

    let mut new_attributes = PrimitiveAttributes {
        positions: Vec::new(),
        normals: Vec::new(),
        texcoords: Vec::new(),
        texcoords_1: Vec::new(),
        colors: Vec::new(),
        tangents: Vec::new(),
        joint_indices: attributes.joint_indices.as_ref().map(|_| Vec::new()),
        joint_weights: attributes.joint_weights.as_ref().map(|_| Vec::new()),
    };
    let mut output_indices = Vec::new();

    for &(start_index, end_index) in segments {
        let start = start_index as usize;
        let end = end_index as usize;
        if start >= attributes.positions.len() || end >= attributes.positions.len() {
            continue;
        }
        let p0 = attributes.positions[start];
        let p1 = attributes.positions[end];
        let direction = p1 - p0;
        if nalgebra_glm::length(&direction) < 1.0e-8 {
            continue;
        }
        let direction_norm = nalgebra_glm::normalize(&direction);
        let up_candidate = if direction_norm.y.abs() < 0.99 {
            vec3(0.0, 1.0, 0.0)
        } else {
            vec3(1.0, 0.0, 0.0)
        };
        let side_a =
            nalgebra_glm::normalize(&nalgebra_glm::cross(&direction_norm, &up_candidate)) * radius;
        let side_b =
            nalgebra_glm::normalize(&nalgebra_glm::cross(&direction_norm, &side_a)) * radius;

        let corners = [
            p0 - side_a - side_b,
            p0 + side_a - side_b,
            p0 + side_a + side_b,
            p0 - side_a + side_b,
            p1 - side_a - side_b,
            p1 + side_a - side_b,
            p1 + side_a + side_b,
            p1 - side_a + side_b,
        ];

        let texcoord = attributes.texcoords[start];
        let texcoord_1 = attributes.texcoords_1[start];
        let color = attributes.colors[start];
        let tangent = attributes.tangents[start];
        let source_joint_indices = attributes
            .joint_indices
            .as_ref()
            .map(|joints| joints[start]);
        let source_joint_weights = attributes
            .joint_weights
            .as_ref()
            .map(|weights| weights[start]);

        let segment_center = p0 + (p1 - p0) * 0.5;
        let base = new_attributes.positions.len() as u32;
        for corner in &corners {
            let outward = corner - segment_center;
            let normal = if nalgebra_glm::length(&outward) > 1.0e-6 {
                nalgebra_glm::normalize(&outward)
            } else {
                direction_norm
            };
            new_attributes.positions.push(*corner);
            new_attributes.normals.push(normal);
            new_attributes.texcoords.push(texcoord);
            new_attributes.texcoords_1.push(texcoord_1);
            new_attributes.colors.push(color);
            new_attributes.tangents.push(tangent);
            if let (Some(target), Some(value)) =
                (new_attributes.joint_indices.as_mut(), source_joint_indices)
            {
                target.push(value);
            }
            if let (Some(target), Some(value)) =
                (new_attributes.joint_weights.as_mut(), source_joint_weights)
            {
                target.push(value);
            }
        }
        for triangle in &prism_triangles {
            output_indices.push(base + triangle[0]);
            output_indices.push(base + triangle[1]);
            output_indices.push(base + triangle[2]);
        }
    }

    *attributes = new_attributes;
    output_indices
}

fn load_morph_targets(
    primitive: &Primitive,
    buffers: &[buffer::Data],
    positions: &[Vec3],
    normals: &[Vec3],
    default_weights: Option<&[f32]>,
) -> Option<MorphTargetData> {
    let reader = primitive.reader(|buffer| buffers.get(buffer.index()).map(|b| &b.0[..]));
    let vertex_count = positions.len();

    let morph_target_iter = reader.read_morph_targets();
    let targets: Vec<MorphTarget> = morph_target_iter
        .filter_map(|morph_target| {
            let position_displacements: Vec<[f32; 3]> =
                morph_target.0?.map(|p| [p[0], p[1], p[2]]).collect();

            if position_displacements.len() != vertex_count {
                return None;
            }

            let mut target = MorphTarget::new(position_displacements);

            if let Some(normals_iter) = morph_target.1 {
                let normal_displacements: Vec<[f32; 3]> =
                    normals_iter.map(|n| [n[0], n[1], n[2]]).collect();
                if normal_displacements.len() == vertex_count {
                    target = target.with_normals(normal_displacements);
                }
            }

            if let Some(tangents_iter) = morph_target.2 {
                let tangent_displacements: Vec<[f32; 3]> =
                    tangents_iter.map(|t| [t[0], t[1], t[2]]).collect();
                if tangent_displacements.len() == vertex_count {
                    target = target.with_tangents(tangent_displacements);
                }
            }

            Some(target)
        })
        .collect();

    if targets.is_empty() {
        None
    } else {
        let base_positions: Vec<[f32; 3]> = positions.iter().map(|p| [p.x, p.y, p.z]).collect();
        let base_normals: Vec<[f32; 3]> = normals.iter().map(|n| [n.x, n.y, n.z]).collect();
        let weights = match default_weights {
            Some(w) => w.iter().take(targets.len()).copied().collect(),
            None => vec![0.0; targets.len()],
        };
        Some(
            MorphTargetData::new(targets)
                .with_base_data(base_positions, base_normals)
                .with_default_weights(weights),
        )
    }
}

fn generate_normals(
    positions: &[Vec3],
    indices_reader: &Option<gltf::mesh::util::ReadIndices>,
) -> Vec<Vec3> {
    let mut normals = vec![Vec3::zeros(); positions.len()];
    let mut face_counts = vec![0u32; positions.len()];

    let indices: Vec<u32> = if let Some(indices_iter) = indices_reader {
        indices_iter.clone().into_u32().collect()
    } else {
        (0..positions.len() as u32).collect()
    };

    for triangle in indices.chunks(3) {
        if triangle.len() != 3 {
            continue;
        }

        let i0 = triangle[0] as usize;
        let i1 = triangle[1] as usize;
        let i2 = triangle[2] as usize;

        if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
            continue;
        }

        let v0 = positions[i0];
        let v1 = positions[i1];
        let v2 = positions[i2];

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let face_normal = nalgebra_glm::cross(&edge1, &edge2);

        let face_normal = if nalgebra_glm::length(&face_normal) > 1e-6 {
            nalgebra_glm::normalize(&face_normal)
        } else {
            vec3(0.0, 1.0, 0.0)
        };

        normals[i0] += face_normal;
        normals[i1] += face_normal;
        normals[i2] += face_normal;

        face_counts[i0] += 1;
        face_counts[i1] += 1;
        face_counts[i2] += 1;
    }

    for (index, normal) in normals.iter_mut().enumerate() {
        if face_counts[index] > 0 {
            *normal /= face_counts[index] as f32;
            let magnitude = nalgebra_glm::length(normal);
            if magnitude > 1e-6 {
                *normal = nalgebra_glm::normalize(normal);
            } else {
                *normal = vec3(0.0, 1.0, 0.0);
            }
        } else {
            *normal = vec3(0.0, 1.0, 0.0);
        }
    }

    normals
}

fn load_gltf_animation(
    animation: &gltf::Animation,
    buffers: &[buffer::Data],
    node_to_morph_target_count: &HashMap<usize, usize>,
) -> AnimationClip {
    let mut channels = Vec::new();

    for (channel_index, channel) in animation.channels().enumerate() {
        let reader = channel.reader(|buffer| buffers.get(buffer.index()).map(|b| &b.0[..]));

        let target_gltf_node = channel.target().node();
        let target_node = target_gltf_node.index();
        let target_bone_name = target_gltf_node.name().map(|s| s.to_string());
        let target_property = match channel.target().property() {
            gltf::animation::Property::Translation => AnimationProperty::Translation,
            gltf::animation::Property::Rotation => AnimationProperty::Rotation,
            gltf::animation::Property::Scale => AnimationProperty::Scale,
            gltf::animation::Property::MorphTargetWeights => AnimationProperty::MorphWeights,
        };

        let interpolation = match channel.sampler().interpolation() {
            gltf::animation::Interpolation::Linear => AnimationInterpolation::Linear,
            gltf::animation::Interpolation::Step => AnimationInterpolation::Step,
            gltf::animation::Interpolation::CubicSpline => AnimationInterpolation::CubicSpline,
        };

        let Some(inputs) = reader.read_inputs() else {
            tracing::warn!("  Channel {}: skipping - no input data", channel_index);
            continue;
        };
        let input: Vec<f32> = inputs.collect();

        let Some(outputs) = reader.read_outputs() else {
            tracing::warn!("  Channel {}: skipping - no output data", channel_index);
            continue;
        };

        let output = match outputs {
            gltf::animation::util::ReadOutputs::Translations(translations) => {
                if interpolation == AnimationInterpolation::CubicSpline {
                    let all_data: Vec<[f32; 3]> = translations.collect();
                    let num_keyframes = all_data.len() / 3;

                    let mut in_tangents = Vec::with_capacity(num_keyframes);
                    let mut values = Vec::with_capacity(num_keyframes);
                    let mut out_tangents = Vec::with_capacity(num_keyframes);

                    for keyframe_index in 0..num_keyframes {
                        let base_idx = keyframe_index * 3;
                        in_tangents.push(vec3(
                            all_data[base_idx][0],
                            all_data[base_idx][1],
                            all_data[base_idx][2],
                        ));
                        values.push(vec3(
                            all_data[base_idx + 1][0],
                            all_data[base_idx + 1][1],
                            all_data[base_idx + 1][2],
                        ));
                        out_tangents.push(vec3(
                            all_data[base_idx + 2][0],
                            all_data[base_idx + 2][1],
                            all_data[base_idx + 2][2],
                        ));
                    }

                    AnimationSamplerOutput::CubicSplineVec3 {
                        values,
                        in_tangents,
                        out_tangents,
                    }
                } else {
                    let values = translations.map(|t| vec3(t[0], t[1], t[2])).collect();
                    AnimationSamplerOutput::Vec3(values)
                }
            }
            gltf::animation::util::ReadOutputs::Rotations(rotations) => {
                if interpolation == AnimationInterpolation::CubicSpline {
                    let all_data: Vec<[f32; 4]> = rotations.into_f32().collect();
                    let num_keyframes = all_data.len() / 3;

                    let mut in_tangents = Vec::with_capacity(num_keyframes);
                    let mut values = Vec::with_capacity(num_keyframes);
                    let mut out_tangents = Vec::with_capacity(num_keyframes);

                    for keyframe_index in 0..num_keyframes {
                        let base_idx = keyframe_index * 3;
                        in_tangents.push(Quat::new(
                            all_data[base_idx][3],
                            all_data[base_idx][0],
                            all_data[base_idx][1],
                            all_data[base_idx][2],
                        ));
                        values.push(
                            Quat::new(
                                all_data[base_idx + 1][3],
                                all_data[base_idx + 1][0],
                                all_data[base_idx + 1][1],
                                all_data[base_idx + 1][2],
                            )
                            .normalize(),
                        );
                        out_tangents.push(Quat::new(
                            all_data[base_idx + 2][3],
                            all_data[base_idx + 2][0],
                            all_data[base_idx + 2][1],
                            all_data[base_idx + 2][2],
                        ));
                    }

                    AnimationSamplerOutput::CubicSplineQuat {
                        values,
                        in_tangents,
                        out_tangents,
                    }
                } else {
                    let values = rotations
                        .into_f32()
                        .map(|r| Quat::new(r[3], r[0], r[1], r[2]).normalize())
                        .collect();
                    AnimationSamplerOutput::Quat(values)
                }
            }
            gltf::animation::util::ReadOutputs::Scales(scales) => {
                if interpolation == AnimationInterpolation::CubicSpline {
                    let all_data: Vec<[f32; 3]> = scales.collect();
                    let num_keyframes = all_data.len() / 3;

                    let mut in_tangents = Vec::with_capacity(num_keyframes);
                    let mut values = Vec::with_capacity(num_keyframes);
                    let mut out_tangents = Vec::with_capacity(num_keyframes);

                    for keyframe_index in 0..num_keyframes {
                        let base_idx = keyframe_index * 3;
                        in_tangents.push(vec3(
                            all_data[base_idx][0],
                            all_data[base_idx][1],
                            all_data[base_idx][2],
                        ));
                        values.push(vec3(
                            all_data[base_idx + 1][0],
                            all_data[base_idx + 1][1],
                            all_data[base_idx + 1][2],
                        ));
                        out_tangents.push(vec3(
                            all_data[base_idx + 2][0],
                            all_data[base_idx + 2][1],
                            all_data[base_idx + 2][2],
                        ));
                    }

                    AnimationSamplerOutput::CubicSplineVec3 {
                        values,
                        in_tangents,
                        out_tangents,
                    }
                } else {
                    let values = scales.map(|s| vec3(s[0], s[1], s[2])).collect();
                    AnimationSamplerOutput::Vec3(values)
                }
            }
            gltf::animation::util::ReadOutputs::MorphTargetWeights(weights) => {
                let weight_count = node_to_morph_target_count
                    .get(&target_node)
                    .copied()
                    .unwrap_or(1);
                if weight_count == 0 {
                    continue;
                }

                if interpolation == AnimationInterpolation::CubicSpline {
                    let all_data: Vec<f32> = weights.into_f32().collect();
                    let elements_per_keyframe = weight_count * 3;
                    if !all_data.len().is_multiple_of(elements_per_keyframe) {
                        continue;
                    }
                    let num_keyframes = all_data.len() / elements_per_keyframe;

                    let mut in_tangents = Vec::with_capacity(num_keyframes);
                    let mut values = Vec::with_capacity(num_keyframes);
                    let mut out_tangents = Vec::with_capacity(num_keyframes);

                    for keyframe_index in 0..num_keyframes {
                        let base_idx = keyframe_index * elements_per_keyframe;
                        let in_tangent: Vec<f32> =
                            all_data[base_idx..base_idx + weight_count].to_vec();
                        let value: Vec<f32> =
                            all_data[base_idx + weight_count..base_idx + weight_count * 2].to_vec();
                        let out_tangent: Vec<f32> = all_data
                            [base_idx + weight_count * 2..base_idx + weight_count * 3]
                            .to_vec();
                        in_tangents.push(in_tangent);
                        values.push(value);
                        out_tangents.push(out_tangent);
                    }

                    AnimationSamplerOutput::CubicSplineWeights {
                        values,
                        in_tangents,
                        out_tangents,
                    }
                } else {
                    let all_data: Vec<f32> = weights.into_f32().collect();
                    if !all_data.len().is_multiple_of(weight_count) {
                        continue;
                    }
                    let values: Vec<Vec<f32>> = all_data
                        .chunks(weight_count)
                        .map(|chunk| chunk.to_vec())
                        .collect();
                    AnimationSamplerOutput::Weights(values)
                }
            }
        };

        let sampler = AnimationSampler {
            input,
            output,
            interpolation,
        };

        channels.push(AnimationChannel {
            target_node,
            target_bone_name,
            target_property,
            sampler,
        });
    }

    let duration = channels
        .iter()
        .flat_map(|channel| channel.sampler.input.last())
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .copied()
        .unwrap_or(0.0);

    AnimationClip {
        name: animation.name().unwrap_or("Animation").to_string(),
        duration,
        channels,
        events: Vec::new(),
    }
}

fn load_gltf_skin(skin: &gltf::Skin, buffers: &[buffer::Data]) -> GltfSkin {
    let joints: Vec<usize> = skin.joints().map(|j| j.index()).collect();

    let mut inverse_bind_matrices = Vec::new();

    if let Some(accessor) = skin.inverse_bind_matrices() {
        let view = accessor
            .view()
            .expect("Inverse bind matrices accessor has no view");
        let buffer_index = view.buffer().index();

        if let Some(buffer_data) = buffers.get(buffer_index) {
            let buffer_bytes = &buffer_data.0;
            let view_offset = view.offset();
            let accessor_offset = accessor.offset();
            let start = view_offset + accessor_offset;
            let stride = view.stride().unwrap_or(64);

            for joint_index in 0..accessor.count() {
                let matrix_start = start + joint_index * stride;
                if matrix_start + 64 <= buffer_bytes.len() {
                    let mut floats = [0.0f32; 16];
                    for (float_index, float_value) in floats.iter_mut().enumerate() {
                        let byte_offset = matrix_start + float_index * 4;
                        let bytes = &buffer_bytes[byte_offset..byte_offset + 4];
                        *float_value = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                    }

                    let matrix = Mat4::new(
                        floats[0], floats[4], floats[8], floats[12], floats[1], floats[5],
                        floats[9], floats[13], floats[2], floats[6], floats[10], floats[14],
                        floats[3], floats[7], floats[11], floats[15],
                    );
                    inverse_bind_matrices.push(matrix);
                }
            }
        }
    }

    while inverse_bind_matrices.len() < joints.len() {
        inverse_bind_matrices.push(Mat4::identity());
    }

    GltfSkin {
        name: skin.name().map(|s| s.to_string()),
        joints,
        inverse_bind_matrices,
    }
}

struct IndexedMesh<'a> {
    positions: &'a [Vec3],
    normals: &'a [Vec3],
    texcoords: &'a [Vec2],
    indices: &'a [u32],
    tangents: Vec<[f32; 4]>,
    written: Vec<bool>,
}

impl bevy_mikktspace::Geometry for IndexedMesh<'_> {
    fn num_faces(&self) -> usize {
        self.indices.len() / 3
    }

    fn num_vertices_of_face(&self, _face: usize) -> usize {
        3
    }

    fn position(&self, face: usize, vert: usize) -> [f32; 3] {
        let position = self.positions[self.indices[face * 3 + vert] as usize];
        [position.x, position.y, position.z]
    }

    fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
        let normal = self.normals[self.indices[face * 3 + vert] as usize];
        [normal.x, normal.y, normal.z]
    }

    fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
        let texcoord = self.texcoords[self.indices[face * 3 + vert] as usize];
        [texcoord.x, 1.0 - texcoord.y]
    }

    fn set_tangent(
        &mut self,
        tangent_space: Option<bevy_mikktspace::TangentSpace>,
        face: usize,
        vert: usize,
    ) {
        let Some(ts) = tangent_space else {
            return;
        };
        let vertex_index = self.indices[face * 3 + vert] as usize;
        if self.written[vertex_index] {
            return;
        }
        let tangent = ts.tangent();
        let sign = if ts.is_orientation_preserving() {
            1.0
        } else {
            -1.0
        };
        self.tangents[vertex_index] = [tangent[0], tangent[1], tangent[2], sign];
        self.written[vertex_index] = true;
    }
}

fn generate_mikktspace_tangents(
    positions: &[Vec3],
    normals: &[Vec3],
    texcoords: &[Vec2],
    indices: &[u32],
) -> Vec<[f32; 4]> {
    let mut mesh = IndexedMesh {
        positions,
        normals,
        texcoords,
        indices,
        tangents: vec![[1.0, 0.0, 0.0, 1.0]; positions.len()],
        written: vec![false; positions.len()],
    };

    if let Err(error) = bevy_mikktspace::generate_tangents(&mut mesh) {
        tracing::warn!(
            "mikktspace tangent generation failed ({:?}); falling back to default tangents",
            error
        );
        return vec![[1.0, 0.0, 0.0, 1.0]; positions.len()];
    }

    mesh.tangents
}
