use crate::renderer::wgpu::pass_sync;
use crate::renderer::wgpu::presentation;
use crate::renderer::wgpu::render_configs::FrameOutputs;
use crate::renderer::wgpu::texture_uploads;
use nightshade_graphics::commands::loading::{
    loading_pipeline_mark_completed, loading_pipeline_poll_decodes, loading_pipeline_pop,
    loading_pipeline_release_recipe_sources, loading_pipeline_requeue_front,
    loading_pipeline_reset_source_arena,
};
use nightshade_graphics::queries::loading::{
    loading_pipeline_is_active, loading_pipeline_recipe_ready,
};

pub fn render_frame(
    renderer: &mut crate::renderer::wgpu::WgpuRenderer,
    hooks: &mut crate::resources::RenderHooks,
    world: &mut nightshade::ecs::World,
) -> Result<(), crate::renderer::wgpu::rendergraph::RenderGraphError> {
    poll_depth_pick_readback(renderer, &world.storage, &mut world.resources);
    drain_loading_tasks(renderer, &mut world.resources);
    super::extract::prepare_text_meshes(renderer, &mut world.storage, &mut world.resources);

    let scene = std::mem::replace(world, nightshade::ecs::World::tracking());
    let mut inputs = super::inputs::compose_render_inputs(scene);
    let mut outputs = FrameOutputs::default();
    let result = crate::renderer::wgpu::frame::render_frame(
        renderer,
        &mut hooks.frame_inputs,
        &mut inputs,
        &mut outputs,
    );
    *world = super::inputs::restore_render_inputs(inputs, outputs);
    world
        .resources
        .res_mut::<nightshade_render::data::config::RendererState>()
        .gpu_frame_time_ms = renderer.timing.milliseconds;
    result
}

pub fn configure_graph(
    renderer: &mut crate::renderer::wgpu::WgpuRenderer,
    hooks: &mut crate::resources::RenderHooks,
) -> Result<(), Box<dyn std::error::Error>> {
    pass_sync::publish_glyph_atlas(renderer);

    hooks.publish_shader_modules(&renderer.shader_composer);

    let surface_format = renderer.surface_format;
    let (graph, build) = renderer.graph_and_pass_build();
    hooks.configure_render_graph(graph, build, surface_format);

    let (graph, build) = renderer.graph_and_pass_build();
    hooks.configure_render_graph_sub_passes(graph, build, surface_format);

    presentation::install_presentation_passes(renderer)?;

    pass_sync::publish_glyph_atlas(renderer);
    Ok(())
}

pub fn update_graph(
    renderer: &mut crate::renderer::wgpu::WgpuRenderer,
    hooks: &mut crate::resources::RenderHooks,
    storage: &nightshade::ecs::Storage,
    resources: &mut nightshade::ecs::ResourceMap,
) -> Result<(), Box<dyn std::error::Error>> {
    let pending_font_loads: Vec<_> = std::mem::take(
        &mut resources
            .res_mut::<nightshade_graphics::resources::loading::LoadingState>()
            .pending_font_loads,
    );
    for pending_load in pending_font_loads {
        if let Some(text_state) =
            resources.get_mut::<nightshade_text::contract::resources::TextState>()
        {
            text_state.font_engine.load_font(pending_load.font_data);
        }
    }

    hooks.update_render_graph(&mut renderer.graph, storage, resources);
    Ok(())
}

pub(crate) fn poll_depth_pick_readback(
    renderer: &mut crate::renderer::wgpu::WgpuRenderer,
    _storage: &nightshade::ecs::Storage,
    resources: &mut nightshade::ecs::ResourceMap,
) {
    let Some(readback) = crate::renderer::wgpu::picking::poll_depth_pick(renderer) else {
        return;
    };

    resources
        .res_mut::<nightshade_render::resources::gpu_picking::GpuPicking>()
        .set_depth_samples(
            readback.depth_values,
            readback.entity_id_values,
            readback.sample_size,
            readback.sample_size,
            readback.center.0,
            readback.center.1,
        );

    if let Some(camera_entity) = readback.camera
        && let Some(matrices) = resources
            .res::<nightshade_render::contract::RenderViews>()
            .view_for(camera_entity)
            .and_then(|view| view.matrices)
    {
        let (texture_width, texture_height) = readback.texture_size;
        let inverse_view_proj = (matrices.projection * matrices.view)
            .try_inverse()
            .unwrap_or_else(nalgebra_glm::Mat4::identity);

        resources
            .res_mut::<nightshade_render::resources::gpu_picking::GpuPicking>()
            .compute_result(
                &inverse_view_proj,
                texture_width as f32,
                texture_height as f32,
            );
    }
}

