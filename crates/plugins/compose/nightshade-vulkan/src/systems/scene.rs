use crate::pre_render::Renderer;
use crate::renderer::capture::CaptureRequest;
use crate::renderer::constraints;
use crate::renderer::environment::load_environment;
use crate::renderer::scene::copies::{CopyBatch, CopyRows};
use crate::renderer::scene::geometry::{
    GeometryArena, flush_ranges, index_buffer, mesh_bounds, mesh_range, mesh_slot, meshlet_buffer,
    meshlet_count, meshlet_triangle_buffer, meshlet_vertex_buffer, morph_buffer, range_buffer,
    release_mesh, reserve_pose, reset_arena, retire_spans, skin_offset, skin_vertex_buffer,
    upload_mesh, vertex_buffer,
};
use crate::renderer::scene::materials::upload_materials;
use crate::renderer::scene::objects::{
    FLAG_DEBUG_VIEW_EXEMPT, FLAG_DOUBLE_SIDED, FLAG_OVERLAY, FLAG_SHADOW_CASTER, FLAG_SKINNED,
    FLAG_VISIBLE, GpuObject, NO_MATERIAL, SOLO_RUN, flips_winding, matrix_columns,
    normal_transform, run_of,
};
use crate::renderer::scene::pass::LOCAL_SHADOW_COUNT;
use crate::renderer::scene::pass::{
    GpuAreaLight, GpuFrame, GpuInstanceBatch, GpuLocalShadow, GpuSkinInstance, MAX_LIGHTS,
    SCENE_PASS, SceneData, SceneHandover, ScenePass, TASK_BATCH, create_pass, hand_over,
    posed_runs,
};
use crate::renderer::scene::probes::MAX_COUNTS as PROBE_MAX_COUNTS;
use crate::renderer::scene::shadow::{
    ATLAS_SIZE, CASCADE_COUNT, LOCAL_ATLAS_SIZE, LOCAL_SLOT_NARROWEST, LOCAL_SLOT_WIDEST, SLOT_SIZE,
};
use crate::renderer::scene::textures::{SceneTextures, texture_sampler, upload_texture};
use crate::renderer::{VulkanRenderer, install_scene_pass, pass_setup};
use nightshade::prelude::*;
use nightshade_graphics::commands::loading::{
    execute_texture_recipe, loading_pipeline_mark_completed, loading_pipeline_poll_decodes,
    loading_pipeline_pop, loading_pipeline_release_recipe_sources, loading_pipeline_requeue_front,
};
use nightshade_graphics::data::queues::RenderCommand;
use nightshade_graphics::queries::clusters::clusters_of;
use nightshade_graphics::queries::loading::loading_pipeline_recipe_ready;
use nightshade_graphics::resources::clusters::ClusterCache;
use nightshade_graphics::resources::loading::{DECODE_POLL_BUDGET, LoadingState, LoadingTask};
use nightshade_graphics::resources::queues::CommandQueues;
use nightshade_render::commands::material_registry::material_registry_resolve_uploaded_textures;
use nightshade_render::commands::mesh_cache::{mesh_cache_mark_dirty, mesh_cache_take_dirty};
use nightshade_render::commands::mesh_state::mesh_state_take_frame;
use nightshade_render::components::light::RenderLightData;
use nightshade_render::components::render_world::{
    CullingMask, Layer, MaterialId, MeshName, MorphWeights, ShadowCaster, Transform, Visible,
};
use nightshade_render::data::config::{
    EffectiveShading, InstancedObjectData, RenderBounds, RenderLighting, RenderSettings,
    RenderView, RendererState,
};
use nightshade_render::data::geometry::{
    create_cone_mesh, create_cube_mesh, create_cylinder_mesh, create_plane_mesh, create_quad_mesh,
    create_sphere_mesh, create_subdivided_plane_mesh, create_torus_mesh,
};
use nightshade_render::data::light_gpu::{AreaLightData, COOKIE_LAYER_NONE, LightData};
use nightshade_render::queries::lights::{
    cascade_texel_sizes, screen_coverage, shadows_enabled, wanted_slot,
};
use nightshade_render::queries::mesh_cache::{mesh_cache_holds, mesh_cache_iter};
use nightshade_render::queries::probes::{
    probe_counts, probe_origin, probe_rotation, probe_settings, probe_spacing,
};
use nightshade_render::queries::projection::{full_frustum_planes, reverse_z_perspective};
use nightshade_render::queries::texture::pack_texture_slot;
use nightshade_render::queries::texture_cache::{
    texture_cache_texture_id, texture_cache_texture_name,
};
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_render::resources::mesh_cache::MeshCache;
use nightshade_render::resources::mesh_state::MeshRenderState;
use nightshade_render::resources::texture_cache::TextureCache;

type ObjectRow<'item> = (
    &'item Transform,
    &'item MeshName,
    &'item MaterialId,
    &'item Visible,
    &'item CullingMask,
    &'item Layer,
    Option<&'item RenderBounds>,
);

struct ObjectSource {
    entity: Entity,
    transform: nalgebra_glm::Mat4,
    mesh_name: String,
    material_id: u32,
    visible: bool,
    culling_mask: u32,
    layer: u32,
    bounds: Option<[f32; 4]>,
}

struct SkinnedEntity {
    instance: GpuSkinInstance,
    mesh_slot: u32,
}

pub fn install(renderer: &mut VulkanRenderer) {
    let texture_layout = renderer.textures.table.layout;
    let environment_layout = renderer.environment.scene_layout;
    let setup = pass_setup(renderer);
    match create_pass(
        &setup,
        texture_layout,
        &renderer.shadows,
        environment_layout,
        &renderer.air,
        &renderer.probes,
    ) {
        Ok(pass) => install_scene_pass(renderer, Box::new(pass)),
        Err(error) => tracing::error!("the scene pass was not installed: {error}"),
    }
}

pub fn drain_commands(
    mut renderer: Renderer,
    mut queues: ResMut<CommandQueues>,
    textures: Res<TextureCache>,
) {
    let commands = std::mem::take(&mut queues.render);
    for command in commands {
        match command {
            RenderCommand::CaptureScreenshot {
                path,
                max_dimension,
            } => {
                renderer.capture = Some(CaptureRequest {
                    path,
                    max_dimension,
                });
            }
            RenderCommand::LoadHdrSkybox { hdr_data } => {
                load_skybox(&mut renderer, &hdr_data);
                renderer.kept.sky = Some(crate::renderer::SkySource::Bytes(hdr_data));
            }
            RenderCommand::RemakeRenderer => renderer.context.mark_lost(),
            RenderCommand::SetColorLut { data } => hand_over_color_lut(&mut renderer, &data),
            RenderCommand::ReloadTexture {
                name,
                rgba_data,
                width,
                height,
            } => put_new_pixels_in(&mut renderer, &textures, &name, &rgba_data, width, height),
            RenderCommand::LoadHdrSkyboxFromPath { path } => match std::fs::read(&path) {
                Ok(bytes) => {
                    load_skybox(&mut renderer, &bytes);
                    renderer.kept.sky = Some(crate::renderer::SkySource::Path(path));
                }
                Err(error) => {
                    tracing::error!("the skybox at {} was not read: {error}", path.display())
                }
            },
        }
    }
}

