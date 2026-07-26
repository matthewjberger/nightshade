//! Streaming-load pipeline for asset work that must not block the frame.
//!
//! Push tasks via [`loading_pipeline_push`]. The render frame drains a budget
//! of tasks per tick, so any subsystem that calls it automatically gets
//! UI-friendly streaming and shows up in the global progress bar.
//!
//! Source images live in a dense per-batch arena indexed by [`SourceImageId`].
//! Final material textures are identified by name strings at the pipeline
//! boundary (the renderer turns those into typed handles when it registers
//! them with the texture cache).

use std::collections::VecDeque;

mod decoder;

use crate::platform::input::resources::DroppedFile;
use crate::render::asset_id::TextureId;
use crate::render::particles::ParticleTextureUpload;
use crate::render::texture_data::{SamplerSettings, TextureUsage};
use decoder::TextureDecoder;

/// Per-frame budget for GPU texture uploads drained on the render thread.
/// Image decoding runs in parallel off this thread on native, so this only
/// throttles the cheap upload step; wasm decodes inline and keeps it small.
#[cfg(not(target_arch = "wasm32"))]
pub const DEFAULT_TASKS_PER_FRAME: usize = 8;
#[cfg(target_arch = "wasm32")]
pub const DEFAULT_TASKS_PER_FRAME: usize = 2;

/// Maximum decoded images collected from the worker pool per frame. On native
/// this is just moving finished buffers, so it is generous; on wasm each poll
/// decodes inline, so it bounds the per-frame decode cost.
#[cfg(not(target_arch = "wasm32"))]
pub const DECODE_POLL_BUDGET: usize = 256;
#[cfg(target_arch = "wasm32")]
pub const DECODE_POLL_BUDGET: usize = 2;

/// A plan to produce one texture in the loading pipeline: the source-image
/// recipe plus the color space and sampler the result should carry.
pub struct TextureProductionPlan {
    pub name: String,
    pub recipe: TextureRecipe,
    pub usage: crate::render::texture_data::TextureUsage,
    pub sampler: crate::render::texture_data::SamplerSettings,
}

/// A queued font-atlas load: raw font bytes and the size to rasterize at.
pub struct PendingFontLoad {
    pub font_data: Vec<u8>,
    pub font_size: f32,
}

#[derive(Default)]
pub struct LoadingState {
    pub pipeline: LoadingPipeline,
    pub pending_font_loads: Vec<PendingFontLoad>,
    pub pending_particle_textures: Vec<ParticleTextureUpload>,
    pub dropped_files: Vec<DroppedFile>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SourceImageId(pub u32);

pub struct DecodedImage {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub enum TextureRecipe {
    Direct {
        image: SourceImageId,
    },
    PackRg {
        a: Option<SourceImageId>,
        b: Option<SourceImageId>,
    },
    PackRgbA {
        rgb: Option<SourceImageId>,
        alpha: Option<SourceImageId>,
    },
    SpecGlossBase {
        diffuse: Option<SourceImageId>,
        diffuse_factor: [f32; 4],
        spec_gloss: Option<SourceImageId>,
        specular_factor: [f32; 3],
        glossiness_factor: f32,
    },
    SpecGlossMr {
        diffuse: Option<SourceImageId>,
        diffuse_factor: [f32; 4],
        spec_gloss: Option<SourceImageId>,
        specular_factor: [f32; 3],
        glossiness_factor: f32,
    },
}

impl TextureRecipe {
    fn remap_sources(&mut self, remap: &[SourceImageId]) {
        let map = |id: &mut SourceImageId| {
            if let Some(target) = remap.get(id.0 as usize) {
                *id = *target;
            }
        };
        let map_opt = |opt: &mut Option<SourceImageId>| {
            if let Some(id) = opt {
                map(id);
            }
        };
        match self {
            TextureRecipe::Direct { image } => map(image),
            TextureRecipe::PackRg { a, b } => {
                map_opt(a);
                map_opt(b);
            }
            TextureRecipe::PackRgbA { rgb, alpha } => {
                map_opt(rgb);
                map_opt(alpha);
            }
            TextureRecipe::SpecGlossBase {
                diffuse,
                spec_gloss,
                ..
            }
            | TextureRecipe::SpecGlossMr {
                diffuse,
                spec_gloss,
                ..
            } => {
                map_opt(diffuse);
                map_opt(spec_gloss);
            }
        }
    }

