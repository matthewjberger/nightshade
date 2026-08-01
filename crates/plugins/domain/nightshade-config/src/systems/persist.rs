use crate::contract::{Config, ConfigSchema};
use crate::data::SETTLE_SECONDS;
use crate::queries::config_text;
use crate::resources::ConfigFile;
use nightshade::prelude::{Res, ResMut};
use nightshade_platform::contract::time::Time;
use nightshade_platform::contract::window::Window;
use nightshade_store::prelude::store_write_text;

pub fn keep_the_settings<S: ConfigSchema>(
    config: Res<Config<S>>,
    time: Res<Time>,
    window: Res<Window>,
    mut file: ResMut<ConfigFile<S>>,
) {
    if file.saved_revision == config.revision() {
        return;
    }
    if file.seen_revision == config.revision() {
        file.settle_seconds += time.delta_time;
    } else {
        file.seen_revision = config.revision();
        file.settle_seconds = 0.0;
    }
    let leaving = window.close_requested || window.should_exit;
    if file.settle_seconds < SETTLE_SECONDS && !leaving {
        return;
    }
    let text = config_text(&config);
    if store_write_text(file.folder, file.name, &text) {
        file.saved_revision = config.revision();
    }
}
