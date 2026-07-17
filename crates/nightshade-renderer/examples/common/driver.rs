//! A tiny reusable host for the `nightshade-renderer` capability examples.
//!
//! The renderer never sees a scene graph or an ECS: every frame the host hands
//! it one composed [`RenderInputs`] and reads back a [`FrameOutputs`]. This
//! module is that host in miniature, with nothing from the engine layer above
//! it. It owns a winit window, brings the renderer up, drives a fly camera, and
//! composes the per-frame input from a small mutable [`Scene`] the example
//! fills. Each example becomes a `setup` that populates the scene once and an
//! `update` that changes it per frame.
//!
//! Controls: W/A/S/D to move, Q/E down/up, arrow keys to look.
//!
//! Each example is its own binary that includes this module and uses a subset
//! of its helpers, so the unused-per-example remainder is not dead across the
//! set.
#![allow(dead_code)]

use std::collections::HashSet;
use std::sync::Arc;

use glm::{Mat4, Vec2, Vec3};
use nalgebra_glm as glm;

use nightshade_renderer::asset_id::TextureId;
use nightshade_renderer::config::{
    DebugDraw, DynamicObjectState, EffectiveShading, InstancedObjectData, RenderAreaLightShape,
    RenderLight, RenderLightData, RenderLightType, RenderMaterialEntry, RenderParticleEmitter,
    RenderSettings, RenderWater, RenderWaterData, RendererState, ViewportUpdateMode,
};
use nightshade_renderer::entity::RenderEntity;
use nightshade_renderer::geometry::Mesh;
use nightshade_renderer::material::{AlphaMode, Material, MaterialTextureIds};
use nightshade_renderer::mesh_cache::{MeshCache, mesh_cache_insert};
use nightshade_renderer::mesh_state::MeshRenderStateInner;
use nightshade_renderer::particles::ParticleEmitter;
use nightshade_renderer::render_layer::RenderLayer;
use nightshade_renderer::texture_data::{SamplerSettings, TextureUsage};
use nightshade_renderer::ui_data::UiRect;
use nightshade_renderer::wgpu::WgpuRenderer;
use nightshade_renderer::wgpu::frame::render_frame;
use nightshade_renderer::wgpu::presentation::install_presentation_passes;
use nightshade_renderer::wgpu::render_configs::{
    CameraFrameInputs, CameraMatricesInputs, CameraProjectionParams, FrameInputs, LinesFrameInputs,
    RenderInputs, RendererCommand, UiFrameInputs, ViewInputs,
};
use nightshade_renderer::wgpu::texture_cache::{TextureCache, texture_cache_reserve_id};
use nightshade_renderer::wgpu::texture_uploads::{
    MaterialTextureUploadRequest, upload_material_texture,
};
use nightshade_renderer::wind::Wind;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

const INITIAL_WIDTH: u32 = 1280;
const INITIAL_HEIGHT: u32 = 720;

/// A first-person fly camera. Yaw rotates about world +Y, pitch about the
/// camera's right axis; zero yaw looks down -Z.
pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub y_fov_rad: f32,
    pub z_near: f32,
    pub z_far: f32,
    pub move_speed: f32,
    pub look_speed: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 1.5, 6.0),
            yaw: 0.0,
            pitch: -0.15,
            y_fov_rad: std::f32::consts::FRAC_PI_4,
            z_near: 0.1,
            z_far: 1000.0,
            move_speed: 6.0,
            look_speed: 1.5,
        }
    }
}

impl Camera {
    fn forward(&self) -> Vec3 {
        Vec3::new(
            self.pitch.cos() * self.yaw.sin(),
            self.pitch.sin(),
            -self.pitch.cos() * self.yaw.cos(),
        )
    }

    fn view(&self) -> Mat4 {
        let target = self.position + self.forward();
        glm::look_at(&self.position, &target, &Vec3::new(0.0, 1.0, 0.0))
    }

