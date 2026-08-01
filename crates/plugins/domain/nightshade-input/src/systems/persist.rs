use crate::contract::{Action, Bindings};
use crate::data::SETTLE_SECONDS;
use crate::queries::bindings_text;
use crate::resources::BindingsFile;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_platform::contract::time::Time;
use nightshade_platform::contract::window::Window;
use nightshade_store::prelude::store_write_text;

pub fn keep_the_bindings<A: Action>(
    bindings: Res<Bindings<A>>,
    time: Res<Time>,
    window: Res<Window>,
    mut file: ResMut<BindingsFile<A>>,
) {
    if file.saved_revision == bindings.revision {
        return;
    }
    if file.seen_revision == bindings.revision {
        file.settle_seconds += time.delta_time;
    } else {
        file.seen_revision = bindings.revision;
        file.settle_seconds = 0.0;
    }
    let leaving = window.close_requested || window.should_exit;
    if file.settle_seconds < SETTLE_SECONDS && !leaving {
        return;
    }
    if store_write_text(file.folder, file.name, &bindings_text(&bindings)) {
        file.saved_revision = bindings.revision;
    }
}
