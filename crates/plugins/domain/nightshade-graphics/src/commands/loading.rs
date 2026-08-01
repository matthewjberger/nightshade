use nightshade_render::queries::image::{linear_to_srgb_byte, srgb_byte_to_linear};
use std::collections::VecDeque;

use crate::resources::loading::decoder::TextureDecoder;
use crate::resources::loading::*;
use nightshade_render::data::asset_id::TextureId;
use nightshade_render::data::decoded_image::DecodedImage;
use nightshade_render::data::texture_data::{SamplerSettings, TextureUsage};

pub fn loading_pipeline_new() -> LoadingPipeline {
    LoadingPipeline {
        blocks: nightshade_render::data::decoded_image::BlockFamily::default(),
        pending: VecDeque::new(),
        completed: 0,
        total: 0,
        last_label: None,
        last_category: None,
        tasks_per_frame: DEFAULT_TASKS_PER_FRAME,
        next_source_image: 0,
        free_source_indices: Vec::new(),
        decoded_images: Vec::new(),
        decoded_image_refs: Vec::new(),
        source_ready: Vec::new(),
        source_pending: Vec::new(),
        decoder: None,
        generation: 0,
        quiet: 0,
    }
}

pub fn loading_pipeline_push(pipeline: &mut LoadingPipeline, task: LoadingTask) {
    if pipeline.pending.is_empty() && pipeline.completed == pipeline.total {
        pipeline.completed = 0;
        pipeline.total = 0;
    }
    pipeline.total += 1;
    pipeline.pending.push_back(task);
}

pub fn loading_pipeline_pop(pipeline: &mut LoadingPipeline) -> Option<LoadingTask> {
    pipeline.pending.pop_front()
}

pub fn loading_pipeline_requeue_front(pipeline: &mut LoadingPipeline, task: LoadingTask) {
    pipeline.pending.push_front(task);
}

pub fn loading_pipeline_submit_decode(
    pipeline: &mut LoadingPipeline,
    image: SourceImageId,
    encoded_bytes: Vec<u8>,
    as_texels: bool,
) {
    let slot = image.0 as usize;
    loading_pipeline_grow_arena_to(pipeline, slot + 1);
    pipeline.source_ready[slot] = false;
    pipeline.source_pending[slot] = true;
    let generation = pipeline.generation;
    let family = if as_texels {
        nightshade_render::data::decoded_image::BlockFamily::Texels
    } else {
        pipeline.blocks
    };
    pipeline
        .decoder
        .get_or_insert_with(TextureDecoder::new)
        .submit(image, encoded_bytes, family, generation);
}

pub fn loading_pipeline_poll_decodes(pipeline: &mut LoadingPipeline, budget: usize) {
    let generation = pipeline.generation;
    let Some(decoder) = pipeline.decoder.as_mut() else {
        return;
    };
    for result in decoder.take(budget) {
        if result.generation != generation {
            continue;
        }
        let slot = result.image.0 as usize;
        loading_pipeline_grow_arena_to(pipeline, slot + 1);
        pipeline.decoded_images[slot] = result.decoded;
        pipeline.source_ready[slot] = true;
        pipeline.source_pending[slot] = false;
    }
}

pub fn loading_pipeline_mark_completed(
    pipeline: &mut LoadingPipeline,
    label: String,
    category: &'static str,
) {
    pipeline.last_label = Some(label);
    pipeline.last_category = Some(category);
    pipeline.completed += 1;
}

pub fn loading_pipeline_allocate_source_image(pipeline: &mut LoadingPipeline) -> SourceImageId {
    if let Some(reused) = pipeline.free_source_indices.pop() {
        let slot = reused as usize;
        pipeline.decoded_images[slot] = None;
        pipeline.decoded_image_refs[slot] = 0;
        pipeline.source_ready[slot] = false;
        pipeline.source_pending[slot] = false;
        return SourceImageId(reused);
    }
    let id = SourceImageId(pipeline.next_source_image);
    pipeline.next_source_image += 1;
    loading_pipeline_grow_arena_to(pipeline, pipeline.next_source_image as usize);
    id
}

