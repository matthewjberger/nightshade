use nightshade::app::App;
use nightshade::plugin::Plugin;

pub struct PrefabPlugin;

impl Plugin for PrefabPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_render::resources::mesh_cache::MeshCache => "GraphicsPlugin",
            nightshade_render::resources::mesh_state::MeshRenderState => "GraphicsPlugin",
            nightshade_render::resources::material_registry::MaterialRegistry => "GraphicsPlugin",
            nightshade_render::resources::texture_cache::TextureCache => "GraphicsPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.add_plugin(nightshade_scene::plugin::ScenePlugin);
    }
}
