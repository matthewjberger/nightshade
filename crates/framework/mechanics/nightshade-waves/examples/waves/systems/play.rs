use crate::commands::slab;
use crate::components::Lane;
use crate::resources::Gate;
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::prelude::*;
use nightshade_platform::contract::input::Input;
use nightshade_platform::contract::time::Time;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_scene::contract::LocalTransform;
use nightshade_waves::commands::{hurry, one_fell, rewind, start, stop};
use nightshade_waves::{Arrival, WaveCleared, WavePlan, WaveProgress};

pub fn take_the_orders(
    input: Res<Input>,
    mut progress: ResMut<WaveProgress>,
    mut gate: ResMut<Gate>,
) {
    if input.keyboard.just_pressed(KeyCode::Space) {
        if progress.running {
            stop(&mut progress);
        } else {
            start(&mut progress);
        }
    }
    if input.keyboard.just_pressed(KeyCode::KeyH) {
        gate.saved += hurry(&mut progress);
    }
    if input.keyboard.just_pressed(KeyCode::KeyR) {
        rewind(&mut progress);
        gate.cleared = 0;
        gate.saved = 0.0;
    }
}

pub fn meet_the_arrivals(
    arrivals: EventReader<Arrival>,
    mut materials: ResMut<MaterialRegistry>,
    mut commands: Commands,
) {
    for arrival in arrivals.iter() {
        let across = (arrival.index as f32 - 6.0) * 1.4;
        let tint = theme::KIND_TINT[arrival.wave % theme::KIND_TINT.len()];
        let tall = if arrival.last { 1.8 } else { 1.0 };
        let walker = slab(
            &mut commands,
            &mut materials,
            vec3(across, tall * 0.5, -theme::LANE),
            vec3(0.8, tall, 0.8),
            tint,
        );
        commands.set(walker, Lane(0.0));
    }
}

pub fn count_the_cleared(cleared: EventReader<WaveCleared>, mut gate: ResMut<Gate>) {
    for wave in cleared.iter() {
        gate.cleared = wave.wave + 1;
    }
}

pub fn walk_the_lane(
    time: Res<Time>,
    mut progress: ResMut<WaveProgress>,
    mut commands: Commands,
    walkers: Query<(&mut LocalTransform, &mut Lane)>,
) {
    let step = theme::WALK * time.delta_time;
    let mut done = Vec::new();
    walkers.for_each(|entity, (transform, lane)| {
        transform.translation.z += step;
        lane.0 += step;
        if lane.0 >= theme::LANE * 2.0 {
            done.push(entity);
        }
    });
    for entity in done {
        one_fell(&mut progress);
        commands.despawn(entity);
    }
}

pub fn show_the_wave(
    plan: Res<WavePlan>,
    progress: Res<WaveProgress>,
    gate: Res<Gate>,
    mut readout: ResMut<Readout>,
) {
    readout.notes.clear();
    readout
        .notes
        .push(format!("wave {} of {}", progress.wave + 1, plan.0.len()));
    readout.notes.push(format!("{} sent", progress.sent));
    readout
        .notes
        .push(format!("{} still walking", progress.alive));
    readout.notes.push(format!("{} cleared", gate.cleared));
    readout
        .notes
        .push(format!("{:.0} seconds saved", gate.saved));
    readout.notes.push(if progress.resting {
        format!("resting {:.1}", progress.rest.max(0.0))
    } else if progress.running {
        "sending".to_string()
    } else {
        "held".to_string()
    });
}