pub fn loading_pipeline_grow_arena_to(pipeline: &mut LoadingPipeline, new_len: usize) {
    if new_len > pipeline.decoded_images.len() {
        pipeline.decoded_images.resize_with(new_len, || None);
        pipeline.decoded_image_refs.resize(new_len, 0);
        pipeline.source_ready.resize(new_len, false);
        pipeline.source_pending.resize(new_len, false);
    }
}

pub fn loading_pipeline_acquire_source(pipeline: &mut LoadingPipeline, id: SourceImageId) {
    let slot = id.0 as usize;
    loading_pipeline_grow_arena_to(pipeline, slot + 1);
    pipeline.decoded_image_refs[slot] += 1;
}

pub fn loading_pipeline_release_source(pipeline: &mut LoadingPipeline, id: SourceImageId) {
    let slot = id.0 as usize;
    let Some(count) = pipeline.decoded_image_refs.get_mut(slot) else {
        return;
    };
    if *count == 0 {
        return;
    }
    *count -= 1;
    if *count == 0 {
        if let Some(entry) = pipeline.decoded_images.get_mut(slot) {
            *entry = None;
        }
        pipeline.free_source_indices.push(id.0);
    }
}

pub fn loading_pipeline_acquire_recipe_sources(
    pipeline: &mut LoadingPipeline,
    recipe: &TextureRecipe,
) {
    recipe.for_each_source(|id| loading_pipeline_acquire_source(pipeline, id));
}

pub fn loading_pipeline_release_recipe_sources(
    pipeline: &mut LoadingPipeline,
    recipe: &TextureRecipe,
) {
    recipe.for_each_source(|id| loading_pipeline_release_source(pipeline, id));
}

pub fn loading_pipeline_reset_source_arena(pipeline: &mut LoadingPipeline) {
    pipeline.generation = pipeline.generation.wrapping_add(1);
    pipeline.next_source_image = 0;
    pipeline.free_source_indices.clear();
    pipeline.free_source_indices.shrink_to_fit();
    pipeline.decoded_images.clear();
    pipeline.decoded_images.shrink_to_fit();
    pipeline.decoded_image_refs.clear();
    pipeline.decoded_image_refs.shrink_to_fit();
    pipeline.source_ready.clear();
    pipeline.source_ready.shrink_to_fit();
    pipeline.source_pending.clear();
    pipeline.source_pending.shrink_to_fit();
}

pub fn load_texture_from_image_bytes(
    load: &mut TextureLoad<'_>,
    name: &str,
    encoded_bytes: &[u8],
    usage: TextureUsage,
    sampler: SamplerSettings,
) -> Option<TextureId> {
    let decoded = match image::load_from_memory(encoded_bytes) {
        Ok(image) => image,
        Err(error) => {
            tracing::error!(?error, name, "failed to decode texture bytes");
            return None;
        }
    };
    let rgba = decoded.to_rgba8();
    let (width, height) = rgba.dimensions();
    let texture = queue_decoded_texture(
        load,
        name.to_string(),
        rgba.into_raw(),
        width,
        height,
        usage,
        sampler,
    );
    nightshade_render::commands::texture_cache::texture_cache_acquire(
        load.cache,
        nightshade_render::resources::texture_cache::TextureOwner::Named(name.to_string()),
        vec![name.to_string()],
    );
    Some(texture)
}

pub fn acquire_named_texture(
    cache: &mut nightshade_render::resources::texture_cache::TextureCache,
    name: &str,
) {
    nightshade_render::commands::texture_cache::texture_cache_acquire(
        cache,
        nightshade_render::resources::texture_cache::TextureOwner::Named(name.to_string()),
        vec![name.to_string()],
    );
}

