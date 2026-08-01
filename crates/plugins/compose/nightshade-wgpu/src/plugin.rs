use nightshade::app::App;
use nightshade::plugin::Plugin;

#[derive(Default)]
pub struct RenderPlugin {
    pub vsync: Option<bool>,
    pub frame_rate_limit: Option<f32>,
}

impl Plugin for RenderPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
            nightshade_platform::contract::input::Input => "PlatformPlugin",
            nightshade_platform::contract::window::Window => "PlatformPlugin",
            nightshade_render::data::config::RenderSettings => "GraphicsPlugin",
            nightshade_render::resources::viewport::Viewport => "GraphicsPlugin",
            nightshade_render::resources::mesh_cache::MeshCache => "GraphicsPlugin",
            nightshade_render::resources::mesh_state::MeshRenderState => "GraphicsPlugin",
            nightshade_render::resources::material_registry::MaterialRegistry => "GraphicsPlugin",
            nightshade_render::resources::texture_cache::TextureCache => "GraphicsPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.add_plugin(nightshade_graphics::GraphicsPlugin);
        app.world
            .res_mut::<nightshade_graphics::resources::loading::LoadingState>()
            .pipeline
            .blocks = nightshade_render::data::decoded_image::BlockFamily::Texels;
        app.world
            .res_mut::<nightshade_render::resources::viewport::Viewport>()
            .create_renderer = true;
        app.world
            .res_mut::<nightshade_render::data::config::RendererState>()
            .poses_on_gpu = true;
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
        app.set_runner("RenderPlugin", crate::systems::run::run_windowed);
    }
}
