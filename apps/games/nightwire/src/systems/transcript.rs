use crate::data::{SECONDS_BETWEEN_LINES, SECONDS_PER_LETTER};
use crate::resources::{NightwireResources, Screen};
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;

const MAX_STEP_SECONDS: f32 = 0.1;
const CARET_SECONDS: f32 = 0.45;

pub fn reveal(time: Res<Time>, mut resources: ResMut<NightwireResources>) {
    if resources.night.screen != Screen::Shift {
        return;
    }

    let step = time.delta_time.min(MAX_STEP_SECONDS);
    resources.night.caret_timer += step;
    if resources.night.caret_timer >= CARET_SECONDS {
        resources.night.caret_timer -= CARET_SECONDS;
        resources.night.caret_on = !resources.night.caret_on;
        resources.rows_dirty = true;
    }

    resources.night.reveal_timer += step;
    let mut changed = false;

    while let Some(length) = resources
        .night
        .queue
        .front()
        .map(|line| line.chars().count())
    {
        if resources.night.revealed >= length {
            if resources.night.reveal_timer < SECONDS_BETWEEN_LINES {
                break;
            }
            resources.night.reveal_timer -= SECONDS_BETWEEN_LINES;
            let line = resources.night.queue.pop_front().unwrap_or_default();
            resources.night.lines.push(line);
            resources.night.revealed = 0;
            changed = true;
            continue;
        }

        if resources.night.reveal_timer < SECONDS_PER_LETTER {
            break;
        }
        resources.night.reveal_timer -= SECONDS_PER_LETTER;
        resources.night.revealed += 1;
        changed = true;
    }

    if changed {
        resources.rows_dirty = true;
    }

    if resources.night.queue.is_empty()
        && let Some(phase) = resources.night.pending_phase.take()
    {
        resources.night.phase = phase;
        resources.controls_dirty = true;
    }
}
