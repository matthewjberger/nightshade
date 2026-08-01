use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub fn show_the_rate(time: Res<Time>, mut window: ResMut<Window>) {
    window.title = format!(
        "{} - {} copies - {:.0} fps",
        theme::TITLE,
        theme::COPIES,
        time.frames_per_second
    );
}
