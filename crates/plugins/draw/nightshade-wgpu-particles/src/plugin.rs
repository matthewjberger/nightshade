use nightshade::app::App;
use nightshade::plugin::Plugin;
use nightshade_wgpu::RenderAppExt;

#[derive(Default)]
pub struct ParticleRenderPlugin;

impl Plugin for ParticleRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_render_graph_sub_pass(crate::systems::particles::install_particle_pass);
        app.on_pre_render(crate::systems::particles::sync_particle_pass);
    }
}
