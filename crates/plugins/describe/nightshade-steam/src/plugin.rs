use nightshade::app::{App, Stage};
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade::plugin::Plugin;

pub struct SteamPlugin;

impl Plugin for SteamPlugin {
    fn build(&self, app: &mut App) {
        app.world
            .insert_resource(crate::resources::SteamResources::default());
        app.add_system(
            Stage::Startup,
            |mut steam: ResMut<crate::resources::SteamResources>| {
                if let Err(error) = crate::commands::steam_initialize(&mut steam) {
                    tracing::warn!("Steam unavailable: {error}");
                }
            },
        );
        app.add_system(
            Stage::First,
            |steam: Res<crate::resources::SteamResources>| {
                crate::commands::steam_run_callbacks(&steam);
            },
        );
    }
}
