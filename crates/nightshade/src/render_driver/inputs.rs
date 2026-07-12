use crate::ecs::world::World;
use crate::render::wgpu::render_configs::{RenderInputs, UiFrameInputs, ViewInputs};

/// Composes the renderer's per-frame input by moving state out of the world.
///
/// Every non-`Copy` input is taken with `std::mem::take`, not cloned, so
/// handing the renderer its inputs costs nothing regardless of scene size.
///
/// INVARIANT: between this call and [`restore_render_inputs`], the world's
/// copies of the taken resources (`render_settings`, `debug_draw`,
/// `editor_selection`, `renderer_state`, `ibl_views`, `shadow_atlas`,
/// `texture_cache`, `assets.mesh_cache`, `loading.pending_particle_textures`,
/// the retained-UI frame and render slots, and the grass and terrain state)
/// are empty defaults. Nothing may read or write them until the restore runs;
/// writes are discarded when the taken state is moved back. The only
/// exception is `loading.pending_particle_textures`, which the restore merges
/// so uploads queued mid-frame survive.
pub fn compose_render_inputs(world: &mut World) -> RenderInputs {
    let resources = &mut world.resources;
    let retained = &mut resources.retained_ui;
    RenderInputs {
        settings: std::mem::take(&mut resources.render_settings),
        debug_draw: std::mem::take(&mut resources.debug_draw),
        editor_selection: std::mem::take(&mut resources.editor_selection),
        scene: std::mem::take(&mut resources.renderer_state),
        ibl_views: std::mem::take(&mut resources.ibl_views),
        shadow_atlas: std::mem::take(&mut resources.shadow_atlas),
        wind: resources.wind,
        mesh_cache: std::mem::take(&mut resources.assets.mesh_cache),
        texture_cache: std::mem::take(&mut resources.texture_cache),
        pending_particle_textures: std::mem::take(&mut resources.loading.pending_particle_textures),
        ui: UiFrameInputs {
            rects: std::mem::take(&mut retained.frame.rects),
            rect_entities: std::mem::take(&mut retained.frame.rect_entities),
            images: std::mem::take(&mut retained.frame.ui_images),
            text_meshes: std::mem::take(&mut retained.frame.text_meshes),
            render_slots: std::mem::take(&mut retained.render_slots),
            background_color: retained.background_color,
        },
        view: ViewInputs {
            active_camera: resources.active_camera,
            uptime_milliseconds: resources.window.timing.uptime_milliseconds,
            delta_time: resources.window.timing.delta_time,
            viewport_size: resources.window.cached_viewport_size,
            camera_tile_rects: resources.window.camera_tile_rects.clone(),
            camera_tile_render_iteration: resources.window.camera_tile_render_iteration,
            active_viewport_rect: resources.window.active_viewport_rect,
        },
        world_id: resources.world_id,
        #[cfg(feature = "grass")]
        grass: std::mem::take(&mut resources.grass),
        #[cfg(feature = "terrain")]
        terrain: std::mem::take(&mut resources.terrain),
        #[cfg(feature = "terrain")]
        terrain_render: std::mem::take(&mut resources.terrain_render),
    }
}

/// Moves the state carried by [`RenderInputs`] back into the world, ending the
/// window in which the world's copies are empty. Everything is a plain move
/// that overwrites the world's default-valued placeholder, except
/// `loading.pending_particle_textures`: the taken contents come back first and
/// anything the engine queued mid-frame is appended after them, preserving the
/// upload cursor the particle pass keeps into that list.
pub fn restore_render_inputs(world: &mut World, inputs: RenderInputs) {
    let resources = &mut world.resources;
    resources.render_settings = inputs.settings;
    resources.debug_draw = inputs.debug_draw;
    resources.editor_selection = inputs.editor_selection;
    resources.renderer_state = inputs.scene;
    resources.ibl_views = inputs.ibl_views;
    resources.shadow_atlas = inputs.shadow_atlas;
    resources.texture_cache = inputs.texture_cache;
    resources.assets.mesh_cache = inputs.mesh_cache;
    let mut pending = inputs.pending_particle_textures;
    pending.extend(std::mem::take(
        &mut resources.loading.pending_particle_textures,
    ));
    resources.loading.pending_particle_textures = pending;
    let retained = &mut resources.retained_ui;
    retained.frame.rects = inputs.ui.rects;
    retained.frame.rect_entities = inputs.ui.rect_entities;
    retained.frame.ui_images = inputs.ui.images;
    retained.frame.text_meshes = inputs.ui.text_meshes;
    retained.render_slots = inputs.ui.render_slots;
    #[cfg(feature = "grass")]
    {
        resources.grass = inputs.grass;
    }
    #[cfg(feature = "terrain")]
    {
        resources.terrain = inputs.terrain;
        resources.terrain_render = inputs.terrain_render;
    }
}
