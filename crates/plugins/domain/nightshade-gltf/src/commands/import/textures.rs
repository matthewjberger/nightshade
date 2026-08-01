use super::*;

pub(super) fn sampler_settings_from_gltf(sampler: &gltf::texture::Sampler) -> SamplerSettings {
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

pub(super) fn texture_transform_from_info(info: &gltf::texture::Info) -> TextureTransform {
    let base_uv = info.tex_coord();
    match info.texture_transform() {
        Some(t) => TextureTransform {
            offset: t.offset(),
            rotation: t.rotation(),
            scale: t.scale(),
            uv_set: t.tex_coord().unwrap_or(base_uv),
            drift: [0.0, 0.0],
        },
        None => TextureTransform {
            offset: [0.0, 0.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
            uv_set: base_uv,
            drift: [0.0, 0.0],
        },
    }
}

pub(super) fn texture_transform_from_extension_value(
    extension: Option<&serde_json::Value>,
    base_uv: u32,
) -> TextureTransform {
    let Some(transform) = extension else {
        return TextureTransform {
            offset: [0.0, 0.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
            uv_set: base_uv,
            drift: [0.0, 0.0],
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
        drift: [0.0, 0.0],
    }
}

pub(super) fn texture_transform_from_normal(
    info: &gltf::material::NormalTexture,
) -> TextureTransform {
    texture_transform_from_extension_value(
        info.extension_value("KHR_texture_transform"),
        info.tex_coord(),
    )
}

pub(super) fn texture_transform_from_occlusion(
    info: &gltf::material::OcclusionTexture,
) -> TextureTransform {
    texture_transform_from_extension_value(
        info.extension_value("KHR_texture_transform"),
        info.tex_coord(),
    )
}

pub(super) fn pack_textures_rg(
    source_a_name: Option<&str>,
    source_b_name: Option<&str>,
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
        "packed::{}_{:x}_{}",
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

pub(super) fn pack_textures_rgb_a(
    source_rgb_name: Option<&str>,
    source_a_name: Option<&str>,
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
        "packed::{}_{:x}_{}",
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

pub(super) fn direct_image_source(
    name: &str,
    texture_specs: &TextureSpecMap,
) -> Option<SourceImageId> {
    match texture_specs.get(name)?.recipe {
        TextureRecipe::Direct { image } => Some(image),
        _ => None,
    }
}

pub(super) fn register_texture(
    texture: gltf::Texture<'_>,
    usage: TextureUsage,
    image_hashes: &[u64],
    image_index_to_id: &HashMap<usize, SourceImageId>,
    texture_specs: &mut TextureSpecMap,
) -> Option<String> {
    let image_index = basisu_source(&texture).or_else(|| Some(texture.source()?.index()))?;
    let image = *image_index_to_id.get(&image_index)?;
    let image_hash = *image_hashes.get(image.0 as usize)?;
    let usage_suffix = match usage {
        TextureUsage::Color => "_srgb",
        TextureUsage::Linear => "_linear",
    };
    let sampler_settings = sampler_settings_from_gltf(&texture.sampler());
    let name = format!(
        "image_{:016x}{}_{}",
        image_hash,
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

pub(super) fn texture_transform_from_json(
    value: &serde_json::Value,
    base_uv: u32,
) -> TextureTransform {
    let ext = value
        .get("extensions")
        .and_then(|extensions| extensions.get("KHR_texture_transform"));
    let Some(ext) = ext else {
        return TextureTransform {
            offset: [0.0, 0.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
            uv_set: base_uv,
            drift: [0.0, 0.0],
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
        drift: [0.0, 0.0],
    }
}

pub(super) fn resolve_extension_texture(
    info: &serde_json::Value,
    doc: &Document,
    usage: TextureUsage,
    image_hashes: &[u64],
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
        image_hashes,
        image_index_to_id,
        texture_specs,
    )?;
    let transform = texture_transform_from_json(info, base_uv);
    Some((name, transform))
}

fn basisu_source(texture: &gltf::Texture<'_>) -> Option<usize> {
    let source = texture
        .extension_value("KHR_texture_basisu")?
        .get("source")?;
    source.as_u64().map(|index| index as usize)
}
