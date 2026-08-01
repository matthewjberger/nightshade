use crate::resources::Save;
use crate::systems;
use nightshade::prelude::*;

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Save::default());
        app.add_system(
            Stage::Startup,
            (
                systems::save::read_the_save,
                systems::save::take_the_save,
                systems::save::sow_the_saved_things,
            ),
        );
        app.add_system(
            Stage::Update,
            (
                systems::fresh::forget_the_save,
                systems::save::keep_the_world,
            ),
        );
    }
}
