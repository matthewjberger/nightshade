use crate::data;
use crate::resources::{
    ColumnJob, Dig, Falling, Ground, Meshes, Sight, Streaming, VoxelSettings, VoxelViewer,
    VoxelWorld,
};
use crate::systems;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;
use nightshade_worker::prelude::*;
use std::marker::PhantomData;

pub struct VoxelsPlugin<D: Dig>(PhantomData<D>);

impl<D: Dig> Default for VoxelsPlugin<D> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<D: Dig> Plugin for VoxelsPlugin<D> {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
            nightshade_render::resources::mesh_cache::MeshCache => "GraphicsPlugin",
            nightshade_render::resources::mesh_state::MeshRenderState => "GraphicsPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.world.resource_or_insert_default::<VoxelWorld>();
        app.world.resource_or_insert_default::<Meshes>();
        app.world.resource_or_insert_default::<VoxelViewer>();
        app.world.resource_or_insert_default::<VoxelSettings>();
        app.world.resource_or_insert_default::<Streaming>();
        app.world.resource_or_insert_default::<Sight>();
        app.world.resource_or_insert_default::<Falling>();
        let hands = workers_free();
        let diggers = (hands * data::DIGGER_SHARE / data::CREW_SHARES).clamp(1, data::DIGGER_LIMIT);
        let masons = hands.saturating_sub(diggers).clamp(1, data::MASON_LIMIT);
        app.insert_resource(workers_hire::<ColumnJob<D>>(diggers));
        app.insert_resource(workers_hire::<Ground>(masons));
        app.add_system(Stage::Update, systems::stream::stream_the_world::<D>);
        app.add_system(Stage::Update, systems::settle::settle_the_columns::<D>);
        app.add_system(Stage::Update, systems::shadow::cast_the_shadows);
        app.add_system(Stage::Update, systems::sight::see_the_slabs);
        app.add_system(Stage::Update, systems::gravity::settle_the_loose_blocks);
        app.add_system(Stage::Update, systems::gravity::drop_the_falling);
        app.add_system(Stage::Update, systems::liquid::spread_the_liquids);
        app.add_system(Stage::Update, systems::mesh::rebuild_dirty);
        app.add_system(Stage::Update, systems::mesh::place_the_meshes);
    }
}
