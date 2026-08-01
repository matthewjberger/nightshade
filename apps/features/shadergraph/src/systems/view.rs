use crate::resources::Preview;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_ui::prelude::DockState;

pub fn measure_the_preview(dock: Res<DockState>, mut preview: ResMut<Preview>) {
    let reserved = &dock.primary.reserved_areas;
    preview.left = reserved.left;
    preview.right = reserved.right;
    preview.top = reserved.top;
    preview.bottom = reserved.bottom;
}