    fn advance(&mut self, delta_time: f32, keys: &HashSet<KeyCode>) {
        let mut look = 0.0;
        let mut pitch = 0.0;
        if keys.contains(&KeyCode::ArrowLeft) {
            look += 1.0;
        }
        if keys.contains(&KeyCode::ArrowRight) {
            look -= 1.0;
        }
        if keys.contains(&KeyCode::ArrowUp) {
            pitch += 1.0;
        }
        if keys.contains(&KeyCode::ArrowDown) {
            pitch -= 1.0;
        }
        self.yaw += look * self.look_speed * delta_time;
        self.pitch = (self.pitch + pitch * self.look_speed * delta_time).clamp(
            -std::f32::consts::FRAC_PI_2 + 0.01,
            std::f32::consts::FRAC_PI_2 - 0.01,
        );

        let forward = self.forward();
        let right = forward.cross(&Vec3::new(0.0, 1.0, 0.0)).normalize();
        let mut motion = Vec3::zeros();
        if keys.contains(&KeyCode::KeyW) {
            motion += forward;
        }
        if keys.contains(&KeyCode::KeyS) {
            motion -= forward;
        }
        if keys.contains(&KeyCode::KeyD) {
            motion += right;
        }
        if keys.contains(&KeyCode::KeyA) {
            motion -= right;
        }
        if keys.contains(&KeyCode::KeyE) {
            motion += Vec3::new(0.0, 1.0, 0.0);
        }
        if keys.contains(&KeyCode::KeyQ) {
            motion -= Vec3::new(0.0, 1.0, 0.0);
        }
        if motion.norm() > 0.0 {
            self.position += motion.normalize() * self.move_speed * delta_time;
        }
    }
}

/// The host-owned scene the example fills. The persistent pieces
/// (`state`, `mesh_cache`, `texture_cache`, `settings`) move into the frame's
/// [`RenderInputs`] and back out each frame, so nothing is cloned per frame.
pub struct Scene {
    pub state: RendererState,
    pub mesh_cache: MeshCache,
    pub texture_cache: TextureCache,
    pub settings: RenderSettings,
    pub camera: Camera,
    pub commands: Vec<RendererCommand>,
    pub ui_rects: Vec<UiRect>,
    pending_textures: Vec<PendingTexture>,
    next_id: u32,
    camera_entity: RenderEntity,
    time: f32,
}

struct PendingTexture {
    texture: TextureId,
    rgba: Vec<u8>,
    width: u32,
    height: u32,
    usage: TextureUsage,
}

impl Scene {
    fn new() -> Self {
        let state = RendererState {
            occlusion_culling_enabled: false,
            ..RendererState::default()
        };
        Self {
            state,
            mesh_cache: MeshCache::default(),
            texture_cache: TextureCache::default(),
            settings: RenderSettings::default(),
            camera: Camera::default(),
            commands: Vec::new(),
            ui_rects: Vec::new(),
            pending_textures: Vec::new(),
            next_id: 1,
            camera_entity: RenderEntity {
                id: 0,
                generation: 0,
            },
            time: 0.0,
        }
    }

    /// Seconds since the first frame, handy for driving animation.
    pub fn time(&self) -> f32 {
        self.time
    }

    fn next_entity(&mut self) -> RenderEntity {
        let entity = RenderEntity {
            id: self.next_id,
            generation: 0,
        };
        self.next_id += 1;
        entity
    }

    /// Registers named geometry the mesh passes resolve by name.
    pub fn mesh(&mut self, name: &str, mesh: Mesh) {
        mesh_cache_insert(&mut self.mesh_cache, name.to_string(), mesh);
    }

    /// Adds a material and returns its id. Material id `N` is `entries[N - 1]`;
    /// id 0 is the renderer's built-in default.
    pub fn material(&mut self, material: Material) -> u32 {
        self.push_material(material, MaterialTextureIds::default())
    }

    /// Adds a material with resolved texture ids (from [`Scene::texture`]).
    pub fn textured_material(&mut self, material: Material, textures: MaterialTextureIds) -> u32 {
        self.push_material(material, textures)
    }

    fn push_material(&mut self, material: Material, texture_ids: MaterialTextureIds) -> u32 {
        let transparent = matches!(material.alpha_mode, AlphaMode::Blend | AlphaMode::Dither);
        let mask = matches!(material.alpha_mode, AlphaMode::Mask);
        let double_sided = material.double_sided;
        self.state
            .render_materials
            .entries
            .push(RenderMaterialEntry {
                material,
                texture_ids,
            });
        let id = self.state.render_materials.entries.len() as u32;
        if transparent {
            self.state.render_materials.transparent_ids.insert(id);
        }
        if mask {
            self.state.render_materials.mask_ids.insert(id);
        }
        if double_sided {
            self.state.render_materials.double_sided_ids.insert(id);
        }
        self.state.render_materials.generation += 1;
        id
    }