fn put_new_pixels_in(
    renderer: &mut VulkanRenderer,
    textures: &TextureCache,
    name: &str,
    pixels: &[u8],
    width: u32,
    height: u32,
) {
    let Some(id) = texture_cache_texture_id(textures, name) else {
        return;
    };
    let VulkanRenderer {
        context,
        textures: table,
        ..
    } = renderer;
    if let Err(error) = crate::renderer::image_table::reload_image(
        &mut table.table,
        context,
        id.index,
        width,
        height,
        pixels,
    ) {
        tracing::error!("the texture {name} was not put on the device again: {error}");
    }
}

pub fn put_back_what_was_kept(renderer: &mut VulkanRenderer, kept: crate::renderer::Kept) {
    match &kept.sky {
        Some(crate::renderer::SkySource::Bytes(bytes)) => load_skybox(renderer, bytes),
        Some(crate::renderer::SkySource::Path(path)) => match std::fs::read(path) {
            Ok(bytes) => load_skybox(renderer, &bytes),
            Err(error) => {
                tracing::error!(
                    "the skybox at {} was not read again: {error}",
                    path.display()
                )
            }
        },
        None => {}
    }
    renderer.kept.sky = kept.sky;
    if let Some(table) = kept.color_lut {
        hand_over_color_lut(renderer, &table);
    }
}

fn hand_over_color_lut(renderer: &mut VulkanRenderer, table: &[u8]) {
    renderer.kept.color_lut = Some(table.to_vec());
    let context = &renderer.context;
    let Some(pass) = renderer
        .passes
        .iter()
        .find(|(_, pass)| pass.name() == crate::renderer::post::POST_PASS)
        .map(|(_, pass)| pass.as_ref())
    else {
        return;
    };
    let Some(pass) = (pass as &dyn std::any::Any).downcast_ref() else {
        return;
    };
    if let Err(error) = crate::renderer::post::set_color_lut(pass, context, table) {
        tracing::error!("the color lookup table was not put on the device: {error}");
    }
}

fn load_skybox(renderer: &mut VulkanRenderer, bytes: &[u8]) {
    let VulkanRenderer {
        context,
        environment,
        ..
    } = renderer;
    if let Err(error) = load_environment(context, environment, bytes) {
        tracing::error!("the environment was not built: {error}");
    }
}

pub fn upload_geometry(
    mut renderer: Renderer,
    mut meshes: ResMut<MeshCache>,
    clusters: Res<ClusterCache>,
) {
    let VulkanRenderer {
        context,
        geometry,
        rays,
        ..
    } = &mut *renderer;

    crate::renderer::rays::take_a_turn(rays);
    if geometry.clear_generation != meshes.clear_generation {
        reset_arena(geometry, meshes.clear_generation);
        crate::renderer::rays::forget_meshes(rays);
    }
    retire_spans(geometry);
    seed_primitives(geometry, context);

    let dirty = mesh_cache_take_dirty(&mut meshes);
    if dirty.is_empty() {
        return;
    }

    let mut uploaded = Vec::new();
    let mut waiting = Vec::new();
    let mut released = Vec::new();
    for (name, mesh) in mesh_cache_iter(&meshes) {
        if !dirty.contains(name) {
            continue;
        }
        let Some(built) = clusters_of(&clusters, name) else {
            waiting.push(name.clone());
            continue;
        };
        match upload_mesh(geometry, context, name, mesh, Some(built)) {
            Ok(slot) => uploaded.push(slot),
            Err(error) => tracing::error!("the mesh {name} was not uploaded: {error}"),
        }
    }
    for name in &dirty {
        if mesh_cache_holds(&meshes, name) {
            continue;
        }
        if let Some(slot) = release_mesh(geometry, name) {
            released.push(slot);
        }
    }
    for name in waiting {
        mesh_cache_mark_dirty(&mut meshes, name);
    }

    if let Err(error) = flush_ranges(geometry, context) {
        tracing::error!("the mesh ranges were not written: {error}");
    }

    for slot in released {
        crate::renderer::rays::forget_mesh(rays, slot);
    }
    for slot in uploaded {
        build_mesh_to_trace(rays, context, geometry, slot);
    }
}

fn traced_sun_reach(
    context: &crate::renderer::context::Context,
    anything: bool,
    lighting: &RenderLighting,
    shading: &EffectiveShading,
    view: &RenderView,
) -> f32 {
    if crate::renderer::rays::traces(context)
        && shading.sun_shadow_traced
        && anything
        && lighting.shadows_enabled > 0.0
    {
        view.z_far.max(1.0)
    } else {
        0.0
    }
}

fn build_world_to_trace(
    rays: &mut crate::renderer::rays::Rays,
    context: &crate::renderer::context::Context,
    shadows: &mut crate::renderer::scene::shadow::ShadowAtlas,
    placements: &[crate::renderer::rays::Placement],
    copies: &[crate::renderer::rays::Placement],
    copies_signature: u64,
    first_copy: u32,
) {
    if !crate::renderer::rays::traces(context) {
        return;
    }
    match crate::renderer::rays::build_world(
        rays,
        context,
        placements,
        copies,
        copies_signature,
        first_copy,
    ) {
        Ok(true) => {
            if let Some(world) = crate::renderer::rays::world_handle(rays) {
                crate::renderer::scene::shadow::point_at_world(shadows, world);
            }
        }
        Ok(false) => {}
        Err(error) => tracing::error!("the world was not made ready to trace: {error}"),
    }
}

fn ready_to_trace(
    rays: &mut crate::renderer::rays::Rays,
    context: &crate::renderer::context::Context,
    geometry: &GeometryArena,
    slot: u32,
) {
    if !crate::renderer::rays::traces(context) || crate::renderer::rays::mesh_is_ready(rays, slot) {
        return;
    }
    build_mesh_to_trace(rays, context, geometry, slot);
}

fn build_mesh_to_trace(
    rays: &mut crate::renderer::rays::Rays,
    context: &crate::renderer::context::Context,
    geometry: &GeometryArena,
    slot: u32,
) {
    if !crate::renderer::rays::traces(context) {
        return;
    }
    let Some(range) = mesh_range(geometry, slot) else {
        return;
    };
    let stride = std::mem::size_of::<nightshade_render::data::geometry::Vertex>() as u64;
    let geometry_of_mesh = crate::renderer::rays::MeshGeometry {
        vertices: range.vertex_offset as u64 * stride,
        vertex_count: range.vertex_count,
        vertex_stride: stride,
        indices: range.index_offset as u64 * 4,
        index_count: range.index_count,
    };
    if let Err(error) = crate::renderer::rays::build_mesh(rays, context, slot, &geometry_of_mesh) {
        tracing::error!("the mesh was not made ready to trace: {error}");
    }
}

