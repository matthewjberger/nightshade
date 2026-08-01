use crate::queries::field_of;
use crate::resources::{Ground, Plan};
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::{Time, Window};
use nightshade_vulkan::resources::GpuDrawn;

pub fn show_the_rate(
    time: Res<Time>,
    ground: Res<Ground>,
    plan: Res<Plan>,
    drawn: Res<GpuDrawn>,
    mut window: ResMut<Window>,
) {
    let (tiles, boulders, triangles) = field_of(&ground, &plan);
    if tiles == 0 {
        window.title = format!("{} - {:.0} fps", theme::TITLE, time.frames_per_second);
        return;
    }
    let held = format!(
        "{} - {tiles} tiles, {boulders} boulders, {:.1}M triangles",
        theme::TITLE,
        triangles as f32 / 1.0e6
    );
    window.title = if drawn.clusters == 0 {
        format!("{held} - {:.0} fps", time.frames_per_second)
    } else {
        format!(
            "{held} - {:.1}M drawn in {} clusters - {:.0} fps",
            drawn.triangles as f32 / 1.0e6,
            drawn.clusters,
            time.frames_per_second
        )
    };
}