    /// Reserves a texture id under `name` and queues its RGBA pixels for upload
    /// once the renderer is live. Pass `color: true` for sRGB color data (base
    /// color, emissive) and `false` for linear data (normal, roughness). The
    /// returned id goes into a [`MaterialTextureIds`] slot or a decal's texture
    /// name.
    pub fn texture(
        &mut self,
        name: &str,
        rgba: Vec<u8>,
        width: u32,
        height: u32,
        color: bool,
    ) -> TextureId {
        let texture = texture_cache_reserve_id(&mut self.texture_cache, name.to_string());
        let usage = if color {
            TextureUsage::Color
        } else {
            TextureUsage::Linear
        };
        self.pending_textures.push(PendingTexture {
            texture,
            rgba,
            width,
            height,
            usage,
        });
        texture
    }

    /// Spawns a static mesh entity with a material and world transform.
    pub fn object(&mut self, mesh: &str, material: u32, transform: Mat4) -> RenderEntity {
        let entity = self.next_entity();
        self.state.render_regular_mesh_entities.push(entity);
        self.state
            .render_object_meshes
            .insert(entity, mesh.to_string());
        self.state.render_dynamic_objects.insert(
            entity,
            DynamicObjectState {
                visible: 1,
                morph_weights: [0.0; 8],
                transform,
                culling_mask: u32::MAX,
                is_overlay: 0,
                render_layer: RenderLayer::WORLD,
            },
        );
        self.state
            .render_materials
            .entity_to_id
            .insert(entity, material);
        self.state.render_shadow_casters.meshes.push(entity);
        entity
    }

    /// Spawns one instanced-mesh entity drawing `transforms` copies of `mesh`
    /// with a single material, in one upload.
    pub fn instanced(&mut self, mesh: &str, material: u32, transforms: &[Mat4]) -> RenderEntity {
        let entity = self.next_entity();
        let world_models = transforms.iter().map(|matrix| (*matrix).into()).collect();
        let world_normals = transforms
            .iter()
            .map(|matrix| {
                let normal = glm::mat4_to_mat3(&glm::transpose(&glm::inverse(matrix)));
                [
                    [normal[(0, 0)], normal[(1, 0)], normal[(2, 0)], 0.0],
                    [normal[(0, 1)], normal[(1, 1)], normal[(2, 1)], 0.0],
                    [normal[(0, 2)], normal[(1, 2)], normal[(2, 2)], 0.0],
                ]
            })
            .collect();
        self.state.render_instanced_objects.insert(
            entity,
            InstancedObjectData {
                world_models,
                world_normals,
                local_matrices: transforms.to_vec(),
                custom_tints: vec![[1.0, 1.0, 1.0, 1.0]; transforms.len()],
                mesh_name: mesh.to_string(),
                render_layer: RenderLayer::WORLD,
                visible: 1,
                parent_transform: Mat4::identity(),
            },
        );
        self.state
            .render_materials
            .entity_to_id
            .insert(entity, material);
        self.state.render_shadow_casters.instanced.push(entity);
        entity
    }

    /// Updates an existing entity's world transform.
    pub fn set_transform(&mut self, entity: RenderEntity, transform: Mat4) {
        if let Some(dynamic) = self.state.render_dynamic_objects.get_mut(&entity) {
            dynamic.transform = transform;
        }
    }

    /// Moves an existing light to a new world transform.
    pub fn set_light_transform(&mut self, entity: RenderEntity, transform: Mat4) {
        if let Some(&index) = self.state.render_lights.entity_to_index.get(&entity) {
            self.state.render_lights.lights[index].transform = transform;
        }
    }

    /// Sets an existing entity's morph-target weights.
    pub fn set_morph_weights(&mut self, entity: RenderEntity, weights: [f32; 8]) {
        if let Some(dynamic) = self.state.render_dynamic_objects.get_mut(&entity) {
            dynamic.morph_weights = weights;
        }
    }

    fn add_light(&mut self, data: RenderLightData, transform: Mat4) -> RenderEntity {
        let entity = self.next_entity();
        let index = self.state.render_lights.lights.len();
        self.state.render_lights.lights.push(RenderLight {
            entity,
            light: data,
            transform,
        });
        self.state
            .render_lights
            .entity_to_index
            .insert(entity, index);
        entity
    }

    /// A shadow-casting directional light (the sun) pointing along `direction`.
    pub fn sun(&mut self, direction: Vec3, color: Vec3, intensity: f32) -> RenderEntity {
        let mut data = light_data(RenderLightType::Directional, color, intensity, 0.0);
        data.cast_shadows = true;
        self.add_light(data, direction_transform(Vec3::zeros(), direction))
    }

