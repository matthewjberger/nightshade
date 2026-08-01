use crate::contract::{Bulk, Crowded, NearFrame};
use nightshade::prelude::*;
use nightshade_scene::contract::GlobalTransform;

pub fn gather_the_points(
    mut frame: ResMut<NearFrame>,
    points: Query<(&Bulk, &GlobalTransform, Option<&Crowded>)>,
) {
    frame.points.clear();
    points.for_each(|entity, (bulk, transform, crowded)| {
        let place = transform.0.column(3);
        frame.points.push((
            entity,
            nalgebra_glm::vec3(place.x, place.y, place.z),
            bulk.0,
            crowded.map(|weight| weight.0).unwrap_or(0.0),
        ));
    });
}
