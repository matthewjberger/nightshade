use crate::components::Parallax;
use crate::resources::CoinfallResources;
use nightshade::prelude::*;
use nightshade_scene::prelude::*;

pub fn drift(resources: Res<CoinfallResources>, sky: Query<(&Parallax, &mut LocalTransform)>) {
    let camera_x = resources.camera_x;
    sky.for_each(|_entity, (parallax, transform)| {
        transform.translation.x = camera_x * parallax.0 - crate::theme::SKY_MARGIN;
    });
}
