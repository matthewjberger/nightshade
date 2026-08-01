use crate::components::Home;
use crate::data::{Avatar, MenuCamera};
use crate::resources::{CoinDashResources, Screen};
use crate::systems::screens::leave_run;
use crate::systems::world::{
    ARENA, COIN_HEIGHT, PICKUP_RADIUS, ROUND_SECONDS, generate_world, scatter,
};
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_ui::prelude::ui_set_text;
use nightshade_ui::resources::RetainedUiDirty;

pub fn update(
    time: Res<Time>,
    mut resources: ResMut<CoinDashResources>,
    mut window: ResMut<Window>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    mut players: Query<(&LocalTransform,), WithTag<Avatar>>,
    mut coins: Query<(&mut Home,)>,
    mut menu_cameras: Query<(&LocalTransform,), WithTag<MenuCamera>>,
) {
    let resources = &mut *resources;
    if resources.round.screen != Screen::Playing {
        return;
    }
    let delta_time = time.delta_time;

    let mut avatars = Vec::new();
    let mut here = None;
    players.for_each_mut(|entity, (transform,)| {
        avatars.push(entity);
        if here.is_none() {
            here = Some(transform.translation);
        }
    });
    let Some(here) = here else {
        return;
    };

    let mut reached = Vec::new();
    coins.for_each_mut(|entity, (home,)| {
        if (home.0.x - here.x).hypot(home.0.z - here.z) < PICKUP_RADIUS {
            reached.push(entity);
        }
    });
    for entity in reached {
        resources.round.score += 1;
        let (x, z) = scatter(&mut resources.round.rng, 5.0, ARENA * 0.85);
        coins.for_entity(entity, |(home,)| {
            home.0 = Vec3::new(x, COIN_HEIGHT, z);
        });
    }

    let round = resources.round.round;
    if resources.round.score >= resources.round.goal {
        let state = &mut resources.round;
        state.round = round + 1;
        state.score = 0;
        state.goal = 5 + (round + 1) * 3;
        state.time = ROUND_SECONDS;
        generate_world(resources, &mut commands);
    }

    resources.round.time = (resources.round.time - delta_time).max(0.0);
    let round = resources.round.round;
    let score = resources.round.score;
    let goal = resources.round.goal;
    let remaining = resources.round.time;
    let handles = resources.ui;

    if let Some(readout) = handles.hud_label {
        ui_set_text(
            &mut commands,
            readout,
            &format!("World {round}   {score}/{goal}   {:.0}s", remaining.ceil()),
        );
    }

    if remaining <= 0.0 {
        if let Some(summary) = handles.over_score {
            ui_set_text(
                &mut commands,
                summary,
                &format!("You reached world {round}"),
            );
        }
        let mut menu_camera = None;
        menu_cameras.for_each_mut(|entity, _| {
            if menu_camera.is_none() {
                menu_camera = Some(entity);
            }
        });
        leave_run(
            resources,
            &mut window,
            &mut dirty,
            &mut commands,
            &avatars,
            menu_camera,
            Screen::Over,
        );
    }
}
