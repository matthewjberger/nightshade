use nightshade::ecs::system_param::{ExtractParam, ResourceNeed};
use nightshade::ecs::{HierarchyIndex, ResourceMap, ResourceSlot, Storage, SystemAccess};
use nightshade_graphics::resources::loading::LoadingState;
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_render::resources::mesh_cache::MeshCache;
use nightshade_render::resources::mesh_state::MeshRenderState;
use nightshade_render::resources::texture_cache::TextureCache;
use nightshade_render::resources::texture_sources::TextureSources;

pub struct PrefabParts<'a> {
    pub meshes: &'a mut MeshCache,
    pub mesh_state: &'a mut MeshRenderState,
    pub materials: &'a mut MaterialRegistry,
    pub textures: &'a mut TextureCache,
    pub sources: &'a mut TextureSources,
    pub loading: &'a mut LoadingState,
    pub hierarchy: &'a mut HierarchyIndex,
}

impl PrefabParts<'_> {
    pub fn reborrow(&mut self) -> PrefabParts<'_> {
        PrefabParts {
            meshes: self.meshes,
            mesh_state: self.mesh_state,
            materials: self.materials,
            textures: self.textures,
            sources: self.sources,
            loading: self.loading,
            hierarchy: self.hierarchy,
        }
    }
}

pub struct PrefabSpawn<'a> {
    pub storage: &'a mut Storage,
    pub parts: PrefabParts<'a>,
}

impl PrefabSpawn<'_> {
    pub fn reborrow(&mut self) -> PrefabSpawn<'_> {
        PrefabSpawn {
            storage: self.storage,
            parts: self.parts.reborrow(),
        }
    }
}

pub fn prefab_parts(resources: &mut ResourceMap) -> PrefabParts<'_> {
    use std::any::TypeId;
    let [
        meshes,
        mesh_state,
        materials,
        textures,
        sources,
        loading,
        hierarchy,
    ] = resources.res_disjoint_mut([
        TypeId::of::<MeshCache>(),
        TypeId::of::<MeshRenderState>(),
        TypeId::of::<MaterialRegistry>(),
        TypeId::of::<TextureCache>(),
        TypeId::of::<TextureSources>(),
        TypeId::of::<LoadingState>(),
        TypeId::of::<HierarchyIndex>(),
    ]);
    PrefabParts {
        meshes: meshes.downcast_mut().expect("MeshCache"),
        mesh_state: mesh_state.downcast_mut().expect("MeshRenderState"),
        materials: materials.downcast_mut().expect("MaterialRegistry"),
        textures: textures.downcast_mut().expect("TextureCache"),
        sources: sources.downcast_mut().expect("TextureSources"),
        loading: loading.downcast_mut().expect("LoadingState"),
        hierarchy: hierarchy.downcast_mut().expect("HierarchyIndex"),
    }
}

impl<W: nightshade::ecs::ResourceHost> ExtractParam<W> for PrefabParts<'_> {
    type State = ();
    type Owned = ();
    type Item<'item> = PrefabParts<'item>;

    fn init() -> Self::State {}

    fn resources() -> Vec<ResourceNeed> {
        vec![
            ResourceNeed::write::<MeshCache>(),
            ResourceNeed::write::<MeshRenderState>(),
            ResourceNeed::write::<MaterialRegistry>(),
            ResourceNeed::write::<TextureCache>(),
            ResourceNeed::write::<TextureSources>(),
            ResourceNeed::write::<LoadingState>(),
            ResourceNeed::write::<HierarchyIndex>(),
        ]
    }

    fn extract(_state: &mut (), _host: &W) {}

    fn build<'a, I: Iterator<Item = Option<ResourceSlot<'a>>>>(
        _owned: &'a mut (),
        slots: &mut I,
        _resources: &'a ResourceMap,
    ) -> PrefabParts<'a> {
        PrefabParts {
            meshes: next_slot(slots),
            mesh_state: next_slot(slots),
            materials: next_slot(slots),
            textures: next_slot(slots),
            sources: next_slot(slots),
            loading: next_slot(slots),
            hierarchy: next_slot(slots),
        }
    }

    fn apply(_state: &mut (), _owned: (), _host: &W) {}

    fn access() -> SystemAccess {
        use nightshade::ecs::Access;
        use std::any::TypeId;
        SystemAccess::empty()
            .with_resource(TypeId::of::<MeshCache>(), Access::Write)
            .with_resource(TypeId::of::<MeshRenderState>(), Access::Write)
            .with_resource(TypeId::of::<MaterialRegistry>(), Access::Write)
            .with_resource(TypeId::of::<TextureCache>(), Access::Write)
            .with_resource(TypeId::of::<TextureSources>(), Access::Write)
            .with_resource(TypeId::of::<LoadingState>(), Access::Write)
            .with_resource(TypeId::of::<HierarchyIndex>(), Access::Write)
    }
}

fn next_slot<'a, T: Send + Sync + 'static>(
    slots: &mut impl Iterator<Item = Option<ResourceSlot<'a>>>,
) -> &'a mut T {
    slots
        .next()
        .expect("the runner borrows one slot for each resource a spawn names")
        .expect("a prefab spawn names no resource it can do without")
        .into_mut::<T>()
}
