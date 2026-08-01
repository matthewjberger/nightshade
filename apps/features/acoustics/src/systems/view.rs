use crate::commands::glaze_the_building;
use crate::data::{CHEST_DROP, VIEW_BACK, VIEW_LIFT};
use crate::resources::AcousticsDemo;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::contract::LocalTransform;
use nightshade_scene::contract::primitives::Visibility;

pub fn hold_the_view(
    input: Res<Input>,
    mut demo: ResMut<AcousticsDemo>,
    mut registry: ResMut<nightshade_render::resources::material_registry::MaterialRegistry>,
    mut commands: Commands,
    mut places: Query<(&mut LocalTransform,)>,
) {
    if !demo.built {
        return;
    }

    let mut eye = demo.eye;
    let mut turn = Quat::identity();
    places.for_entity(demo.camera, |(transform,)| {
        eye = transform.translation;
        turn = transform.rotation;
    });

    let moved = eye - demo.eye;
    if demo.chasing {
        demo.stand += moved;
    } else {
        demo.stand = eye;
    }

    if input.keyboard.just_pressed(KeyCode::KeyC) {
        demo.chasing = !demo.chasing;
        let visible = demo.chasing;
        commands.set(demo.head, Visibility { visible });
        commands.set(demo.chest, Visibility { visible });
        glaze_the_building(&mut registry, visible);
    }

    let ahead = nalgebra_glm::quat_rotate_vec3(&turn, &Vec3::new(0.0, 0.0, -1.0));
    let stand = demo.stand;
    let place = if demo.chasing {
        stand - ahead * VIEW_BACK + Vec3::new(0.0, VIEW_LIFT, 0.0)
    } else {
        stand
    };
    if (place - eye).magnitude() > 1.0e-5 {
        places.for_entity(demo.camera, |(transform,)| {
            transform.translation = place;
        });
    }
    demo.eye = place;

    let facing = nalgebra_glm::quat_angle_axis((-ahead.x).atan2(-ahead.z), &Vec3::y());
    places.for_entity(demo.head, |(transform,)| {
        transform.translation = stand;
        transform.rotation = facing;
    });
    places.for_entity(demo.chest, |(transform,)| {
        transform.translation = stand - Vec3::new(0.0, CHEST_DROP, 0.0);
        transform.rotation = facing;
    });
}