    fn for_each_source(&self, mut visit: impl FnMut(SourceImageId)) {
        match self {
            TextureRecipe::Direct { image } => visit(*image),
            TextureRecipe::PackRg { a, b } => {
                if let Some(id) = a {
                    visit(*id);
                }
                if let Some(id) = b {
                    visit(*id);
                }
            }
            TextureRecipe::PackRgbA { rgb, alpha } => {
                if let Some(id) = rgb {
                    visit(*id);
                }
                if let Some(id) = alpha {
                    visit(*id);
                }
            }
            TextureRecipe::SpecGlossBase {
                diffuse,
                spec_gloss,
                ..
            }
            | TextureRecipe::SpecGlossMr {
                diffuse,
                spec_gloss,
                ..
            } => {
                if let Some(id) = diffuse {
                    visit(*id);
                }
                if let Some(id) = spec_gloss {
                    visit(*id);
                }
            }
        }
    }
}

pub enum LoadingTask {
    ReserveLayer {
        texture: TextureId,
        usage: TextureUsage,
        sampler: SamplerSettings,
    },
    UploadDecodedTexture {
        texture: TextureId,
        rgba_data: Vec<u8>,
        width: u32,
        height: u32,
        usage: TextureUsage,
        sampler: SamplerSettings,
    },
    MaterializeTexture {
        texture: TextureId,
        recipe: TextureRecipe,
        usage: TextureUsage,
        sampler: SamplerSettings,
    },
}

impl LoadingTask {
    pub fn category(&self) -> &'static str {
        match self {
            LoadingTask::ReserveLayer { .. } => "texture",
            LoadingTask::UploadDecodedTexture { .. } => "texture",
            LoadingTask::MaterializeTexture { .. } => "texture",
        }
    }
}

#[derive(Default)]
pub struct LoadingPipeline {
    pending: VecDeque<LoadingTask>,
    completed: usize,
    total: usize,
    last_label: Option<String>,
    last_category: Option<&'static str>,
    pub tasks_per_frame: usize,
    next_source_image: u32,
    free_source_indices: Vec<u32>,
    pub decoded_images: Vec<Option<DecodedImage>>,
    decoded_image_refs: Vec<u32>,
    source_ready: Vec<bool>,
    source_pending: Vec<bool>,
    decoder: Option<TextureDecoder>,
    generation: u64,
}

pub fn loading_pipeline_new() -> LoadingPipeline {
    LoadingPipeline {
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

/// Returns a task to the front of the queue so it is retried before any newer
/// work. Used by the drain to defer a `MaterializeTexture` whose source images
/// are still decoding in the worker pool.
pub fn loading_pipeline_requeue_front(pipeline: &mut LoadingPipeline, task: LoadingTask) {
    pipeline.pending.push_front(task);
}

/// Submits an encoded source image to the decode worker pool. The decoded RGBA
/// is collected later by [`loading_pipeline_poll_decodes`].
pub fn loading_pipeline_submit_decode(
    pipeline: &mut LoadingPipeline,
    image: SourceImageId,
    encoded_bytes: Vec<u8>,
) {
    let slot = image.0 as usize;
    loading_pipeline_grow_arena_to(pipeline, slot + 1);
    pipeline.source_ready[slot] = false;
    pipeline.source_pending[slot] = true;
    let generation = pipeline.generation;
    pipeline
        .decoder
        .get_or_insert_with(TextureDecoder::new)
        .submit(image, encoded_bytes, generation);
}

/// Collects up to `budget` finished decodes from the worker pool into the
/// arena, marking each source image ready. Results from a superseded load (an
/// older generation) are discarded.
pub fn loading_pipeline_poll_decodes(pipeline: &mut LoadingPipeline, budget: usize) {
    let generation = pipeline.generation;
    for _ in 0..budget {
        let Some(decoder) = pipeline.decoder.as_mut() else {
            return;
        };
        let Some(result) = decoder.poll() else {
            return;
        };
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

/// Reports whether a recipe may sample this source now: either its decode has
/// arrived, or no decode is outstanding for it. A source that was acquired but
/// never submitted (or whose decode was orphaned by a superseded load) has no
/// outstanding decode and so is resolved to the recipe's neutral default rather
/// than blocking the recipe forever.
fn loading_pipeline_source_ready(pipeline: &LoadingPipeline, id: SourceImageId) -> bool {
    let slot = id.0 as usize;
    if pipeline.source_ready.get(slot).copied().unwrap_or(false) {
        return true;
    }
    !pipeline.source_pending.get(slot).copied().unwrap_or(false)
}

/// Reports whether every source image a recipe samples has finished decoding
/// (successfully or not), so the recipe can be materialized.
pub fn loading_pipeline_recipe_ready(pipeline: &LoadingPipeline, recipe: &TextureRecipe) -> bool {
    let mut ready = true;
    recipe.for_each_source(|id| {
        if !loading_pipeline_source_ready(pipeline, id) {
            ready = false;
        }
    });
    ready
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

pub fn loading_pipeline_pending_len(pipeline: &LoadingPipeline) -> usize {
    pipeline.pending.len()
}

pub fn loading_pipeline_completed(pipeline: &LoadingPipeline) -> usize {
    pipeline.completed
}

pub fn loading_pipeline_total(pipeline: &LoadingPipeline) -> usize {
    pipeline.total
}

pub fn loading_pipeline_fraction(pipeline: &LoadingPipeline) -> f32 {
    if pipeline.total == 0 {
        return 1.0;
    }
    pipeline.completed as f32 / pipeline.total as f32
}

pub fn loading_pipeline_is_active(pipeline: &LoadingPipeline) -> bool {
    !pipeline.pending.is_empty()
}

pub fn loading_pipeline_last_label(pipeline: &LoadingPipeline) -> Option<&str> {
    pipeline.last_label.as_deref()
}

pub fn loading_pipeline_last_category(pipeline: &LoadingPipeline) -> Option<&'static str> {
    pipeline.last_category
}

/// Decodes a PNG/JPEG/etc. byte slice via the `image` crate, queues
/// the resulting RGBA data for GPU upload under `name`, and adds a
/// reference so the texture isn't evicted before materials sample
/// it. Returns the `TextureId` for callers that want to retain it.
/// Game code can call this once per texture at startup instead of
/// reimplementing the decode + queue + ref-count dance.
#[cfg(feature = "assets")]
pub fn load_texture_from_image_bytes(
    world: &mut crate::ecs::world::World,
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
        world,
        name.to_string(),
        rgba.into_raw(),
        width,
        height,
        usage,
        sampler,
    );
    crate::render::wgpu::texture_cache::texture_cache_acquire(
        world.res_mut::<crate::render::wgpu::texture_cache::TextureCache>(),
        crate::render::wgpu::texture_cache::TextureOwner::Named(name.to_string()),
        vec![name.to_string()],
    );
    Some(texture)
}

/// Marks the named texture resident, keeping it alive under a name-owned
/// reference until released. Pair with [`queue_encoded_texture`] or
/// [`queue_decoded_texture`] to register the pixels first.
pub fn acquire_named_texture(world: &mut crate::ecs::world::World, name: &str) {
    crate::render::wgpu::texture_cache::texture_cache_acquire(
        world.res_mut::<crate::render::wgpu::texture_cache::TextureCache>(),
        crate::render::wgpu::texture_cache::TextureOwner::Named(name.to_string()),
        vec![name.to_string()],
    );
}

/// Marks `names` resident under `entity`'s material, so they stay alive while
/// the entity references them and are released when it is despawned.
pub fn acquire_entity_material_textures(
    world: &mut crate::ecs::world::World,
    entity: nightshade_ecs::Entity,
    names: Vec<String>,
) {
    crate::render::wgpu::texture_cache::texture_cache_acquire(
        world.res_mut::<crate::render::wgpu::texture_cache::TextureCache>(),
        crate::render::wgpu::texture_cache::TextureOwner::EntityMaterial(entity),
        names,
    );
}

/// Bulk variant of `load_texture_from_image_bytes` for cases where a
/// game wants to ship a pack of textures together. Each entry is
/// loaded with the same `usage` and `sampler`. Pack textures are
/// referenced by name without reference counting, so every name is
/// protected from the load drain's zero-reference eviction.
#[cfg(feature = "assets")]
pub fn load_texture_pack_from_image_bytes(
    world: &mut crate::ecs::world::World,
    entries: &[(&str, &[u8])],
    usage: TextureUsage,
    sampler: SamplerSettings,
) {
    for (name, bytes) in entries {
        load_texture_from_image_bytes(world, name, bytes, usage, sampler);
        crate::render::wgpu::texture_cache::texture_cache_protect(
            world.res_mut::<crate::render::wgpu::texture_cache::TextureCache>(),
            (*name).to_string(),
        );
    }
}

pub fn queue_decoded_texture(
    world: &mut crate::ecs::world::World,
    name: String,
    rgba_data: Vec<u8>,
    width: u32,
    height: u32,
    usage: TextureUsage,
    sampler: SamplerSettings,
) -> TextureId {
    use crate::assets::texture_source::{TextureSourceBytes, TextureSourceData};

    world
        .res_mut::<crate::assets::texture_source::TextureSources>()
        .insert(
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
    let texture = crate::render::wgpu::texture_cache::texture_cache_reserve_id(
        world.res_mut::<crate::render::wgpu::texture_cache::TextureCache>(),
        name,
    );
    loading_pipeline_push(
        &mut world
            .res_mut::<crate::assets::loading::LoadingState>()
            .pipeline,
        LoadingTask::ReserveLayer {
            texture,
            usage,
            sampler,
        },
    );
    loading_pipeline_push(
        &mut world
            .res_mut::<crate::assets::loading::LoadingState>()
            .pipeline,
        LoadingTask::UploadDecodedTexture {
            texture,
            rgba_data,
            width,
            height,
            usage,
            sampler,
        },
    );
    texture
}

pub fn queue_encoded_texture(
    world: &mut crate::ecs::world::World,
    name: String,
    encoded_bytes: Vec<u8>,
    usage: TextureUsage,
    sampler: SamplerSettings,
) -> TextureId {
    let texture = crate::render::wgpu::texture_cache::texture_cache_reserve_id(
        world.res_mut::<crate::render::wgpu::texture_cache::TextureCache>(),
        name,
    );
    loading_pipeline_push(
        &mut world
            .res_mut::<crate::assets::loading::LoadingState>()
            .pipeline,
        LoadingTask::ReserveLayer {
            texture,
            usage,
            sampler,
        },
    );
    let image = loading_pipeline_allocate_source_image(
        &mut world
            .res_mut::<crate::assets::loading::LoadingState>()
            .pipeline,
    );
    loading_pipeline_submit_decode(
        &mut world
            .res_mut::<crate::assets::loading::LoadingState>()
            .pipeline,
        image,
        encoded_bytes,
    );
    let recipe = TextureRecipe::Direct { image };
    loading_pipeline_acquire_recipe_sources(
        &mut world
            .res_mut::<crate::assets::loading::LoadingState>()
            .pipeline,
        &recipe,
    );
    loading_pipeline_push(
        &mut world
            .res_mut::<crate::assets::loading::LoadingState>()
            .pipeline,
        LoadingTask::MaterializeTexture {
            texture,
            recipe,
            usage,
            sampler,
        },
    );
    texture
}

/// Queues decoded images and meshes for GPU upload.
///
/// The mechanical half of loading a model. Callers holding a glTF import should
/// use [`queue_gltf_load`](crate::assets::prefab::queue_gltf_load), which hands
/// these three pieces over without disassembling the import by hand; this stays
/// public for anyone assembling the pieces from somewhere else.
#[cfg(feature = "assets")]
pub fn queue_texture_and_mesh_upload(
    world: &mut crate::ecs::world::World,
    encoded: Vec<Vec<u8>>,
    plans: Vec<TextureProductionPlan>,
    meshes: std::collections::HashMap<String, crate::render::geometry::Mesh>,
) {
    use crate::assets::texture_source::{TextureSourceBytes, TextureSourceData};
    use crate::render::mesh_cache::mesh_cache_insert;

    for plan in &plans {
        if let TextureRecipe::Direct { image } = &plan.recipe {
            let index = image.0 as usize;
            if let Some(bytes) = encoded.get(index) {
                world
                    .res_mut::<crate::assets::texture_source::TextureSources>()
                    .insert(
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

    let mut remap: Vec<SourceImageId> = Vec::with_capacity(encoded.len());
    for encoded_bytes in encoded {
        let image = loading_pipeline_allocate_source_image(
            &mut world
                .res_mut::<crate::assets::loading::LoadingState>()
                .pipeline,
        );
        remap.push(image);
        loading_pipeline_submit_decode(
            &mut world
                .res_mut::<crate::assets::loading::LoadingState>()
                .pipeline,
            image,
            encoded_bytes,
        );
    }

    for mut plan in plans {
        plan.recipe.remap_sources(&remap);
        let name = plan.name.clone();
        let texture = crate::render::wgpu::texture_cache::texture_cache_reserve_id(
            world.res_mut::<crate::render::wgpu::texture_cache::TextureCache>(),
            plan.name,
        );
        crate::render::wgpu::texture_cache::texture_cache_protect(
            world.res_mut::<crate::render::wgpu::texture_cache::TextureCache>(),
            name,
        );
        loading_pipeline_acquire_recipe_sources(
            &mut world
                .res_mut::<crate::assets::loading::LoadingState>()
                .pipeline,
            &plan.recipe,
        );
        loading_pipeline_push(
            &mut world
                .res_mut::<crate::assets::loading::LoadingState>()
                .pipeline,
            LoadingTask::ReserveLayer {
                texture,
                usage: plan.usage,
                sampler: plan.sampler,
            },
        );
        loading_pipeline_push(
            &mut world
                .res_mut::<crate::assets::loading::LoadingState>()
                .pipeline,
            LoadingTask::MaterializeTexture {
                texture,
                recipe: plan.recipe,
                usage: plan.usage,
                sampler: plan.sampler,
            },
        );
    }

    for (name, mesh) in meshes {
        mesh_cache_insert(
            world.res_mut::<crate::render::mesh_cache::MeshCache>(),
            name,
            mesh,
        );
    }
}

pub fn decode_to_rgba8(encoded_bytes: &[u8]) -> Result<DecodedImage, image::ImageError> {
    let dynamic = image::load_from_memory(encoded_bytes)?;
    let width = dynamic.width();
    let height = dynamic.height();
    let rgba = dynamic.to_rgba8().into_raw();
    Ok(DecodedImage {
        rgba,
        width,
        height,
    })
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
    match recipe {
        TextureRecipe::Direct { image } => lookup(*image).map(clone_decoded),
        TextureRecipe::PackRg { a, b } => Some(pack_rg(a.and_then(lookup), b.and_then(lookup))),
        TextureRecipe::PackRgbA { rgb, alpha } => {
            Some(pack_rgb_a(rgb.and_then(lookup), alpha.and_then(lookup)))
        }
        TextureRecipe::SpecGlossBase {
            diffuse,
            diffuse_factor,
            spec_gloss,
            specular_factor,
            glossiness_factor,
        } => {
            let (base, _, width, height) = convert_spec_gloss(
                diffuse.and_then(lookup),
                *diffuse_factor,
                spec_gloss.and_then(lookup),
                *specular_factor,
                *glossiness_factor,
            );
            Some(DecodedImage {
                rgba: base,
                width,
                height,
            })
        }
        TextureRecipe::SpecGlossMr {
            diffuse,
            diffuse_factor,
            spec_gloss,
            specular_factor,
            glossiness_factor,
        } => {
            let (_, mr, width, height) = convert_spec_gloss(
                diffuse.and_then(lookup),
                *diffuse_factor,
                spec_gloss.and_then(lookup),
                *specular_factor,
                *glossiness_factor,
            );
            Some(DecodedImage {
                rgba: mr,
                width,
                height,
            })
        }
    }
}

fn clone_decoded(source: &DecodedImage) -> DecodedImage {
    DecodedImage {
        rgba: source.rgba.clone(),
        width: source.width,
        height: source.height,
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
    DecodedImage {
        rgba: packed,
        width: target_w,
        height: target_h,
    }
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
    DecodedImage {
        rgba: packed,
        width: target_w,
        height: target_h,
    }
}

fn srgb_byte_to_linear(value: u8) -> f32 {
    let normalized = value as f32 / 255.0;
    if normalized <= 0.04045 {
        normalized / 12.92
    } else {
        ((normalized + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb_byte(value: f32) -> u8 {
    let clamped = value.clamp(0.0, 1.0);
    let encoded = if clamped <= 0.0031308 {
        clamped * 12.92
    } else {
        1.055 * clamped.powf(1.0 / 2.4) - 0.055
    };
    (encoded * 255.0).round().clamp(0.0, 255.0) as u8
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
                crate::ecs::material::spec_gloss::spec_gloss_to_metallic_roughness(
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

/// Queues a font for the renderer to register on the next frame. Returns the
/// slot the glyph cache will use.
pub fn load_font_from_bytes(
    world: &mut crate::ecs::world::World,
    font_data: Vec<u8>,
    font_size: f32,
) -> Result<usize, String> {
    world
        .res_mut::<LoadingState>()
        .pending_font_loads
        .push(PendingFontLoad {
            font_data,
            font_size,
        });
    Ok(0)
}
