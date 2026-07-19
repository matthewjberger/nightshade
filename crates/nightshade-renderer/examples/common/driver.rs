//! A tiny reusable host for the `nightshade-renderer` capability examples.
//!
//! Every frame the host spawns the scene into the renderer's own world, hands
//! it one composed [`RenderInputs`], and reads back a [`FrameOutputs`]. This
//! module is that host in miniature, a pure renderer client with no engine layer
//! above it. It owns a winit window, drives a fly camera, and composes the
//! per-frame input from a small mutable [`Scene`] the example fills. Each example
//! is a `setup` that populates the scene once and an `update` run every frame.
//!
//! Controls: W/A/S/D to move, Q/E down/up, arrow keys to look.
//!
//! Almost every helper here is a thin wrapper over renderer API
//! (`RenderInputs::single_view`, `CameraFrameInputs::perspective`,
//! `RenderLight::directional`, `RenderMaterials::add`, ...), so the host owns
//! little more than the window loop and the fly camera.
#![allow(dead_code)]

use std::collections::HashSet;
use std::sync::Arc;

use glm::{Mat4, Vec3};
use nalgebra_glm as glm;

use freecs::Entity;
use nightshade_renderer::asset_id::TextureId;
use nightshade_renderer::config::{
    EffectiveShading, InstancedObjectData, RenderDecalData, RenderLight, RenderSettings,
    RenderWaterData, RendererState,
};
use nightshade_renderer::geometry::Mesh;
use nightshade_renderer::material::{Material, MaterialTextureIds};
use nightshade_renderer::mesh_cache::{MeshCache, mesh_cache_insert};
use nightshade_renderer::particles::{ParticleEmitter, ParticleTextureUpload};
use nightshade_renderer::render_layer::RenderLayer;
use nightshade_renderer::render_world::{
    CullingMask, Layer, MaterialId, MeshName, MorphWeights, RenderEmitter, ShadowCaster, Transform,
    Visible, register_render_components,
};
use nightshade_renderer::texture_data::{SamplerSettings, TextureUsage};
use nightshade_renderer::ui_data::UiRect;
use nightshade_renderer::wgpu::WgpuRenderer;
use nightshade_renderer::wgpu::frame::render_frame;
use nightshade_renderer::wgpu::presentation::install_presentation_passes;
#[cfg(feature = "screenshot")]
use nightshade_renderer::wgpu::presentation::{poll_screenshot_readback, request_screenshot_copy};
use nightshade_renderer::wgpu::render_configs::{
    CameraFrameInputs, CameraProjectionParams, RenderInputs, RendererCommand, ViewFrame,
};
use nightshade_renderer::wgpu::texture_cache::{TextureCache, texture_cache_reserve_id};
use nightshade_renderer::wgpu::texture_uploads::{
    MaterialTextureUploadRequest, upload_material_texture,
};

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
    pub world: freecs::dynamic::DynWorld,
    pub mesh_cache: MeshCache,
    pub texture_cache: TextureCache,
    pub settings: RenderSettings,
    pub camera: Camera,
    pub commands: Vec<RendererCommand>,
    pub ui_rects: Vec<UiRect>,
    pending_textures: Vec<PendingTexture>,
    particle_textures: Vec<ParticleTextureUpload>,
    next_id: u32,
    camera_entity: Entity,
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
            world: freecs::dynamic::DynWorld::from_registry(register_render_components()),
            mesh_cache: MeshCache::default(),
            texture_cache: TextureCache::default(),
            settings: RenderSettings::default(),
            camera: Camera::default(),
            commands: Vec::new(),
            ui_rects: Vec::new(),
            pending_textures: Vec::new(),
            particle_textures: Vec::new(),
            next_id: 1,
            camera_entity: Entity {
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

    fn next_entity(&mut self) -> Entity {
        let entity = Entity {
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

    /// Adds a material and returns its id.
    pub fn material(&mut self, material: Material) -> u32 {
        self.state.render_materials.add(material)
    }

    /// Adds a material with resolved texture ids (from [`Scene::texture`]).
    pub fn textured_material(&mut self, material: Material, textures: MaterialTextureIds) -> u32 {
        self.state.render_materials.add_textured(material, textures)
    }

    /// Reserves a texture id under `name` and queues its RGBA pixels for upload
    /// once the renderer is live. `color: true` for sRGB color data, `false`
    /// for linear (normal, roughness). The id goes into a [`MaterialTextureIds`]
    /// slot or a decal's texture name.
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

    /// Spawns a static mesh entity into the renderer's own ECS world with a
    /// material and world transform — one `spawn`, no coordinated map writes.
    pub fn object(&mut self, mesh: &str, material: u32, transform: Mat4) -> Entity {
        let entity = self.world.spawn((
            Transform(transform),
            MeshName(mesh.to_string()),
            MaterialId(material),
            Visible(true),
            MorphWeights([0.0; 8]),
            CullingMask(u32::MAX),
            Layer(RenderLayer::WORLD),
        ));
        self.world.add_tag_type::<ShadowCaster>(entity);
        Entity {
            id: entity.id,
            generation: entity.generation,
        }
    }

    /// Spawns one instanced-mesh entity drawing `transforms` copies of `mesh`
    /// with a single material, in one upload.
    pub fn instanced(&mut self, mesh: &str, material: u32, transforms: &[Mat4]) -> Entity {
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
        let entity = self.world.spawn((
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
            MaterialId(material),
        ));
        self.world.add_tag_type::<ShadowCaster>(entity);
        entity
    }

    /// Updates an existing entity's world transform.
    pub fn set_transform(&mut self, entity: Entity, transform: Mat4) {
        self.world.set(entity, Transform(transform));
    }

    /// Moves an existing light to a new world transform.
    pub fn set_light_transform(&mut self, entity: Entity, transform: Mat4) {
        self.set_transform(entity, transform);
    }

    fn spawn_light(&mut self, light: RenderLight) -> Entity {
        let entity = self.world.spawn((light.light, Transform(light.transform)));
        Entity {
            id: entity.id,
            generation: entity.generation,
        }
    }

    /// Sets an existing entity's morph-target weights.
    pub fn set_morph_weights(&mut self, entity: Entity, weights: [f32; 8]) {
        self.world.set(entity, MorphWeights(weights));
    }

    /// A shadow-casting directional light (the sun) pointing along `direction`.
    pub fn sun(&mut self, direction: Vec3, color: Vec3, intensity: f32) -> Entity {
        self.spawn_light(RenderLight::directional(
            Entity::default(),
            direction,
            color,
            intensity,
        ))
    }

    /// A point light at `position` reaching out to `range`.
    pub fn point_light(
        &mut self,
        position: Vec3,
        color: Vec3,
        intensity: f32,
        range: f32,
    ) -> Entity {
        self.spawn_light(RenderLight::point(
            Entity::default(),
            position,
            color,
            intensity,
            range,
        ))
    }

    /// A spot light at `position` aimed along `direction`, with `[inner, outer]`
    /// cone half-angles in radians.
    pub fn spot_light(
        &mut self,
        position: Vec3,
        direction: Vec3,
        color: Vec3,
        intensity: f32,
        range: f32,
        cone: [f32; 2],
    ) -> Entity {
        self.spawn_light(RenderLight::spot(
            Entity::default(),
            position,
            direction,
            color,
            intensity,
            range,
            cone,
        ))
    }

    /// Uploads a particle sprite into a texture-array slot the emitters sample
    /// (emitters default to slot 0). Without one, particles draw invisibly.
    pub fn particle_texture(&mut self, slot: u32, rgba: Vec<u8>, width: u32, height: u32) {
        self.particle_textures.push(ParticleTextureUpload {
            slot,
            rgba_data: rgba,
            width,
            height,
        });
    }

    /// Adds a GPU particle emitter. The emitter's `position` field is its world
    /// origin.
    pub fn emitter(&mut self, emitter: ParticleEmitter) -> Entity {
        let entity = self.world.spawn((RenderEmitter(emitter),));
        Entity {
            id: entity.id,
            generation: entity.generation,
        }
    }

    /// Spawns a projected decal into the world.
    pub fn decal(&mut self, decal: RenderDecalData, transform: Mat4) {
        self.world
            .spawn((decal, Transform(transform), Visible(true)));
    }

    /// Adds a water body and enables the water pass.
    pub fn water(&mut self, water: RenderWaterData, transform: Mat4) {
        self.world.spawn((water, Transform(transform)));
        self.settings.water_enabled = true;
    }

    /// Queues a retained-UI rectangle for this frame. UI is immediate here:
    /// push the rects an example wants every frame from its `update`.
    pub fn rect(&mut self, rect: UiRect) {
        self.ui_rects.push(rect);
    }

    fn compose(&mut self, width: u32, height: u32, delta_time: f32) -> RenderInputs {
        // The particle pass spawns from each emitter's accumulated_spawn counter,
        // which the host advances every frame by spawn_rate * dt.
        self.world
            .query::<&mut RenderEmitter>()
            .for_each(|_entity, emitter| {
                emitter.0.accumulated_spawn += emitter.0.spawn_rate * delta_time;
            });

        let aspect = width as f32 / height.max(1) as f32;
        let camera = CameraFrameInputs::perspective(
            self.camera_entity,
            self.camera.view(),
            self.camera.position,
            CameraProjectionParams {
                z_near: self.camera.z_near,
                z_far: self.camera.z_far,
                y_fov_rad: self.camera.y_fov_rad,
                aspect,
                orthographic: None,
            },
            EffectiveShading::from_settings(&self.settings, &Default::default()),
        );
        let uptime = (self.time * 1000.0) as u64;

        let scene_world = std::mem::replace(&mut self.world, freecs::dynamic::DynWorld::new());
        let mut inputs = RenderInputs::single_view(
            scene_world,
            std::mem::take(&mut self.state),
            std::mem::take(&mut self.mesh_cache),
            std::mem::take(&mut self.texture_cache),
            std::mem::take(&mut self.settings),
            Default::default(),
            ViewFrame {
                camera,
                viewport: (width.max(1), height.max(1)),
                uptime_milliseconds: uptime,
                delta_time,
            },
        );
        inputs.pending_particle_textures = std::mem::take(&mut self.particle_textures);
        inputs.ui.rects = std::mem::take(&mut self.ui_rects);
        inputs.frame.commands = std::mem::take(&mut self.commands);
        inputs
    }

    fn restore(&mut self, inputs: RenderInputs) {
        self.state = inputs.scene;
        self.world = inputs.scene_world;
        self.mesh_cache = inputs.mesh_cache;
        self.texture_cache = inputs.texture_cache;
        self.settings = inputs.settings;
    }
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
    shot: Option<std::path::PathBuf>,
    shot_frames: u32,
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

    /// Renders a fixed number of frames offscreen and writes the final one to a
    /// PNG. Used by `NS_SHOT=<path>` so a scene can be captured headless (the
    /// GPU-driven passes need a few frames of warmup before the first draws
    /// land, so this renders many).
    fn capture(&mut self, renderer: &mut WgpuRenderer, window: &Window, path: &std::path::Path) {
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);
        let state = self.state.as_mut().expect("setup ran");
        for _ in 0..self.shot_frames.max(1) {
            let delta_time = 1.0 / 60.0;
            self.scene.time += delta_time;
            let time = self.scene.time;
            (self.update)(
                &mut self.scene,
                state,
                &Frame {
                    delta_time,
                    time,
                    width,
                    height,
                },
            );
            let mut inputs = self.scene.compose(width, height, delta_time);
            let mut outputs = Default::default();
            if let Err(error) = render_frame(renderer, &mut inputs, &mut outputs) {
                eprintln!("render_frame failed: {error}");
            }
            self.scene.restore(inputs);
        }
        #[cfg(feature = "screenshot")]
        {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            request_screenshot_copy(renderer, Some(path.to_path_buf()), None);
            let mut guard = 0;
            while renderer.screenshot.pending && guard < 600 {
                let _ = renderer
                    .device
                    .poll(nightshade_renderer::wgpu::PollType::wait_indefinitely());
                poll_screenshot_readback(renderer);
                guard += 1;
            }
            println!("wrote {}", path.display());
        }
        #[cfg(not(feature = "screenshot"))]
        {
            let _ = path;
            eprintln!("NS_SHOT needs the screenshot feature: --features wgpu,hdr,screenshot");
        }
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
            .with_visible(self.shot.is_none())
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

        if let Some(path) = self.shot.clone() {
            self.capture(&mut renderer, &window, &path);
            event_loop.exit();
            return;
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
    let shot = std::env::var("NS_SHOT").ok().map(std::path::PathBuf::from);
    let shot_frames = std::env::var("NS_SHOT_FRAMES")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(120);
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
        shot,
        shot_frames,
    };
    event_loop.run_app(&mut host).expect("event loop failed");
}
