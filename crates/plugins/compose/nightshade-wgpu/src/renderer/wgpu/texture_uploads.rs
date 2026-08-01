use crate::renderer::wgpu::WgpuRenderer;
use crate::renderer::wgpu::material_texture_arrays::{MaterialTextureLayer, MaterialTextureUpload};
use crate::renderer::wgpu::texture_cache::{
    TextureCache, TextureReloadResult, TextureUploadRequest, TextureUploadSpec,
    texture_cache_load_from_raw_rgba_with_format, texture_cache_reload, texture_store_get,
    texture_store_remove,
};
use nightshade_render::data::asset_id::TextureId;
use nightshade_render::data::generational_registry::registry_name_for;
use nightshade_render::data::texture_data::{SamplerSettings, TextureUsage};

pub struct MaterialTextureUploadRequest<'a> {
    pub texture: TextureId,
    pub rgba_data: &'a [u8],
    pub width: u32,
    pub height: u32,
    pub usage: TextureUsage,
    pub sampler: SamplerSettings,
}

pub fn upload_material_texture(
    renderer: &mut WgpuRenderer,
    texture_cache: &mut TextureCache,
    request: MaterialTextureUploadRequest<'_>,
) {
    let MaterialTextureUploadRequest {
        texture,
        rgba_data,
        width,
        height,
        usage,
        sampler,
    } = request;
    let Some(name) = registry_name_for(&texture_cache.registry, texture.index).map(str::to_string)
    else {
        tracing::error!(
            "upload_material_texture: no name registered for texture id {}",
            texture.index
        );
        return;
    };
    if let Err(e) = texture_cache_load_from_raw_rgba_with_format(
        texture_cache,
        &mut renderer.texture_store,
        &renderer.device,
        &renderer.queue,
        &renderer.mip_generator,
        TextureUploadRequest {
            name: name.clone(),
            rgba_data,
            dimensions: (width, height),
            spec: TextureUploadSpec {
                format: crate::renderer::wgpu::texture_cache::texture_usage_format(usage),
                sampler,
            },
        },
    ) {
        tracing::error!("Failed to load texture: {}", e);
        return;
    }

    let layer_index = renderer.material_texture_arrays.upload(
        &renderer.device,
        &renderer.queue,
        &renderer.mip_generator,
        MaterialTextureUpload {
            name: name.clone(),
            rgba_data,
            width,
            height,
            usage,
            wrap_u: sampler.wrap_u,
            wrap_v: sampler.wrap_v,
            filter: sampler.mag_filter,
        },
    );

    let mapping = layer_index.map(|layer| {
        (
            texture,
            MaterialTextureLayer {
                usage,
                layer,
                wrap_u: sampler.wrap_u,
                wrap_v: sampler.wrap_v,
                filter: sampler.mag_filter,
            },
        )
    });
    announce_register(renderer, &name, mapping);
}

pub fn reserve_material_layer(
    renderer: &mut WgpuRenderer,
    texture_cache: &TextureCache,
    texture: TextureId,
    usage: TextureUsage,
    sampler: SamplerSettings,
) -> bool {
    let Some(name) = registry_name_for(&texture_cache.registry, texture.index).map(str::to_string)
    else {
        return false;
    };
    let Some(layer) = renderer.material_texture_arrays.reserve_layer(
        &renderer.queue,
        name,
        usage,
        sampler.wrap_u,
        sampler.wrap_v,
        sampler.mag_filter,
    ) else {
        return false;
    };
    let mapping = MaterialTextureLayer {
        usage,
        layer,
        wrap_u: sampler.wrap_u,
        wrap_v: sampler.wrap_v,
        filter: sampler.mag_filter,
    };
    announce_mapping(renderer, (texture, mapping));
    true
}

