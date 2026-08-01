use nightshade::app::App;
use nightshade::plugin::Plugin;
use nightshade_wgpu::RenderAppExt;

#[derive(Default)]
pub struct AcousticsRenderPlugin;

impl Plugin for AcousticsRenderPlugin {
    fn build(&self, app: &mut App) {
        app.world
            .resource_or_insert_default::<crate::resources::AcousticsGpu>();
        app.on_pre_render(crate::systems::acoustics::sync_acoustics_pass);
    }
}
