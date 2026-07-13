use crate::ecs::loading::{
    loading_pipeline_is_active, loading_pipeline_mark_completed, loading_pipeline_poll_decodes,
    loading_pipeline_pop, loading_pipeline_recipe_ready, loading_pipeline_release_recipe_sources,
    loading_pipeline_requeue_front, loading_pipeline_reset_source_arena,
};
use crate::render::generational_registry::registry_name_for;
use crate::render::wgpu::DEPTH_PICK_SAMPLE_SIZE;
use crate::render::wgpu::pass_sync;
use crate::render::wgpu::presentation;
use crate::render::wgpu::render_configs::FrameOutputs;
use crate::render::wgpu::texture_uploads;

/// Renders one frame: the pre-frame steps that need both the world and the
/// renderer run first, then the frame inputs are composed, the renderer's
/// frame driver renders from them alone, and the outputs are restored into
/// the world.
pub fn render_frame(
    renderer: &mut crate::render::wgpu::WgpuRenderer,
    world: &mut crate::ecs::world::World,
) -> Result<(), Box<dyn std::error::Error>> {
    poll_depth_pick_readback(renderer, world);
    drain_loading_tasks(renderer, world);
    super::extract::prepare_text_meshes(renderer, world);
    #[cfg(feature = "egui")]
    super::egui::prepare_egui_pass(renderer, world);

    let mut inputs = super::inputs::compose_render_inputs(world);
    let mut outputs = FrameOutputs::default();
    let result = crate::render::wgpu::frame::render_frame(renderer, &mut inputs, &mut outputs);
    super::inputs::restore_render_inputs(world, inputs, outputs);
    result
}

pub fn configure_with_state(
    renderer: &mut crate::render::wgpu::WgpuRenderer,
    state: &mut dyn crate::run::State,
) -> Result<(), Box<dyn std::error::Error>> {
    pass_sync::rebind_glyph_atlas(renderer);

    state.configure_render_graph(
        &mut renderer.graph,
        &renderer.device,
        renderer.surface_format,
        crate::run::RenderResources {
            scene_color: renderer.targets.scene_color,
            depth: renderer.targets.depth,
            compute_output: renderer.targets.compute_output,
            swapchain: renderer.targets.swapchain,
            view_normals: renderer.targets.view_normals,
            velocity: renderer.targets.velocity,
            ssao_raw: renderer.targets.ssao_raw,
            ssao: renderer.targets.ssao,
            ssgi_raw: renderer.targets.ssgi_raw,
            ssgi: renderer.targets.ssgi,
            ssr_raw: renderer.targets.ssr_raw,
            ssr: renderer.targets.ssr,
            surface_width: renderer.surface_config.width,
            surface_height: renderer.surface_config.height,
        },
    );

    presentation::install_presentation_passes(renderer)
}

pub fn update_with_state(
    renderer: &mut crate::render::wgpu::WgpuRenderer,
    state: &mut dyn crate::run::State,
    world: &mut crate::ecs::world::World,
) -> Result<(), Box<dyn std::error::Error>> {
    for pending_load in world.resources.loading.pending_font_loads.drain(..) {
        world
            .resources
            .text
            .font_engine
            .load_font(pending_load.font_data);
    }

    state.update_render_graph(&mut renderer.graph, world);
    sync_cloth_write_targets(renderer, world);
    Ok(())
}

/// Gathers each cloth's mesh name and widened cull bounds from the world and
/// hands the cloth pass its vertex buffer write targets for this frame.
pub(crate) fn sync_cloth_write_targets(
    renderer: &mut crate::render::wgpu::WgpuRenderer,
    world: &crate::ecs::world::World,
) {
    let mut cloths: Vec<pass_sync::ClothWriteBounds> = Vec::new();
    for (_entity, (cloth, render_mesh)) in world
        .query_ref::<(
            &crate::ecs::cloth::components::Cloth,
            &crate::ecs::mesh::components::RenderMesh,
        )>()
        .iter()
    {
        let bounds = crate::ecs::cloth::systems::cloth_bounding_volume(cloth);
        cloths.push(pass_sync::ClothWriteBounds {
            mesh_name: render_mesh.name.clone(),
            center: [
                bounds.obb.center.x,
                bounds.obb.center.y,
                bounds.obb.center.z,
            ],
            sphere_radius: bounds.sphere_radius,
        });
    }
    pass_sync::set_cloth_write_targets(renderer, &cloths);
}

