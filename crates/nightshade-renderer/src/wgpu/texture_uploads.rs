//! Material texture upload, reload, and eviction. Each operation writes the
//! texture cache and material texture arrays, then fans the new views out to
//! every pass that binds them (mesh, skinned mesh, decal).

use crate::asset_id::TextureId;
use crate::generational_registry::{registry_entry_by_name, registry_name_for};
use crate::texture_data::{SamplerSettings, TextureUsage};
use crate::wgpu::WgpuRenderer;
use crate::wgpu::material_texture_arrays::{MaterialTextureLayer, MaterialTextureUpload};
use crate::wgpu::texture_cache::{
    TextureCache, TextureReloadResult, TextureUploadRequest, TextureUploadSpec,
    texture_cache_load_from_raw_rgba_with_format, texture_cache_reload,
};

/// Inputs for uploading one material texture into the cache and texture arrays.
pub struct MaterialTextureUploadRequest<'a> {
    /// Target texture id in the cache registry.
    pub texture: TextureId,
    /// Tightly packed 8-bit RGBA source pixels.
    pub rgba_data: &'a [u8],
    /// Source width in pixels.
    pub width: u32,
    /// Source height in pixels.
    pub height: u32,
    /// How the texture is sampled, selecting its GPU format.
    pub usage: TextureUsage,
    /// Sampler wrap and filtering settings.
    pub sampler: SamplerSettings,
}

/// Uploads a material texture and fans its views out to every pass that binds it.
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
        &renderer.device,
        &renderer.queue,
        &renderer.mip_generator,
        TextureUploadRequest {
            name: name.clone(),
            rgba_data,
            dimensions: (width, height),
            spec: TextureUploadSpec {
                format: usage.wgpu_format(),
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
        },
    );

    if let Some(mesh_pass) = crate::wgpu::pass_access::mesh_pass_mut(&mut renderer.graph)
        && let Some(texture_entry) = registry_entry_by_name(&texture_cache.registry, &name)
    {
        mesh_pass.register_texture_with_data(
            name.clone(),
            texture_entry.view.clone(),
            texture_entry.sampler.clone(),
        );
        if let Some(layer) = layer_index {
            mesh_pass.add_material_layer_mapping(
                texture,
                MaterialTextureLayer {
                    usage,
                    layer,
                    wrap_u: sampler.wrap_u,
                    wrap_v: sampler.wrap_v,
                },
            );
        }
    }

    #[cfg(feature = "meshlet")]
    if let Some(meshlet_pass) = crate::wgpu::pass_access::meshlet_pass_mut(&mut renderer.graph)
        && let Some(layer) = layer_index
    {
        meshlet_pass.add_material_layer_mapping(
            texture,
            MaterialTextureLayer {
                usage,
                layer,
                wrap_u: sampler.wrap_u,
                wrap_v: sampler.wrap_v,
            },
        );
    }

    if let Some(skinned_mesh_pass) =
        crate::wgpu::pass_access::skinned_mesh_pass_mut(&mut renderer.graph)
        && let Some(texture_entry) = registry_entry_by_name(&texture_cache.registry, &name)
    {
        skinned_mesh_pass.register_texture(
            name.clone(),
            texture_entry.view.clone(),
            texture_entry.sampler.clone(),
        );
        if let Some(layer) = layer_index {
            skinned_mesh_pass.add_material_layer_mapping(
                texture,
                MaterialTextureLayer {
                    usage,
                    layer,
                    wrap_u: sampler.wrap_u,
                    wrap_v: sampler.wrap_v,
                },
            );
        }
    }

    if let Some(decal_pass) = crate::wgpu::pass_access::decal_pass_mut(&mut renderer.graph)
        && let Some(texture_entry) = registry_entry_by_name(&texture_cache.registry, &name)
    {
        decal_pass.register_texture(
            name,
            texture_entry.view.clone(),
            texture_entry.sampler.clone(),
        );
    }
}

/// Reserves a material texture-array layer without uploading pixels, returning whether it succeeded.
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
    ) else {
        return false;
    };
    let mapping = MaterialTextureLayer {
        usage,
        layer,
        wrap_u: sampler.wrap_u,
        wrap_v: sampler.wrap_v,
    };
    if let Some(mesh_pass) = crate::wgpu::pass_access::mesh_pass_mut(&mut renderer.graph) {
        mesh_pass.add_material_layer_mapping(texture, mapping);
    }
    if let Some(skinned_mesh_pass) =
        crate::wgpu::pass_access::skinned_mesh_pass_mut(&mut renderer.graph)
    {
        skinned_mesh_pass.add_material_layer_mapping(texture, mapping);
    }
    true
}

