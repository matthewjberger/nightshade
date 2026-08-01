use nightshade::ecs::ResourceMap;

pub fn install_render_resources(resources: &mut ResourceMap) {
    hold::<crate::resources::capture::ScheduledCapture>(resources);
    hold::<crate::resources::texture_sources::TextureSources>(resources);
    hold::<crate::resources::cleanup::CleanupState>(resources);
    hold::<crate::data::config::DebugDraw>(resources);
    hold::<crate::data::config::FramePacing>(resources);
    hold::<crate::data::config::RenderSettings>(resources);
    hold::<crate::data::config::RendererState>(resources);
    hold::<crate::data::config::RenderViews>(resources);
    hold::<crate::resources::mesh_cache::MeshCache>(resources);
    hold::<crate::resources::light_state::LightSyncState>(resources);
    hold::<crate::resources::mesh_state::MeshRenderState>(resources);
    hold::<crate::resources::render_sync::RenderSceneSync>(resources);
    hold::<crate::resources::texture_cache::TextureCache>(resources);
    hold::<crate::resources::gpu_picking::GpuPicking>(resources);
    hold::<crate::resources::material_registry::MaterialRegistry>(resources);
    hold::<crate::resources::scratch::Scratch>(resources);
    hold::<crate::resources::outline::Outline>(resources);
    hold::<crate::resources::viewport::Viewport>(resources);
    hold::<crate::components::mesh::InstancedMeshMatrices>(resources);
}

fn hold<T: Default + Send + Sync + 'static>(resources: &mut ResourceMap) {
    if resources.get::<T>().is_none() {
        resources.insert(T::default());
    }
}
