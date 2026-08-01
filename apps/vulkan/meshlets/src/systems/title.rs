use crate::resources::Field;
use crate::theme;
use nightshade::prelude::{Res, ResMut};
use nightshade_platform::prelude::*;

#[cfg(feature = "vulkan")]
pub fn show_the_rate(
    time: Res<Time>,
    field: Res<Field>,
    drawn: Option<Res<nightshade_vulkan::resources::GpuDrawn>>,
    mut window: ResMut<Window>,
) {
    let drawn =
        drawn.and_then(|drawn| (drawn.clusters != 0).then_some((drawn.triangles, drawn.clusters)));
    window.title = compose(*field, drawn, time.frames_per_second);
}

#[cfg(not(feature = "vulkan"))]
pub fn show_the_rate(time: Res<Time>, field: Res<Field>, mut window: ResMut<Window>) {
    window.title = compose(*field, None, time.frames_per_second);
}

fn compose(field: Field, drawn: Option<(u32, u32)>, rate: f32) -> String {
    if field.copies == 0 {
        return format!("{} - {rate:.0} fps", theme::TITLE);
    }
    let held = format!(
        "{} - {} copies, {:.1}M triangles",
        theme::TITLE,
        field.copies,
        field.triangles as f32 / 1.0e6
    );
    match drawn {
        Some((triangles, clusters)) => format!(
            "{held} - {:.1}M drawn in {clusters} clusters - {rate:.0} fps",
            triangles as f32 / 1.0e6
        ),
        None => format!("{held} - {rate:.0} fps"),
    }
}