pub fn acquire_entity_material_textures(
    cache: &mut nightshade_render::resources::texture_cache::TextureCache,
    entity: nightshade_ecs::Entity,
    names: Vec<String>,
) {
    nightshade_render::commands::texture_cache::texture_cache_acquire(
        cache,
        nightshade_render::resources::texture_cache::TextureOwner::EntityMaterial(entity),
        names,
    );
}

pub fn load_texture_pack_from_image_bytes(
    load: &mut TextureLoad<'_>,
    entries: &[(&str, &[u8])],
    usage: TextureUsage,
    sampler: SamplerSettings,
) {
    for (name, bytes) in entries {
        load_texture_from_image_bytes(load, name, bytes, usage, sampler);
        nightshade_render::commands::texture_cache::texture_cache_protect(
            load.cache,
            (*name).to_string(),
        );
    }
}

pub fn queue_decoded_texture(
    load: &mut TextureLoad<'_>,
    name: String,
    rgba_data: Vec<u8>,
    width: u32,
    height: u32,
    usage: TextureUsage,
    sampler: SamplerSettings,
) -> TextureId {
    use nightshade_render::resources::texture_sources::{TextureSourceBytes, TextureSourceData};

    load.sources.insert(
        name.clone(),
        TextureSourceBytes {
            data: TextureSourceData::Rgba {
                rgba: rgba_data.clone(),
                width,
                height,
            },
            usage,
            sampler,
        },
    );
    let texture =
        nightshade_render::commands::texture_cache::texture_cache_reserve_id(load.cache, name);
    loading_pipeline_push(
        &mut load.loading.pipeline,
        LoadingTask::ReserveLayer {
            texture,
            usage,
            sampler,
        },
    );
    loading_pipeline_push(
        &mut load.loading.pipeline,
        LoadingTask::UploadDecodedTexture {
            texture,
            image: DecodedImage::rgba8(rgba_data, width, height),
            usage,
            sampler,
        },
    );
    texture
}

pub fn queue_encoded_texture(
    load: &mut TextureLoad<'_>,
    name: String,
    encoded_bytes: Vec<u8>,
    usage: TextureUsage,
    sampler: SamplerSettings,
) -> TextureId {
    let texture =
        nightshade_render::commands::texture_cache::texture_cache_reserve_id(load.cache, name);
    loading_pipeline_push(
        &mut load.loading.pipeline,
        LoadingTask::ReserveLayer {
            texture,
            usage,
            sampler,
        },
    );
    let image = loading_pipeline_allocate_source_image(&mut load.loading.pipeline);
    loading_pipeline_submit_decode(&mut load.loading.pipeline, image, encoded_bytes, false);
    let recipe = TextureRecipe::Direct { image };
    loading_pipeline_acquire_recipe_sources(&mut load.loading.pipeline, &recipe);
    loading_pipeline_push(
        &mut load.loading.pipeline,
        LoadingTask::MaterializeTexture {
            texture,
            recipe,
            usage,
            sampler,
        },
    );
    texture
}