pub fn upload_textures(
    mut renderer: Renderer,
    mut loading: ResMut<LoadingState>,
    mut registry: ResMut<MaterialRegistry>,
    mut mesh_state: ResMut<MeshRenderState>,
    mut textures: ResMut<TextureCache>,
    views: Res<nightshade_render::data::config::RenderViews>,
) {
    let shading = views
        .active
        .as_ref()
        .map(|camera| camera.effective_shading)
        .unwrap_or(views.fallback_shading);
    loading.pipeline.blocks = if renderer.context.compression.bc {
        nightshade_render::data::decoded_image::BlockFamily::Bc
    } else if renderer.context.compression.astc {
        nightshade_render::data::decoded_image::BlockFamily::Astc
    } else {
        nightshade_render::data::decoded_image::BlockFamily::Texels
    };
    let wanted = shading.material_anisotropy_filtering as f32;
    let VulkanRenderer {
        context,
        textures: table,
        ..
    } = &mut *renderer;
    if let Err(error) =
        crate::renderer::image_table::set_anisotropy(&mut table.table, context, wanted)
    {
        tracing::error!("the samplers were not built again: {error}");
    }

    loading_pipeline_poll_decodes(&mut loading.pipeline, DECODE_POLL_BUDGET);

    let budget = loading.pipeline.tasks_per_frame.max(1);
    let mut deferred = Vec::new();
    let mut uploaded = Vec::new();
    let mut reserved_any = false;
    let mut uploads = 0;

    while uploads < budget {
        let Some(task) = loading_pipeline_pop(&mut loading.pipeline) else {
            break;
        };
        let category = task.category();
        let label = match &task {
            LoadingTask::ReserveLayer { texture, .. }
            | LoadingTask::UploadDecodedTexture { texture, .. }
            | LoadingTask::MaterializeTexture { texture, .. } => {
                texture_cache_texture_name(&textures, texture.index)
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("texture {}", texture.index))
            }
        };
        match task {
            LoadingTask::ReserveLayer { .. } => reserved_any = true,
            LoadingTask::UploadDecodedTexture {
                texture,
                image,
                usage,
                sampler,
            } => {
                let VulkanRenderer {
                    context, textures, ..
                } = &mut *renderer;
                if let Err(error) =
                    upload_texture(textures, context, texture.index, &image, usage, sampler)
                {
                    tracing::error!("a scene texture was not uploaded: {error}");
                }
                uploaded.push(texture);
                uploads += 1;
            }
            LoadingTask::MaterializeTexture {
                texture,
                recipe,
                usage,
                sampler,
            } => {
                if !loading_pipeline_recipe_ready(&loading.pipeline, &recipe) {
                    deferred.push(LoadingTask::MaterializeTexture {
                        texture,
                        recipe,
                        usage,
                        sampler,
                    });
                    continue;
                }
                let produced = execute_texture_recipe(&recipe, &loading.pipeline.decoded_images);
                loading_pipeline_release_recipe_sources(&mut loading.pipeline, &recipe);
                let Some(decoded) = produced else {
                    continue;
                };
                let VulkanRenderer {
                    context, textures, ..
                } = &mut *renderer;
                if let Err(error) =
                    upload_texture(textures, context, texture.index, &decoded, usage, sampler)
                {
                    tracing::error!("a scene texture was not materialized: {error}");
                }
                uploaded.push(texture);
                uploads += 1;
            }
        }
        loading_pipeline_mark_completed(&mut loading.pipeline, label, category);
    }

    for task in deferred.into_iter().rev() {
        loading_pipeline_requeue_front(&mut loading.pipeline, task);
    }

    if reserved_any || !uploaded.is_empty() || !registry.pending_resolve.is_empty() {
        material_registry_resolve_uploaded_textures(&mut registry, &textures, &uploaded);
        mesh_state.full_rebuild_needed = true;
    }

    let evicted =
        nightshade_render::commands::texture_cache::texture_cache_remove_unused(&mut textures);
    if !evicted.is_empty() {
        let VulkanRenderer {
            context,
            textures: table,
            ..
        } = &mut *renderer;
        for (_name, slot) in evicted {
            crate::renderer::image_table::free_image(&mut table.table, context, slot);
        }
        mesh_state.full_rebuild_needed = true;
    }
}