pub(crate) fn drain_loading_tasks(
    renderer: &mut crate::renderer::wgpu::WgpuRenderer,
    resources: &mut nightshade::ecs::ResourceMap,
) {
    loading_pipeline_poll_decodes(
        &mut resources
            .res_mut::<nightshade_graphics::resources::loading::LoadingState>()
            .pipeline,
        nightshade_graphics::resources::loading::DECODE_POLL_BUDGET,
    );

    let budget = resources
        .res::<nightshade_graphics::resources::loading::LoadingState>()
        .pipeline
        .tasks_per_frame
        .max(1);
    let mut uploaded_textures: Vec<nightshade_render::data::asset_id::TextureId> = Vec::new();
    let mut deferred: Vec<nightshade_graphics::resources::loading::LoadingTask> = Vec::new();
    let mut reserved_any = false;
    let mut uploads = 0usize;
    while uploads < budget {
        let Some(task) = loading_pipeline_pop(
            &mut resources
                .res_mut::<nightshade_graphics::resources::loading::LoadingState>()
                .pipeline,
        ) else {
            break;
        };
        let category = task.category();
        let label = match &task {
            nightshade_graphics::resources::loading::LoadingTask::ReserveLayer {
                texture, ..
            }
            | nightshade_graphics::resources::loading::LoadingTask::UploadDecodedTexture {
                texture,
                ..
            }
            | nightshade_graphics::resources::loading::LoadingTask::MaterializeTexture {
                texture,
                ..
            } => nightshade_render::queries::texture_cache::texture_cache_texture_name(
                resources.res::<nightshade_render::resources::texture_cache::TextureCache>(),
                texture.index,
            )
            .map(str::to_string)
            .unwrap_or_else(|| format!("texture {}", texture.index)),
        };
        match task {
            nightshade_graphics::resources::loading::LoadingTask::ReserveLayer {
                texture,
                usage,
                sampler,
            } => {
                if texture_uploads::reserve_material_layer(
                    renderer,
                    resources.res::<nightshade_render::resources::texture_cache::TextureCache>(),
                    texture,
                    usage,
                    sampler,
                ) {
                    reserved_any = true;
                }
            }
            nightshade_graphics::resources::loading::LoadingTask::UploadDecodedTexture {
                texture,
                image,
                usage,
                sampler,
            } => {
                if image.format.is_compressed() {
                    tracing::error!(
                        "the texture {label} is block compressed, which this backend does not draw"
                    );
                } else {
                    texture_uploads::upload_material_texture(
                        renderer,
                        resources
                            .res_mut::<nightshade_render::resources::texture_cache::TextureCache>(),
                        texture_uploads::MaterialTextureUploadRequest {
                            texture,
                            rgba_data: &image.rgba,
                            width: image.width,
                            height: image.height,
                            usage,
                            sampler,
                        },
                    );
                    uploaded_textures.push(texture);
                }
                uploads += 1;
            }
            nightshade_graphics::resources::loading::LoadingTask::MaterializeTexture {
                texture,
                recipe,
                usage,
                sampler,
            } => {
                if !loading_pipeline_recipe_ready(
                    &resources
                        .res::<nightshade_graphics::resources::loading::LoadingState>()
                        .pipeline,
                    &recipe,
                ) {
                    deferred.push(
                        nightshade_graphics::resources::loading::LoadingTask::MaterializeTexture {
                            texture,
                            recipe,
                            usage,
                            sampler,
                        },
                    );
                    continue;
                }
                let produced = nightshade_graphics::commands::loading::execute_texture_recipe(
                    &recipe,
                    &resources
                        .res::<nightshade_graphics::resources::loading::LoadingState>()
                        .pipeline
                        .decoded_images,
                );
                loading_pipeline_release_recipe_sources(
                    &mut resources
                        .res_mut::<nightshade_graphics::resources::loading::LoadingState>()
                        .pipeline,
                    &recipe,
                );
                if let Some(decoded) = produced.filter(|decoded| {
                    if decoded.format.is_compressed() {
                        tracing::error!(
                            "the texture {label} is block compressed, which this backend does not draw"
                        );
                    }
                    !decoded.format.is_compressed()
                }) {
                    let texture_name =
                        nightshade_render::queries::texture_cache::texture_cache_texture_name(
                            resources
                                .res::<nightshade_render::resources::texture_cache::TextureCache>(),
                            texture.index,
                        )
                        .map(str::to_string);
                    if let Some(name) = texture_name {
                        resources
                            .res_mut::<nightshade_render::resources::texture_sources::TextureSources>()
                            .entry(name)
                            .or_insert_with(|| {
                                nightshade_render::resources::texture_sources::TextureSourceBytes {
                                data: nightshade_render::resources::texture_sources::TextureSourceData::Rgba {
                                    rgba: decoded.rgba.clone(),
                                    width: decoded.width,
                                    height: decoded.height,
                                },
                                usage,
                                sampler,
                            }
                            });
                    }
                    texture_uploads::upload_material_texture(
                        renderer,
                        resources
                            .res_mut::<nightshade_render::resources::texture_cache::TextureCache>(),
                        texture_uploads::MaterialTextureUploadRequest {
                            texture,
                            rgba_data: &decoded.rgba,
                            width: decoded.width,
                            height: decoded.height,
                            usage,
                            sampler,
                        },
                    );
                    uploaded_textures.push(texture);
                } else {
                    tracing::warn!("texture recipe produced no data for '{}'", label);
                }
                uploads += 1;
            }
        }
        loading_pipeline_mark_completed(
            &mut resources
                .res_mut::<nightshade_graphics::resources::loading::LoadingState>()
                .pipeline,
            label,
            category,
        );
    }
    for task in deferred.into_iter().rev() {
        loading_pipeline_requeue_front(
            &mut resources
                .res_mut::<nightshade_graphics::resources::loading::LoadingState>()
                .pipeline,
            task,
        );
    }
    let needs_resolve = reserved_any
        || !resources
            .res::<nightshade_render::resources::material_registry::MaterialRegistry>()
            .pending_resolve
            .is_empty();
    if needs_resolve {
        let texture_cache = std::mem::take(
            resources.res_mut::<nightshade_render::resources::texture_cache::TextureCache>(),
        );
        nightshade_render::commands::material_registry::material_registry_resolve_uploaded_textures(
            resources
                .res_mut::<nightshade_render::resources::material_registry::MaterialRegistry>(),
            &texture_cache,
            &uploaded_textures,
        );
        *resources.res_mut::<nightshade_render::resources::texture_cache::TextureCache>() =
            texture_cache;
        resources
            .res_mut::<nightshade_render::resources::mesh_state::MeshRenderState>()
            .full_rebuild_needed = true;
    }
    if !loading_pipeline_is_active(
        &resources
            .res::<nightshade_graphics::resources::loading::LoadingState>()
            .pipeline,
    ) {
        loading_pipeline_reset_source_arena(
            &mut resources
                .res_mut::<nightshade_graphics::resources::loading::LoadingState>()
                .pipeline,
        );
    }

    let evicted = texture_uploads::evict_unused_textures(
        renderer,
        resources.res_mut::<nightshade_render::resources::texture_cache::TextureCache>(),
    );
    if !evicted.is_empty() {
        for name in &evicted {
            resources
                .res_mut::<nightshade_render::resources::texture_sources::TextureSources>()
                .remove(name);
        }
        resources
            .res_mut::<nightshade_render::resources::mesh_state::MeshRenderState>()
            .full_rebuild_needed = true;
    }
}