pub fn queue_texture_and_mesh_upload(
    load: &mut TextureLoad<'_>,
    mesh_cache: &mut nightshade_render::resources::mesh_cache::MeshCache,
    encoded: Vec<Vec<u8>>,
    plans: Vec<TextureProductionPlan>,
    meshes: std::collections::HashMap<String, nightshade_render::data::geometry::Mesh>,
) {
    use nightshade_render::commands::mesh_cache::{mesh_cache_insert, mesh_cache_protect};
    use nightshade_render::data::generational_registry::registry_lookup_index;
    use nightshade_render::resources::texture_sources::{TextureSourceBytes, TextureSourceData};

    let plans: Vec<TextureProductionPlan> = plans
        .into_iter()
        .filter(|plan| registry_lookup_index(&load.cache.registry, &plan.name).is_none())
        .collect();

    for plan in &plans {
        if let TextureRecipe::Direct { image } = &plan.recipe {
            let index = image.0 as usize;
            if let Some(bytes) = encoded.get(index) {
                load.sources.insert(
                    plan.name.clone(),
                    TextureSourceBytes {
                        data: TextureSourceData::Png(bytes.clone()),
                        usage: plan.usage,
                        sampler: plan.sampler,
                    },
                );
            }
        }
    }

    let mut packed = vec![false; encoded.len()];
    let mut wanted = vec![false; encoded.len()];
    for plan in &plans {
        let direct = matches!(plan.recipe, TextureRecipe::Direct { .. });
        plan.recipe.for_each_source(|source| {
            if let Some(slot) = wanted.get_mut(source.0 as usize) {
                *slot = true;
            }
            if !direct && let Some(slot) = packed.get_mut(source.0 as usize) {
                *slot = true;
            }
        });
    }

    let mut remap: Vec<SourceImageId> = Vec::with_capacity(encoded.len());
    for (index, encoded_bytes) in encoded.into_iter().enumerate() {
        let image = loading_pipeline_allocate_source_image(&mut load.loading.pipeline);
        remap.push(image);
        if wanted[index] {
            loading_pipeline_submit_decode(
                &mut load.loading.pipeline,
                image,
                encoded_bytes,
                packed[index],
            );
        } else {
            load.loading.pipeline.free_source_indices.push(image.0);
        }
    }

    for mut plan in plans {
        plan.recipe.remap_sources(&remap);
        let name = plan.name.clone();
        let texture = nightshade_render::commands::texture_cache::texture_cache_reserve_id(
            load.cache, plan.name,
        );
        nightshade_render::commands::texture_cache::texture_cache_protect(load.cache, name);
        loading_pipeline_acquire_recipe_sources(&mut load.loading.pipeline, &plan.recipe);
        loading_pipeline_push(
            &mut load.loading.pipeline,
            LoadingTask::ReserveLayer {
                texture,
                usage: plan.usage,
                sampler: plan.sampler,
            },
        );
        loading_pipeline_push(
            &mut load.loading.pipeline,
            LoadingTask::MaterializeTexture {
                texture,
                recipe: plan.recipe,
                usage: plan.usage,
                sampler: plan.sampler,
            },
        );
    }

    for (name, mesh) in meshes {
        mesh_cache_protect(mesh_cache, name.clone());
        mesh_cache_insert(mesh_cache, name, mesh);
    }
}

pub fn execute_texture_recipe(
    recipe: &TextureRecipe,
    decoded_images: &[Option<DecodedImage>],
) -> Option<DecodedImage> {
    let lookup = |id: SourceImageId| -> Option<&DecodedImage> {
        decoded_images
            .get(id.0 as usize)
            .and_then(|slot| slot.as_ref())
    };
    let flat = |id: SourceImageId| -> Option<&DecodedImage> {
        let image = lookup(id)?;
        if image.format.is_compressed() {
            tracing::error!(
                "a block compressed image cannot be packed with another, so it is left out"
            );
            return None;
        }
        Some(image)
    };
    match recipe {
        TextureRecipe::Direct { image } => lookup(*image).cloned(),
        TextureRecipe::PackRg { a, b } => Some(pack_rg(a.and_then(flat), b.and_then(flat))),
        TextureRecipe::PackRgbA { rgb, alpha } => {
            Some(pack_rgb_a(rgb.and_then(flat), alpha.and_then(flat)))
        }
        TextureRecipe::SpecGlossBase {
            diffuse,
            diffuse_factor,
            spec_gloss,
            specular_factor,
            glossiness_factor,
        } => {
            let (base, _, width, height) = convert_spec_gloss(
                diffuse.and_then(flat),
                *diffuse_factor,
                spec_gloss.and_then(flat),
                *specular_factor,
                *glossiness_factor,
            );
            Some(DecodedImage::rgba8(base, width, height))
        }
        TextureRecipe::SpecGlossMr {
            diffuse,
            diffuse_factor,
            spec_gloss,
            specular_factor,
            glossiness_factor,
        } => {
            let (_, mr, width, height) = convert_spec_gloss(
                diffuse.and_then(flat),
                *diffuse_factor,
                spec_gloss.and_then(flat),
                *specular_factor,
                *glossiness_factor,
            );
            Some(DecodedImage::rgba8(mr, width, height))
        }
    }
}

