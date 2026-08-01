use crate::resources::Bounce;
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub fn show_the_rate(time: Res<Time>, bounce: Res<Bounce>, mut window: ResMut<Window>) {
    let grid = if bounce.enabled { "probes" } else { "sky only" };
    window.title = format!(
        "{} - {} - {:.0} fps",
        theme::TITLE,
        grid,
        time.frames_per_second
    );
}