pub fn upload_scene(
    mut renderer: Renderer,
    state: Res<RendererState>,
    mut mesh_state: ResMut<MeshRenderState>,
    views: Res<nightshade_render::data::config::RenderViews>,
    settings: Res<RenderSettings>,
    time: Res<nightshade_platform::contract::time::Time>,
    meshes: Res<MeshCache>,
    clusters: Res<ClusterCache>,
    texture_cache: Res<TextureCache>,
    storage: &mut nightshade::ecs::Storage,
) {
    let VulkanRenderer {
        context,
        geometry,
        materials: material_table,
        textures,
        shadows,
        environment,
        passes,
        view,
        rays,
        copies,
        frames_sent,
        ..
    } = &mut *renderer;
    let frame_number = *frames_sent;

    if let Err(error) = upload_materials(material_table, context, &state.render_materials, textures)
    {
        tracing::error!("the material table was not uploaded: {error}");
        return;
    }
    mesh_state_take_frame(&mut mesh_state);

    let skinned = gather_skins(geometry, context, &meshes, &clusters, &state);
    let mut skin_instances = Vec::new();
    let mut objects = Vec::new();
    let mut run_capacity = 0;
    let mut posed_meshlets = 0;
    let mut posed_meshlet_width = 0;
    let mut placements: Vec<crate::renderer::rays::Placement> = Vec::new();
    let mut work_capacity = 0;
    let mut has_overlay = false;
    let sources: Vec<ObjectSource> = storage
        .query_ref::<ObjectRow<'_>>()
        .without::<InstancedObjectData>()
        .iter()
        .map(
            |(entity, (transform, name, material, visible, culling_mask, layer, bounds))| {
                ObjectSource {
                    entity,
                    transform: transform.0,
                    mesh_name: name.0.clone(),
                    material_id: material.0,
                    visible: visible.0,
                    culling_mask: culling_mask.0,
                    layer: layer.0,
                    bounds: bounds.map(|bounds| {
                        [
                            bounds.center.x,
                            bounds.center.y,
                            bounds.center.z,
                            bounds.sphere_radius,
                        ]
                    }),
                }
            },
        )
        .collect();

    for source in &sources {
        if !layer_enabled(&settings, source.layer) {
            continue;
        }
        let Some(slot) = slot_for(geometry, context, &meshes, &clusters, &source.mesh_name) else {
            continue;
        };
        let batches = meshlet_count(geometry, slot).div_ceil(TASK_BATCH);
        let material_index = source.material_id.checked_sub(1).unwrap_or(NO_MATERIAL);
        let sphere = source
            .bounds
            .or_else(|| mesh_bounds(geometry, slot))
            .unwrap_or([0.0, 0.0, 0.0, f32::MAX]);
        let sided =
            if flips_winding(&source.transform) {
                FLAG_DOUBLE_SIDED
            } else {
                0
            } | if source.layer == nightshade_render::data::render_layer::RenderLayer::OVERLAY {
                has_overlay = true;
                FLAG_OVERLAY
            } else {
                0
            };
        let shadow_flag = if storage.has_tag_type::<ShadowCaster>(source.entity) {
            FLAG_SHADOW_CASTER
        } else {
            0
        } | exempt_flag(storage, source.entity);

        work_capacity += batches;
        if source.visible
            && shadow_flag & FLAG_SHADOW_CASTER != 0
            && !state
                .render_materials
                .transparent_ids
                .contains(&source.material_id)
        {
            ready_to_trace(rays, context, geometry, slot);
            placements.push(crate::renderer::rays::Placement {
                transform: matrix_columns(&source.transform),
                traced: crate::renderer::rays::Traced::Mesh(slot),
                object: objects.len() as u32,
                masked: state
                    .render_materials
                    .mask_ids
                    .contains(&source.material_id),
            });
        }
        objects.push(GpuObject {
            transform: matrix_columns(&source.transform),
            previous_transform: matrix_columns(&source.transform),
            previous_vertex_offset: 0,
            normal_transform: normal_transform(&source.transform),
            bounds: sphere,
            mesh_slot: slot,
            material_index,
            culling_mask: source.culling_mask,
            flags: if source.visible {
                FLAG_VISIBLE | shadow_flag | sided
            } else {
                shadow_flag | sided
            },
            vertex_offset: 0,
            entity_id: source.entity.id,
            run: SOLO_RUN,
            posed_meshlets: 0,
            morph_weights: storage
                .get::<MorphWeights>(source.entity)
                .map(|weights| weights.0)
                .unwrap_or([0.0; 8]),
            tint: [1.0; 4],
        });
    }

    for (entity, skin) in &skinned {
        let Some(dynamic) = state.render_skinned_dynamic_state.get(entity) else {
            continue;
        };
        let shadow_flag = if state.render_shadow_casters.skinned.contains(entity) {
            FLAG_SHADOW_CASTER
        } else {
            0
        } | exempt_flag(storage, *entity);
        let material = storage
            .get::<MaterialId>(*entity)
            .map(|material| material.0)
            .unwrap_or(0);
        let meshlets = meshlet_count(geometry, skin.mesh_slot);
        work_capacity += meshlets.div_ceil(TASK_BATCH);
        let posed_meshlet_offset = posed_meshlets;
        posed_meshlets += meshlets;
        posed_meshlet_width = posed_meshlet_width.max(meshlets);
        let mut instance = skin.instance;
        instance.object_index = objects.len() as u32;
        skin_instances.push(instance);
        objects.push(GpuObject {
            transform: matrix_columns(&dynamic.transform),
            previous_transform: matrix_columns(&dynamic.transform),
            previous_vertex_offset: skin.instance.posed_vertex_offset,
            normal_transform: normal_transform(&dynamic.transform),
            bounds: [0.0; 4],
            mesh_slot: skin.mesh_slot,
            material_index: material.checked_sub(1).unwrap_or(NO_MATERIAL),
            culling_mask: dynamic.culling_mask,
            flags: if dynamic.visible != 0 {
                FLAG_VISIBLE | FLAG_SKINNED | shadow_flag
            } else {
                FLAG_SKINNED | shadow_flag
            },
            vertex_offset: skin.instance.posed_vertex_offset,
            entity_id: entity.id,
            run: SOLO_RUN,
            posed_meshlets: posed_meshlet_offset,
            morph_weights: dynamic.morph_weights,
            tint: [1.0; 4],
        });
    }

    if copies_changed(copies, storage, &settings) {
        rebuild_copies(
            copies,
            storage,
            geometry,
            context,
            &meshes,
            &clusters,
            &settings,
            &state.render_materials,
        );
    }
    let instance_batches: Vec<GpuInstanceBatch> = copies
        .batches
        .iter()
        .map(|batch| GpuInstanceBatch {
            parent: batch.parent,
            first_object: objects.len() as u32 + batch.first_row,
            count: batch.count,
            first_local: batch.first_local,
            padding: 0,
        })
        .collect();
    for batch in &copies.batches {
        ready_to_trace(rays, context, geometry, batch.slot);
    }
    work_capacity += copies.work_capacity;
    run_capacity += copies.run_capacity;
    has_overlay |= copies.overlay;
    let copies_signature = copies.batches.iter().fold(
        copies.generation.wrapping_mul(0x9E37_79B9_7F4A_7C15),
        |settled, batch| {
            settled.rotate_left(7) ^ crate::renderer::rays::mesh_address(rays, batch.slot)
        },
    );

    let first_copy = objects.len() as u32;

    let Some(view) = view.as_ref() else {
        return;
    };
    let shading = views
        .active
        .as_ref()
        .map(|camera| camera.effective_shading)
        .unwrap_or(views.fallback_shading);

    let lighting = state.render_lighting.clone().unwrap_or_default();
    let mut lights = lighting.lights_data.clone();
    lights.truncate(MAX_LIGHTS);
    let mut area_lights = fold_area_lights(&mut lights, &lighting.area_lights_data);
    resolve_light_pictures(
        &mut lights,
        &mut area_lights,
        &lighting,
        storage,
        textures,
        &texture_cache,
    );
    let biases = light_biases(&lighting, storage, lights.len());
    let local_shadows = fit_local_shadows(&mut lights, &biases, view);
    let traced_reach = traced_sun_reach(
        context,
        !placements.is_empty() || !copies.placements.is_empty() || !skinned.is_empty(),
        &lighting,
        &shading,
        view,
    );

    let skinning = &state.render_skinning;
    let bones: Vec<[[f32; 4]; 4]> = skinning
        .bone_transforms
        .iter()
        .map(matrix_columns)
        .collect();
    let inverse_binds: Vec<[[f32; 4]; 4]> = skinning
        .cache
        .inverse_bind_matrices
        .iter()
        .map(matrix_columns)
        .collect();
    let skins = skinning.cache.skin_data.clone();

    let data = SceneData {
        frame: frame_uniform(
            view,
            &lighting,
            &shading,
            time.uptime_seconds as f32,
            frame_number,
            Counts {
                objects: (objects.len() + copies.rows.len()) as u32,
                lights: lights.len() as u32,
                work_capacity,
                runs: run_capacity,
                joints: skinning.cache.total_joints,
                skins: skins.len() as u32,
                skin_instances: skin_instances.len() as u32,
                culling_mask: shading.culling_mask,
                directional: lighting.num_directional_lights.min(lights.len() as u32),
            },
            traced_reach,
            state.view_projection,
            environment.crossed,
            state.min_screen_pixel_size.max(0.0),
        ),
        objects,
        copies: copies.snapshot(),
        lights,
        area_lights,
        local_shadows,
        has_overlay,
        instance_batches,
        bones,
        constraints: constraints::build(&state.render_animation, &skinning.cache),
        inverse_binds,
        skins,
        skin_instances,
        joints: skinning.cache.total_joints,
        posed_meshlets,
        posed_meshlet_width,
        work_capacity,
        run_capacity,
    };
    let handover = SceneHandover {
        mesh_ranges: range_buffer(geometry).buffer,
        indices: index_buffer(geometry).buffer,
        meshlets: meshlet_buffer(geometry).buffer,
        meshlet_vertices: meshlet_vertex_buffer(geometry).buffer,
        meshlet_triangles: meshlet_triangle_buffer(geometry).buffer,
        vertices: vertex_buffer(geometry).buffer,
        materials: material_table.buffer.buffer,
        skin_vertices: skin_vertex_buffer(geometry).buffer,
        morph_displacements: morph_buffer(geometry).buffer,
        textures: textures.table.set,
        shadows: shadows.sets,
        environment: environment.scene_sets,
    };

    let Some(pass) = passes
        .iter_mut()
        .find(|(_, pass)| pass.name() == SCENE_PASS)
        .map(|(_, pass)| pass.as_mut())
    else {
        return;
    };
    if let Some(pass) = (pass as &mut dyn std::any::Any).downcast_mut::<ScenePass>() {
        hand_over(pass, handover, data, state.view_projection);
        trace_the_posed(
            rays,
            context,
            geometry,
            &posed_runs(pass),
            &state.render_materials,
            &mut placements,
        );
    }
    build_world_to_trace(
        rays,
        context,
        shadows,
        &placements,
        &copies.placements,
        copies_signature,
        first_copy,
    );
}

