use crate::resources::{Ground, Plan, Terrain, TileJob};
use crate::systems;
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_worker::prelude::workers_hire;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = theme::TITLE.to_string();
        app.insert_resource(Plan::default());
        app.insert_resource(Ground::default());
        app.insert_resource(Terrain::default());
        app.insert_resource(workers_hire::<TileJob>(theme::TILES_IN_FLIGHT));
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(
            Stage::Update,
            (
                systems::flight::keep_above_the_ground,
                systems::streaming::stream_the_ground,
                systems::streaming::place_the_tiles,
                systems::streaming::clear_the_tiles,
                systems::title::show_the_rate,
            ),
        );
    }
}