fn resize_rgba(source: &DecodedImage, target_w: u32, target_h: u32) -> Vec<u8> {
    if source.width == target_w && source.height == target_h {
        return source.rgba.clone();
    }
    let mut out = vec![0u8; (target_w * target_h * 4) as usize];
    for y in 0..target_h {
        let sy = ((y as u64) * (source.height as u64) / (target_h as u64)) as u32;
        for x in 0..target_w {
            let sx = ((x as u64) * (source.width as u64) / (target_w as u64)) as u32;
            let src_offset = ((sy * source.width + sx) * 4) as usize;
            let dst_offset = ((y * target_w + x) * 4) as usize;
            out[dst_offset..dst_offset + 4]
                .copy_from_slice(&source.rgba[src_offset..src_offset + 4]);
        }
    }
    out
}

fn pack_rg(a: Option<&DecodedImage>, b: Option<&DecodedImage>) -> DecodedImage {
    let target_w = a
        .map(|image| image.width)
        .unwrap_or(1)
        .max(b.map(|image| image.width).unwrap_or(1));
    let target_h = a
        .map(|image| image.height)
        .unwrap_or(1)
        .max(b.map(|image| image.height).unwrap_or(1));
    let pixel_count = (target_w * target_h) as usize;
    let mut packed = vec![255u8; pixel_count * 4];
    if let Some(image) = a {
        let resized = resize_rgba(image, target_w, target_h);
        for index in 0..pixel_count {
            packed[index * 4] = resized[index * 4];
        }
    }
    if let Some(image) = b {
        let resized = resize_rgba(image, target_w, target_h);
        for index in 0..pixel_count {
            packed[index * 4 + 1] = resized[index * 4 + 1];
        }
    }
    DecodedImage::rgba8(packed, target_w, target_h)
}

fn pack_rgb_a(rgb: Option<&DecodedImage>, alpha: Option<&DecodedImage>) -> DecodedImage {
    let target_w = rgb
        .map(|image| image.width)
        .unwrap_or(1)
        .max(alpha.map(|image| image.width).unwrap_or(1));
    let target_h = rgb
        .map(|image| image.height)
        .unwrap_or(1)
        .max(alpha.map(|image| image.height).unwrap_or(1));
    let pixel_count = (target_w * target_h) as usize;
    let mut packed = vec![255u8; pixel_count * 4];
    if let Some(image) = rgb {
        let resized = resize_rgba(image, target_w, target_h);
        for index in 0..pixel_count {
            let offset = index * 4;
            packed[offset] = resized[offset];
            packed[offset + 1] = resized[offset + 1];
            packed[offset + 2] = resized[offset + 2];
        }
    }
    if let Some(image) = alpha {
        let resized = resize_rgba(image, target_w, target_h);
        for index in 0..pixel_count {
            packed[index * 4 + 3] = resized[index * 4 + 3];
        }
    }
    DecodedImage::rgba8(packed, target_w, target_h)
}

