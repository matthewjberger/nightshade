pub(crate) mod decoder;

use std::collections::VecDeque;

use nightshade_render::data::asset_id::TextureId;
use nightshade_render::data::decoded_image::DecodedImage;

use nightshade_render::data::texture_data::{SamplerSettings, TextureUsage};

use decoder::TextureDecoder;

use nightshade_platform::contract::DroppedFile;

#[cfg(not(target_arch = "wasm32"))]
pub const DEFAULT_TASKS_PER_FRAME: usize = 8;

#[cfg(target_arch = "wasm32")]
pub const DEFAULT_TASKS_PER_FRAME: usize = 2;

#[cfg(not(target_arch = "wasm32"))]
pub const DECODE_POLL_BUDGET: usize = 256;

#[cfg(target_arch = "wasm32")]
pub const DECODE_POLL_BUDGET: usize = 2;

pub struct TextureProductionPlan {
    pub name: String,
    pub recipe: TextureRecipe,
    pub usage: nightshade_render::data::texture_data::TextureUsage,
    pub sampler: nightshade_render::data::texture_data::SamplerSettings,
}

pub struct PendingFontLoad {
    pub font_data: Vec<u8>,
    pub font_size: f32,
}

#[derive(Default)]
pub struct LoadingState {
    pub pipeline: LoadingPipeline,
    pub pending_font_loads: Vec<PendingFontLoad>,
    pub dropped_files: Vec<DroppedFile>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SourceImageId(pub u32);

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
    pub(crate) fn remap_sources(&mut self, remap: &[SourceImageId]) {
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

    pub(crate) fn for_each_source(&self, mut visit: impl FnMut(SourceImageId)) {
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
        image: DecodedImage,
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
    pub blocks: nightshade_render::data::decoded_image::BlockFamily,
    pub(crate) pending: VecDeque<LoadingTask>,
    pub(crate) completed: usize,
    pub(crate) total: usize,
    pub(crate) last_label: Option<String>,
    pub(crate) last_category: Option<&'static str>,
    pub tasks_per_frame: usize,
    pub(crate) next_source_image: u32,
    pub(crate) free_source_indices: Vec<u32>,
    pub decoded_images: Vec<Option<DecodedImage>>,
    pub(crate) decoded_image_refs: Vec<u32>,
    pub(crate) source_ready: Vec<bool>,
    pub(crate) source_pending: Vec<bool>,
    pub(crate) decoder: Option<TextureDecoder>,
    pub(crate) generation: u64,
    pub(crate) quiet: u32,
}

pub struct TextureLoad<'a> {
    pub sources: &'a mut nightshade_render::resources::texture_sources::TextureSources,
    pub cache: &'a mut nightshade_render::resources::texture_cache::TextureCache,
    pub loading: &'a mut LoadingState,
}

impl<W: nightshade_ecs::ResourceHost> nightshade_ecs::system_param::ExtractParam<W>
    for TextureLoad<'_>
{
    type State = ();
    type Owned = ();
    type Item<'item> = TextureLoad<'item>;

    fn init() -> Self::State {}

    fn resources() -> Vec<nightshade_ecs::system_param::ResourceNeed> {
        use nightshade_ecs::system_param::ResourceNeed;
        vec![
            ResourceNeed::write::<nightshade_render::resources::texture_sources::TextureSources>(),
            ResourceNeed::write::<nightshade_render::resources::texture_cache::TextureCache>(),
            ResourceNeed::write::<LoadingState>(),
        ]
    }

    fn extract(_state: &mut (), _host: &W) {}

    fn build<'a, I: Iterator<Item = Option<nightshade_ecs::ResourceSlot<'a>>>>(
        _owned: &'a mut (),
        slots: &mut I,
        _resources: &'a nightshade_ecs::ResourceMap,
    ) -> TextureLoad<'a> {
        TextureLoad {
            sources: next_slot(slots),
            cache: next_slot(slots),
            loading: next_slot(slots),
        }
    }

    fn apply(_state: &mut (), _owned: (), _host: &W) {}

    fn access() -> nightshade_ecs::SystemAccess {
        use nightshade_ecs::{Access, SystemAccess};
        SystemAccess::empty()
            .with_resource(
                std::any::TypeId::of::<nightshade_render::resources::texture_sources::TextureSources>(),
                Access::Write,
            )
            .with_resource(
                std::any::TypeId::of::<nightshade_render::resources::texture_cache::TextureCache>(),
                Access::Write,
            )
            .with_resource(std::any::TypeId::of::<LoadingState>(), Access::Write)
    }
}

fn next_slot<'a, T: Send + Sync + 'static>(
    slots: &mut impl Iterator<Item = Option<nightshade_ecs::ResourceSlot<'a>>>,
) -> &'a mut T {
    slots
        .next()
        .expect("the runner borrows one slot per resource a load names")
        .expect("a texture load names no resource it can do without")
        .into_mut::<T>()
}

pub fn texture_load(resources: &mut nightshade_ecs::ResourceMap) -> TextureLoad<'_> {
    use std::any::TypeId;
    let [sources, cache, loading] = resources.res_disjoint_mut([
        TypeId::of::<nightshade_render::resources::texture_sources::TextureSources>(),
        TypeId::of::<nightshade_render::resources::texture_cache::TextureCache>(),
        TypeId::of::<LoadingState>(),
    ]);
    TextureLoad {
        sources: sources.downcast_mut().expect("TextureSources"),
        cache: cache.downcast_mut().expect("TextureCache"),
        loading: loading.downcast_mut().expect("LoadingState"),
    }
}

pub fn asset_load(
    resources: &mut nightshade_ecs::ResourceMap,
) -> (
    TextureLoad<'_>,
    &mut nightshade_render::resources::mesh_cache::MeshCache,
) {
    use std::any::TypeId;
    let [sources, cache, loading, meshes] = resources.res_disjoint_mut([
        TypeId::of::<nightshade_render::resources::texture_sources::TextureSources>(),
        TypeId::of::<nightshade_render::resources::texture_cache::TextureCache>(),
        TypeId::of::<LoadingState>(),
        TypeId::of::<nightshade_render::resources::mesh_cache::MeshCache>(),
    ]);
    (
        TextureLoad {
            sources: sources.downcast_mut().expect("TextureSources"),
            cache: cache.downcast_mut().expect("TextureCache"),
            loading: loading.downcast_mut().expect("LoadingState"),
        },
        meshes.downcast_mut().expect("MeshCache"),
    )
}
