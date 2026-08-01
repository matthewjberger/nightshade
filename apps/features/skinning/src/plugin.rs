use crate::resources::{Gait, Model, Watch};
use crate::systems;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub struct SkinningPlugin;

impl Plugin for SkinningPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "Skinning - whole clip and filtered clip".to_string();
        app.insert_resource(Model::default());
        app.insert_resource(Gait::default());
        app.insert_resource(Watch::default());
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(Stage::Update, systems::foxes::stand_the_foxes);
        app.add_system(Stage::Update, systems::gait::change_gait);
        app.add_system(Stage::Update, systems::watch::report);
    }
}