const COPIES_IN_A_RUN: u32 = 1024;

fn run_length_of(meshlets: u32) -> u32 {
    if meshlets == 0 || meshlets > TASK_BATCH {
        1
    } else {
        COPIES_IN_A_RUN
    }
}

fn copies_changed(
    copies: &CopyRows,
    storage: &nightshade::ecs::Storage,
    settings: &RenderSettings,
) -> bool {
    if copies.generation == 0
        || copies.waiting
        || copies.overlay_enabled != settings.render_layer_overlay_enabled
    {
        return true;
    }
    let standing = storage
        .query_entities::<(InstancedObjectData, MaterialId)>()
        .count();
    if standing != copies.batches.len() {
        return true;
    }
    storage
        .query_entities_changed_since::<(InstancedObjectData,)>(copies.seen_tick)
        .next()
        .is_some()
        || storage
            .query_entities_changed_since::<(MaterialId,)>(copies.seen_tick)
            .any(|entity| storage.has::<InstancedObjectData>(entity))
}

fn rebuild_copies(
    copies: &mut CopyRows,
    storage: &nightshade::ecs::Storage,
    geometry: &mut GeometryArena,
    context: &crate::renderer::context::Context,
    meshes: &MeshCache,
    clusters: &ClusterCache,
    settings: &RenderSettings,
    materials: &nightshade_render::data::config::RenderMaterials,
) {
    let mut rows = Vec::new();
    let mut locals = Vec::new();
    let mut placements = Vec::new();
    let mut batches = Vec::new();
    let mut work_capacity = 0;
    let mut run_capacity = 0;
    let mut overlay = false;
    let mut waiting = false;
    let held: Vec<(Entity, u32)> = storage
        .query_ref::<(&InstancedObjectData, &MaterialId)>()
        .iter()
        .map(|(entity, (_, material))| (entity, material.0))
        .collect();
    for (entity, material_id) in held {
        let Some(instances) = storage.get::<InstancedObjectData>(entity) else {
            continue;
        };
        if !layer_enabled(settings, instances.render_layer)
            || instances.visible == 0
            || instances.local_matrices.is_empty()
        {
            continue;
        }
        let Some(slot) = slot_for(geometry, context, meshes, clusters, &instances.mesh_name) else {
            waiting = true;
            continue;
        };
        let per_copy = meshlet_count(geometry, slot).div_ceil(TASK_BATCH);
        let material_index = material_id.checked_sub(1).unwrap_or(NO_MATERIAL);
        let sphere = mesh_bounds(geometry, slot).unwrap_or([0.0, 0.0, 0.0, f32::MAX]);
        let layer_flag = if instances.render_layer
            == nightshade_render::data::render_layer::RenderLayer::OVERLAY
        {
            overlay = true;
            FLAG_OVERLAY
        } else {
            0
        };
        let shadow_flag = if storage.has_tag_type::<ShadowCaster>(entity) {
            FLAG_SHADOW_CASTER
        } else {
            0
        } | exempt_flag(storage, entity);
        let traced = shadow_flag & FLAG_SHADOW_CASTER != 0
            && !materials.transparent_ids.contains(&material_id);
        let masked = materials.mask_ids.contains(&material_id);
        let count = instances.local_matrices.len() as u32;
        let run_length = run_length_of(meshlet_count(geometry, slot));
        batches.push(CopyBatch {
            parent: matrix_columns(&instances.parent_transform),
            first_row: rows.len() as u32,
            count,
            first_local: locals.len() as u32,
            slot,
        });
        if run_length > 1 {
            run_capacity += count.div_ceil(run_length);
        } else {
            work_capacity += count * per_copy;
        }
        for (index, local) in instances.local_matrices.iter().enumerate() {
            let position = index as u32 % run_length;
            let length = run_length.min(count - (index as u32 - position));
            let bounds = if run_length > 1 && position == 0 {
                let head = index;
                nightshade_render::data::bounding_volume::instanced_bounding_sphere(
                    &instances.parent_transform,
                    &instances.local_matrices[head..head + length as usize],
                    sphere,
                )
            } else {
                sphere
            };
            locals.push(matrix_columns(local));
            if traced {
                placements.push(crate::renderer::rays::Placement {
                    transform: matrix_columns(&(instances.parent_transform * local)),
                    traced: crate::renderer::rays::Traced::Mesh(slot),
                    object: rows.len() as u32,
                    masked,
                });
            }
            rows.push(GpuObject {
                transform: [[0.0; 4]; 4],
                previous_transform: [[0.0; 4]; 4],
                previous_vertex_offset: 0,
                normal_transform: [[0.0; 4]; 4],
                bounds,
                mesh_slot: slot,
                material_index,
                culling_mask: u32::MAX,
                flags: FLAG_VISIBLE | shadow_flag | layer_flag,
                vertex_offset: 0,
                entity_id: entity.id,
                run: run_of(length, position),
                posed_meshlets: 0,
                morph_weights: [0.0; 8],
                tint: instances
                    .custom_tints
                    .get(index)
                    .copied()
                    .unwrap_or([1.0; 4]),
            });
        }
    }
    copies.rows = std::sync::Arc::new(rows);
    copies.locals = std::sync::Arc::new(locals);
    copies.placements = std::sync::Arc::new(placements);
    copies.batches = batches;
    copies.work_capacity = work_capacity;
    copies.run_capacity = run_capacity;
    copies.overlay = overlay;
    copies.waiting = waiting;
    copies.overlay_enabled = settings.render_layer_overlay_enabled;
    copies.seen_tick = storage.current_tick().wrapping_sub(1);
    copies.generation = copies.generation.wrapping_add(1).max(1);
}