    /// A point light at `position` reaching out to `range`.
    pub fn point_light(
        &mut self,
        position: Vec3,
        color: Vec3,
        intensity: f32,
        range: f32,
    ) -> RenderEntity {
        let data = light_data(RenderLightType::Point, color, intensity, range);
        self.add_light(data, glm::translation(&position))
    }

    /// A spot light at `position` aimed along `direction` with a soft cone
    /// given as `[inner, outer]` half-angles in radians.
    pub fn spot_light(
        &mut self,
        position: Vec3,
        direction: Vec3,
        color: Vec3,
        intensity: f32,
        range: f32,
        cone: [f32; 2],
    ) -> RenderEntity {
        let mut data = light_data(RenderLightType::Spot, color, intensity, range);
        data.inner_cone_angle = cone[0];
        data.outer_cone_angle = cone[1];
        self.add_light(data, direction_transform(position, direction))
    }

    /// Adds a GPU particle emitter. The emitter's `position` field is its world
    /// origin.
    pub fn emitter(&mut self, emitter: ParticleEmitter) -> RenderEntity {
        let entity = self.next_entity();
        self.state
            .render_particle_emitters
            .push(RenderParticleEmitter { entity, emitter });
        entity
    }

    /// Adds a water body and enables the water pass.
    pub fn water(&mut self, water: RenderWaterData, transform: Mat4) {
        self.state
            .render_water
            .push(RenderWater { water, transform });
        self.settings.water_enabled = true;
    }

    /// Queues a retained-UI rectangle for this frame. UI is immediate here:
    /// push the rects an example wants every frame from its `update`.
    pub fn rect(&mut self, rect: UiRect) {
        self.ui_rects.push(rect);
    }

    fn effective_shading(&self) -> EffectiveShading {
        let settings = &self.settings;
        EffectiveShading {
            unlit_mode: settings.unlit_mode,
            bloom_enabled: settings.bloom_enabled,
            ssao_enabled: settings.ssao_enabled,
            ssgi_enabled: settings.ssgi_enabled,
            ssr_enabled: settings.ssr_enabled,
            show_normals: false,
            show_bounding_volumes: false,
            show_wireframe: false,
            selection_outline_enabled: false,
            flat_shading_color: None,
            shadow_depth_enabled: true,
            lines_enabled: false,
            color_grading: settings.color_grading,
            depth_of_field: settings.depth_of_field,
            bloom_intensity: settings.bloom_intensity,
            bloom_threshold: settings.bloom_threshold,
            bloom_knee: settings.bloom_knee,
            bloom_filter_radius: settings.bloom_filter_radius,
            ssao_radius: settings.ssao_radius,
            ssao_bias: settings.ssao_bias,
            ssao_intensity: settings.ssao_intensity,
            ssao_sample_count: settings.ssao_sample_count,
            ssao_visualization: false,
            ssao_blur_depth_threshold: settings.ssao_blur_depth_threshold,
            ssao_blur_normal_power: settings.ssao_blur_normal_power,
            ssgi_radius: settings.ssgi_radius,
            ssgi_intensity: settings.ssgi_intensity,
            ssgi_max_steps: settings.ssgi_max_steps,
            ssr_max_steps: settings.ssr_max_steps,
            ssr_thickness: settings.ssr_thickness,
            ssr_max_distance: settings.ssr_max_distance,
            ssr_stride: settings.ssr_stride,
            ssr_fade_start: settings.ssr_fade_start,
            ssr_fade_end: settings.ssr_fade_end,
            ssr_intensity: settings.ssr_intensity,
            ambient_light: settings.ambient_light,
            culling_mask: !0,
            fog: settings.fog,
            atmosphere: settings.atmosphere,
        }
    }

