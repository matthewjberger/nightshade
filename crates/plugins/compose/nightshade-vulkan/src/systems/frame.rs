use crate::commands::{run_install_hooks, run_pre_render_hooks};
use crate::renderer::VulkanRenderer;
use crate::renderer::picking::SAMPLE_SIZE;
use crate::resources::RenderHooks;
use nightshade::World;
use nightshade::ecs::{ResourceMap, Storage};
use nightshade_render::data::config::{RenderViews, RendererState, build_render_view};
use nightshade_render::queries::lights::build_render_lighting;
use nightshade_render::resources::gpu_picking::GpuPicking;
use nightshade_window::renderer::{RenderError, Renderer};

pub struct VulkanFrameRenderer {
    pub renderer: VulkanRenderer,
    pub hooks: RenderHooks,
    pub frames: u64,
    pub window: std::sync::Arc<nightshade_platform::winit::window::Window>,
    pub vsync: bool,
}

fn lost_the_device(error: &(dyn std::error::Error + 'static)) -> bool {
    error.downcast_ref::<ash::vk::Result>() == Some(&ash::vk::Result::ERROR_DEVICE_LOST)
}

fn put_the_world_back(world: &mut World) {
    use nightshade_graphics::commands::loading::{queue_decoded_texture, queue_encoded_texture};
    use nightshade_render::resources::texture_sources::{TextureSourceData, TextureSources};
    let sources: Vec<(
        String,
        nightshade_render::resources::texture_sources::TextureSourceBytes,
    )> = world
        .res::<TextureSources>()
        .iter()
        .map(|(name, source)| (name.clone(), source.clone()))
        .collect();
    let mut load = nightshade_graphics::resources::loading::texture_load(&mut world.resources);
    for (name, source) in sources {
        match source.data {
            TextureSourceData::Rgba {
                rgba,
                width,
                height,
            } => {
                queue_decoded_texture(
                    &mut load,
                    name,
                    rgba,
                    width,
                    height,
                    source.usage,
                    source.sampler,
                );
            }
            TextureSourceData::Png(bytes) => {
                queue_encoded_texture(&mut load, name, bytes, source.usage, source.sampler);
            }
        }
    }
    world
        .res_mut::<nightshade_render::resources::mesh_state::MeshRenderState>()
        .full_rebuild_needed = true;
}

const JITTER_PERIOD: u64 = 8;

fn halton(index: u32, base: u32) -> f32 {
    let mut result = 0.0;
    let mut fraction = 1.0;
    let mut current = index;
    while current > 0 {
        fraction /= base as f32;
        result += fraction * (current % base) as f32;
        current /= base;
    }
    result
}

fn camera_jitter(frames: u64, screen_size: (u32, u32), wanted: bool) -> [f32; 2] {
    if !wanted {
        return [0.0, 0.0];
    }
    let index = (frames % JITTER_PERIOD) as u32 + 1;
    [
        (halton(index, 2) - 0.5) * 2.0 / screen_size.0.max(1) as f32,
        (halton(index, 3) - 0.5) * 2.0 / screen_size.1.max(1) as f32,
    ]
}

impl Renderer for VulkanFrameRenderer {
    fn configure(&mut self) -> Result<(), RenderError> {
        run_install_hooks(&mut self.hooks, &mut self.renderer);
        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) -> Result<(), RenderError> {
        crate::renderer::resize_surface(&mut self.renderer, width, height)
            .map_err(|error| RenderError(error.to_string()))
    }

    fn release_surface(&mut self) {}

    fn recreate_surface(
        &mut self,
        _window: std::sync::Arc<nightshade_platform::winit::window::Window>,
        width: u32,
        height: u32,
    ) -> Result<(), RenderError> {
        crate::renderer::resize_surface(&mut self.renderer, width, height)
            .map_err(|error| RenderError(error.to_string()))
    }

    fn update(
        &mut self,
        storage: &Storage,
        resources: &mut ResourceMap,
    ) -> Result<(), RenderError> {
        let shading = resources
            .get::<RenderViews>()
            .map(|views| {
                views
                    .active
                    .as_ref()
                    .map(|camera| camera.effective_shading)
                    .unwrap_or(views.fallback_shading)
            })
            .unwrap_or_default();
        let wanted_scale = shading.render_scale.clamp(0.25, 4.0);
        if (wanted_scale - self.renderer.render_scale).abs() > 0.001 {
            self.renderer.render_scale = wanted_scale;
            let extent = self.renderer.swapchain.extent;
            if let Err(error) =
                crate::renderer::resize_surface(&mut self.renderer, extent.width, extent.height)
            {
                tracing::error!("the scene was not drawn at the new size: {error}");
            }
        }
        self.renderer.paper_white = shading.hdr_paper_white.max(1.0);
        let wanted_hdr = shading.hdr_output;
        if wanted_hdr != self.renderer.hdr_output {
            self.renderer.hdr_output = wanted_hdr;
            self.renderer.display = crate::renderer::display::display_light(self.window.as_ref());
            let extent = self.renderer.swapchain.extent;
            if let Err(error) =
                crate::renderer::resize_surface(&mut self.renderer, extent.width, extent.height)
            {
                tracing::error!("the screen was not made again for the new range: {error}");
            }
        }
        self.renderer.viewport_rect = shading
            .viewport_rect
            .map(|rect| [rect.x, rect.y, rect.width, rect.height]);
        let extent = self.renderer.swapchain.render_extent;
        let screen_size = (extent.width, extent.height);
        let camera = resources
            .get::<RenderViews>()
            .and_then(|views| views.active.clone());
        let jitter = camera_jitter(
            self.frames,
            screen_size,
            shading.anti_aliasing.is_temporal(),
        );
        self.renderer.view = camera
            .as_ref()
            .and_then(|camera| build_render_view(camera, jitter, screen_size));
        self.frames = self.frames.wrapping_add(1);
        let lighting = build_render_lighting(
            storage,
            camera.as_ref(),
            crate::renderer::scene::pass::MAX_LIGHTS,
            crate::renderer::scene::pass::MAX_AREA_LIGHTS,
            crate::renderer::scene::shadow::SLOT_SIZE as f32,
        );
        let settled = camera
            .as_ref()
            .and_then(|camera| camera.matrices.as_ref())
            .map(|matrices| matrices.projection * matrices.view);
        let state = resources.res_mut::<RendererState>();
        state.render_lighting = Some(lighting);
        state.taa_jitter = jitter;
        if let Some(settled) = settled {
            state.prev_view_projection = state.view_projection;
            state.view_projection = settled.into();
        }
        Ok(())
    }

    fn prepare(&mut self, storage: &mut Storage, resources: &mut ResourceMap) {
        settle_pick(&mut self.renderer, resources);
        self.renderer.picking.wanted = resources
            .res_mut::<GpuPicking>()
            .take_pending_request()
            .map(|request| (request.screen_x, request.screen_y));
        run_pre_render_hooks(&mut self.hooks, &mut self.renderer, storage, resources);
    }

    fn render(&mut self, world: &mut World) -> Result<(), RenderError> {
        if self.renderer.is_torn_down() || self.renderer.context.is_lost() {
            return self.make_again(world);
        }
        match crate::renderer::render_frame(&mut self.renderer) {
            Ok(()) => {}
            Err(error) if lost_the_device(error.as_ref()) || self.renderer.context.is_lost() => {
                return self.make_again(world);
            }
            Err(error) => return Err(RenderError(error.to_string())),
        }
        let timings = world.res_mut::<crate::resources::GpuTimings>();
        timings.frame_milliseconds = self.renderer.timings.frame_milliseconds;
        timings.passes.clear();
        timings
            .passes
            .extend(
                self.renderer
                    .timings
                    .passes
                    .iter()
                    .map(|pass| crate::resources::PassTiming {
                        name: pass.name,
                        milliseconds: pass.milliseconds,
                    }),
            );
        let counted = self.count_what_was_drawn();
        let drawn = world.res_mut::<crate::resources::GpuDrawn>();
        drawn.clusters = counted.clusters;
        drawn.triangles = counted.triangles;
        Ok(())
    }
}

impl VulkanFrameRenderer {
    fn count_what_was_drawn(&self) -> crate::renderer::scene::pass::DrawnCounts {
        let Some(pass) = self
            .renderer
            .passes
            .iter()
            .find(|(_, pass)| pass.name() == crate::renderer::scene::pass::SCENE_PASS)
            .map(|(_, pass)| pass.as_ref())
        else {
            return crate::renderer::scene::pass::DrawnCounts::default();
        };
        (pass as &dyn std::any::Any)
            .downcast_ref::<crate::renderer::scene::pass::ScenePass>()
            .map(crate::renderer::scene::pass::drawn_counts)
            .unwrap_or_default()
    }

    fn make_again(&mut self, world: &mut World) -> Result<(), RenderError> {
        tracing::error!("the device was lost, so the renderer is made again");
        let extent = self.renderer.swapchain.extent;
        let kept = std::mem::take(&mut self.renderer.kept);
        self.renderer.tear_down();
        let made = crate::renderer::create_renderer(
            self.window.as_ref(),
            extent.width,
            extent.height,
            self.vsync,
        )
        .map_err(|error| RenderError(format!("the renderer was not made again: {error}")))?;
        self.renderer = made;
        run_install_hooks(&mut self.hooks, &mut self.renderer);
        crate::systems::scene::put_back_what_was_kept(&mut self.renderer, kept);
        put_the_world_back(world);
        Ok(())
    }
}

fn settle_pick(renderer: &mut VulkanRenderer, resources: &mut ResourceMap) {
    let Some(samples) = renderer.picking.arrived.take() else {
        return;
    };
    let window = samples.window;
    resources.res_mut::<GpuPicking>().set_depth_samples(
        samples.depth,
        samples.entity_ids,
        SAMPLE_SIZE,
        SAMPLE_SIZE,
        window.center.0,
        window.center.1,
    );
    let Some(matrices) = resources
        .get::<RenderViews>()
        .and_then(|views| views.active.as_ref().and_then(|camera| camera.matrices))
    else {
        return;
    };
    let inverse = (matrices.projection * matrices.view)
        .try_inverse()
        .unwrap_or_else(nalgebra_glm::Mat4::identity);
    resources.res_mut::<GpuPicking>().compute_result(
        &inverse,
        window.extent.0 as f32,
        window.extent.1 as f32,
    );
}
