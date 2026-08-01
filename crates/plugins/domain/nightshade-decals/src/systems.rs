use crate::contract::render_data::DecalInstance;
use crate::contract::{Decal, DecalFrame};
use nightshade::prelude::*;

pub fn gather_decals_system(
    mut frame: ResMut<DecalFrame>,
    projectors: Query<(
        &Decal,
        &nightshade_scene::contract::GlobalTransform,
        Option<&nightshade_scene::contract::primitives::Visibility>,
    )>,
) {
    frame.decals.clear();
    projectors.for_each(|_entity, (decal, global_transform, visibility)| {
        let visible = visibility.is_none_or(|visibility| visibility.visible);
        frame
            .decals
            .push(DecalInstance::new(decal, global_transform.0, visible));
    });
}
