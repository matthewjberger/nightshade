use nightshade::prelude::*;
use nightshade_platform::prelude::Window;
use nightshade_render::components::lines::{Line, Lines};
use nightshade_scene::contract::GlobalTransform;
use nightshade_scene::contract::primitives::{Name, Visibility};

use crate::queries::markers::marker_shape;
use crate::resources::ViewerResources;

pub fn draw_markers(
    mut viewer: ResMut<ViewerResources>,
    window: Res<Window>,
    mut commands: Commands,
    storage: &mut nightshade::ecs::Storage,
) {
    let viewer = &mut *viewer;
    let segments = viewer
        .selection
        .selected
        .filter(|entity| storage.is_alive(*entity))
        .map(|entity| marker_shape(storage, &window, entity))
        .unwrap_or_default();

    let entity = match viewer.markers.lines {
        Some(entity) if storage.has::<Lines>(entity) => entity,
        _ => {
            let entity = commands.spawn((
                Name("Selection Markers".to_string()),
                Lines::new_always_on_top(Vec::new()),
                Visibility { visible: false },
                GlobalTransform::default(),
            ));
            viewer.markers.lines = Some(entity);
            viewer.markers.drawn.clear();
            return;
        }
    };

    if segments == viewer.markers.drawn {
        return;
    }
    if let Some(visibility) = storage.get_mut::<Visibility>(entity) {
        visibility.visible = !segments.is_empty();
    }
    if let Some(lines) = storage.get_mut::<Lines>(entity) {
        lines.clear();
        for (start, end, color) in &segments {
            lines.push(Line {
                start: *start,
                end: *end,
                color: *color,
            });
        }
    }
    viewer.markers.drawn = segments;
}
