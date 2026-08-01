use crate::theme;
use blocker_figure::components::{Eye, Gaze};
use blocker_figure::resources::Hero;
use blocker_player::commands::{Standing, float_the_player};
use blocker_player::components::Yaw;
use blocker_screen::data::fresh::NewWorld;
use blocker_screen::data::screen::Screen;
use blocker_screen::queries::is_pregame;
use blocker_world::queries::city_gate;
use blocker_world::resources::{Bench, Draft, Generation};
use nalgebra_glm::{Vec3, quat_angle_axis, quat_rotate_vec3};
use nightshade::ecs::system_param::{EventReader, EventWriter};
use nightshade::prelude::*;
use nightshade_camera::prelude::Camera;
use nightshade_platform::prelude::Time;
use nightshade_render::data::config::RendererState;
use nightshade_scene::prelude::LocalTransform;
use nightshade_state::prelude::*;

pub fn start_the_world(
    mut draft: ResMut<Draft>,
    mut next: ResMut<NextState<Screen>>,
    mut fresh: EventWriter<NewWorld>,
) {
    if !draft.asked {
        return;
    }
    draft.asked = false;
    fresh.send(NewWorld);
    next.pending = Some(Screen::Loading);
}

pub fn raise_the_new_world(
    fresh: EventReader<NewWorld>,
    draft: Res<Draft>,
    hero: Res<Hero>,
    mut generation: ResMut<Generation>,
    mut bench: ResMut<Bench>,
    mut renderer_state: ResMut<RendererState>,
    mut bodies: Standing<'_, '_>,
) {
    if fresh.into_iter().next().is_none() {
        return;
    }
    let Some(body) = hero.0 else {
        return;
    };
    generation.seed = draft.seed;
    generation.enabled = draft.enabled;
    bench.reseed(draft.seed);
    let gate = city_gate(&bench, 0);
    float_the_player(&mut bodies, body, &mut generation, None, gate);
    renderer_state.day_night.hour = blocker_screen::theme::START_HOUR;
}

pub fn look_over_the_world(
    time: Res<Time>,
    state: Res<State<Screen>>,
    hero: Res<Hero>,
    mut cameras: Query<(&Camera, &mut LocalTransform)>,
    mut bodies: Query<(&mut Eye, &mut Gaze, &mut Yaw, &LocalTransform)>,
) {
    if !is_pregame(state.current) {
        return;
    }
    let Some(body) = hero.0 else {
        return;
    };
    let mut aimed = None;
    bodies.for_entity(body, |(eye, gaze, yaw, transform)| {
        yaw.0 += time.delta_time * theme::TITLE_TURN;
        let rotation =
            quat_angle_axis(yaw.0, &Vec3::y()) * quat_angle_axis(theme::TITLE_PITCH, &Vec3::x());
        let place = transform.translation + Vec3::new(0.0, blocker_player::theme::EYE_HEIGHT, 0.0);
        eye.0 = place;
        gaze.0 = quat_rotate_vec3(&rotation, &Vec3::new(0.0, 0.0, -1.0));
        aimed = Some((place, rotation));
    });
    let Some((place, rotation)) = aimed else {
        return;
    };
    cameras.for_each_mut(|_, (_, transform)| {
        transform.translation = place;
        transform.rotation = rotation;
    });
}
