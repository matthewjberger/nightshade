use crate::ecs::world::World;
use crate::render::wgpu::render_configs::{RenderInputs, UiFrameInputs, ViewInputs};

/// Composes the renderer's per-frame input from the world. Large snapshot
/// state is moved out of the resources rather than cloned; small configs are
/// copied. [`restore_render_inputs`] puts the moved state back after the
/// frame so the engine and game code observe it as usual between frames.
pub fn compose_render_inputs(world: &mut World) -> RenderInputs {
    let resources = &mut world.resources;
    let retained = &mut resources.retained_ui;
    RenderInputs {
        settings: resources.render_settings.clone(),
        debug_draw: resources.debug_draw.clone(),
        editor_selection: resources.editor_selection.clone(),
        scene: std::mem::take(&mut resources.renderer_state),
        ibl_views: resources.ibl_views.clone(),
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
        grass: resources.grass.clone(),
        #[cfg(feature = "terrain")]
        terrain: resources.terrain.clone(),
        #[cfg(feature = "terrain")]
        terrain_render: std::mem::take(&mut resources.terrain_render),
    }
}

/// Moves the snapshot state carried by [`RenderInputs`] back into the world.
/// Containers the engine may have pushed to while the input was taken keep
/// both sides: the taken contents come back first, then anything new.
pub fn restore_render_inputs(world: &mut World, inputs: RenderInputs) {
    let resources = &mut world.resources;
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
    #[cfg(feature = "terrain")]
    {
        resources.terrain_render = inputs.terrain_render;
    }
}
