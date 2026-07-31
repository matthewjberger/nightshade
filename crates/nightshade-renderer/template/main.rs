//! A standalone `nightshade-renderer` starter in one file: open a window, build
//! a small scene in the renderer's own nightshade_ecs world, and draw it every frame —
//! no engine layer, no shared harness. Copy this file plus the dependencies in
//! `Cargo.toml` into a new binary crate and it runs.
//!
//! The renderer-facing state lives in a [`SingleViewHost`]; each frame builds a
//! [`ViewFrame`] for the camera and hands both to [`render_single_view`], which
//! draws the scene and hands the state back for the next frame.
//!
//! Controls: W/A/S/D move, Q/E down/up, mouse to look, Esc quits.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

use glm::{Mat4, Vec3};
use nalgebra_glm as glm;

use nightshade_ecs::Entity;
use nightshade_ecs::dynamic::DynWorld;
use nightshade_renderer::config::{
    Atmosphere, EffectiveShading, RenderLight, RenderSettings, RendererState,
};
use nightshade_renderer::material::Material;
use nightshade_renderer::mesh_cache::{MeshCache, mesh_cache_insert};
use nightshade_renderer::procedural_meshes::{
    create_cube_mesh, create_plane_mesh, create_sphere_mesh,
};
use nightshade_renderer::render_layer::RenderLayer;
use nightshade_renderer::render_world::{
    CullingMask, Layer, MaterialId, MeshName, MorphWeights, ShadowCaster, Transform, Visible,
    register_render_components,
};
use nightshade_renderer::wgpu::WgpuRenderer;
use nightshade_renderer::wgpu::frame::render_single_view;
use nightshade_renderer::wgpu::presentation::install_presentation_passes;
use nightshade_renderer::wgpu::render_configs::{
    CameraFrameInputs, CameraProjectionParams, SingleViewHost, ViewFrame,
};
use nightshade_renderer::wgpu::texture_cache::TextureCache;

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

/// The renderer keys per-camera state by this handle; the scene never spawns it.
const CAMERA: Entity = Entity {
    id: 0,
    generation: 0,
};

struct Camera {
    position: Vec3,
    yaw: f32,
    pitch: f32,
}

fn camera_forward(camera: &Camera) -> Vec3 {
    glm::vec3(
        camera.pitch.cos() * camera.yaw.sin(),
        camera.pitch.sin(),
        -camera.pitch.cos() * camera.yaw.cos(),
    )
}

fn camera_advance(camera: &mut Camera, delta_time: f32, keys: &HashSet<KeyCode>) {
    let held = |code| keys.contains(&code) as i32 as f32;
    let forward = camera_forward(camera);
    let right = glm::normalize(&glm::cross(&forward, &glm::vec3(0.0, 1.0, 0.0)));
    let motion = forward * (held(KeyCode::KeyW) - held(KeyCode::KeyS))
        + right * (held(KeyCode::KeyD) - held(KeyCode::KeyA))
        + glm::vec3(0.0, 1.0, 0.0) * (held(KeyCode::KeyE) - held(KeyCode::KeyQ));
    if glm::length(&motion) > 0.0 {
        camera.position += glm::normalize(&motion) * 6.0 * delta_time;
    }
}

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<WgpuRenderer>,
    host: SingleViewHost,
    spinners: Vec<(Entity, Vec3)>,
    camera: Camera,
    keys: HashSet<KeyCode>,
    last_frame: Instant,
    time: f32,
}

impl App {
    fn render(&mut self) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        let size = window.inner_size();
        let (width, height) = (size.width.max(1), size.height.max(1));
        let delta_time = self.last_frame.elapsed().as_secs_f32().min(0.1);
        self.last_frame = Instant::now();
        self.time += delta_time;

        camera_advance(&mut self.camera, delta_time, &self.keys);
        for (entity, base) in &self.spinners {
            let spin =
                glm::translation(base) * glm::rotation(self.time * 0.6, &glm::vec3(0.0, 1.0, 0.0));
            self.host.scene_world.set(*entity, Transform(spin));
        }

