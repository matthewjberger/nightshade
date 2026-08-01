use crate::components::{Home, Offset, Reach, Yaw};
use crate::contract::WorldSide;
use crate::data;
use crate::data::Ask;
use crate::resources::{MashupResources, Phase};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::{LocalTransform, Visibility};

pub fn turn_camera(
    time: Res<Time>,
    actions: Res<Actions<Ask>>,
    mut resources: ResMut<MashupResources>,
) {
    if awake(&resources) {
        if actions.just_pressed(Ask::TurnLeft) {
            resources.meadow.yaw_index -= 1;
        }
        if actions.just_pressed(Ask::TurnRight) {
            resources.meadow.yaw_index += 1;
        }
    }
    let wanted = resources.meadow.yaw_index as f32 * std::f32::consts::FRAC_PI_2;
    let share = (data::CAMERA_TURN * time.delta_time).clamp(0.0, 1.0);
    let yaw = resources.meadow.yaw;
    resources.meadow.yaw = yaw + (wanted - yaw) * share;
    if (wanted - resources.meadow.yaw).abs() < 0.0005 {
        resources.meadow.yaw = wanted;
    }
}

pub fn walk(time: Res<Time>, actions: Res<Actions<Ask>>, mut resources: ResMut<MashupResources>) {
    let delta_time = time.delta_time.min(0.05);
    let yaw = resources.meadow.yaw;
    let forward = Vec3::new(-yaw.sin(), 0.0, -yaw.cos());
    let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());

    let mut wish = Vec3::zeros();
    if awake(&resources) {
        let asked = actions.axis2(Ask::Walk);
        wish = right * asked.x + forward * asked.y;
    }

    if wish.norm() > 1.0e-4 {
        wish = wish.normalize();
        resources.meadow.facing = (-wish.x).atan2(-wish.z);
    }

    let reach = data::MEADOW_HALF - 1.0;
    let step = wish * data::MEADOW_SPEED * delta_time;
    let feet = resources.meadow.feet + step;
    resources.meadow.feet = Vec3::new(
        feet.x.clamp(-reach, reach),
        data::MEADOW_GROUND,
        feet.z.clamp(-reach, reach),
    );

    let share = (data::CAMERA_FOLLOW * delta_time).clamp(0.0, 1.0);
    let focus = resources.meadow.focus;
    resources.meadow.focus = focus + (resources.meadow.feet - focus) * share;
}

pub fn wander(time: Res<Time>, villagers: Query<(&Home, &Offset, &Reach, &mut LocalTransform)>) {
    let clock = time.uptime_seconds as f32;
    villagers.for_each(|_entity, (home, offset, reach, transform)| {
        let turn = clock * data::VILLAGER_SPEED * 0.25 + offset.0;
        transform.translation =
            home.0 + Vec3::new(turn.cos() * reach.0, 0.0, (turn * 0.7).sin() * reach.0);
    });
}

pub fn place(resources: Res<MashupResources>, mut commands: Commands) {
    let meadow = &resources.meadow;
    let shown = resources.here == WorldSide::Meadow && resources.crossing.phase != Phase::Diving;
    if meadow.body != Entity::default() {
        commands.set(meadow.body, Visibility { visible: shown });
        commands.set(
            meadow.body,
            LocalTransform {
                translation: meadow.feet,
                rotation: nalgebra_glm::quat_angle_axis(meadow.yaw, &Vec3::y()),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
        );
    }
    if meadow.shadow != Entity::default() {
        commands.set(meadow.shadow, Visibility { visible: shown });
        commands.set(
            meadow.shadow,
            LocalTransform {
                translation: meadow.feet + Vec3::new(0.0, data::SHADOW_LIFT, 0.3),
                rotation: nalgebra_glm::quat_angle_axis(-std::f32::consts::FRAC_PI_2, &Vec3::x()),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
        );
    }
}

pub fn turn_uprights(
    resources: Res<MashupResources>,
    uprights: Query<(&mut Yaw, &mut LocalTransform)>,
) {
    let yaw = resources.meadow.yaw;
    let rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y());
    uprights.for_each(|_entity, (upright_yaw, transform)| {
        if upright_yaw.0 == yaw {
            return;
        }
        upright_yaw.0 = yaw;
        transform.rotation = rotation;
    });
}

fn awake(resources: &MashupResources) -> bool {
    resources.here == WorldSide::Meadow && resources.crossing.phase == Phase::Idle
}
