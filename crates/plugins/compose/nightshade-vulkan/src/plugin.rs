use crate::commands::VulkanAppExt;
use nightshade::app::App;
use nightshade::plugin::Plugin;

pub fn install_render_resources(app: &mut App) {
    app.add_plugin(nightshade_paint::PaintPlugin);
    app.world
        .resource_or_insert_default::<crate::resources::TextMeshCache>();
    app.world
        .resource_or_insert_default::<crate::resources::GpuTimings>();
    app.world
        .resource_or_insert_default::<crate::resources::GpuDrawn>();
}

#[derive(Default)]
pub struct RenderPlugin {
    pub vsync: Option<bool>,
    pub frame_rate_limit: Option<f32>,
}

impl Plugin for RenderPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
            nightshade_render::data::config::RenderSettings => "GraphicsPlugin",
            nightshade_render::resources::viewport::Viewport => "GraphicsPlugin",
            nightshade_render::resources::mesh_state::MeshRenderState => "GraphicsPlugin",
            nightshade_render::resources::material_registry::MaterialRegistry => "GraphicsPlugin",
            nightshade_render::resources::texture_cache::TextureCache => "GraphicsPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        crate::commands::claim_hooks_slot(app);
        app.add_plugin(nightshade_graphics::GraphicsPlugin);
        install_render_resources(app);
        app.world
            .res_mut::<nightshade_graphics::resources::loading::LoadingState>()
            .pipeline
            .blocks = nightshade_render::data::decoded_image::BlockFamily::Bc;
        app.world
            .res_mut::<nightshade_render::resources::viewport::Viewport>()
            .create_renderer = true;
        if let Some(vsync) = self.vsync {
            app.world
                .res_mut::<nightshade_render::data::config::FramePacing>()
                .vsync_enabled = vsync;
        }
        if let Some(limit) = self.frame_rate_limit {
            app.world
                .res_mut::<nightshade_render::data::config::FramePacing>()
                .frame_rate_limit = Some(limit);
        }
        app.install_vulkan_pass(crate::systems::sky::install);
        app.install_vulkan_pass(crate::systems::scene::install);
        app.install_vulkan_pass(crate::systems::grid::install);
        app.install_vulkan_pass(crate::systems::lines::install);
        app.install_vulkan_pass(crate::systems::sprites::install);
        app.install_vulkan_pass(crate::systems::particles::install);
        app.install_vulkan_pass(crate::systems::text::install);
        app.install_vulkan_pass(crate::systems::decals::install);
        app.install_vulkan_pass(crate::systems::post::install);
        app.install_vulkan_pass(crate::systems::outline::install);
        app.install_vulkan_pass(crate::systems::paint::install);
        app.on_vulkan_pre_render(crate::systems::scene::drain_commands);
        app.on_vulkan_pre_render(crate::systems::scene::upload_geometry);
        app.on_vulkan_pre_render(crate::systems::scene::upload_textures);
        app.on_vulkan_pre_render(crate::systems::sky::light_from_the_sky);
        app.on_vulkan_pre_render(crate::systems::scene::upload_scene);
        app.on_vulkan_pre_render(crate::systems::sky::upload_sky);
        app.on_vulkan_pre_render(crate::systems::grid::upload_grid);
        app.on_vulkan_pre_render(crate::systems::lines::upload_lines);
        app.on_vulkan_pre_render(crate::systems::text::build_text_meshes);
        app.on_vulkan_pre_render(crate::systems::text::upload_text);
        app.on_vulkan_pre_render(crate::systems::sprites::upload_sprites);
        app.on_vulkan_pre_render(crate::systems::particles::upload_particles);
        app.on_vulkan_pre_render(crate::systems::decals::upload_decals);
        app.on_vulkan_pre_render(crate::systems::post::upload_post);
        app.on_vulkan_pre_render(crate::systems::outline::upload_outline);
        app.on_vulkan_pre_render(crate::systems::paint::gather);
        app.world
            .resource_or_insert_default::<crate::resources::SpawnOffsets>();
        app.set_runner("RenderPlugin", crate::systems::run::run_windowed);
    }
}