pub fn reload_material_texture(
    renderer: &mut WgpuRenderer,
    texture_cache: &TextureCache,
    name: String,
    rgba_data: &[u8],
    width: u32,
    height: u32,
) {
    let result = texture_cache_reload(
        texture_cache,
        &mut renderer.texture_store,
        &renderer.device,
        &renderer.queue,
        &name,
        rgba_data,
        (width, height),
    );
    if !matches!(result, TextureReloadResult::Recreated) {
        return;
    }
    let usage_opt = texture_store_get(&renderer.texture_store, &name).map(|entry| {
        match entry.texture.format() {
            wgpu::TextureFormat::Rgba8UnormSrgb | wgpu::TextureFormat::Bgra8UnormSrgb => {
                TextureUsage::Color
            }
            _ => TextureUsage::Linear,
        }
    });
    if let Some(usage) = usage_opt {
        let (wrap_u, wrap_v, filter) = renderer
            .material_texture_arrays
            .get_layer(&name)
            .map(|layer| (layer.wrap_u, layer.wrap_v, layer.filter))
            .unwrap_or((
                nightshade_render::data::texture_data::SamplerWrap::Repeat,
                nightshade_render::data::texture_data::SamplerWrap::Repeat,
                nightshade_render::data::texture_data::SamplerFilter::Linear,
            ));
        renderer.material_texture_arrays.upload(
            &renderer.device,
            &renderer.queue,
            &renderer.mip_generator,
            MaterialTextureUpload {
                name: name.clone(),
                rgba_data,
                width,
                height,
                usage,
                wrap_u,
                wrap_v,
                filter,
            },
        );
    }
    announce_register(renderer, &name, None);
}

pub fn unregister_evicted_textures(renderer: &mut WgpuRenderer, evicted: &[String]) {
    for name in evicted {
        renderer.material_texture_arrays.release(name);
    }
    for name in evicted {
        announce_unregister(renderer, name);
    }
}

pub fn evict_unused_textures(
    renderer: &mut WgpuRenderer,
    cache: &mut nightshade_render::resources::texture_cache::TextureCache,
) -> Vec<String> {
    let evicted: Vec<String> =
        nightshade_render::commands::texture_cache::texture_cache_remove_unused(cache)
            .into_iter()
            .map(|(name, _slot)| name)
            .collect();
    if !evicted.is_empty() {
        unregister_evicted_textures(renderer, &evicted);
        for name in &evicted {
            texture_store_remove(&mut renderer.texture_store, name);
        }
    }
    evicted
}

pub fn sync_material_texture_bindings(renderer: &mut WgpuRenderer, requested_anisotropy: u16) {
    let anisotropy_changed = renderer
        .material_texture_arrays
        .set_anisotropy(&renderer.device, requested_anisotropy);
    if !(renderer.material_texture_arrays.take_bindless_dirty() || anisotropy_changed) {
        return;
    }
    announce_rebind(renderer);
}

fn announce_register(
    renderer: &mut WgpuRenderer,
    name: &str,
    mapping: Option<(TextureId, MaterialTextureLayer)>,
) {
    let Some(entry) = texture_store_get(&renderer.texture_store, name) else {
        return;
    };
    let view = entry.view.clone();
    let sampler = entry.sampler.clone();
    let device = renderer.device.clone();
    let update = crate::renderer::wgpu::rendergraph::MaterialUpdate::Register {
        name,
        view: &view,
        sampler: &sampler,
        mapping: mapping.as_ref().map(|pair| pair as &dyn std::any::Any),
    };
    crate::renderer::wgpu::rendergraph::render_graph_material_update(
        &mut renderer.graph,
        &device,
        &update,
    );
}

fn announce_mapping(renderer: &mut WgpuRenderer, mapping: (TextureId, MaterialTextureLayer)) {
    let device = renderer.device.clone();
    let update = crate::renderer::wgpu::rendergraph::MaterialUpdate::Mapping {
        mapping: &mapping as &dyn std::any::Any,
    };
    crate::renderer::wgpu::rendergraph::render_graph_material_update(
        &mut renderer.graph,
        &device,
        &update,
    );
}

fn announce_unregister(renderer: &mut WgpuRenderer, name: &str) {
    let device = renderer.device.clone();
    let update = crate::renderer::wgpu::rendergraph::MaterialUpdate::Unregister { name };
    crate::renderer::wgpu::rendergraph::render_graph_material_update(
        &mut renderer.graph,
        &device,
        &update,
    );
}

fn announce_rebind(renderer: &mut WgpuRenderer) {
    let device = renderer.device.clone();
    let WgpuRenderer {
        graph,
        material_texture_arrays,
        ..
    } = renderer;
    let update = crate::renderer::wgpu::rendergraph::MaterialUpdate::Rebind {
        arrays: material_texture_arrays as &dyn std::any::Any,
    };
    crate::renderer::wgpu::rendergraph::render_graph_material_update(graph, &device, &update);
}
