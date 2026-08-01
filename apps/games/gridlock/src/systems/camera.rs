use crate::components::{Velocity, Yaw};
use crate::data::motion::{
    CAMERA_AIM, CAMERA_SKIN, CAMERA_STEP, FIRST_PERSON_DISTANCE, LOT_HEIGHT,
};
use crate::data::settings::Settings;
use crate::queries::collide::lots_near;
use crate::resources::{City, Indoors, Player};
use nightshade::ecs::{HierarchyIndex, Storage};
use nightshade::prelude::*;
use nightshade_camera::queries::camera_look::camera_look_rotation;
use nightshade_camera_controller::contract::ThirdPersonCamera;
use nightshade_config::contract::Config;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::{LocalTransform, Visibility};

pub fn follow(
    player: Res<Player>,
    time: Res<Time>,
    config: Res<Config<Settings>>,
    storage: &mut Storage,
) {
    let (Some(car), Some(camera)) = (player.car, player.chase_camera) else {
        return;
    };
    if player.flying {
        return;
    }
    let back = config.number(Settings::ChaseBack);
    let height = config.number(Settings::ChaseHeight);
    let lag = config.number(Settings::ChaseLag);
    let Some(focus) = storage
        .get::<LocalTransform>(car)
        .map(|transform| transform.translation)
    else {
        return;
    };
    let Some(velocity) = storage.get::<Velocity>(car).map(|velocity| velocity.0) else {
        return;
    };
    let Some(yaw) = storage.get::<Yaw>(car).map(|yaw| yaw.0) else {
        return;
    };
    let travel = if velocity.norm() > 1.0 {
        velocity.normalize()
    } else {
        Vec3::new(yaw.sin(), 0.0, yaw.cos())
    };
    let eye = focus - travel * back + Vec3::y() * height;
    let Some(transform) = storage.get_mut::<LocalTransform>(camera) else {
        return;
    };
    transform.translation += (eye - transform.translation) * (lag * time.delta_time).min(1.0);
    transform.rotation =
        camera_look_rotation(transform.translation, focus + Vec3::y() * CAMERA_AIM, 0.0);
}

pub fn keep_out_of_the_lots(
    player: Res<Player>,
    city: Res<City>,
    indoors: Res<Indoors>,
    config: Res<Config<Settings>>,
    mut bodies: Query<(&LocalTransform,)>,
    mut cameras: Query<(&mut LocalTransform,)>,
) {
    let (Some(camera), Some(body)) = (player.walk_camera, player.body) else {
        return;
    };
    if player.car.is_some() || player.flying {
        return;
    }
    let mut head = None;
    bodies.for_entity(body, |(transform,)| {
        head = Some(transform.translation + Vec3::new(0.0, config.number(Settings::WalkHead), 0.0));
    });
    let Some(head) = head else {
        return;
    };
    let lots = if indoors.visit.is_some() {
        indoors.lots.clone()
    } else {
        lots_near(&city, head)
    };
    cameras.for_entity(camera, |(transform,)| {
        let offset = transform.translation - head;
        let distance = offset.magnitude();
        if distance < FIRST_PERSON_DISTANCE {
            return;
        }
        let direction = offset / distance;
        let mut reach = CAMERA_STEP;
        while reach < distance {
            let point = head + direction * reach;
            let inside = point.y < head.y + LOT_HEIGHT
                && lots.iter().any(|lot| {
                    point.x > lot.minimum.x
                        && point.x < lot.maximum.x
                        && point.z > lot.minimum.y
                        && point.z < lot.maximum.y
                });
            if inside {
                let pulled = (reach - CAMERA_SKIN).max(0.0);
                transform.translation = head + direction * pulled;
                return;
            }
            reach += CAMERA_STEP;
        }
    });
}

pub fn first_person(
    mut player: ResMut<Player>,
    hierarchy: Res<HierarchyIndex>,
    mut commands: Commands,
    mut cameras: Query<(&ThirdPersonCamera,)>,
) {
    let (Some(camera), Some(body)) = (player.walk_camera, player.body) else {
        return;
    };
    let mut hidden = player.car.is_some();
    if player.car.is_none() && !player.flying {
        cameras.for_entity(camera, |(third_person,)| {
            hidden = third_person.distance < FIRST_PERSON_DISTANCE;
        });
    }
    if player.first_person == hidden {
        return;
    }
    player.first_person = hidden;
    let mut pending = vec![body];
    while let Some(entity) = pending.pop() {
        commands.set(entity, Visibility { visible: !hidden });
        if let Some(children) = hierarchy.children.get(&entity) {
            pending.extend(children.iter().copied());
        }
    }
}
