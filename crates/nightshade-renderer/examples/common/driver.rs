//! A thin windowing and frame-loop harness for the nightshade-renderer
//! examples. It owns only the window, a fly camera, and the render loop; each
//! example fills a plain [`Scene`] of renderer data using the raw renderer and
//! freecs API (`world.spawn`, `RenderMaterials::add`, `mesh_cache_insert`,
//! `world.set`) and the loop composes one [`RenderInputs`] per frame and draws
//! it. Everything here is a free function or a plain data struct.
//!
//! Controls: W/A/S/D to move, Q/E down/up, arrow keys to look.

use std::collections::HashSet;
use std::sync::Arc;

use glm::{Mat4, Vec3};
use nalgebra_glm as glm;

use freecs::Entity;
use freecs::dynamic::DynWorld;
use nightshade_renderer::config::{
    EffectiveShading, RenderLight, RenderLightData, RenderSettings, RendererState,
};
use nightshade_renderer::mesh_cache::MeshCache;
use nightshade_renderer::render_world::{Transform, register_render_components};
use nightshade_renderer::wgpu::WgpuRenderer;
use nightshade_renderer::wgpu::frame::render_frame;
use nightshade_renderer::wgpu::presentation::install_presentation_passes;
#[cfg(feature = "screenshot")]
use nightshade_renderer::wgpu::presentation::{poll_screenshot_readback, request_screenshot_copy};
use nightshade_renderer::wgpu::render_configs::{
    CameraFrameInputs, CameraProjectionParams, RenderInputs, ViewFrame,
};
use nightshade_renderer::wgpu::texture_cache::TextureCache;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

const INITIAL_WIDTH: u32 = 1280;
const INITIAL_HEIGHT: u32 = 720;

// The renderer keys its per-camera state by this handle. The scene never spawns
// the camera, so any stable id works.
const CAMERA_ENTITY: Entity = Entity {
    id: 0,
    generation: 0,
};

/// A first-person fly camera as plain data. Yaw rotates about world +Y, pitch
/// about the camera's right axis; zero yaw looks down -Z.
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

fn camera_forward(camera: &Camera) -> Vec3 {
    Vec3::new(
        camera.pitch.cos() * camera.yaw.sin(),
        camera.pitch.sin(),
        -camera.pitch.cos() * camera.yaw.cos(),
    )
}

fn camera_view(camera: &Camera) -> Mat4 {
    let target = camera.position + camera_forward(camera);
    glm::look_at(&camera.position, &target, &Vec3::new(0.0, 1.0, 0.0))
}

fn camera_advance(camera: &mut Camera, delta_time: f32, keys: &HashSet<KeyCode>) {
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
    camera.yaw += look * camera.look_speed * delta_time;
    camera.pitch = (camera.pitch + pitch * camera.look_speed * delta_time).clamp(
        -std::f32::consts::FRAC_PI_2 + 0.01,
        std::f32::consts::FRAC_PI_2 - 0.01,
    );

    let forward = camera_forward(camera);
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
        camera.position += motion.normalize() * camera.move_speed * delta_time;
    }
}

/// The directional-light components (a sun) an example spawns into the scene
/// world with `world.spawn(directional_light(..))`. A reusable helper because
/// `RenderLightData` carries far more fields than a sun sets.
pub fn directional_light(direction: Vec3, color: Vec3, intensity: f32) -> (RenderLightData, Transform) {
    let light = RenderLight::directional(CAMERA_ENTITY, direction, color, intensity);
    (light.light, Transform(light.transform))
}

/// Plain data: the renderer-side scene an example builds and the loop feeds the
/// renderer each frame. Every field is public; fill it with the raw renderer and
/// freecs API. The persistent pieces move into the frame's [`RenderInputs`] and
/// back out, so nothing is cloned per frame.
pub struct Scene {
    pub world: DynWorld,
    pub state: RendererState,
    pub mesh_cache: MeshCache,
    pub texture_cache: TextureCache,
    pub settings: RenderSettings,
    pub camera: Camera,
}

fn new_scene() -> Scene {
    Scene {
        world: DynWorld::from_registry(register_render_components()),
        state: RendererState {
            occlusion_culling_enabled: false,
            ..RendererState::default()
        },
        mesh_cache: MeshCache::default(),
        texture_cache: TextureCache::default(),
        settings: RenderSettings::default(),
        camera: Camera::default(),
    }
}

/// What an example does: build the scene once in [`Example::setup`], then
/// optionally advance it each frame in [`Example::update`], which receives the
/// seconds elapsed since the first frame. A static example implements only
/// `setup` and inherits the no-op `update`.
pub trait Example {
    type State;
    fn setup(scene: &mut Scene) -> Self::State;
    fn update(_: &mut Scene, _: &mut Self::State, _: f32) {}
}

