use nightshade::prelude::*;
use nightshade_render::components::lines::{Line, Lines};
use nightshade_scene::contract::GlobalTransform;
use nightshade_scene::contract::primitives::{Name, Visibility};

use crate::queries::skeleton::bone_segments;
use crate::resources::ViewerResources;
use crate::theme::BONE_COLOR;

pub fn update_skeleton(
    mut viewer: ResMut<ViewerResources>,
    mut commands: Commands,
    storage: &mut nightshade::ecs::Storage,
) {
    let viewer = &mut *viewer;

    if !viewer.skeleton.enabled {
        if let Some(entity) = viewer.skeleton.entity
            && let Some(visibility) = storage.get_mut::<Visibility>(entity)
            && visibility.visible
        {
            visibility.visible = false;
            if let Some(lines) = storage.get_mut::<Lines>(entity) {
                lines.clear();
            }
        }
        return;
    }

    let entity = match viewer.skeleton.entity {
        Some(entity) if storage.has::<Lines>(entity) => entity,
        _ => {
            let entity = commands.spawn((
                Name("Skeleton Overlay".to_string()),
                Lines::new_always_on_top(Vec::new()),
                Visibility { visible: true },
                GlobalTransform::default(),
            ));
            viewer.skeleton.entity = Some(entity);
            return;
        }
    };

    if let Some(visibility) = storage.get_mut::<Visibility>(entity) {
        visibility.visible = true;
    }

    let segments = bone_segments(storage);
    if let Some(lines) = storage.get_mut::<Lines>(entity) {
        lines.clear();
        for (start, end) in segments {
            lines.push(Line {
                start,
                end,
                color: BONE_COLOR,
            });
        }
    }
}
