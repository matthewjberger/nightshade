use crate::commands::wire::open_wire;
use crate::data::WireSettings;
use nightshade::app::App;
use nightshade::plugin::Plugin;

#[derive(Default)]
pub struct NetPlugin {
    pub settings: WireSettings,
}

impl NetPlugin {
    pub fn new(settings: WireSettings) -> Self {
        Self { settings }
    }
}

impl Plugin for NetPlugin {
    fn build(&self, app: &mut App) {
        install(app, &self.settings);
    }
}

fn install(app: &mut App, settings: &WireSettings) {
    app.insert_resource(open_wire(WireSettings {
        role: settings.role,
        name: settings.name.clone(),
        bind: settings.bind.clone(),
        connect: settings.connect.clone(),
        subscribe: settings.subscribe.clone(),
    }));
}
