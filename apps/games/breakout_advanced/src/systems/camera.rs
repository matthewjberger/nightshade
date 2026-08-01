use crate::data;
use crate::queries;
use crate::resources::BreakoutAdvancedResources;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn frame(
    time: Res<Time>,
    resources: Res<BreakoutAdvancedResources>,
    cameras: Query<(&mut Camera, &mut LocalTransform)>,
) {
    let clock = time.uptime_seconds as f32;
    let offset = queries::shake_offset(resources.juice.shake, clock);
    let zoom = resources.juice.zoom;

    cameras.for_each(|_entity, (camera, transform)| {
        transform.translation = Vec3::new(offset.x, offset.y, 30.0);
        camera.projection = Projection::Orthographic(OrthographicCamera {
            x_mag: None,
            y_mag: (data::FIELD_HALF_HEIGHT + 1.5) * zoom,
            z_near: 0.1,
            z_far: 100.0,
        });
    });
}