/// Polls a pending GPU pick readback and, once the staging buffer is mapped,
/// stores the depth and entity-id samples on the picking resource and
/// resolves the picked world position through the requesting camera.
pub(crate) fn poll_depth_pick_readback(
    renderer: &mut crate::render::wgpu::WgpuRenderer,
    world: &mut crate::ecs::world::World,
) {
    if !renderer.depth_pick.pending {
        return;
    }
    let _ = renderer.device.poll(wgpu::PollType::Poll);

    if !renderer
        .depth_pick
        .map_complete
        .load(std::sync::atomic::Ordering::Relaxed)
    {
        return;
    }
    let buffer_slice = renderer.depth_pick.staging_buffer.slice(..);
    let data = buffer_slice.get_mapped_range();
    let mut depth_values = Vec::new();
    let mut entity_id_values = Vec::new();
    for chunk in data.chunks_exact(8) {
        let depth = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        let entity_id = u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
        depth_values.push(depth);
        entity_id_values.push(entity_id);
    }
    drop(data);
    renderer.depth_pick.staging_buffer.unmap();

    world.resources.gpu_picking.set_depth_samples(
        depth_values,
        entity_id_values,
        DEPTH_PICK_SAMPLE_SIZE,
        DEPTH_PICK_SAMPLE_SIZE,
        renderer.depth_pick.center.0,
        renderer.depth_pick.center.1,
    );

    if let Some(camera_entity) = renderer.depth_pick.camera
        && let Some(matrices) =
            crate::ecs::camera::queries::query_camera_matrices(world, camera_entity)
    {
        let (texture_width, texture_height) = renderer.depth_pick.texture_size;
        let inverse_view_proj = (matrices.projection * matrices.view)
            .try_inverse()
            .unwrap_or_else(nalgebra_glm::Mat4::identity);

        world.resources.gpu_picking.compute_result(
            &inverse_view_proj,
            texture_width as f32,
            texture_height as f32,
        );
    }

    renderer.depth_pick.pending = false;
    renderer
        .depth_pick
        .map_complete
        .store(false, std::sync::atomic::Ordering::Relaxed);
}

