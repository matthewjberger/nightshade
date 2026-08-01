use crate::commands::spawn_glider_wing;
use crate::components::{Grounded, Motion, Submerged, Velocity, Wing, Yaw, Zoom};
use crate::data::settings::Settings;
use crate::resources::Profile;
use crate::theme;
use blocker_figure as figure;
use blocker_figure::commands::{advance_the_stride, plate_the_body, show_the_tree};
use blocker_figure::components::{Gait, Gaze, Pace, Plate, Soar};
use blocker_figure::queries::read_the_hero;
use blocker_figure::resources::Hero;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::ecs::HierarchyIndex;
use nightshade::prelude::*;
use nightshade_config::prelude::Config;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::{LocalTransform, Visibility};
use nightshade_text::prelude::TextState;

pub fn stride_the_body(
    time: Res<Time>,
    mut bodies: Query<(
        &Velocity,
        &Motion,
        &Submerged,
        &Grounded,
        &mut Gait,
        &mut Pace,
        &mut Soar,
    )>,
) {
    let delta_time = time.delta_time.min(0.05);
    bodies.for_each_mut(
        |_, (velocity, motion, submerged, grounded, gait, pace, soar)| {
            let gliding = if *motion == Motion::Glide { 1.0 } else { 0.0 };
            soar.0 += (gliding - soar.0) * (figure::theme::SOAR_EASE * delta_time).clamp(0.0, 1.0);
            let pace_speed = Vec2::new(velocity.0.x, velocity.0.z).magnitude();
            let floating =
                matches!(motion, Motion::Fly | Motion::Watch) || submerged.0 || !grounded.0;
            let animation = if *motion == Motion::Glide {
                theme::SOAR_GAIT
            } else if floating {
                theme::FLOAT_GAIT
            } else {
                1.0
            };
            advance_the_stride(
                gait,
                pace,
                pace_speed,
                pace_speed > 0.6,
                animation,
                delta_time,
            );
        },
    );
}

pub fn name_the_player(
    profile: Res<Profile>,
    hero: Res<Hero>,
    mut text_state: ResMut<TextState>,
    mut commands: Commands,
    mut plated: Query<(&Plate,)>,
    mut bodies: Query<(&LocalTransform,), (With<Gait>, With<Yaw>)>,
) {
    let Some(body) = hero.0 else {
        return;
    };
    let Some(place) = read_the_hero(&hero, &mut bodies, |(transform,)| transform.translation)
    else {
        return;
    };
    let mut has_plate = false;
    plated.for_entity(body, |_| has_plate = true);
    if has_plate {
        return;
    }
    plate_the_body(
        &mut text_state,
        &mut commands,
        body,
        &profile.name,
        place,
        figure::queries::plate_properties(),
    );
}

type Gliding<'world, 'item> = Query<
    'world,
    (
        &'item Motion,
        &'item Yaw,
        &'item Gaze,
        &'item Zoom,
        &'item LocalTransform,
        Option<&'item Wing>,
    ),
>;

pub fn carry_the_glider(
    hero: Res<Hero>,
    config: Res<Config<Settings>>,
    mut commands: Commands,
    mut seen: Query<(&Visibility,)>,
    mut bodies: Gliding<'_, '_>,
) {
    let Some(body) = hero.0 else {
        return;
    };
    let Some((motion, yaw, gaze, zoom, stood, wing)) = read_the_hero(
        &hero,
        &mut bodies,
        |(motion, yaw, gaze, zoom, transform, wing)| {
            (
                *motion,
                yaw.0,
                gaze.0,
                zoom.0,
                transform.translation,
                wing.map(|wing| wing.0),
            )
        },
    ) else {
        return;
    };
    let wing = match wing {
        Some(wing) => wing,
        None => {
            let wing = spawn_glider_wing(&mut commands);
            commands.set(body, Wing(wing));
            wing
        }
    };
    let third = zoom >= theme::ZOOM_FIRST_PERSON;
    let wanted = (third || config.toggle(Settings::GliderInFirstPerson)) && motion == Motion::Glide;
    let mut current = None;
    seen.for_entity(wing, |(visibility,)| current = Some(visibility.visible));
    if current != Some(wanted) {
        commands.set(wing, Visibility { visible: wanted });
    }
    if wanted {
        let rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y())
            * nalgebra_glm::quat_angle_axis(theme::WING_PITCH, &Vec3::x());
        let up = nalgebra_glm::quat_rotate_vec3(&rotation, &Vec3::y());
        let place = stood + Vec3::new(0.0, theme::GLIDER_LIFT, 0.0) + up * theme::WING_RISE
            - gaze * theme::WING_BACK;
        commands.set(
            wing,
            LocalTransform {
                translation: place,
                rotation,
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
        );
    }
}

pub fn show_the_body(
    hierarchy: Res<HierarchyIndex>,
    hero: Res<Hero>,
    mut commands: Commands,
    mut seen: Query<(&Visibility,)>,
    mut bodies: Query<(&Zoom,), With<Gait>>,
) {
    let Some(body) = hero.0 else {
        return;
    };
    let Some(zoom) = read_the_hero(&hero, &mut bodies, |(zoom,)| zoom.0) else {
        return;
    };
    let third = zoom >= theme::ZOOM_FIRST_PERSON;
    show_the_tree(&hierarchy, &mut commands, &mut seen, body, third);
}
