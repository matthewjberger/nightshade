use crate::assets::loading::{
    loading_pipeline_is_active, loading_pipeline_mark_completed, loading_pipeline_poll_decodes,
    loading_pipeline_pop, loading_pipeline_recipe_ready, loading_pipeline_release_recipe_sources,
    loading_pipeline_requeue_front, loading_pipeline_reset_source_arena,
};
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
) -> Result<(), crate::render::wgpu::rendergraph::RenderGraphError> {
    poll_depth_pick_readback(renderer, world);
    drain_loading_tasks(renderer, world);
    super::extract::prepare_text_meshes(renderer, world);

    let mut inputs = super::inputs::compose_render_inputs(world);
    let mut outputs = FrameOutputs::default();
    let result = crate::render::wgpu::frame::render_frame(renderer, &mut inputs, &mut outputs);
    super::inputs::restore_render_inputs(world, inputs, outputs);
    result
}

pub fn configure_with_state(
    renderer: &mut crate::render::wgpu::WgpuRenderer,
    state: &mut dyn crate::state::State,
) -> Result<(), Box<dyn std::error::Error>> {
    pass_sync::rebind_glyph_atlas(renderer);

    state.configure_render_graph(
        &mut renderer.graph,
        &renderer.device,
        renderer.surface_format,
        crate::state::RenderResources {
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
    state: &mut dyn crate::state::State,
    world: &mut crate::ecs::world::World,
) -> Result<(), Box<dyn std::error::Error>> {
    let pending_font_loads: Vec<_> = world
        .res_mut::<crate::assets::loading::LoadingState>()
        .pending_font_loads
        .drain(..)
        .collect();
    for pending_load in pending_font_loads {
        world
            .res_mut::<crate::ecs::text::resources::TextState>()
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
    let Some(readback) = crate::render::wgpu::picking::poll_depth_pick(renderer) else {
        return;
    };

    world
        .res_mut::<crate::ecs::gpu_picking::GpuPicking>()
        .set_depth_samples(
            readback.depth_values,
            readback.entity_id_values,
            readback.sample_size,
            readback.sample_size,
            readback.center.0,
            readback.center.1,
        );

    if let Some(camera_entity) = readback.camera
        && let Some(matrices) =
            crate::ecs::camera::queries::query_camera_matrices(world, camera_entity)
    {
        let (texture_width, texture_height) = readback.texture_size;
        let inverse_view_proj = (matrices.projection * matrices.view)
            .try_inverse()
            .unwrap_or_else(nalgebra_glm::Mat4::identity);

        world
            .res_mut::<crate::ecs::gpu_picking::GpuPicking>()
            .compute_result(
                &inverse_view_proj,
                texture_width as f32,
                texture_height as f32,
            );
    }
}

pub(crate) fn drain_loading_tasks(
    renderer: &mut crate::render::wgpu::WgpuRenderer,
    world: &mut crate::ecs::world::World,
) {
    // Collect images the worker pool finished decoding. The GPU upload below
    // is the only throttled step; decoding already ran off this thread.
    loading_pipeline_poll_decodes(
        &mut world
            .res_mut::<crate::assets::loading::LoadingState>()
            .pipeline,
        crate::assets::loading::DECODE_POLL_BUDGET,
    );

    let budget = world
        .res::<crate::assets::loading::LoadingState>()
        .pipeline
        .tasks_per_frame
        .max(1);
    let mut uploaded_textures: Vec<crate::render::asset_id::TextureId> = Vec::new();
    let mut deferred: Vec<crate::assets::loading::LoadingTask> = Vec::new();
    let mut reserved_any = false;
    let mut uploads = 0usize;
    while uploads < budget {
        let Some(task) = loading_pipeline_pop(
            &mut world
                .res_mut::<crate::assets::loading::LoadingState>()
                .pipeline,
        ) else {
            break;
        };
        let category = task.category();
        let label = match &task {
            crate::assets::loading::LoadingTask::ReserveLayer { texture, .. }
            | crate::assets::loading::LoadingTask::UploadDecodedTexture { texture, .. }
            | crate::assets::loading::LoadingTask::MaterializeTexture { texture, .. } => {
                crate::render::wgpu::texture_cache::texture_cache_texture_name(
                    world.res::<crate::render::wgpu::texture_cache::TextureCache>(),
                    texture.index,
                )
                .map(str::to_string)
                .unwrap_or_else(|| format!("texture {}", texture.index))
            }
        };
        match task {
            crate::assets::loading::LoadingTask::ReserveLayer {
                texture,
                usage,
                sampler,
            } => {
                // Reserving a stable layer and writing its placeholder is cheap and
                // must happen before geometry first renders, so it does not draw from
                // the per-frame upload budget.
                if texture_uploads::reserve_material_layer(
                    renderer,
                    world.res::<crate::render::wgpu::texture_cache::TextureCache>(),
                    texture,
                    usage,
                    sampler,
                ) {
                    reserved_any = true;
                }
            }
            crate::assets::loading::LoadingTask::UploadDecodedTexture {
                texture,
                rgba_data,
                width,
                height,
                usage,
                sampler,
            } => {
                texture_uploads::upload_material_texture(
                    renderer,
                    world.res_mut::<crate::render::wgpu::texture_cache::TextureCache>(),
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
            crate::assets::loading::LoadingTask::MaterializeTexture {
                texture,
                recipe,
                usage,
                sampler,
            } => {
                // Hold the task until every source image it samples has finished
                // decoding in the worker pool, then upload into the reserved layer.
                if !loading_pipeline_recipe_ready(
                    &world.res::<crate::assets::loading::LoadingState>().pipeline,
                    &recipe,
                ) {
                    deferred.push(crate::assets::loading::LoadingTask::MaterializeTexture {
                        texture,
                        recipe,
                        usage,
                        sampler,
                    });
                    continue;
                }
                let produced = crate::assets::loading::execute_texture_recipe(
                    &recipe,
                    &world
                        .res::<crate::assets::loading::LoadingState>()
                        .pipeline
                        .decoded_images,
                );
                loading_pipeline_release_recipe_sources(
                    &mut world
                        .res_mut::<crate::assets::loading::LoadingState>()
                        .pipeline,
                    &recipe,
                );
                if let Some(decoded) = produced {
                    // Capture the materialized RGBA into texture_sources for any
                    // texture whose recipe is composite (RG-pack, RGB+A pack,
                    // spec/gloss conversion). Direct-recipe textures already have
                    // their PNG bytes captured at queue_gltf_load and are skipped
                    // here via the `entry`-style insert.
                    let texture_name =
                        crate::render::wgpu::texture_cache::texture_cache_texture_name(
                            world.res::<crate::render::wgpu::texture_cache::TextureCache>(),
                            texture.index,
                        )
                        .map(str::to_string);
                    if let Some(name) = texture_name {
                        world
                            .res_mut::<crate::assets::texture_source::TextureSources>()
                            .entry(name)
                            .or_insert_with(|| crate::assets::texture_source::TextureSourceBytes {
                                data: crate::assets::texture_source::TextureSourceData::Rgba {
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
                        world.res_mut::<crate::render::wgpu::texture_cache::TextureCache>(),
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
            &mut world
                .res_mut::<crate::assets::loading::LoadingState>()
                .pipeline,
            label,
            category,
        );
    }
    for task in deferred.into_iter().rev() {
        loading_pipeline_requeue_front(
            &mut world
                .res_mut::<crate::assets::loading::LoadingState>()
                .pipeline,
            task,
        );
    }
    // Stable layers mean a streamed pixel upload never changes a material's layer
    // index, so only a fresh reservation (or a freshly inserted material) needs the
    // resolve plus instance rebuild.
    let needs_resolve = reserved_any
        || !world
            .res::<crate::ecs::material::resources::MaterialRegistry>()
            .pending_resolve
            .is_empty();
    if needs_resolve {
        let texture_cache =
            std::mem::take(world.res_mut::<crate::render::wgpu::texture_cache::TextureCache>());
        crate::ecs::material::resources::material_registry_resolve_uploaded_textures(
            world.res_mut::<crate::ecs::material::resources::MaterialRegistry>(),
            &texture_cache,
            &uploaded_textures,
        );
        *world.res_mut::<crate::render::wgpu::texture_cache::TextureCache>() = texture_cache;
        world
            .res_mut::<crate::render::mesh_state::MeshRenderState>()
            .request_full_rebuild();
    }
    if !loading_pipeline_is_active(&world.res::<crate::assets::loading::LoadingState>().pipeline) {
        loading_pipeline_reset_source_arena(
            &mut world
                .res_mut::<crate::assets::loading::LoadingState>()
                .pipeline,
        );
    }

    let evicted = texture_uploads::evict_unused_textures(
        renderer,
        world.res_mut::<crate::render::wgpu::texture_cache::TextureCache>(),
    );
    if !evicted.is_empty() {
        for name in &evicted {
            world
                .res_mut::<crate::assets::texture_source::TextureSources>()
                .remove(name);
        }
        world
            .res_mut::<crate::render::mesh_state::MeshRenderState>()
            .request_full_rebuild();
    }
}
