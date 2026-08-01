use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

pub struct SdfPlugin;

impl Plugin for SdfPlugin {
    fn build(&self, app: &mut App) {
        app.world
            .resource_or_insert_default::<crate::contract::render_data::SdfFrame>();
        app.world
            .resource_or_insert_default::<crate::contract::render_data::SdfClipmap>();
        app.world
            .resource_or_insert_default::<crate::contract::render_data::SdfDebugView>();
        app.world
            .resource_or_insert_default::<crate::contract::render_data::SdfMarchScale>();
        app.world
            .resource_or_insert_default::<crate::resources::SdfCollision>();
        app.add_system(Stage::Render, crate::systems::gather::gather_edits_system);
        app.add_system(
            Stage::Render,
            crate::systems::collision::sdf_collision_system,
        );
    }
}