    fn compose(&mut self, width: u32, height: u32, delta_time: f32) -> RenderInputs {
        let aspect = width as f32 / height.max(1) as f32;
        let shading = self.effective_shading();
        let camera = CameraFrameInputs {
            entity: self.camera_entity,
            matrices: Some(CameraMatricesInputs {
                view: self.camera.view(),
                projection: perspective(
                    self.camera.y_fov_rad,
                    aspect,
                    self.camera.z_near,
                    self.camera.z_far,
                ),
                camera_position: self.camera.position,
            }),
            projection: CameraProjectionParams {
                z_near: self.camera.z_near,
                z_far: self.camera.z_far,
                y_fov_rad: self.camera.y_fov_rad,
                aspect,
                orthographic: None,
            },
            constrained_aspect: None,
            effective_shading: shading,
            update_mode: ViewportUpdateMode::Always,
            world_transform: None,
        };

        let has_skinned = !self.state.render_skinned_meshes.is_empty();
        let uptime = (self.time * 1000.0) as u64;

        RenderInputs {
            settings: std::mem::take(&mut self.settings),
            debug_draw: DebugDraw::default(),
            scene: std::mem::take(&mut self.state),
            ibl_views: Default::default(),
            shadow_atlas: Default::default(),
            wind: Wind::default(),
            #[cfg(feature = "grass")]
            grass: Default::default(),
            #[cfg(feature = "terrain")]
            terrain: Default::default(),
            #[cfg(feature = "terrain")]
            terrain_render: Default::default(),
            mesh_cache: std::mem::take(&mut self.mesh_cache),
            texture_cache: std::mem::take(&mut self.texture_cache),
            pending_particle_textures: Vec::new(),
            ui: UiFrameInputs {
                rects: std::mem::take(&mut self.ui_rects),
                rect_entities: Vec::new(),
                images: Vec::new(),
                text_meshes: Vec::new(),
                render_slots: Default::default(),
                background_color: None,
            },
            view: ViewInputs {
                active_camera: Some(self.camera_entity),
                uptime_milliseconds: uptime,
                delta_time,
                viewport_size: Some((width.max(1), height.max(1))),
                camera_tile_rects: Default::default(),
                camera_tile_render_iteration: 0,
                active_viewport_rect: None,
            },
            world_id: 0,
            frame: FrameInputs {
                commands: std::mem::take(&mut self.commands),
                cameras: Vec::new(),
                active_camera_frame: Some(camera),
                fallback_shading: shading,
                pick_request: None,
                has_skinned_meshes: has_skinned,
                mesh_frame_state: Some(MeshRenderStateInner {
                    full_rebuild_needed: true,
                    ..Default::default()
                }),
                dirty_world_spheres: Vec::new(),
                global_dirty_signal: true,
                mouse_position: Vec2::zeros(),
                force_render_cameras: HashSet::new(),
                debug_lines: LinesFrameInputs::default(),
            },
        }
    }

    fn restore(&mut self, inputs: RenderInputs) {
        self.state = inputs.scene;
        self.mesh_cache = inputs.mesh_cache;
        self.texture_cache = inputs.texture_cache;
        self.settings = inputs.settings;
    }
}

fn light_data(
    light_type: RenderLightType,
    color: Vec3,
    intensity: f32,
    range: f32,
) -> RenderLightData {
    RenderLightData {
        light_type,
        color,
        intensity,
        range,
        inner_cone_angle: 0.0,
        outer_cone_angle: std::f32::consts::FRAC_PI_4,
        cast_shadows: false,
        shadow_bias: 0.0015,
        shadow_resolution: 2048,
        shadow_distance: 100.0,
        cookie_texture: None,
        area_shape: RenderAreaLightShape::Rectangle,
        area_width: 1.0,
        area_height: 1.0,
        area_radius: 0.5,
        area_two_sided: false,
        area_emissive_texture: None,
        shadow_normal_bias: 0.02,
        shadow_softness: 1.0,
    }
}

/// A world transform whose forward axis (the renderer reads `-column2`) is
/// `direction`, placed at `position`.
fn direction_transform(position: Vec3, direction: Vec3) -> Mat4 {
    let forward = direction.normalize();
    let world_up = if forward.y.abs() > 0.99 {
        Vec3::new(0.0, 0.0, 1.0)
    } else {
        Vec3::new(0.0, 1.0, 0.0)
    };
    let right = forward.cross(&world_up).normalize();
    let up = right.cross(&forward);
    Mat4::new(
        right.x, up.x, -forward.x, position.x, right.y, up.y, -forward.y, position.y, right.z,
        up.z, -forward.z, position.z, 0.0, 0.0, 0.0, 1.0,
    )
}

/// The reverse-Z perspective matrix the passes expect (near maps to depth 1,
/// far to depth 0). A stock `glm::perspective` will not line up with the depth
/// the passes read.
fn perspective(y_fov_rad: f32, aspect: f32, z_near: f32, z_far: f32) -> Mat4 {
    let focal_length = 1.0 / (y_fov_rad / 2.0).tan();
    Mat4::new(
        focal_length / aspect,
        0.0,
        0.0,
        0.0,
        0.0,
        focal_length,
        0.0,
        0.0,
        0.0,
        0.0,
        z_near / (z_far - z_near),
        z_near * z_far / (z_far - z_near),
        0.0,
        0.0,
        -1.0,
        0.0,
    )
}

