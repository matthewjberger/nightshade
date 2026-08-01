use nightshade::app::App;
use nightshade::plugin::Plugin;
use nightshade_wgpu::RenderAppExt;

#[derive(Default)]
pub struct ShadingPlugin;

impl Plugin for ShadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_shader_modules(crate::systems::modules::publish);
    }
}
