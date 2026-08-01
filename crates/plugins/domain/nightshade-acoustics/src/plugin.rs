use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

pub struct AcousticsPlugin;

impl Plugin for AcousticsPlugin {
    fn build(&self, app: &mut App) {
        install(app);
        app.add_system(
            Stage::Render,
            crate::systems::gather::gather_acoustics_system,
        );
    }
}

pub fn install(app: &mut App) {
    app.world
        .resource_or_insert_default::<crate::contract::AcousticsFrame>();
    app.world
        .resource_or_insert_default::<crate::contract::AcousticsEcho>();
    app.world
        .resource_or_insert_default::<crate::contract::AcousticsGrid>();
    app.world
        .resource_or_insert_default::<crate::contract::AcousticsHeard>();
    app.world
        .resource_or_insert_default::<crate::contract::AcousticsRays>();
    app.world
        .resource_or_insert_default::<crate::contract::AcousticsRoom>();
    app.world
        .resource_or_insert_default::<crate::contract::AcousticsSettings>();
}
