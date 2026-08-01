use crate::resources::PickingResources;
use crate::systems;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub struct PickingPlugin;

impl Plugin for PickingPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "Picking Demo".to_string();
        app.insert_resource(PickingResources::default());
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(
            Stage::Update,
            (
                systems::panel::poll,
                systems::world::spin_meshes,
                systems::world::sync_debug_volumes,
                systems::hover::update,
                systems::outline::apply_click,
                systems::outline::sync_outline,
                systems::panel::refresh_text,
                systems::panel::drain_keyboard,
            ),
        );
    }
}
