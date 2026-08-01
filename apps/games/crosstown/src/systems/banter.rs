use crate::data;
use crate::resources::{Interface, Talk};
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_ui::prelude::ui_set_text;

pub fn speak(
    time: Res<Time>,
    handles: Res<Interface>,
    mut talk: ResMut<Talk>,
    mut commands: Commands,
) {
    if handles.caption == Entity::default() {
        return;
    }
    talk.time = (talk.time - time.delta_time).max(0.0);
    if talk.time <= 0.0 {
        talk.line = if talk.queue.is_empty() {
            String::new()
        } else {
            talk.time = data::LINE_TIME;
            talk.queue.remove(0)
        };
    }
    if talk.line != talk.shown {
        talk.shown = talk.line.clone();
        let line = talk.shown.clone();
        ui_set_text(&mut commands, handles.caption, &line);
    }
}