fn trace_the_posed(
    rays: &mut crate::renderer::rays::Rays,
    context: &crate::renderer::context::Context,
    geometry: &GeometryArena,
    runs: &[crate::renderer::scene::pass::PosedRun],
    materials: &nightshade_render::data::config::RenderMaterials,
    placements: &mut Vec<crate::renderer::rays::Placement>,
) {
    if !crate::renderer::rays::traces(context) {
        return;
    }
    let stride = std::mem::size_of::<nightshade_render::data::geometry::Vertex>() as u64;
    let mut poses = Vec::with_capacity(runs.len());
    for run in runs {
        let Some(range) = mesh_range(geometry, run.mesh_slot) else {
            continue;
        };
        let material_id = run.material_index.wrapping_add(1);
        if !run.casts_shadow || materials.transparent_ids.contains(&material_id) {
            continue;
        }
        placements.push(crate::renderer::rays::Placement {
            transform: STANDS_WHERE_IT_IS,
            traced: crate::renderer::rays::Traced::Posed(poses.len() as u32),
            object: run.object_index,
            masked: materials.mask_ids.contains(&material_id),
        });
        poses.push(crate::renderer::rays::MeshGeometry {
            vertices: run.vertex_offset as u64 * stride,
            vertex_count: run.vertex_count,
            vertex_stride: stride,
            indices: range.index_offset as u64 * 4,
            index_count: range.index_count,
        });
    }
    if let Err(error) = crate::renderer::rays::build_posed(rays, context, &poses) {
        tracing::error!("the posed runs were not made ready to trace: {error}");
    }
}

const STANDS_WHERE_IT_IS: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

struct Counts {
    objects: u32,
    lights: u32,
    work_capacity: u32,
    runs: u32,
    joints: u32,
    skins: u32,
    skin_instances: u32,
    culling_mask: u32,
    directional: u32,
}

fn seed_primitives(geometry: &mut GeometryArena, context: &crate::renderer::context::Context) {
    if mesh_slot(geometry, "Cube").is_some() {
        return;
    }
    let primitives = [
        ("Cube", create_cube_mesh()),
        ("Sphere", create_sphere_mesh(1.0, 16)),
        ("Plane", create_plane_mesh(2.0)),
        ("SubdividedPlane", create_subdivided_plane_mesh(2.0, 20)),
        ("Cylinder", create_cylinder_mesh(0.5, 1.0, 16)),
        ("Cone", create_cone_mesh(0.5, 1.0, 16)),
        ("Torus", create_torus_mesh(1.0, 0.3, 16, 16)),
        ("Quad", create_quad_mesh(1.0)),
    ];
    for (name, mesh) in primitives {
        if let Err(error) = upload_mesh(geometry, context, name, &mesh, None) {
            tracing::error!("the primitive {name} was not uploaded: {error}");
        }
    }
    if let Err(error) = flush_ranges(geometry, context) {
        tracing::error!("the mesh ranges were not written: {error}");
    }
}

fn slot_for(
    geometry: &mut GeometryArena,
    context: &crate::renderer::context::Context,
    meshes: &MeshCache,
    clusters: &ClusterCache,
    name: &str,
) -> Option<u32> {
    if let Some(slot) = mesh_slot(geometry, name) {
        return Some(slot);
    }
    let mesh = mesh_cache_iter(meshes)
        .find(|(entry, _)| entry.as_str() == name)?
        .1;
    let built = clusters_of(clusters, name)?;
    match upload_mesh(geometry, context, name, mesh, Some(built)) {
        Ok(slot) => {
            if let Err(error) = flush_ranges(geometry, context) {
                tracing::error!("the mesh ranges were not written: {error}");
            }
            Some(slot)
        }
        Err(error) => {
            tracing::error!("the mesh {name} was not uploaded: {error}");
            None
        }
    }
}

fn gather_skins(
    geometry: &mut GeometryArena,
    context: &crate::renderer::context::Context,
    meshes: &MeshCache,
    clusters: &ClusterCache,
    state: &RendererState,
) -> std::collections::HashMap<Entity, SkinnedEntity> {
    let skinning = &state.render_skinning;
    let mut gathered = std::collections::HashMap::new();
    for entity in &state.render_skinned_meshes {
        let Some(name) = state.render_skinned_mesh_names.get(entity) else {
            continue;
        };
        let Some(slot) = slot_for(geometry, context, meshes, clusters, name) else {
            continue;
        };
        let Some(skin_vertex_offset) = skin_offset(geometry, slot) else {
            continue;
        };
        let Some(&skin_index) = skinning.cache.entity_skin_indices.get(entity) else {
            continue;
        };
        let Some(skin) = skinning.cache.skin_data.get(skin_index as usize) else {
            continue;
        };
        let Some(range) = mesh_range(geometry, slot) else {
            continue;
        };
        let vertex_count = range.vertex_count;
        let source_vertex_offset = range.vertex_offset;
        let posed_vertex_offset = match reserve_pose(geometry, context, *entity, vertex_count) {
            Ok(offset) => offset,
            Err(error) => {
                tracing::error!("a skinned pose was not reserved: {error}");
                continue;
            }
        };
        gathered.insert(
            *entity,
            SkinnedEntity {
                instance: GpuSkinInstance {
                    source_vertex_offset,
                    skin_vertex_offset,
                    posed_vertex_offset,
                    vertex_count,
                    joint_offset: skin.base_output_index,
                    bone_offset: skin.base_bone_index,
                    bone_count: skin.joint_count,
                    object_index: 0,
                },
                mesh_slot: slot,
            },
        );
    }
    gathered
}

fn layer_enabled(settings: &RenderSettings, layer: u32) -> bool {
    match layer {
        nightshade_render::data::render_layer::RenderLayer::WORLD => {
            settings.render_layer_world_enabled
        }
        nightshade_render::data::render_layer::RenderLayer::OVERLAY => {
            settings.render_layer_overlay_enabled
        }
        _ => true,
    }
}

const CUBE_FACES: [([f32; 3], [f32; 3]); 6] = [
    ([1.0, 0.0, 0.0], [0.0, -1.0, 0.0]),
    ([-1.0, 0.0, 0.0], [0.0, -1.0, 0.0]),
    ([0.0, 1.0, 0.0], [0.0, 0.0, 1.0]),
    ([0.0, -1.0, 0.0], [0.0, 0.0, -1.0]),
    ([0.0, 0.0, 1.0], [0.0, -1.0, 0.0]),
    ([0.0, 0.0, -1.0], [0.0, -1.0, 0.0]),
];

