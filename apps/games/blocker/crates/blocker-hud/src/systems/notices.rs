use blocker_screen::data::notice::Notice;
use nightshade::ecs::system_param::EventReader;
use nightshade::prelude::*;
use nightshade_toast::commands::say;
use nightshade_toast::{Toast, ToastSettings};

pub fn take_the_notices(
    notices: EventReader<Notice>,
    settings: Res<ToastSettings>,
    mut toast: ResMut<Toast>,
) {
    for notice in &notices {
        say(&mut toast, &settings, notice.0.clone());
    }
}
