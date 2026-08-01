use crate::contract::{NearAnswers, NearAsks, NearFrame, NearSettings};
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

#[derive(Default)]
pub struct NearPlugin {
    pub settings: NearSettings,
}

impl Plugin for NearPlugin {
    fn build(&self, app: &mut App) {
        install(app, &self.settings);
        app.add_system(Stage::Render, crate::systems::gather::gather_the_points);
    }
}

fn install(app: &mut App, settings: &NearSettings) {
    app.insert_resource(NearSettings {
        cell_size: settings.cell_size,
        room: settings.room,
        cell_capacity: settings.cell_capacity,
    });
    app.insert_resource(NearFrame::default());
    app.insert_resource(NearAsks::default());
    app.insert_resource(NearAnswers::default());
}
