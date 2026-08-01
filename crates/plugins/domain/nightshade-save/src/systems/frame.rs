use crate::data::SaveDue;
use crate::resources::Autosave;
use nightshade::ecs::system_param::EventWriter;
use nightshade::prelude::{Res, ResMut};
use nightshade_platform::contract::{Time, Window};

pub fn announce_a_due_save(
    time: Res<Time>,
    window: Res<Window>,
    mut autosave: ResMut<Autosave>,
    mut due: EventWriter<SaveDue>,
) {
    if let Some(reason) = autosave.due(time.delta_time, window.close_requested) {
        due.send(reason);
    }
}
