use crate::data::{Arrival, WaveCleared};
use crate::queries::current;
use crate::resources::{WavePlan, WaveProgress};
use nightshade::ecs::system_param::EventWriter;
use nightshade::prelude::*;
use nightshade_platform::contract::time::Time;

pub fn send_the_wave(
    time: Res<Time>,
    plan: Res<WavePlan>,
    mut progress: ResMut<WaveProgress>,
    mut arrivals: EventWriter<Arrival>,
    mut cleared: EventWriter<WaveCleared>,
) {
    if !progress.running {
        return;
    }
    let step = time.delta_time
        * if progress.pace > 0.0 {
            progress.pace
        } else {
            1.0
        };
    if progress.resting {
        progress.rest -= step;
        if progress.rest <= 0.0 {
            progress.resting = false;
            progress.sent = 0;
            progress.wait = 0.0;
        }
        return;
    }
    let Some(wave) = current(&plan, &progress) else {
        progress.running = false;
        return;
    };
    if progress.sent < wave.count {
        progress.wait -= step;
        if progress.wait > 0.0 {
            return;
        }
        progress.wait += wave.spacing;
        progress.sent += 1;
        progress.alive += 1;
        arrivals.send(Arrival {
            wave: progress.wave,
            index: progress.sent - 1,
            last: progress.sent == wave.count,
        });
        return;
    }
    if progress.alive > 0 {
        return;
    }
    cleared.send(WaveCleared {
        wave: progress.wave,
    });
    progress.wave += 1;
    progress.resting = true;
    progress.rest = wave.rest;
}