fn light_biases(
    lighting: &nightshade_render::data::config::RenderLighting,
    storage: &nightshade::ecs::Storage,
    count: usize,
) -> Vec<f32> {
    let mut biases = vec![lighting.shadow_bias; count];
    for (entity, index) in &lighting.entity_to_lights_index {
        if let Some(bias) = biases.get_mut(*index)
            && let Some(light) =
                storage.get::<nightshade_render::components::light::RenderLightData>(*entity)
        {
            *bias = light.shadow_bias;
        }
    }
    biases
}

const LIGHT_AREA: u32 = 3;

fn fold_area_lights(
    lights: &mut Vec<LightData>,
    area_lights: &[AreaLightData],
) -> Vec<GpuAreaLight> {
    let mut surfaces = Vec::with_capacity(area_lights.len());
    for area in area_lights {
        if lights.len() >= MAX_LIGHTS {
            break;
        }
        lights.push(LightData {
            position: [area.position[0], area.position[1], area.position[2], 1.0],
            direction: [area.position[3], area.right[3], area.up[3], 0.0],
            color: area.color,
            light_type: LIGHT_AREA,
            range: area.range,
            inner_cone: 0.0,
            outer_cone: 0.0,
            shadow_index: surfaces.len() as i32,
            light_size: area.radius,
            cookie_layer: COOKIE_LAYER_NONE,
            casts_shadow: 0.0,
        });
        surfaces.push(GpuAreaLight {
            right: [area.right[0], area.right[1], area.right[2], 0.0],
            up: [area.up[0], area.up[1], area.up[2], 0.0],
            shape: area.shape,
            two_sided: area.two_sided,
            emissive_layer: area.emissive_layer,
            radius: area.radius,
        });
    }
    surfaces
}

fn resolve_light_pictures(
    lights: &mut [LightData],
    area_lights: &mut [GpuAreaLight],
    lighting: &RenderLighting,
    storage: &nightshade::ecs::Storage,
    textures: &SceneTextures,
    cache: &TextureCache,
) {
    for (entity, index) in &lighting.entity_to_lights_index {
        let Some(light) = storage.get::<RenderLightData>(*entity) else {
            continue;
        };
        let Some(slot) = picture_slot(light.cookie_texture.as_deref(), textures, cache) else {
            continue;
        };
        if let Some(target) = lights.get_mut(*index) {
            target.cookie_layer = slot;
        }
    }
    for (entity, index) in &lighting.area_entity_to_index {
        let Some(light) = storage.get::<RenderLightData>(*entity) else {
            continue;
        };
        let named = light.area_emissive_texture.as_deref();
        let Some(slot) = picture_slot(named, textures, cache) else {
            continue;
        };
        if let Some(target) = area_lights.get_mut(*index) {
            target.emissive_layer = slot;
        }
    }
}

fn picture_slot(name: Option<&str>, textures: &SceneTextures, cache: &TextureCache) -> Option<u32> {
    let name = name?;
    if name.is_empty() {
        return None;
    }
    let id = texture_cache_texture_id(cache, name)?;
    let sampler = texture_sampler(textures, id.index);
    Some(pack_texture_slot(
        id.index,
        sampler.wrap_u,
        sampler.wrap_v,
        sampler.mag_filter,
    ))
}

struct SlotQuadtree {
    across: u32,
    taken: Vec<bool>,
}

impl SlotQuadtree {
    fn new() -> Self {
        let across = LOCAL_ATLAS_SIZE / LOCAL_SLOT_NARROWEST;
        Self {
            across,
            taken: vec![false; (across * across) as usize],
        }
    }

    fn claim(&mut self, wanted: u32) -> Option<([f32; 2], f32)> {
        let mut size = wanted.clamp(LOCAL_SLOT_NARROWEST, LOCAL_SLOT_WIDEST);
        loop {
            let cells = size / LOCAL_SLOT_NARROWEST;
            if let Some((column, row)) = self.free_square(cells) {
                self.mark(column, row, cells);
                let atlas = LOCAL_ATLAS_SIZE as f32;
                return Some((
                    [
                        (column * LOCAL_SLOT_NARROWEST) as f32 / atlas,
                        (row * LOCAL_SLOT_NARROWEST) as f32 / atlas,
                    ],
                    size as f32 / atlas,
                ));
            }
            if size == LOCAL_SLOT_NARROWEST {
                return None;
            }
            size /= 2;
        }
    }

    fn free_square(&self, cells: u32) -> Option<(u32, u32)> {
        let mut row = 0;
        while row + cells <= self.across {
            let mut column = 0;
            while column + cells <= self.across {
                if self.square_is_free(column, row, cells) {
                    return Some((column, row));
                }
                column += cells;
            }
            row += cells;
        }
        None
    }

    fn square_is_free(&self, column: u32, row: u32, cells: u32) -> bool {
        (row..row + cells)
            .all(|y| (column..column + cells).all(|x| !self.taken[(y * self.across + x) as usize]))
    }

    fn mark(&mut self, column: u32, row: u32, cells: u32) {
        for y in row..row + cells {
            for x in column..column + cells {
                self.taken[(y * self.across + x) as usize] = true;
            }
        }
    }
}

fn fit_local_shadows(
    lights: &mut [LightData],
    biases: &[f32],
    view: &RenderView,
) -> Vec<GpuLocalShadow> {
    let mut fitted = Vec::new();
    let mut quadtree = SlotQuadtree::new();
    let mut order: Vec<(usize, f32, f32)> = Vec::new();
    for (index, light) in lights.iter_mut().enumerate() {
        let is_local = light.light_type == 1 || light.light_type == 2;
        if !is_local {
            continue;
        }
        light.shadow_index = -1;
        if light.casts_shadow == 0.0 {
            continue;
        }
        let reach = if light.range > 0.0 {
            light.range
        } else {
            100.0
        };
        order.push((index, screen_coverage(light, reach, view), reach));
    }
    order.sort_by(|first, second| second.1.total_cmp(&first.1));

    for (index, coverage, reach) in order {
        let light = &mut lights[index];
        let bias = biases[index];
        let faces = if light.light_type == 1 { 6 } else { 1 };
        if fitted.len() + faces > LOCAL_SHADOW_COUNT {
            continue;
        }
        let wanted = wanted_slot(coverage, view, LOCAL_SLOT_WIDEST, LOCAL_SLOT_NARROWEST);
        let mut places = Vec::with_capacity(faces);
        for _ in 0..faces {
            let Some(place) = quadtree.claim(wanted) else {
                break;
            };
            places.push(place);
        }
        if places.len() < faces {
            continue;
        }
        let position = nalgebra_glm::vec3(light.position[0], light.position[1], light.position[2]);
        let first = fitted.len();

        for (cube, (origin, span)) in CUBE_FACES.iter().take(faces).zip(places) {
            let (forward, up) = if faces == 6 {
                let (forward, up) = *cube;
                (
                    nalgebra_glm::vec3(forward[0], forward[1], forward[2]),
                    nalgebra_glm::vec3(up[0], up[1], up[2]),
                )
            } else {
                let forward = nalgebra_glm::normalize(&nalgebra_glm::vec3(
                    light.direction[0],
                    light.direction[1],
                    light.direction[2],
                ));
                let up = if forward.y.abs() > 0.99 {
                    nalgebra_glm::vec3(0.0, 0.0, 1.0)
                } else {
                    nalgebra_glm::vec3(0.0, 1.0, 0.0)
                };
                (forward, up)
            };
            let spread = if faces == 6 {
                std::f32::consts::FRAC_PI_2
            } else {
                (light.outer_cone.clamp(-1.0, 1.0).acos() * 2.0 + 0.2)
                    .clamp(0.2, std::f32::consts::PI * 0.9)
            };
            let looking = nalgebra_glm::look_at(&position, &(position + forward), &up);
            let projection = reverse_z_perspective(spread, 1.0, 0.05, reach);
            let view_projection = projection * looking;

            let mut planes = [[0.0f32; 4]; 6];
            for (index, plane) in
                nightshade_render::data::bounding_volume::extract_frustum_planes(&view_projection)
                    .iter()
                    .enumerate()
            {
                planes[index] = [plane.x, plane.y, plane.z, plane.w];
            }
            fitted.push(GpuLocalShadow {
                view_projection: matrix_columns(&view_projection),
                planes,
                slot: [origin[0], origin[1], span, bias],
                light: [position.x, position.y, position.z, reach],
            });
        }

        light.shadow_index = first as i32;
    }
    fitted
}

