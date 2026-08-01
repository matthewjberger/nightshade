use nalgebra_glm::{Vec3, look_at, vec2, vec3, vec4};
use nightshade::ecs::Entity;
use nightshade::prelude::nalgebra_glm;
use nightshade_paint::PaintDrawList;
use nightshade_render::data::config::{EffectiveShading, RenderSettings, RendererState};
use nightshade_render::resources::mesh_cache::MeshCache;
use nightshade_render::resources::texture_cache::TextureCache;
use nightshade_ui::prelude::*;
use nightshade_ui::{default_ui_systems, insert_ui_resources};
use nightshade_wgpu::backend::FrameInputsFn;
use nightshade_wgpu::backend::WgpuRenderer;
use nightshade_wgpu::backend::install_presentation_passes;
use nightshade_wgpu::backend::render_frame;
use nightshade_wgpu::backend::{
    CameraFrameInputs, CameraProjectionParams, SingleViewHost, ViewFrame, compose_single_view,
    restore_single_view,
};
use nightshade_wgpu_paint::systems::paint::{
    install_paint_passes_into, paint_image_pass_mut, paint_pass_mut,
};
use nightshade_window::platform::app::{FrameContext, InitContext, WindowApp, WindowConfig};

const CAMERA: Entity = Entity {
    id: 0,
    generation: 0,
};

struct UiExample {
    renderer: WgpuRenderer,
    hooks: Vec<FrameInputsFn>,
    host: SingleViewHost,
}

impl WindowApp for UiExample {
    fn init(
        storage: &mut nightshade::ecs::Storage,
        resources: &mut nightshade::ecs::ResourceMap,
        context: InitContext,
    ) -> Self {
        let InitContext {
            window,
            width,
            height,
        } = context;

        let mut renderer =
            pollster::block_on(WgpuRenderer::new_async(window.clone(), width, height))
                .expect("failed to create renderer");
        let format = renderer.surface_format;
        let color = renderer.targets.swapchain;
        let depth = renderer.targets.swapchain_depth;
        let (graph, build) = renderer.graph_and_pass_build();
        install_paint_passes_into(graph, build, format, color, depth);
        install_presentation_passes(&mut renderer).expect("failed to install presentation passes");
        nightshade_wgpu::backend::publish_glyph_atlas(&mut renderer);

        insert_ui_resources(resources);
        resources.insert(nightshade_platform::contract::window::Window::default());
        resources.insert(nightshade_platform::contract::input::Input::default());
        resources.insert(nightshade_text::contract::resources::TextState::default());
        resources.insert(nightshade::ecs::HierarchyIndex::default());
        resources.insert(nightshade_platform::contract::time::Time::default());
        {
            let window_resource =
                resources.res_mut::<nightshade_platform::contract::window::Window>();
            window_resource.cached_viewport_size = Some((width, height));
            window_resource.cached_scale_factor = 1.0;
        }
        let runtime = resources.res_mut::<RetainedUiRuntime>();
        runtime.visible = true;
        runtime.background_color = Some(vec4(0.06, 0.07, 0.09, 1.0));
        build_tree(storage, resources);

        let host = SingleViewHost {
            vsync: true,
            scene_world: nightshade::ecs::World::new(),
            state: RendererState {
                occlusion_culling_enabled: false,
                ..RendererState::default()
            },
            mesh_cache: MeshCache::default(),
            texture_cache: TextureCache::default(),
            settings: RenderSettings::default(),
            debug_draw: Default::default(),
        };

        Self {
            renderer,
            hooks: Vec::new(),
            host,
        }
    }