pub(crate) fn drain_loading_tasks(
    renderer: &mut crate::render::wgpu::WgpuRenderer,
    world: &mut crate::ecs::world::World,
) {
    // Collect images the worker pool finished decoding. The GPU upload below
    // is the only throttled step; decoding already ran off this thread.
    loading_pipeline_poll_decodes(
        &mut world.resources.loading.pipeline,
        crate::ecs::loading::DECODE_POLL_BUDGET,
    );

    let budget = world.resources.loading.pipeline.tasks_per_frame.max(1);
    let mut uploaded_textures: Vec<crate::render::asset_id::TextureId> = Vec::new();
    let mut deferred: Vec<crate::ecs::loading::LoadingTask> = Vec::new();
    let mut reserved_any = false;
    let mut uploads = 0usize;
    while uploads < budget {
        let Some(task) = loading_pipeline_pop(&mut world.resources.loading.pipeline) else {
            break;
        };
        let category = task.category();
        let label = match &task {
            crate::ecs::loading::LoadingTask::ReserveLayer { texture, .. }
            | crate::ecs::loading::LoadingTask::UploadDecodedTexture { texture, .. }
            | crate::ecs::loading::LoadingTask::MaterializeTexture { texture, .. } => {
                registry_name_for(&world.resources.texture_cache.registry, texture.index)
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("texture {}", texture.index))
            }
        };
        match task {
            crate::ecs::loading::LoadingTask::ReserveLayer {
                texture,
                usage,
                sampler,
            } => {
                // Reserving a stable layer and writing its placeholder is cheap and
                // must happen before geometry first renders, so it does not draw from
                // the per-frame upload budget.
                if texture_uploads::reserve_material_layer(
                    renderer,
                    &world.resources.texture_cache,
                    texture,
                    usage,
                    sampler,
                ) {
                    reserved_any = true;
                }
            }
            crate::ecs::loading::LoadingTask::UploadDecodedTexture {
                texture,
                rgba_data,
                width,
                height,
                usage,
                sampler,
            } => {
                texture_uploads::upload_material_texture(
                    renderer,
                    &mut world.resources.texture_cache,
                    texture_uploads::MaterialTextureUploadRequest {
                        texture,
                        rgba_data: &rgba_data,
                        width,
                        height,
                        usage,
                        sampler,
                    },
                );
                uploaded_textures.push(texture);
                uploads += 1;
            }
            crate::ecs::loading::LoadingTask::MaterializeTexture {
                texture,
                recipe,
                usage,
                sampler,
            } => {
                // Hold the task until every source image it samples has finished
                // decoding in the worker pool, then upload into the reserved layer.
                if !loading_pipeline_recipe_ready(&world.resources.loading.pipeline, &recipe) {
                    deferred.push(crate::ecs::loading::LoadingTask::MaterializeTexture {
                        texture,
                        recipe,
                        usage,
                        sampler,
                    });
                    continue;
                }
                let produced = crate::ecs::loading::execute_texture_recipe(
                    &recipe,
                    &world.resources.loading.pipeline.decoded_images,
                );
                loading_pipeline_release_recipe_sources(
                    &mut world.resources.loading.pipeline,
                    &recipe,
                );
                if let Some(decoded) = produced {
                    // Capture the materialized RGBA into texture_sources for any
                    // texture whose recipe is composite (RG-pack, RGB+A pack,
                    // spec/gloss conversion). Direct-recipe textures already have
                    // their PNG bytes captured at queue_gltf_load and are skipped
                    // here via the `entry`-style insert.
                    let texture_name = world
                        .resources
                        .texture_cache
                        .registry
                        .index_to_name
                        .get(texture.index as usize)
                        .and_then(|slot| slot.clone());
                    if let Some(name) = texture_name {
                        world
                            .resources
                            .assets
                            .texture_sources
                            .entry(name)
                            .or_insert_with(|| crate::ecs::asset_state::TextureSourceBytes {
                                data: crate::ecs::asset_state::TextureSourceData::Rgba {
                                    rgba: decoded.rgba.clone(),
                                    width: decoded.width,
                                    height: decoded.height,
                                },
                                usage,
                                sampler,
                            });
                    }
                    texture_uploads::upload_material_texture(
                        renderer,
                        &mut world.resources.texture_cache,
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
        loading_pipeline_mark_completed(&mut world.resources.loading.pipeline, label, category);
    }
    for task in deferred.into_iter().rev() {
        loading_pipeline_requeue_front(&mut world.resources.loading.pipeline, task);
    }
    // Stable layers mean a streamed pixel upload never changes a material's layer
    // index, so only a fresh reservation (or a freshly inserted material) needs the
    // resolve plus instance rebuild.
    let needs_resolve = reserved_any
        || !world
            .resources
            .assets
            .material_registry
            .pending_resolve
            .is_empty();
    if needs_resolve {
        crate::ecs::material::resources::material_registry_resolve_uploaded_textures(
            &mut world.resources.assets.material_registry,
            &world.resources.texture_cache,
            &uploaded_textures,
        );
        world.resources.mesh_render_state.request_full_rebuild();
    }
    if !loading_pipeline_is_active(&world.resources.loading.pipeline) {
        loading_pipeline_reset_source_arena(&mut world.resources.loading.pipeline);
    }

    let evicted = crate::render::wgpu::texture_cache::texture_cache_remove_unused(
        &mut world.resources.texture_cache,
    );
    if !evicted.is_empty() {
        texture_uploads::unregister_evicted_textures(renderer, &evicted);
        for name in &evicted {
            world.resources.assets.texture_sources.remove(name);
        }
        world.resources.mesh_render_state.request_full_rebuild();
    }
}