        let target = self.camera.position + camera_forward(&self.camera);
        let camera = CameraFrameInputs::perspective(
            CAMERA,
            glm::look_at(&self.camera.position, &target, &glm::vec3(0.0, 1.0, 0.0)),
            self.camera.position,
            CameraProjectionParams {
                z_near: 0.1,
                z_far: 1000.0,
                y_fov_rad: std::f32::consts::FRAC_PI_4,
                aspect: width as f32 / height as f32,
                orthographic: None,
            },
            EffectiveShading::from_settings(&self.host.settings, &Default::default()),
        );
        let view = ViewFrame {
            camera,
            viewport: (width, height),
            uptime_milliseconds: (self.time * 1000.0) as u64,
            delta_time,
        };
        if let Some(renderer) = self.renderer.as_mut() {
            render_single_view(renderer, &mut self.host, view);
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("nightshade-renderer standalone")
                        .with_inner_size(winit::dpi::PhysicalSize::new(1280, 720)),
                )
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
        window
            .set_cursor_grab(CursorGrabMode::Locked)
            .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined))
            .ok();
        window.set_cursor_visible(false);
        self.renderer = Some(renderer);
        self.window = Some(window);
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            self.camera.yaw -= delta.0 as f32 * 0.003;
            self.camera.pitch = (self.camera.pitch - delta.1 as f32 * 0.003).clamp(-1.5, 1.5);
        }
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
            } => match (code, state) {
                (KeyCode::Escape, _) => event_loop.exit(),
                (_, ElementState::Pressed) => {
                    self.keys.insert(code);
                }
                (_, ElementState::Released) => {
                    self.keys.remove(&code);
                }
            },
            WindowEvent::Resized(size) if size.width > 0 && size.height > 0 => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize_surface(size.width, size.height).ok();
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

fn main() {
    let mut host = SingleViewHost {
        scene_world: DynWorld::from_registry(register_render_components()),
        state: RendererState::default(),
        mesh_cache: MeshCache::default(),
        texture_cache: TextureCache::default(),
        settings: RenderSettings {
            atmosphere: Atmosphere::Sky,
            show_sky: true,
            ambient_light: [0.3, 0.35, 0.45, 0.4],
            ..Default::default()
        },
        debug_draw: Default::default(),
    };

    let sun = RenderLight::directional(
        CAMERA,
        glm::vec3(-0.6, -1.0, -0.4),
        glm::vec3(1.0, 0.96, 0.88),
        4.0,
    );
    host.scene_world.spawn((sun.light, Transform(sun.transform)));

    mesh_cache_insert(&mut host.mesh_cache, "floor".to_string(), create_plane_mesh(40.0));
    mesh_cache_insert(&mut host.mesh_cache, "cube".to_string(), create_cube_mesh());
    mesh_cache_insert(&mut host.mesh_cache, "sphere".to_string(), create_sphere_mesh(0.6, 24));

    let floor = host.state.render_materials.add(Material {
        base_color: [0.72, 0.72, 0.74, 1.0],
        roughness: 0.9,
        ..Material::default()
    });
    host.scene_world.spawn((
        Transform(Mat4::identity()),
        MeshName("floor".to_string()),
        MaterialId(floor),
        Visible(true),
        MorphWeights([0.0; 8]),
        CullingMask(u32::MAX),
        Layer(RenderLayer::WORLD),
    ));

    let shapes = [
        ("cube", [0.85, 0.3, 0.25, 1.0]),
        ("sphere", [0.25, 0.5, 0.85, 1.0]),
    ];
    let mut spinners = Vec::new();
    for (index, (mesh, color)) in shapes.into_iter().enumerate() {
        let base = glm::vec3((index as f32 - 0.5) * 2.4, 0.9, 0.0);
        let material = host.state.render_materials.add(Material {
            base_color: color,
            roughness: 0.4,
            ..Material::default()
        });
        let entity = host.scene_world.spawn((
            Transform(glm::translation(&base)),
            MeshName(mesh.to_string()),
            MaterialId(material),
            Visible(true),
            MorphWeights([0.0; 8]),
            CullingMask(u32::MAX),
            Layer(RenderLayer::WORLD),
        ));
        host.scene_world.add_tag_type::<ShadowCaster>(entity);
        spinners.push((entity, base));
    }

    let event_loop = EventLoop::new().expect("failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App {
        window: None,
        renderer: None,
        host,
        spinners,
        camera: Camera {
            position: glm::vec3(0.0, 4.0, 11.0),
            yaw: 0.0,
            pitch: -0.25,
        },
        keys: HashSet::new(),
        last_frame: Instant::now(),
        time: 0.0,
    };
    event_loop.run_app(&mut app).expect("event loop failed");
}