fn frame_uniform(
    view: &RenderView,
    lighting: &RenderLighting,
    shading: &EffectiveShading,
    seconds: f32,
    frame_number: u64,
    counts: Counts,
    traced_reach: f32,
    unjittered: [[f32; 4]; 4],
    crossed: f32,
    least_pixels: f32,
) -> GpuFrame {
    let mut planes = [[0.0f32; 4]; 6];
    for (slot, plane) in view.frustum_planes.iter().enumerate() {
        planes[slot] = [plane.x, plane.y, plane.z, plane.w];
    }
    GpuFrame {
        view: matrix_columns(&view.view),
        projection: matrix_columns(&view.projection),
        view_projection: matrix_columns(&view.view_projection),
        camera_position: [
            view.camera_position.x,
            view.camera_position.y,
            view.camera_position.z,
            1.0,
        ],
        frustum_planes: planes,
        counts: [
            counts.objects,
            counts.lights,
            counts.runs,
            counts.work_capacity,
        ],
        settings: [
            if shading.anti_aliasing.is_temporal() {
                1.0
            } else {
                0.0
            },
            EXPOSURE,
            seconds,
            (frame_number % 4096) as f32,
        ],
        skin_counts: [counts.joints, counts.skins, counts.skin_instances, 0],
        masks: [counts.culling_mask, counts.directional, 0, 0],
        debug: [
            if shading.unlit_mode { 1.0 } else { 0.0 },
            if shading.show_normals { 1.0 } else { 0.0 },
            shading.pbr_debug_mode.as_u32() as f32,
            if shading.material_filtering { 1.0 } else { 0.0 },
        ],
        flat_color: shading
            .flat_shading_color
            .map(|color| [color.x, color.y, color.z, color.w])
            .unwrap_or([0.0, 0.0, 0.0, -1.0]),
        cluster: nightshade_render::data::config::cluster_slicing(view),
        screen: [
            view.screen_size.0.max(1) as f32,
            view.screen_size.1.max(1) as f32,
            least_pixels,
            0.0,
        ],
        cascade_view_projections: lighting.cascade_view_projections,
        cascade_planes: cascade_planes(lighting),
        cascade_splits: lighting.cascade_split_distances,
        cascade_texel_sizes: cascade_texel_sizes(lighting, SLOT_SIZE),
        shadow_settings: [
            lighting.shadow_bias,
            lighting.shadow_normal_bias,
            shadows_enabled(lighting),
            ATLAS_SIZE as f32,
        ],
        sun_shadow: [
            lighting.directional_light_size.max(0.0),
            TRACED_STEP_OFF,
            0.01,
            traced_reach,
        ],
        previous_view_projection: unjittered,
        environment: [crossed, 0.0, 0.0, 0.0],
        pyramid: [0.0; 4],
        fog_color: shading
            .fog
            .as_ref()
            .map(|fog| {
                [
                    fog.color[0],
                    fog.color[1],
                    fog.color[2],
                    fog.mode.as_u32() as f32,
                ]
            })
            .unwrap_or([0.0; 4]),
        style: [
            shading
                .vertex_snap
                .map(|snap| snap.resolution[0])
                .unwrap_or(0.0),
            shading
                .vertex_snap
                .map(|snap| snap.resolution[1])
                .unwrap_or(0.0),
            if shading.vertex_snap.is_some() {
                1.0
            } else {
                0.0
            },
            if shading.affine_texture_mapping {
                1.0
            } else {
                0.0
            },
        ],
        fog: shading
            .fog
            .as_ref()
            .map(|fog| {
                [
                    fog.start,
                    fog.end,
                    if fog.volumetric { 1.0 } else { 0.0 },
                    0.0,
                ]
            })
            .unwrap_or([0.0; 4]),
        fog_air: shading
            .fog
            .as_ref()
            .map(|fog| {
                [
                    fog.density.max(0.0),
                    fog.anisotropy.clamp(-0.9, 0.9),
                    fog.height_falloff.max(0.0),
                    fog.height,
                ]
            })
            .unwrap_or([0.0; 4]),
        local_shadow: [LOCAL_ATLAS_SIZE as f32, LOCAL_SLOT_WIDEST as f32, 0.0, 0.0],
        probe_origin: probe_origin(shading),
        probe_spacing: probe_spacing(shading),
        probe_counts: probe_counts(shading, PROBE_MAX_COUNTS),
        probe_settings: probe_settings(shading),
        probe_rotation: probe_rotation(frame_number),
    }
}

fn cascade_planes(lighting: &RenderLighting) -> [[[f32; 4]; 6]; CASCADE_COUNT] {
    let mut cascades = [[[0.0f32; 4]; 6]; CASCADE_COUNT];
    for (cascade, matrix) in lighting.cascade_view_projections.iter().enumerate() {
        for (slot, plane) in full_frustum_planes(matrix).iter().enumerate() {
            cascades[cascade][slot] = [plane.x, plane.y, plane.z, plane.w];
        }
    }
    cascades
}

const EXPOSURE: f32 = 1.0;

const TRACED_STEP_OFF: f32 = 1.0e-3;

fn exempt_flag(storage: &nightshade::ecs::Storage, entity: nightshade::ecs::Entity) -> u32 {
    if storage
        .get::<nightshade_render::components::mesh::DebugViewExempt>(entity)
        .is_some()
    {
        FLAG_DEBUG_VIEW_EXEMPT
    } else {
        0
    }
}
