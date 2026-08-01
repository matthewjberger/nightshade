use crate::resources::{Field, Pending};
use crate::systems;
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub struct MeshletsPlugin;

impl Plugin for MeshletsPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = theme::TITLE.to_string();
        app.insert_resource(Pending::default());
        app.insert_resource(Field::default());
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(
            Stage::Update,
            (
                systems::setup::stand_the_field,
                systems::title::show_the_rate,
            ),
        );
    }
}
