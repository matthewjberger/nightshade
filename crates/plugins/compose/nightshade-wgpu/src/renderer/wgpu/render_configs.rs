pub use nightshade_render::data::config::{FrameInputs, LinesFrameInputs, ViewFrame, ViewInputs};
use std::collections::HashMap;

pub use nightshade_render::contract::{CameraFrameInputs, CameraProjectionParams, CameraTarget};
pub use nightshade_render::data::config::{FrameOutputs, RendererCommand};

pub struct RenderInputs {
    pub scene_world: nightshade::ecs::World,
    pub settings: crate::renderer::config::RenderSettings,
    pub vsync: bool,
    pub debug_draw: crate::renderer::config::DebugDraw,
    pub scene: crate::renderer::config::RendererState,
    pub mesh_cache: nightshade_render::resources::mesh_cache::MeshCache,
    pub texture_cache: nightshade_render::resources::texture_cache::TextureCache,
    pub view: ViewInputs,
    pub frame: FrameInputs,
    pub mesh_frame_state: Option<nightshade_render::resources::mesh_state::MeshRenderState>,
    pub hdr_skybox_data: Vec<Vec<u8>>,
    pub dirty_meshes: std::collections::HashSet<String>,
    pub dirty_skinned_meshes: std::collections::HashSet<String>,
    pub owned: RendererOwned,
}

#[derive(Default)]
pub struct RendererOwned {
    pub shadow_atlas: nightshade_render::resources::shadow::SpotlightAtlasAssignment,
    pub point_shadows: Vec<nightshade_render::resources::shadow::PointShadowSlot>,
    pub texture_store: crate::renderer::wgpu::texture_cache::TextureStore,
}

impl RenderInputs {
    pub fn single_view(
        scene_world: nightshade::ecs::World,
        scene: crate::renderer::config::RendererState,
        mesh_cache: nightshade_render::resources::mesh_cache::MeshCache,
        texture_cache: nightshade_render::resources::texture_cache::TextureCache,
        settings: crate::renderer::config::RenderSettings,
        vsync: bool,
        debug_draw: crate::renderer::config::DebugDraw,
        frame: ViewFrame,
    ) -> Self {
        let ViewFrame {
            camera,
            viewport,
            uptime_milliseconds,
            delta_time,
        } = frame;
        let shading = camera.effective_shading;
        let active_camera = camera.entity;
        let has_skinned_meshes = !scene.render_skinned_meshes.is_empty();
        Self {
            hdr_skybox_data: Vec::new(),
            scene_world,
            settings,
            vsync,
            debug_draw,
            scene,
            mesh_cache,
            texture_cache,
            owned: Default::default(),
            view: ViewInputs {
                active_camera: Some(active_camera),
                uptime_milliseconds,
                delta_time,
                viewport_size: Some(viewport),
                scene_target_size: None,
                camera_tile_rects: HashMap::new(),
                camera_tile_render_iteration: 0,
                active_viewport_rect: None,
            },
            frame: FrameInputs {
                commands: Vec::new(),
                cameras: Vec::new(),
                active_camera_frame: Some(camera),
                fallback_shading: shading,
                pick_request: None,
                has_skinned_meshes,
                dirty_world_spheres: Vec::new(),
                global_dirty_signal: true,
                mouse_position: nalgebra_glm::Vec2::zeros(),
                force_render_cameras: std::collections::HashSet::new(),
                debug_lines: LinesFrameInputs::default(),
                background_clear_color: None,
            },
            dirty_meshes: Default::default(),
            dirty_skinned_meshes: Default::default(),
            mesh_frame_state: Some(nightshade_render::resources::mesh_state::MeshRenderState {
                full_rebuild_needed: true,
                ..Default::default()
            }),
        }
    }
}

pub struct SingleViewHost {
    pub scene_world: nightshade::ecs::World,
    pub state: crate::renderer::config::RendererState,
    pub mesh_cache: nightshade_render::resources::mesh_cache::MeshCache,
    pub texture_cache: nightshade_render::resources::texture_cache::TextureCache,
    pub settings: crate::renderer::config::RenderSettings,
    pub vsync: bool,
    pub debug_draw: crate::renderer::config::DebugDraw,
}

pub fn compose_single_view(host: &mut SingleViewHost, frame: ViewFrame) -> RenderInputs {
    RenderInputs::single_view(
        std::mem::replace(&mut host.scene_world, nightshade::ecs::World::tracking()),
        std::mem::take(&mut host.state),
        std::mem::take(&mut host.mesh_cache),
        std::mem::take(&mut host.texture_cache),
        std::mem::take(&mut host.settings),
        host.vsync,
        std::mem::take(&mut host.debug_draw),
        frame,
    )
}

pub fn restore_single_view(host: &mut SingleViewHost, inputs: RenderInputs) {
    *host = SingleViewHost {
        scene_world: inputs.scene_world,
        state: inputs.scene,
        mesh_cache: inputs.mesh_cache,
        texture_cache: inputs.texture_cache,
        settings: inputs.settings,
        vsync: inputs.vsync,
        debug_draw: inputs.debug_draw,
    };
}
