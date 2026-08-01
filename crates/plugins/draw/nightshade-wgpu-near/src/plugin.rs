use nightshade::app::App;
use nightshade::plugin::Plugin;
use nightshade_wgpu::RenderAppExt;

#[derive(Default)]
pub struct NearRenderPlugin;

impl Plugin for NearRenderPlugin {
    fn build(&self, app: &mut App) {
        app.world
            .resource_or_insert_default::<crate::resources::NearGpu>();
        app.on_pre_render(crate::systems::near::sync_near_pass);
    }
}