    fn frame(&mut self, ecs: &mut nightshade::ecs::World, context: FrameContext) {
        let FrameContext {
            width,
            height,
            delta_time,
            elapsed,
        } = context;

        {
            let window_resource = ecs.res_mut::<nightshade_platform::contract::window::Window>();
            window_resource.cached_viewport_size = Some((width, height));
        }

        ecs.resources
            .res_mut::<nightshade::ecs::HierarchyIndex>()
            .sync(&mut ecs.storage);
        for mut system in default_ui_systems() {
            system(ecs);
        }

        let position = vec3(0.0, 1.5, 4.0);
        let view = look_at(&position, &vec3(0.0, 0.0, 0.0), &Vec3::new(0.0, 1.0, 0.0));
        let camera = CameraFrameInputs::perspective(
            CAMERA,
            view,
            position,
            CameraProjectionParams {
                z_near: 0.1,
                z_far: 1000.0,
                y_fov_rad: std::f32::consts::FRAC_PI_4,
                aspect: width as f32 / height.max(1) as f32,
                orthographic: None,
            },
            EffectiveShading::from_settings(&self.host.settings, &Default::default()),
        );
        let view_frame = ViewFrame {
            camera,
            viewport: (width.max(1), height.max(1)),
            uptime_milliseconds: (elapsed * 1000.0) as u64,
            delta_time,
        };

        let mut inputs = compose_single_view(&mut self.host, view_frame);
        inputs.frame.cameras.clear();
        inputs.settings.render_world_to_swapchain = false;
        inputs.frame.background_clear_color = ecs.res::<RetainedUiRuntime>().background_color;

        let render_slots = ecs.res::<RenderSlotAllocator>().clone();
        let frame = ecs.res_mut::<RetainedUiFrame>();
        let rects = std::mem::take(&mut frame.rects);
        let rect_entities = std::mem::take(&mut frame.rect_entities);
        let images = std::mem::take(&mut frame.ui_images);
        let text_meshes = std::mem::take(&mut frame.text_meshes);

        let returned_images = paint_image_pass_mut(&mut self.renderer).map(|pass| {
            pass.swap_draw_list(PaintDrawList {
                images,
                render_slots: render_slots.clone(),
                ..Default::default()
            })
        });
        let returned_draws = paint_pass_mut(&mut self.renderer).map(|pass| {
            pass.swap_draw_list(PaintDrawList {
                rects,
                rect_entities,
                text_meshes,
                render_slots,
                ..Default::default()
            })
        });

        let mut outputs = Default::default();
        if let Err(error) = render_frame(
            &mut self.renderer,
            &mut self.hooks,
            &mut inputs,
            &mut outputs,
        ) {
            eprintln!("render_frame failed: {error}");
        }

        let frame = ecs.res_mut::<RetainedUiFrame>();
        if let Some(returned) = returned_draws {
            frame.rects = returned.rects;
            frame.rect_entities = returned.rect_entities;
            frame.text_meshes = returned.text_meshes;
        }
        if let Some(returned) = returned_images {
            frame.ui_images = returned.images;
        }
        restore_single_view(&mut self.host, inputs);
    }

    fn resize(&mut self, width: u32, height: u32) {
        if let Err(error) = self.renderer.resize_surface(width, height) {
            eprintln!("resize_surface failed: {error}");
        }
    }
}

fn build_tree(
    storage: &mut nightshade::ecs::Storage,
    resources: &mut nightshade::ecs::ResourceMap,
) {
    nightshade::ecs::with_commands_over(storage, resources, |_storage, resources, commands| {
        let mut tree = UiTreeBuilder::new(nightshade_ui::commands::builder::ui_build_context(
            resources, commands,
        ));
        let root = tree
            .add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .entity();
        tree.in_parent(root, |tree| {
            tree.add_node()
                .window(
                    Ab(vec2(40.0, 40.0)),
                    Ab(vec2(320.0, 160.0)),
                    Anchor::TopLeft,
                )
                .with_rect(12.0, 2.0, vec4(0.12, 0.13, 0.18, 0.95))
                .entity();
            tree.add_node()
                .window(
                    Ab(vec2(400.0, 60.0)),
                    Ab(vec2(160.0, 160.0)),
                    Anchor::TopLeft,
                )
                .with_rect(80.0, 0.0, vec4(0.85, 0.4, 0.32, 0.95))
                .entity();
            tree.add_node()
                .window(
                    Ab(vec2(120.0, 240.0)),
                    Ab(vec2(260.0, 90.0)),
                    Anchor::TopLeft,
                )
                .with_rect(8.0, 1.0, vec4(0.28, 0.6, 0.5, 0.9))
                .entity();
        });
        tree.finish();
    });
}

fn main() {
    nightshade_window::platform::app::run::<UiExample>(WindowConfig {
        title: String::from("nightshade-ui: window"),
        width: 1280,
        height: 720,
    })
    .expect("event loop failed");
}