/// Per-frame context handed to the example's `update`.
pub struct Frame {
    pub delta_time: f32,
    pub time: f32,
    pub width: u32,
    pub height: u32,
}

type SetupFn<State> = Box<dyn FnOnce(&mut Scene) -> State>;

struct Host<State, Update> {
    title: String,
    window: Option<Arc<Window>>,
    renderer: Option<WgpuRenderer>,
    scene: Scene,
    state: Option<State>,
    setup: Option<SetupFn<State>>,
    update: Update,
    keys: HashSet<KeyCode>,
    last_frame: std::time::Instant,
}

impl<State, Update> Host<State, Update>
where
    Update: FnMut(&mut Scene, &mut State, &Frame),
{
    fn render(&mut self) {
        let (Some(window), Some(renderer), Some(state)) = (
            self.window.as_ref(),
            self.renderer.as_mut(),
            self.state.as_mut(),
        ) else {
            return;
        };
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            return;
        }

        let now = std::time::Instant::now();
        let delta_time = (now - self.last_frame).as_secs_f32().min(0.1);
        self.last_frame = now;
        self.scene.time += delta_time;

        self.scene.camera.advance(delta_time, &self.keys);
        let time = self.scene.time;
        (self.update)(
            &mut self.scene,
            state,
            &Frame {
                delta_time,
                time,
                width: size.width,
                height: size.height,
            },
        );

        let mut inputs = self.scene.compose(size.width, size.height, delta_time);
        let mut outputs = Default::default();
        if let Err(error) = render_frame(renderer, &mut inputs, &mut outputs) {
            eprintln!("render_frame failed: {error}");
        }
        self.scene.restore(inputs);
    }
}

impl<State, Update> ApplicationHandler for Host<State, Update>
where
    Update: FnMut(&mut Scene, &mut State, &Frame),
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attributes = Window::default_attributes()
            .with_title(self.title.clone())
            .with_inner_size(winit::dpi::PhysicalSize::new(INITIAL_WIDTH, INITIAL_HEIGHT));
        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .expect("failed to create window"),
        );
        let size = window.inner_size();
        let mut renderer = pollster::block_on(WgpuRenderer::new_async(
            window.clone(),
            size.width.max(1),
            size.height.max(1),
        ))
        .expect("failed to create renderer");
        install_presentation_passes(&mut renderer).expect("failed to install presentation passes");

        let setup = self.setup.take().expect("setup already ran");
        self.state = Some(setup(&mut self.scene));

        for pending in std::mem::take(&mut self.scene.pending_textures) {
            upload_material_texture(
                &mut renderer,
                &mut self.scene.texture_cache,
                MaterialTextureUploadRequest {
                    texture: pending.texture,
                    rgba_data: &pending.rgba,
                    width: pending.width,
                    height: pending.height,
                    usage: pending.usage,
                    sampler: SamplerSettings::DEFAULT,
                },
            );
        }

        self.window = Some(window);
        self.renderer = Some(renderer);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        ..
                    },
                ..
            } => {
                if code == KeyCode::Escape {
                    event_loop.exit();
                } else if state == ElementState::Pressed {
                    self.keys.insert(code);
                } else {
                    self.keys.remove(&code);
                }
            }
            WindowEvent::Resized(size) if size.width > 0 && size.height > 0 => {
                if let Some(renderer) = self.renderer.as_mut()
                    && let Err(error) = renderer.resize_surface(size.width, size.height)
                {
                    eprintln!("resize_surface failed: {error}");
                }
            }
            WindowEvent::RedrawRequested => {
                self.render();
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

/// Opens a window, brings the renderer up, runs `setup` once to populate the
/// scene, then calls `update` every frame before composing and rendering.
pub fn run<State: 'static>(
    title: &str,
    setup: impl FnOnce(&mut Scene) -> State + 'static,
    update: impl FnMut(&mut Scene, &mut State, &Frame) + 'static,
) {
    let event_loop = EventLoop::new().expect("failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut host = Host {
        title: title.to_string(),
        window: None,
        renderer: None,
        scene: Scene::new(),
        state: None,
        setup: Some(Box::new(setup)),
        update,
        keys: HashSet::new(),
        last_frame: std::time::Instant::now(),
    };
    event_loop.run_app(&mut host).expect("event loop failed");
}