fn compose(scene: &mut Scene, width: u32, height: u32, delta_time: f32, uptime_ms: u64) -> RenderInputs {
    let aspect = width as f32 / height.max(1) as f32;
    let camera = CameraFrameInputs::perspective(
        CAMERA_ENTITY,
        camera_view(&scene.camera),
        scene.camera.position,
        CameraProjectionParams {
            z_near: scene.camera.z_near,
            z_far: scene.camera.z_far,
            y_fov_rad: scene.camera.y_fov_rad,
            aspect,
            orthographic: None,
        },
        EffectiveShading::from_settings(&scene.settings, &Default::default()),
    );
    RenderInputs::single_view(
        std::mem::replace(&mut scene.world, DynWorld::new()),
        std::mem::take(&mut scene.state),
        std::mem::take(&mut scene.mesh_cache),
        std::mem::take(&mut scene.texture_cache),
        std::mem::take(&mut scene.settings),
        Default::default(),
        ViewFrame {
            camera,
            viewport: (width.max(1), height.max(1)),
            uptime_milliseconds: uptime_ms,
            delta_time,
        },
    )
}

fn restore(scene: &mut Scene, inputs: RenderInputs) {
    scene.world = inputs.scene_world;
    scene.state = inputs.scene;
    scene.mesh_cache = inputs.mesh_cache;
    scene.texture_cache = inputs.texture_cache;
    scene.settings = inputs.settings;
}

struct Host<E: Example> {
    title: String,
    window: Option<Arc<Window>>,
    renderer: Option<WgpuRenderer>,
    scene: Scene,
    state: Option<E::State>,
    keys: HashSet<KeyCode>,
    last_frame: std::time::Instant,
    time: f32,
    shot: Option<std::path::PathBuf>,
    shot_frames: u32,
}

impl<E: Example> Host<E> {
    fn frame(&mut self, width: u32, height: u32, delta_time: f32) {
        self.time += delta_time;
        camera_advance(&mut self.scene.camera, delta_time, &self.keys);
        if let Some(state) = self.state.as_mut() {
            E::update(&mut self.scene, state, self.time);
        }
        let uptime = (self.time * 1000.0) as u64;
        let mut inputs = compose(&mut self.scene, width, height, delta_time, uptime);
        let mut outputs = Default::default();
        if let Some(renderer) = self.renderer.as_mut()
            && let Err(error) = render_frame(renderer, &mut inputs, &mut outputs)
        {
            eprintln!("render_frame failed: {error}");
        }
        restore(&mut self.scene, inputs);
    }

    fn render(&mut self) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            return;
        }
        let now = std::time::Instant::now();
        let delta_time = (now - self.last_frame).as_secs_f32().min(0.1);
        self.last_frame = now;
        self.frame(size.width, size.height, delta_time);
    }

    // Renders a fixed number of frames offscreen and writes the last to a PNG,
    // used by NS_SHOT=<path> for headless capture. The GPU-driven passes need a
    // few frames of warmup before the first draws land, so this renders many.
    fn capture(&mut self, width: u32, height: u32, path: &std::path::Path) {
        for _ in 0..self.shot_frames.max(1) {
            self.frame(width, height, 1.0 / 60.0);
        }
        #[cfg(feature = "screenshot")]
        {
            if let Some(renderer) = self.renderer.as_mut() {
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
                let mut written = 0;
                let mut waited = 0;
                while written == 0 && waited < 5000 {
                    written = std::fs::metadata(path).map(|meta| meta.len()).unwrap_or(0);
                    if written == 0 {
                        std::thread::sleep(std::time::Duration::from_millis(2));
                        waited += 1;
                    }
                }
                if written > 0 {
                    println!("wrote {}", path.display());
                } else {
                    eprintln!("screenshot was not written");
                }
            }
        }
        #[cfg(not(feature = "screenshot"))]
        {
            let _ = path;
            eprintln!("NS_SHOT needs the screenshot feature: --features wgpu,hdr,screenshot");
        }
    }
}

impl<E: Example> ApplicationHandler for Host<E> {
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

        self.renderer = Some(renderer);
        self.state = Some(E::setup(&mut self.scene));

        if let Some(path) = self.shot.clone() {
            self.capture(size.width.max(1), size.height.max(1), &path);
            event_loop.exit();
            return;
        }
        self.window = Some(window);
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

/// Opens a window, brings the renderer up, runs the example's `setup` once, then
/// its `update` every frame before composing and drawing. `NS_SHOT=<path>`
/// renders offscreen and writes a PNG instead of presenting.
pub fn run<E: Example>(title: &str)
where
    E::State: 'static,
{
    let shot = std::env::var("NS_SHOT").ok().map(std::path::PathBuf::from);
    let shot_frames = std::env::var("NS_SHOT_FRAMES")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(120);
    let event_loop = EventLoop::new().expect("failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut host: Host<E> = Host {
        title: title.to_string(),
        window: None,
        renderer: None,
        scene: new_scene(),
        state: None,
        keys: HashSet::new(),
        last_frame: std::time::Instant::now(),
        time: 0.0,
        shot,
        shot_frames,
    };
    event_loop.run_app(&mut host).expect("event loop failed");
}