fn sample_image_linear(
    image: Option<&DecodedImage>,
    x: u32,
    y: u32,
    target_width: u32,
    target_height: u32,
    rgb_is_srgb: bool,
) -> [f32; 4] {
    let Some(image) = image else {
        return [1.0, 1.0, 1.0, 1.0];
    };
    let src_x = ((x as f32 / target_width.max(1) as f32) * image.width as f32) as u32;
    let src_y = ((y as f32 / target_height.max(1) as f32) * image.height as f32) as u32;
    let src_x = src_x.min(image.width.saturating_sub(1));
    let src_y = src_y.min(image.height.saturating_sub(1));
    let pixel_index = ((src_y * image.width + src_x) * 4) as usize;
    if pixel_index + 3 >= image.rgba.len() {
        return [1.0, 1.0, 1.0, 1.0];
    }
    let r = image.rgba[pixel_index];
    let g = image.rgba[pixel_index + 1];
    let b = image.rgba[pixel_index + 2];
    let a = image.rgba[pixel_index + 3];
    let r_lin = if rgb_is_srgb {
        srgb_byte_to_linear(r)
    } else {
        r as f32 / 255.0
    };
    let g_lin = if rgb_is_srgb {
        srgb_byte_to_linear(g)
    } else {
        g as f32 / 255.0
    };
    let b_lin = if rgb_is_srgb {
        srgb_byte_to_linear(b)
    } else {
        b as f32 / 255.0
    };
    [r_lin, g_lin, b_lin, a as f32 / 255.0]
}

fn convert_spec_gloss(
    diffuse_image: Option<&DecodedImage>,
    diffuse_factor: [f32; 4],
    spec_gloss_image: Option<&DecodedImage>,
    specular_factor: [f32; 3],
    glossiness_factor: f32,
) -> (Vec<u8>, Vec<u8>, u32, u32) {
    let width = diffuse_image
        .map(|image| image.width)
        .unwrap_or(1)
        .max(spec_gloss_image.map(|image| image.width).unwrap_or(1));
    let height = diffuse_image
        .map(|image| image.height)
        .unwrap_or(1)
        .max(spec_gloss_image.map(|image| image.height).unwrap_or(1));
    let texel_count = (width as usize) * (height as usize);
    let mut base_rgba = vec![0u8; texel_count * 4];
    let mut mr_rgba = vec![0u8; texel_count * 4];
    for y in 0..height {
        for x in 0..width {
            let dst_idx = ((y * width + x) * 4) as usize;
            let diffuse_sample = sample_image_linear(diffuse_image, x, y, width, height, true);
            let spec_gloss_sample =
                sample_image_linear(spec_gloss_image, x, y, width, height, true);
            let diffuse = [
                diffuse_sample[0] * diffuse_factor[0],
                diffuse_sample[1] * diffuse_factor[1],
                diffuse_sample[2] * diffuse_factor[2],
                diffuse_sample[3] * diffuse_factor[3],
            ];
            let specular = [
                spec_gloss_sample[0] * specular_factor[0],
                spec_gloss_sample[1] * specular_factor[1],
                spec_gloss_sample[2] * specular_factor[2],
            ];
            let glossiness = (spec_gloss_sample[3] * glossiness_factor).clamp(0.0, 1.0);
            let (base_rgb, metallic) =
                nightshade_render::data::spec_gloss::spec_gloss_to_metallic_roughness(
                    diffuse, specular,
                );
            base_rgba[dst_idx] = linear_to_srgb_byte(base_rgb[0]);
            base_rgba[dst_idx + 1] = linear_to_srgb_byte(base_rgb[1]);
            base_rgba[dst_idx + 2] = linear_to_srgb_byte(base_rgb[2]);
            base_rgba[dst_idx + 3] = (diffuse[3].clamp(0.0, 1.0) * 255.0).round() as u8;
            let roughness = 1.0 - glossiness;
            mr_rgba[dst_idx] = 0;
            mr_rgba[dst_idx + 1] = (roughness.clamp(0.0, 1.0) * 255.0).round() as u8;
            mr_rgba[dst_idx + 2] = (metallic.clamp(0.0, 1.0) * 255.0).round() as u8;
            mr_rgba[dst_idx + 3] = 255;
        }
    }
    (base_rgba, mr_rgba, width, height)
}

pub fn load_font_from_bytes(
    loading: &mut LoadingState,
    font_data: Vec<u8>,
    font_size: f32,
) -> Result<usize, String> {
    loading.pending_font_loads.push(PendingFontLoad {
        font_data,
        font_size,
    });
    Ok(0)
}
