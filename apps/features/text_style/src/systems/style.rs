use crate::components::{BaseScale, FollowsCamera};
use crate::resources::TextStyleResources;
use nightshade::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_text::contract::Text;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;

pub fn apply(
    mut resources: ResMut<TextStyleResources>,
    mut dirty: ResMut<RetainedUiDirty>,
    texts: Query<(&BaseScale, &FollowsCamera, &mut Text, &mut LocalTransform)>,
    mut nodes: Query<(&mut UiNodeContent,)>,
) {
    let resources = &mut *resources;
    let style = resources.style;
    if resources.applied == Some(style) {
        return;
    }
    resources.applied = Some(style);

    texts.for_each(|_entity, (base_scale, follows_camera, text, transform)| {
        text.properties.outline_width = style.outline_width;
        text.properties.outline_color = style.outline_color;
        text.properties.smoothing = style.smoothing;
        text.billboard = follows_camera.0 && style.billboard;
        let scale = base_scale.0 * style.scale;
        transform.scale = Vec3::new(scale, scale, scale);
    });

    nodes.for_entity(resources.handles.heading, |(content,)| {
        if let UiNodeContent::Text {
            outline_color,
            outline_width,
            ..
        } = content
        {
            *outline_color = style.outline_color;
            *outline_width = style.outline_width;
        }
    });
    dirty.render_dirty = true;
}