/// Replaces an existing texture's pixels and refreshes the passes that bind it.
pub fn reload_material_texture(
    renderer: &mut WgpuRenderer,
    texture_cache: &mut TextureCache,
    name: String,
    rgba_data: &[u8],
    width: u32,
    height: u32,
) {
    let result = texture_cache_reload(
        texture_cache,
        &renderer.device,
        &renderer.queue,
        &name,
        rgba_data,
        width,
        height,
    );
    if !matches!(result, TextureReloadResult::Recreated) {
        return;
    }
    let usage_opt = registry_entry_by_name(&texture_cache.registry, &name).map(|entry| match entry
        .texture
        .format()
    {
        wgpu::TextureFormat::Rgba8UnormSrgb | wgpu::TextureFormat::Bgra8UnormSrgb => {
            TextureUsage::Color
        }
        _ => TextureUsage::Linear,
    });
    if let Some(usage) = usage_opt {
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
                wrap_u: crate::texture_data::SamplerWrap::Repeat,
                wrap_v: crate::texture_data::SamplerWrap::Repeat,
            },
        );
    }
    if let Some(mesh_pass) = crate::wgpu::pass_access::mesh_pass_mut(&mut renderer.graph)
        && let Some(texture_entry) = registry_entry_by_name(&texture_cache.registry, &name)
    {
        mesh_pass.register_texture_with_data(
            name.clone(),
            texture_entry.view.clone(),
            texture_entry.sampler.clone(),
        );
    }

    if let Some(skinned_mesh_pass) =
        crate::wgpu::pass_access::skinned_mesh_pass_mut(&mut renderer.graph)
        && let Some(texture_entry) = registry_entry_by_name(&texture_cache.registry, &name)
    {
        skinned_mesh_pass.register_texture(
            name.clone(),
            texture_entry.view.clone(),
            texture_entry.sampler.clone(),
        );
    }

    if let Some(decal_pass) = crate::wgpu::pass_access::decal_pass_mut(&mut renderer.graph)
        && let Some(texture_entry) = registry_entry_by_name(&texture_cache.registry, &name)
    {
        decal_pass.register_texture(
            name,
            texture_entry.view.clone(),
            texture_entry.sampler.clone(),
        );
    }
}

/// Releases evicted textures from the arrays and every pass that referenced them.
pub fn unregister_evicted_textures(renderer: &mut WgpuRenderer, evicted: &[String]) {
    for name in evicted {
        renderer.material_texture_arrays.release(name);
    }
    if let Some(mesh_pass) = crate::wgpu::pass_access::mesh_pass_mut(&mut renderer.graph) {
        for name in evicted {
            mesh_pass.unregister_texture(name);
        }
    }
    if let Some(skinned_mesh_pass) =
        crate::wgpu::pass_access::skinned_mesh_pass_mut(&mut renderer.graph)
    {
        for name in evicted {
            skinned_mesh_pass.unregister_texture(name);
        }
    }
    if let Some(decal_pass) = crate::wgpu::pass_access::decal_pass_mut(&mut renderer.graph) {
        for name in evicted {
            decal_pass.unregister_texture(name);
        }
    }
}

/// Applies the requested anisotropy level and, when the sampler or the
/// bindless table changed, rebinds the material texture arrays on the mesh
/// and skinned mesh passes.
pub fn sync_material_texture_bindings(renderer: &mut WgpuRenderer, requested_anisotropy: u16) {
    let anisotropy_changed = renderer
        .material_texture_arrays
        .set_anisotropy(&renderer.device, requested_anisotropy);
    if !(renderer.material_texture_arrays.take_bindless_dirty() || anisotropy_changed) {
        return;
    }
    if let Some(mesh_pass) = crate::wgpu::pass_access::mesh_pass_mut(&mut renderer.graph) {
        mesh_pass.apply_material_textures(&renderer.device, &renderer.material_texture_arrays);
    }
    if let Some(skinned) = crate::wgpu::pass_access::skinned_mesh_pass_mut(&mut renderer.graph) {
        skinned.apply_material_textures(&renderer.device, &renderer.material_texture_arrays);
    }
    #[cfg(feature = "meshlet")]
    if let Some(meshlet) = crate::wgpu::pass_access::meshlet_pass_mut(&mut renderer.graph) {
        meshlet.apply_material_textures(&renderer.device, &renderer.material_texture_arrays);
    }
}
